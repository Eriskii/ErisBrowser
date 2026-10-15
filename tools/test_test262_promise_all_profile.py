"""Frozen Promise.all corpus/profile controls; network and adapters stay mocked."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner
import test_test262_promise_runner as legacy

PROFILE = 'promise-all'
OLD_PROFILE = 'promise-methods-async'
CORPUS = importer.ROOT / 'tests/upstream/test262-promise-all'
OLD_CORPUS = importer.ROOT / 'tests/upstream/test262-promise-methods-async'
ORACLE = Path(__file__).with_name('promise-all-profile-oracle.json')
ORACLE_SHA = '0a8c1db76b33216daec60e0062363708f0a8bbb3910f8b9b150b0b227faab2d2'
ALL_TREE = 'ce292ad759c9c9c91d7b67b134af45c93668d702'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def proof_reader(changed=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == changed:
            value = json.loads(data)
            mutate(value)
            return json.dumps(value).encode()
        return data
    return read


def identities(cases):
    return [dict(id=case['id'], case_sha256=case['case_sha256'],
                 policy=case['completion_policy'],
                 harness=[name for name, _ in case['harness']],
                 class_excluded='class' in case['metadata']['features']) for case in cases]


class PromiseAllProfileTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.manifest = self.oracle['expected_manifest']
        for owner, name in [(importer, 'fetch'), (runner, 'bounded_process')]:
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def files(self, profile=PROFILE):
        corpus = OLD_CORPUS if profile == OLD_PROFILE else CORPUS
        return {name: (corpus / name).read_bytes() for name in self.oracle['auxiliary_files']}

    def cached_fetch(self, url):
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        self.assertNotIn('/contents/', url)
        if url.startswith(api):
            return proof_reader()(url[len(api):])
        self.assertTrue(url.startswith(raw), url)
        name = url[len(raw):]
        self.assertIn(name, {row['path'] for row in self.manifest['files']})
        return (CORPUS / name).read_bytes()

    def test_complete_98_file_selection_and_all_original_blob_bindings(self):
        self.assertEqual(importer.PROFILES[PROFILE], {'Promise/all': 98})
        self.assertEqual(importer.PROFILE_ROOTS[PROFILE], 'test/built-ins')
        self.assertIn(PROFILE, importer.TREE_PROFILES)
        self.assertEqual(importer.corpus_name(PROFILE), 'test262-promise-all')
        actual = json.loads((CORPUS / 'manifest.json').read_bytes())
        self.assertEqual(actual, self.manifest)
        self.assertEqual((actual['test_files'], len(actual['files']),
                          len(actual['inventory_proof']['files'])), (98, 106, 7))
        self.assertEqual(sum(row['bytes'] for row in actual['inventory_proof']['files']), 67567)
        for row in actual['files'] + actual['inventory_proof']['files']:
            data = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(data), digest(data)), (row['bytes'], row['sha256']), row['path'])
        listings, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        self.assertEqual({k: sorted(row['name'] for row in v) for k, v in listings.items()}, actual['directories'])
        self.assertEqual(description, {k: v for k, v in actual['inventory_proof'].items() if k != 'files'})
        self.assertEqual(set(proof), {row['path'] for row in actual['inventory_proof']['files']})

    def test_class_and_async_sources_are_retained_and_helpers_are_exact(self):
        counts = dict(async_files=0, sync_files=0, modes=0, class_files=0, bytes=0)
        helpers = {'assert.js', 'sta.js'}
        for row in self.manifest['files']:
            if not row['path'].startswith('test/'):
                continue
            self.assertNotIn('_FIXTURE', row['path'])
            data = (CORPUS / row['path']).read_bytes()
            metadata = importer.parse_metadata(data.decode())
            self.assertIsNone(metadata['negative'])
            self.assertFalse(set(metadata['flags']) - {'async'})
            counts['async_files' if 'async' in metadata['flags'] else 'sync_files'] += 1
            counts['modes'] += 2
            counts['class_files'] += 'class' in metadata['features']
            counts['bytes'] += len(data)
            helpers.update(metadata['includes'])
            if 'async' in metadata['flags']:
                helpers.add('doneprintHandle.js')
        self.assertEqual(counts, dict(async_files=57, sync_files=41, modes=196, class_files=2, bytes=109176))
        self.assertEqual(helpers, {'assert.js', 'sta.js', 'doneprintHandle.js',
                                  'isConstructor.js', 'promiseHelper.js', 'propertyHelper.js'})

    def test_selected_complete_tree_and_ancestor_proofs_refuse_mutation(self):
        for row in self.manifest['inventory_proof']['files']:
            route = row['url'].split('/git/', 1)[1]
            with self.subTest(path=row['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        edits = [lambda d: d.update(truncated=True), lambda d: d['tree'].pop(),
                 lambda d: d['tree'][0].update(sha='0' * 40),
                 lambda d: d['tree'][0].update(path='replacement.js'),
                 lambda d: d['tree'].append(copy.deepcopy(d['tree'][0]))]
        for edit in edits:
            with self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + ALL_TREE, edit))
        for selected in [{}, {'Promise/all': 97}, {'Promise/all': 98, 'Promise/any': 70}]:
            with patch.dict(importer.PROFILES, {PROFILE: selected}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_and_raw_auxiliary_tampering_remain_authenticated(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch), contextlib.redirect_stdout(io.StringIO()):
            destination = Path(temporary)
            importer.import_corpus(destination, PROFILE)
            self.assertEqual(json.loads((destination / 'manifest.json').read_bytes()), self.manifest)
            for row in self.manifest['files'] + self.manifest['inventory_proof']['files']:
                self.assertEqual((destination / row['path']).read_bytes(), (CORPUS / row['path']).read_bytes())
        names = ['test/built-ins/Promise/all/name.js']
        names += sorted(self.oracle['auxiliary_files'])
        for name in set(names):
            def altered(url):
                data = self.cached_fetch(url)
                return data + b'\n' if url.endswith('/' + name) else data
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=altered), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)
                self.fail('mutated source import returned')

    def test_runner_load_retains_196_exact_identities_and_four_class_modes(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(files), len(cases), fixtures), (98, 106, 196, []))
        self.assertEqual(identities(cases), self.oracle['case_identities'])
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ['sloppy', 'strict']], [98, 98])
        self.assertEqual(sum(c['completion_policy'] for c in cases), 114)
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], set(self.oracle['features']))
        excluded = []
        for case in cases:
            self.assertIs(type(case['completion_policy']), int)
            self.assertNotIn('timeout', case['metadata'])
            self.assertEqual(runner.encode_request(case)[:8], b'ERJS2' + bytes([int(case['mode'] == 'strict'), 0, case['completion_policy']]))
            reason = runner.unsupported_reason(case, set(self.oracle['features']), PROFILE)
            if reason:
                excluded.append(case['file'])
                self.assertEqual(reason, 'unimplemented declared features: class')
            for bad in [True, False, 2]:
                poisoned = dict(case, completion_policy=bad)
                self.assertEqual(runner.unsupported_reason(poisoned, set(self.oracle['features']), PROFILE),
                                 'Promise completion policy does not match metadata')
        self.assertEqual(sorted(excluded), sorted(self.oracle['class_sources'] * 2))

    def test_rehashed_source_and_each_auxiliary_cannot_bypass_root_blob_authentication(self):
        names = ['test/built-ins/Promise/all/name.js'] + sorted(self.oracle['auxiliary_files'])
        for name in names:
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, destination)
                data = (destination / name).read_bytes() + b'\n'
                (destination / name).write_bytes(data)
                manifest = json.loads((destination / 'manifest.json').read_bytes())
                next(row for row in manifest['files'] if row['path'] == name).update(bytes=len(data), sha256=digest(data))
                (destination / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(destination, PROFILE)

    def capture(self, change=None, profile=PROFILE):
        held = {'promise-methods-' + row['name']: row for row in self.oracle['old_profile']['controls']}
        held.update({'promise-all-' + row['name']: row for row in self.oracle['controls']})
        captured = []
        def observe(case, *args):
            captured.append(copy.deepcopy(case))
            row = held[case['id'].removeprefix('harness-preflight:')]
            result = legacy.selected_observation(row)
            if change:
                result = change(case, row, result)
            return dict(id=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                        source_sha256=digest(case['source']), **result)
        with patch.object(runner, 'run_case', side_effect=observe):
            results = runner.harness_preflight(self.files(profile), Path('/inert'), 3.0, profile)
        return captured, results

    def test_all_90_preflight_rows_match_frozen_literals_and_dependencies(self):
        self.assertEqual(runner.promise_all_preflight_variants(), self.oracle['controls'])
        captured, results = self.capture()
        old_captured, old_results = self.capture(profile=OLD_PROFILE)
        self.assertEqual((len(captured), len(results)), (90, 90))
        self.assertEqual(captured[:74], old_captured)
        self.assertEqual(results[:74], old_results)
        expected = [(mode, row) for mode in ['sloppy', 'strict'] for row in self.oracle['controls']]
        for case, result, (mode, row) in zip(captured[74:], results[74:], expected):
            name = 'promise-all-' + row['name']
            self.assertEqual((case['id'], case['mode'], case['source']), ('harness-preflight:' + name, mode, row['source'].encode()))
            self.assertEqual(case['completion_policy'], row['completion_policy'])
            names = ['assert.js', 'sta.js'] + (['doneprintHandle.js'] if row['completion_policy'] else []) + row['includes']
            self.assertEqual(case['harness'], [(n, self.files()['harness/' + n]) for n in names])
            self.assertEqual((result['name'], result['expected'], result['verified']), (name, row['expected'], True))
            dependencies = {('promise-methods-' + old['name'], mode) for old in self.oracle['old_profile']['controls']}
            if row['prerequisite']:
                dependencies.add(('promise-all-' + row['prerequisite'], mode))
            self.assertEqual({(r['name'], r['mode']) for r in result['prerequisites']}, dependencies)
            self.assertTrue(all(r['verified'] for r in result['prerequisites']))
        self.assertTrue(runner.promise_all_preflight_healthy(results))

    def test_pair_mismatch_requires_positive_and_common_host_health(self):
        def broken_positive(case, row, result):
            if case['id'] == 'harness-preflight:promise-all-order' and case['mode'] == 'strict':
                return legacy.selected_observation(dict(row, expected='completion-error'))
            return result
        _, results = self.capture(broken_positive)
        rows = {(r['name'], r['result']['mode']): r for r in results}
        self.assertFalse(rows[('promise-all-order', 'strict')]['verified'])
        self.assertFalse(rows[('promise-all-order-mismatch', 'strict')]['verified'])
        self.assertTrue(rows[('promise-all-order-mismatch', 'sloppy')]['verified'])
        def broken_host(case, row, result):
            if case['id'] == 'harness-preflight:promise-methods-one-async-completion' and case['mode'] == 'strict':
                return legacy.selected_observation(dict(row, expected='missing-completion'))
            return result
        _, results = self.capture(broken_host)
        self.assertTrue(all(not r['verified'] for r in results[74:] if r['result']['mode'] == 'strict'))
        self.assertTrue(all(r['verified'] for r in results[74:] if r['result']['mode'] == 'sloppy'))
        for fake in [dict(status='failed'), dict(status='passed'), dict(status='resource'),
                     dict(status='failed', observation=dict(status='exception', phase='runtime', error_type='TypeError'))]:
            _, results = self.capture(lambda case, row, expected: copy.deepcopy(fake) if 'promise-all-' in case['id'] else expected)
            self.assertFalse(any(r['verified'] for r in results[74:]))

    def test_bad_preflight_blocks_all_196_without_dropping_exclusions_or_recording_baseline(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, healthy = self.capture()
        duplicate = copy.deepcopy(healthy)
        duplicate[-1] = copy.deepcopy(duplicate[0])
        wrong_mode = copy.deepcopy(healthy)
        wrong_mode[-1]['result']['mode'] = 'raw'
        unverified = copy.deepcopy(healthy)
        unverified[0]['verified'] = False
        fixtures = [[], healthy[:74], healthy[:-1], duplicate, healthy + healthy[:1], wrong_mode, unverified]
        helper = legacy.PromiseRunnerTests()
        for bad in fixtures:
            self.assertFalse(runner.promise_all_preflight_healthy(bad))
            code, report, recorded, calls = helper.report(PROFILE, bad, cases)
            self.assertEqual((code, recorded, calls), (1, False, []))
            self.assertEqual((report['mode_cases'], report['counts']), (196, {'blocked': 196}))
            self.assertEqual({r['id'] for r in report['cases']}, {c['id'] for c in cases})
            self.assertEqual(report['runner_policy'], self.oracle['policy'])
            self.assertEqual(report['runner_policy_sha256'], self.oracle['policy_sha256'])

    def test_mock_healthy_report_keeps_host_policy_failure_and_all_modes_in_denominator(self):
        # Synthetic verifier outcomes only; untagged syntax may fail in a real adapter.
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, preflight = self.capture()
        def outcome(case):
            reason = runner.unsupported_reason(case, set(self.oracle['features']), PROFILE)
            if reason:
                return dict(status='unsupported', reason=reason)
            return legacy.selected_observation(dict(name='mock-unhandled', completion_policy=case['completion_policy'], expected='unhandled-rejection'))
        code, report, recorded, calls = legacy.PromiseRunnerTests().report(PROFILE, preflight, cases, outcome)
        self.assertEqual((code, recorded, len(calls)), (0, True, 196))
        self.assertEqual(report['counts'], {'failed': 192, 'unsupported': 4})
        self.assertFalse(report['full_test262_conformance'])
        self.assertEqual(report['runner_policy'], self.oracle['policy'])
        self.assertEqual(report['runner_policy_sha256'], self.oracle['policy_sha256'])
        self.assertTrue(all(row['observation']['status'] == 'host-failure' and row['observation']['message'] == 'unhandled-rejection'
                            for row in report['cases'] if row['status'] == 'failed'))

    def test_existing_232_458_profile_bytes_identities_guards_and_policy_are_unchanged(self):
        old = self.oracle['old_profile']
        self.assertEqual(digest((OLD_CORPUS / 'manifest.json').read_bytes()), old['manifest_sha256'])
        self.assertEqual(json.loads((OLD_CORPUS / 'manifest.json').read_bytes()), old['manifest'])
        manifest, _, cases, fixtures, _ = runner.load_corpus(OLD_CORPUS, OLD_PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (232, 458, []))
        self.assertTrue(all('timeout' not in case['metadata'] for case in cases))
        self.assertEqual(identities(cases), old['case_identities'])
        self.assertEqual(digest(canonical(identities(cases))), old['case_identities_sha256'])
        self.assertEqual(runner.promise_preflight_variants(), old['controls'])
        self.assertEqual(digest(canonical(runner.promise_preflight_variants())), old['controls_sha256'])
        _, preflight = self.capture(profile=OLD_PROFILE)
        self.assertEqual(len(preflight), 74)
        code, report, recorded, _ = legacy.PromiseRunnerTests().report(OLD_PROFILE, preflight)
        self.assertEqual((code, recorded), (0, True))
        self.assertEqual((report['runner_policy'], report['runner_policy_sha256']), (old['policy'], old['policy_sha256']))


if __name__ == '__main__':
    unittest.main()
