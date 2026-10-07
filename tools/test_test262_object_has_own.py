"""Pinned Object.hasOwn corpus, original proofs and guarded control health."""
import contextlib
import copy
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner
from test_test262_conformance import response
from test_test262_object_integrity import canonical, capture_contracts

PROFILE = 'object-has-own'
CORPUS = runner.ROOT / 'tests/upstream/test262-object-has-own'
TREE = 'c415403752259885fd6cfda1999381beef67ee70'
MANIFEST = 'ec06ac5c6d9df81904821234389ed31d6638d7e2421df43345818ca8a12f05b9'
CASE_ROSTER = '0b8bf07ff2d7c961d529585a5bd8480013bf842a6a892f19e0a4a17514a14a8e'
CONTROL_IDENTITIES = 'cf7cd2dfca216ebfe40d3e0e97aafa8d2bf257d35c3b1e30546265b79e2a1187'
OLD_CONTRACTS = '5124e0e1d79ca93a46dab73218c6ec2188ad8c7a2ae3279936722afff4f71786'
FEATURES = {'Object.hasOwn', 'Symbol', 'Symbol.toPrimitive', 'Reflect.construct', 'arrow-function'}
PAIR_NAMES = ['own-inherited', 'accessor-presence', 'target-before-key', 'symbol-exotic',
              'symbol-to-string', 'symbol-value-of', 'property-metadata', 'nonconstructor']


def proof_reader(altered_route=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(data)
            mutate(value)
            return json.dumps(value).encode()
        return data
    return read


def capture_controls(fake=None):
    _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
    captured = []

    def execute(case, *args):
        captured.append(case)
        result = fake(case) if fake else {'status': 'passed'}
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)

    with patch.object(runner, 'run_case', side_effect=execute):
        outcomes = runner.harness_preflight(files, Path('/capture-only-no-adapter'), 3, PROFILE)
    return captured, outcomes


class ObjectHasOwnCorpusTests(unittest.TestCase):
    def setUp(self):
        # Every test is inert. Nested mocks supply exact retained bytes or
        # fabricated protocol responses; no real adapter/network fallback exists.
        adapter = patch.object(runner, 'bounded_process',
                               side_effect=AssertionError('adapter execution forbidden'))
        network = patch.object(importer, 'fetch',
                               side_effect=AssertionError('network access forbidden'))
        adapter.start()
        network.start()
        self.addCleanup(adapter.stop)
        self.addCleanup(network.stop)

    def test_complete_pinned_inventory_and_literal_case_identities(self):
        manifest, files, cases, fixtures, digest = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (62, 124, []))
        self.assertEqual(manifest['revision'], '7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd')
        self.assertEqual({name: len(rows) for name, rows in manifest['directories'].items()},
                         {'Object/hasOwn': 62})
        self.assertEqual(digest, MANIFEST)
        self.assertEqual(sum(len(raw) for name, raw in files.items() if name.startswith('test/')), 33881)
        self.assertEqual(len(files), 68)
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [62, 62])
        self.assertTrue(all(c['metadata']['flags'] == [] and c['metadata']['negative'] is None
                            and c['metadata']['locale'] == [] for c in cases))
        self.assertEqual({f for c in cases for f in c['metadata']['features']}, FEATURES)
        self.assertTrue(all(runner.unsupported_reason(c, FEATURES) is None for c in cases))
        self.assertEqual(runner.digest(canonical([(c['id'], c['case_sha256']) for c in cases])), CASE_ROSTER)
        self.assertEqual({p[8:] for p in files if p.startswith('harness/')},
                         {'assert.js', 'sta.js', 'propertyHelper.js', 'isConstructor.js'})
        proof = manifest['inventory_proof']
        self.assertEqual(proof['subtree_trees'], {'Object/hasOwn': TREE})
        self.assertEqual(proof['directory_trees'], {'Object/hasOwn': TREE})
        self.assertEqual(len(proof['files']), 7)
        recursive = CORPUS / f'inventory-proof/recursive-{TREE}.json'
        self.assertEqual(runner.digest(recursive.read_bytes()),
                         'd12cf03ee0caf86e00bf64d53fe3c86525e1e8ea94a9fea9874315c7fcea2193')
        for case in cases:
            self.assertEqual([name for name, _ in case['harness']],
                             ['assert.js', 'sta.js'] + case['metadata']['includes'])

    def test_profile_scoped_feature_and_all44_predecessor_contracts(self):
        self.assertEqual(runner.OBJECT_HAS_OWN_FEATURES, FEATURES)
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES)
        self.assertNotIn('Object.hasOwn', runner.SUPPORTED_FEATURES)
        self.assertEqual(len(set(runner.PROFILES) - {'typedarray-foundation'}), 45)
        for name, features in runner.PROFILE_FEATURES.items():
            if name != PROFILE:
                self.assertNotIn('Object.hasOwn', features)
        contract = capture_contracts(excluded={'typedarray-foundation', PROFILE})
        self.assertEqual(contract['counts'], {'profiles': 44, 'cases': 19727, 'preflights': 4564})
        self.assertEqual(runner.digest(canonical(contract)), OLD_CONTRACTS)

    def test_exact_control_bytes_order_guards_and_helper_rosters(self):
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (64, 64))
        self.assertEqual(len({(c['id'], c['mode']) for c in captured}), 64)
        feature = list(zip(captured[32:], outcomes[32:]))
        expected_order = [('object-has-own-' + name + '-' + variant, mode)
                          for name in PAIR_NAMES for variant in ('positive', 'wrong')
                          for mode in ('sloppy', 'strict')]
        self.assertEqual([(o['name'], c['mode']) for c, o in feature], expected_order)
        identities = [(o['name'], c['mode'], runner.digest(c['source']), o['expected'])
                      for c, o in feature]
        self.assertEqual(runner.digest(canonical(identities)), CONTROL_IDENTITIES)
        by_key = {(o['name'], c['mode']): (c, o) for c, o in feature}
        for case, outcome in feature:
            source = case['source']
            self.assertTrue(source.startswith(b"assert.sameValue(typeof Object.hasOwn,'function');"))
            for guard in (b"Object.hasOwn(ownGuardTarget,'own'),true",
                          b"Object.hasOwn(ownGuardTarget,'inherited'),false",
                          b"Object.hasOwn(ownGuardTarget,'missing'),false"):
                self.assertIn(guard, source)
            expected_helpers = ['assert.js', 'sta.js']
            if '-property-metadata-' in outcome['name']:
                expected_helpers.append('propertyHelper.js')
            self.assertEqual([name for name, _ in case['harness']], expected_helpers)
            if outcome['name'].endswith('-wrong'):
                name = outcome['name'].removesuffix('-wrong') + '-positive'
                positive_case, positive = by_key[(name, case['mode'])]
                self.assertEqual(source.rsplit(b'assert.sameValue(', 1)[0],
                                 positive_case['source'].rsplit(b'assert.sameValue(', 1)[0])
                self.assertNotEqual(source, positive_case['source'])
                self.assertEqual(outcome['prerequisite'], dict(name=name, mode=case['mode'],
                    case_sha256=positive_case['case_sha256'],
                    source_sha256=runner.digest(positive_case['source']), expected='passed', verified=True))
                self.assertFalse(outcome['verified'])  # fabricated completion cannot satisfy a wrong assertion
            if '-nonconstructor-' in outcome['name']:
                self.assertLess(source.index(b'Reflect.construct(Guard,[])'),
                                source.index(b'Reflect.construct(Object.hasOwn,[])'))
                self.assertIn(b'assert.sameValue(made.marker,7)', source)
                self.assertIn(b'assert.sameValue(Object.getPrototypeOf(made),Guard.prototype)', source)

    def test_common_controls_unchanged_and_incidental_exceptions_do_not_verify(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            current = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            common = runner.harness_preflight(files, Path('/fake'), 1)
        self.assertEqual(current[:32], common)
        self.assertEqual(sum(o['verified'] for o in current[32:]), 16)
        for error in ('Test262Error', 'TypeError', 'ReferenceError', 'SyntaxError', 'RangeError'):
            with self.subTest(error=error), patch.object(runner, 'bounded_process',
                    return_value=(0, response('exception', 'runtime', error), b'')):
                outcomes = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(o['verified'] for o in outcomes[32:]))
            self.assertTrue(all(o['result']['observation']['error_type'] == error for o in outcomes[32:]))

    def test_negative_health_requires_same_mode_positive_and_actual_wrong_error(self):
        def same_mode(case):
            if case['id'].endswith('-wrong') or case['mode'] == 'strict':
                return {'status': 'failed', 'observation': {'error_type': 'Test262Error'}}
            return {'status': 'passed'}
        _, outcomes = capture_controls(same_mode)
        for item in outcomes[32:]:
            self.assertEqual(item['verified'], item['result']['mode'] == 'sloppy')
            if item['expected'] == 'failed':
                self.assertEqual(item['prerequisite']['mode'], item['result']['mode'])
        for observation in ({}, {'error_type': 'TypeError'}, {'error_type': 'ReferenceError'}):
            def wrong_error(case):
                return ({'status': 'failed', 'observation': observation}
                        if case['id'].endswith('-wrong') else {'status': 'passed'})
            _, outcomes = capture_controls(wrong_error)
            self.assertTrue(all(not o['verified'] for o in outcomes[32:] if o['expected'] == 'failed'))
        for status in ('resource', 'timeout', 'adapter-error', 'unsupported', 'harness-error'):
            _, outcomes = capture_controls(lambda case: {'status': status})
            self.assertFalse(any(o['verified'] for o in outcomes[32:]))
            self.assertTrue(all(o['result']['status'] == status for o in outcomes[32:]))

    def test_missing_malformed_and_exited_adapter_responses_remain_unhealthy(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for returned in ((0, b'', b''), (0, b'ERJR2', b''), (0, b'null', b''),
                         (7, response('complete'), b'failed adapter')):
            with self.subTest(response=returned), patch.object(runner, 'bounded_process', return_value=returned):
                outcomes = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertEqual(len(outcomes), 64)
            self.assertTrue(all(not o['verified'] and o['result']['status'] == 'adapter-error'
                                for o in outcomes))

    def test_recursive_proof_rejects_incomplete_substituted_and_unsafe_entries(self):
        route = f'trees/{TREE}?recursive=1'
        mutations = {
            'truncated': lambda d: d.update(truncated=True),
            'omitted': lambda d: d['tree'].pop(),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'wrong-blob': lambda d: d['tree'][0].update(sha='0' * 40),
            'symlink-mode': lambda d: d['tree'][0].update(mode='120000'),
            'unhashable-mode': lambda d: d['tree'][0].update(mode=[]),
            'parent-path': lambda d: d['tree'][0].update(path='../outside.js'),
            'absolute-path': lambda d: d['tree'][0].update(path='/outside.js'),
            'orphan': lambda d: d['tree'][0].update(path='missing/source.js'),
            'entry-limit': lambda d: d.update(tree=[d['tree'][0]] * 4097),
            'boolean-size': lambda d: d['tree'][0].update(size=True),
            'source-size': lambda d: d['tree'][0].update(size=importer.MAX_FILE + 1),
            'aggregate-size': lambda d: [row.update(size=importer.MAX_FILE) for row in d['tree']],
        }
        for name, mutate in mutations.items():
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, mutate))
        for directories in ({}, {'Object/hasOwn': 61}, {'Object/hasOwn': 62, 'Object/hasOwn/extra': 0}):
            with self.subTest(directories=directories), patch.dict(importer.PROFILES, {PROFILE: directories}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_ancestor_harness_and_legal_proofs_are_root_linked(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(proof=entry['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_roundtrip_and_raw_blob_corruption_rejection(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'

        def fetch(url):
            if url.startswith(api):
                return proof_reader()(url[len(api):])
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]

        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), PROFILE)
            actual, retained, imported, _, _ = runner.load_corpus(Path(temporary), PROFILE)
            self.assertEqual(actual, manifest)
            self.assertEqual(retained, files)
            self.assertEqual([(c['id'], c['case_sha256']) for c in imported],
                             [(c['id'], c['case_sha256']) for c in cases])
            for entry in manifest['inventory_proof']['files']:
                self.assertEqual((Path(temporary) / entry['path']).read_bytes(),
                                 (CORPUS / entry['path']).read_bytes())
        for suffix in ('test/built-ins/Object/hasOwn/length.js', 'harness/assert.js',
                       'harness/propertyHelper.js', 'harness/isConstructor.js', 'LICENSE', 'INTERPRETING.md'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(source=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_changed_extra_or_symlinked_inputs(self):
        for name in ('test/built-ins/Object/hasOwn/length.js', 'harness/assert.js',
                     'harness/propertyHelper.js', 'harness/isConstructor.js', 'LICENSE',
                     'INTERPRETING.md', f'inventory-proof/recursive-{TREE}.json'):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                target = dest / name
                if name.startswith('inventory-proof/'):
                    value = json.loads(target.read_bytes())
                    value['tree'].pop()
                    data = json.dumps(value).encode()
                else:
                    data = target.read_bytes() + b'\n'
                target.write_bytes(data)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                entries = manifest['inventory_proof']['files'] if name.startswith('inventory-proof/') else manifest['files']
                next(row for row in entries if row['path'] == name).update(bytes=len(data), sha256=runner.digest(data))
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)
        for mutation in ('extra-helper', 'extra-proof', 'unlisted-script', 'helper-symlink', 'proof-symlink'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                if mutation == 'extra-helper':
                    name = 'harness/unpinned.js'
                    data = b'// unpinned\n'
                    (dest / name).write_bytes(data)
                    manifest['files'].append(dict(path=name, upstream_path=name, bytes=len(data), sha256=runner.digest(data)))
                elif mutation == 'extra-proof':
                    manifest['inventory_proof']['files'].append(dict(path='inventory-proof/unrequested.json',
                        bytes=2, sha256=runner.digest(b'{}'), url='unrequested'))
                elif mutation == 'unlisted-script':
                    (dest / 'harness/unlisted.js').write_text('// unlisted\n')
                else:
                    name = 'harness/assert.js' if mutation == 'helper-symlink' else manifest['inventory_proof']['files'][0]['path']
                    target = dest / name
                    target.unlink()
                    target.symlink_to(CORPUS / name)
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)
