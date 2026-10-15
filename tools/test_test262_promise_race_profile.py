"""Frozen race profile admission controls; network and adapters are mocked."""
import copy
import contextlib
import hashlib
import io
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner
import test_test262_promise_runner as legacy

PROFILE = 'promise-race'
CORPUS = importer.ROOT / 'tests/upstream/test262-promise-race'
ORACLE = Path(__file__).with_name('promise-race-profile-oracle.json')
ORACLE_SHA = '252b2740d577254459311092942a04ef5d9b8b03b58b313397bb031997fc0427'
RACE_TREE = 'fe8e0bf36a2323ffb4f03d63d56e1516e0855900'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def identities(cases):
    return [dict(id=c['id'], case_sha256=c['case_sha256'], policy=c['completion_policy'],
                 harness=[n for n, _ in c['harness']],
                 class_excluded='class' in c['metadata']['features']) for c in cases]


def proof_reader(changed=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == changed:
            document = json.loads(data)
            mutate(document)
            return json.dumps(document).encode()
        return data
    return read


class PromiseRaceProfileTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.manifest = self.oracle['expected_manifest']
        self.existing = {}
        for name, binding in self.oracle['existing_oracles'].items():
            data = Path(__file__).with_name(name).read_bytes()
            self.assertEqual((len(data), digest(data)), (binding['bytes'], binding['sha256']))
            self.existing[name] = json.loads(data)
        self.all_oracle = self.existing['promise-all-profile-oracle.json']
        self.common = self.existing['promise-methods-runner-oracle.json']['controls']
        for owner, name in [(importer, 'fetch'), (runner, 'bounded_process')]:
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def files(self, profile=PROFILE):
        corpus = importer.ROOT / 'tests/upstream' / ('test262-' + profile)
        names = self.oracle['auxiliary_files'] if profile == PROFILE else self.all_oracle['auxiliary_files']
        return {name: (corpus / name).read_bytes() for name in names}

    def cached_fetch(self, url):
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        self.assertNotIn('/contents/', url)
        if url.startswith(api):
            return proof_reader()(url[len(api):])
        self.assertTrue(url.startswith(raw), url)
        name = url[len(raw):]
        self.assertIn(name, {r['path'] for r in self.manifest['files']})
        return (CORPUS / name).read_bytes()

    def capture(self, change=None, profile=PROFILE):
        rows = {'promise-methods-' + r['name']: r for r in self.common}
        rows.update({'promise-all-' + r['name']: r for r in self.all_oracle['controls']})
        rows.update({'promise-race-' + r['name']: r for r in self.oracle['controls']})
        captured = []
        def observe(case, *args):
            captured.append(copy.deepcopy(case))
            held = rows[case['id'].removeprefix('harness-preflight:')]
            result = legacy.selected_observation(held)
            if change:
                result = change(case, held, result)
            return dict(id=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                        source_sha256=digest(case['source']), **result)
        with patch.object(runner, 'run_case', side_effect=observe):
            outcomes = runner.harness_preflight(self.files(profile), Path('/inert'), 3.0, profile)
        return captured, outcomes

    def test_complete_94_selection_and_original_blob_proof_bindings(self):
        self.assertEqual(importer.PROFILES[PROFILE], {'Promise/race': 94})
        self.assertEqual(importer.PROFILE_ROOTS[PROFILE], 'test/built-ins')
        self.assertIn(PROFILE, importer.TREE_PROFILES)
        self.assertEqual(importer.corpus_name(PROFILE), 'test262-promise-race')
        data = (CORPUS / 'manifest.json').read_bytes()
        self.assertEqual(digest(data), self.oracle['manifest_sha256'])
        self.assertEqual(json.loads(data), self.manifest)
        self.assertEqual((self.manifest['test_files'], len(self.manifest['files']),
                          len(self.manifest['inventory_proof']['files'])), (94, 103, 7))
        self.assertEqual(sum(r['bytes'] for r in self.manifest['inventory_proof']['files']), 66506)
        for row in self.manifest['files'] + self.manifest['inventory_proof']['files']:
            actual = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(actual), digest(actual)), (row['bytes'], row['sha256']), row['path'])
        listings, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        self.assertEqual({k: sorted(r['name'] for r in v) for k, v in listings.items()}, self.manifest['directories'])
        self.assertEqual(description, {k: v for k, v in self.manifest['inventory_proof'].items() if k != 'files'})
        self.assertEqual(set(proof), {r['path'] for r in self.manifest['inventory_proof']['files']})

    def test_all_raw_metadata_modes_and_requested_helpers_match_independent_census(self):
        helpers = {'assert.js', 'sta.js'}
        async_count = total_bytes = 0
        for path, expected in self.oracle['metadata'].items():
            data = (CORPUS / path).read_bytes()
            actual = importer.parse_metadata(data.decode())
            self.assertEqual(actual, expected, path)
            self.assertNotIn('timeout', actual)
            self.assertIsNone(actual['negative'])
            self.assertFalse(set(actual['flags']) - {'async'})
            self.assertEqual(runner.modes(actual), ['sloppy', 'strict'])
            async_count += 'async' in actual['flags']
            total_bytes += len(data)
            helpers.update(actual['includes'])
            if 'async' in actual['flags']:
                helpers.add('doneprintHandle.js')
        self.assertEqual((len(self.oracle['metadata']), async_count, total_bytes), (94, 63, 102171))
        self.assertEqual(helpers, {'assert.js', 'sta.js', 'doneprintHandle.js', 'compareArray.js',
                                   'isConstructor.js', 'promiseHelper.js', 'propertyHelper.js'})
        self.assertEqual(len(self.oracle['class_sources']), 2)

    def test_complete_tree_and_every_ancestor_proof_refuse_mutation(self):
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
                importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + RACE_TREE, edit))
        for selected in [{}, {'Promise/race': 93}, {'Promise/race': 94, 'Promise/all': 98}]:
            with patch.dict(importer.PROFILES, {PROFILE: selected}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_is_exact_and_rejects_changed_source_helper_or_legal_blob(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch), contextlib.redirect_stdout(io.StringIO()):
            destination = Path(temporary)
            importer.import_corpus(destination, PROFILE)
            self.assertEqual((destination / 'manifest.json').read_bytes(), (CORPUS / 'manifest.json').read_bytes())
            for row in self.manifest['files'] + self.manifest['inventory_proof']['files']:
                self.assertEqual((destination / row['path']).read_bytes(), (CORPUS / row['path']).read_bytes())
        names = ['test/built-ins/Promise/race/name.js'] + sorted(self.oracle['auxiliary_files'])
        for name in names:
            def altered(url):
                original = self.cached_fetch(url)
                return original + b'\n' if url.endswith('/' + name) else original
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=altered), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_loader_keeps_188_exact_identities_ordered_harness_and_integer_policies(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(files), len(cases), fixtures), (94, 103, 188, []))
        self.assertEqual(identities(cases), self.oracle['case_identities'])
        self.assertEqual(digest(canonical(identities(cases))), self.oracle['case_identities_sha256'])
        self.assertEqual([sum(c['mode'] == m for c in cases) for m in ('sloppy', 'strict')], [94, 94])
        self.assertEqual(sum(c['completion_policy'] for c in cases), 126)
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], set(self.oracle['features']))
        for case in cases:
            self.assertIs(type(case['completion_policy']), int)
            self.assertNotIn('timeout', case['metadata'])
            self.assertEqual(runner.encode_request(case)[:8], b'ERJS2' + bytes([case['mode'] == 'strict', 0, case['completion_policy']]))
            for bad in (True, False, 2):
                forged = dict(case, completion_policy=bad)
                with self.assertRaises(ValueError):
                    runner.encode_request(forged)
                self.assertIsNotNone(runner.unsupported_reason(forged, set(self.oracle['features']), PROFILE))
        excluded = [c['id'] for c in cases if runner.unsupported_reason(c, set(self.oracle['features']), PROFILE)]
        self.assertEqual(excluded, [p + ':' + m for p in self.oracle['class_sources'] for m in ('sloppy', 'strict')])

    def test_rehashed_loader_sources_helpers_and_legal_bytes_still_require_root_blobs(self):
        names = ['test/built-ins/Promise/race/name.js'] + sorted(self.oracle['auxiliary_files'])
        for name in names:
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, destination)
                data = (destination / name).read_bytes() + b'\n'
                (destination / name).write_bytes(data)
                manifest = copy.deepcopy(self.manifest)
                next(r for r in manifest['files'] if r['path'] == name).update(bytes=len(data), sha256=digest(data))
                (destination / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(destination, PROFILE)

    def test_deleted_or_unbound_sources_helpers_and_declared_proofs_are_rejected(self):
        sample = 'test/built-ins/Promise/race/name.js'
        proof = 'inventory-proof/tree-' + RACE_TREE + '.json'
        for name in (sample, 'harness/compareArray.js', proof):
            for drop_entry in (False, True):
                with self.subTest(deleted=name, drop_entry=drop_entry), tempfile.TemporaryDirectory() as temporary:
                    destination = Path(temporary) / 'corpus'
                    shutil.copytree(CORPUS, destination)
                    (destination / name).unlink()
                    manifest = copy.deepcopy(self.manifest)
                    if drop_entry:
                        if name.startswith('inventory-proof/'):
                            manifest['inventory_proof']['files'] = [r for r in manifest['inventory_proof']['files'] if r['path'] != name]
                        else:
                            manifest['files'] = [r for r in manifest['files'] if r['path'] != name]
                            if name == sample:
                                manifest['directories']['Promise/race'].remove('name.js')
                                manifest['test_files'] -= 1
                    (destination / 'manifest.json').write_text(json.dumps(manifest))
                    with self.assertRaises((ValueError, OSError)):
                        runner.load_corpus(destination, PROFILE)
        # Inert unlisted JSON is not execution input. The proof witness is listed
        # and locally checksum-valid, but absent from the consumed root proof set.
        additions = [(sample.replace('name.js', 'unbound.js'), False),
                     ('harness/unbound.js', False), ('harness/unbound.js', True),
                     ('inventory-proof/unbound.json', True)]
        for name, listed in additions:
            with self.subTest(extra=name, listed=listed), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, destination)
                data = b'{}' if name.endswith('.json') else b'0;\n'
                (destination / name).write_bytes(data)
                manifest = copy.deepcopy(self.manifest)
                if listed:
                    row = dict(path=name, bytes=len(data), sha256=digest(data))
                    if name.endswith('.json'):
                        row['url'] = 'https://api.github.com/repos/tc39/test262/git/trees/' + '0' * 40
                        manifest['inventory_proof']['files'].append(row)
                    else:
                        row['upstream_path'] = name
                        manifest['files'].append(row)
                (destination / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(destination, PROFILE)

    def test_exact_90_preflights_keep_literals_identity_and_same_mode_dependencies(self):
        self.assertEqual(runner.promise_race_preflight_variants(), self.oracle['controls'])
        captured, results = self.capture()
        common_cases, common_results = self.capture(profile='promise-methods-async')
        self.assertEqual((len(captured), len(results)), (90, 90))
        self.assertEqual(captured[:74], common_cases)
        self.assertEqual(results[:74], common_results)
        by_identity = {(r['name'], r['result']['mode']): r for r in results}
        expected = [(mode, row) for mode in ('sloppy', 'strict') for row in self.oracle['controls']]
        for case, result, held, (mode, row) in zip(captured[74:], results[74:], self.oracle['preflight'], expected):
            name = 'promise-race-' + row['name']
            self.assertEqual((case['id'], case['mode'], case['source']), ('harness-preflight:' + name, mode, row['source'].encode()))
            self.assertEqual(case['case_sha256'], held['case_sha256'])
            self.assertEqual(digest(case['source']), held['source_sha256'])
            names = ['assert.js', 'sta.js'] + (['doneprintHandle.js'] if row['completion_policy'] else []) + row['includes']
            self.assertEqual(case['harness'], [(n, self.files()['harness/' + n]) for n in names])
            self.assertEqual(case['completion_policy'], row['completion_policy'])
            self.assertEqual((result['name'], result['expected'], result['verified']), (name, row['expected'], True))
            required = [('promise-methods-' + old['name'], mode) for old in self.common]
            if row['prerequisite']:
                required.append(('promise-race-' + row['prerequisite'], mode))
            self.assertEqual(result['prerequisites'], [dict(name=n, mode=m, verified=True,
                case_sha256=by_identity[(n, m)]['result']['case_sha256'],
                source_sha256=by_identity[(n, m)]['result']['source_sha256']) for n, m in required])
        self.assertTrue(runner.promise_race_preflight_healthy(results))

    def test_pairs_require_their_positive_common_health_and_authenticated_error_evidence(self):
        for positive in [r for r in self.oracle['controls'] if r['prerequisite'] is None]:
            def break_positive(case, row, result):
                return dict(status='failed') if case['mode'] == 'strict' and case['id'] == 'harness-preflight:promise-race-' + positive['name'] else result
            _, results = self.capture(break_positive)
            rows = {(r['name'], r['result']['mode']): r for r in results}
            name = 'promise-race-' + positive['name']
            self.assertFalse(rows[(name, 'strict')]['verified'])
            self.assertFalse(rows[(name + '-mismatch', 'strict')]['verified'])
            self.assertTrue(rows[(name + '-mismatch', 'sloppy')]['verified'])
        def break_host(case, row, result):
            return dict(status='failed') if case['mode'] == 'strict' and case['id'] == 'harness-preflight:promise-methods-one-async-completion' else result
        _, results = self.capture(break_host)
        self.assertTrue(all(not r['verified'] for r in results[74:] if r['result']['mode'] == 'strict'))
        self.assertTrue(all(r['verified'] for r in results[74:] if r['result']['mode'] == 'sloppy'))
        for fake in [dict(status='failed'), dict(status='passed'), dict(status='resource')]:
            _, results = self.capture(lambda c, r, good: copy.deepcopy(fake) if 'promise-race-' in c['id'] else good)
            self.assertFalse(any(r['verified'] for r in results[74:]))
        # Keep positives healthy: isolate the negative verifier, not dependency failure.
        for mutation in ('wrong-error', 'unauthenticated', 'fatal'):
            def corrupt(case, row, result):
                if 'promise-race-' not in case['id'] or not row['prerequisite']:
                    return result
                value = copy.deepcopy(result)
                observation = value['observation']
                if mutation == 'fatal':
                    return legacy.selected_observation(dict(row, expected='resource'))
                if mutation == 'wrong-error' and row['completion_policy'] == 0:
                    observation['error_type'] = 'TypeError'
                elif mutation == 'unauthenticated' and row['completion_policy'] == 1:
                    observation['completion'].update(done_calls=0, failure_prints=0, flags=16)
                    observation['message'] = 'unauthenticated-completion'
                elif row['completion_policy'] == 0:
                    observation['error_type'] = 'TypeError'
                else:
                    observation['completion'].update(done_calls=0, success_prints=0, failure_prints=0, flags=0)
                    observation['message'] = 'missing-completion'
                return value
            _, results = self.capture(corrupt)
            self.assertTrue(all(r['verified'] for r in results[74:] if not r['name'].endswith('-mismatch')))
            self.assertFalse(any(r['verified'] for r in results[74:] if r['name'].endswith('-mismatch')))

    def test_bad_gate_blocks_all_188_without_case_execution_or_baseline(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, healthy = self.capture()
        duplicate, wrong_mode, wrong_name, unverified = [copy.deepcopy(healthy) for _ in range(4)]
        duplicate[-1] = copy.deepcopy(duplicate[0])
        wrong_mode[-1]['result']['mode'] = 'raw'
        wrong_name[-1]['name'] = 'promise-race-forged'
        unverified[0]['verified'] = False
        for bad in ([], healthy[:74], healthy[:-1], duplicate, healthy + healthy[:1], wrong_mode, wrong_name, unverified):
            self.assertFalse(runner.promise_race_preflight_healthy(bad))
            code, report, recorded, calls = legacy.PromiseRunnerTests().report(PROFILE, bad, cases)
            self.assertEqual((code, recorded, calls), (1, False, []))
            self.assertEqual((report['mode_cases'], report['counts']), (188, {'blocked': 188}))
            self.assertEqual({r['id'] for r in report['cases']}, {c['id'] for c in cases})
            self.assertTrue(all(r['reason'] == 'Promise host preflight failed' for r in report['cases']))
            self.assertEqual((report['runner_policy'], report['runner_policy_sha256']), (self.oracle['policy'], self.oracle['policy_sha256']))

    def test_missing_host_capability_or_legacy_complete_cannot_admit_the_profile(self):
        # Complete and valid ERJR2, deliberately lacking authenticated evidence.
        frame = bytearray(b'ERJR2')
        for text in ('complete', 'runtime', '', '', '', ''):
            data = text.encode()
            frame.extend(struct.pack('<I', len(data)))
            frame.extend(data)
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for response, expected in [((2, b'', b'test262-host disabled'), 'adapter-error'),
                                   ((0, bytes(frame), b''), 'failed')]:
            with self.subTest(expected=expected), patch.object(runner, 'bounded_process', return_value=response) as process:
                results = runner.harness_preflight(self.files(), Path('/inert'), 3.0, PROFILE)
            self.assertEqual(len(results), 90)
            self.assertTrue(all(r['result']['status'] == expected and not r['verified'] for r in results))
            self.assertEqual(process.call_count, 90)
            self.assertTrue(all(call.args[1].startswith(b'ERJS2') for call in process.call_args_list))
            code, report, recorded, calls = legacy.PromiseRunnerTests().report(PROFILE, results, cases)
            self.assertEqual((code, recorded, calls, report['counts']), (1, False, [], {'blocked': 188}))

    def test_synthetic_reporting_retains_host_veto_and_refuses_bad_run_baselines(self):
        # These are mock observations, not predictions for any upstream body.
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, preflight = self.capture()
        def outcome(case):
            reason = runner.unsupported_reason(case, set(self.oracle['features']), PROFILE)
            if reason:
                return dict(status='unsupported', reason=reason)
            return legacy.selected_observation(dict(name='mock-unhandled', completion_policy=case['completion_policy'], expected='unhandled-rejection'))
        code, report, recorded, calls = legacy.PromiseRunnerTests().report(PROFILE, preflight, cases, outcome)
        self.assertEqual((code, recorded, len(calls)), (0, True, 188))
        self.assertEqual(report['counts'], {'failed': 184, 'unsupported': 4})
        self.assertFalse(report['full_test262_conformance'])
        self.assertEqual((report['runner_policy'], report['runner_policy_sha256']), (self.oracle['policy'], self.oracle['policy_sha256']))
        self.assertTrue(all(r['observation']['status'] == 'host-failure' and r['observation']['message'] == 'unhandled-rejection'
                            for r in report['cases'] if r['status'] == 'failed'))
        for status in ('resource', 'timeout', 'adapter-error'):
            def unhealthy(case):
                return dict(status=status, reason='injected mock failure') if case['id'] == cases[0]['id'] else outcome(case)
            code, report, recorded, calls = legacy.PromiseRunnerTests().report(PROFILE, preflight, cases, unhealthy)
            self.assertEqual((code, recorded, len(calls), report['counts'][status]), (1, False, 188, 1))

    def test_existing_55_tables_and_both_promise_profiles_remain_exact(self):
        # Parent-compatible control: no lookup of the new profile occurs here.
        tables = dict(profiles={k: v for k, v in importer.PROFILES.items() if k not in {PROFILE, 'promise-all-settled', 'aggregate-error', 'promise-any', 'promise-with-resolvers', 'promise-try'}},
                      roots={k: v for k, v in importer.PROFILE_ROOTS.items() if k not in {PROFILE, 'promise-all-settled', 'aggregate-error', 'promise-any', 'promise-with-resolvers', 'promise-try'}},
                      tree_profiles=sorted(importer.TREE_PROFILES - {PROFILE, 'promise-all-settled', 'aggregate-error', 'promise-any', 'promise-with-resolvers', 'promise-try'}))
        self.assertEqual(tables, self.oracle['old_tables'])
        self.assertEqual((len(tables['profiles']), digest(canonical(tables))), (55, self.oracle['old_tables_sha256']))
        self.assertEqual(runner.promise_preflight_variants(), self.common)
        self.assertEqual(runner.promise_all_preflight_variants(), self.all_oracle['controls'])
        for profile in ('promise-methods-async', 'promise-all'):
            corpus = importer.ROOT / 'tests/upstream' / ('test262-' + profile)
            old = self.all_oracle['old_profile'] if profile == 'promise-methods-async' else self.all_oracle
            expected_manifest = old['manifest'] if profile == 'promise-methods-async' else old['expected_manifest']
            self.assertEqual(digest((corpus / 'manifest.json').read_bytes()), self.oracle['old_manifest_hashes'][profile])
            self.assertEqual(json.loads((corpus / 'manifest.json').read_bytes()), expected_manifest)
            if profile == 'promise-methods-async':
                self.assertEqual(digest((corpus / 'manifest.json').read_bytes()), old['manifest_sha256'])
            manifest, _, cases, fixtures, _ = runner.load_corpus(corpus, profile)
            self.assertEqual(identities(cases), old['case_identities'])
            self.assertTrue(all('timeout' not in c['metadata'] for c in cases))
            self.assertEqual(fixtures, [])
            expected_counts = (232, 458, 74) if profile == 'promise-methods-async' else (98, 196, 90)
            _, preflight = self.capture(profile=profile)
            self.assertEqual((manifest['test_files'], len(cases), len(preflight)), expected_counts)
            code, report, recorded, _ = legacy.PromiseRunnerTests().report(profile, preflight)
            self.assertEqual((code, recorded), (0, True))
            self.assertEqual((report['runner_policy'], report['runner_policy_sha256']), (old['policy'], old['policy_sha256']))


if __name__ == '__main__':
    unittest.main()
