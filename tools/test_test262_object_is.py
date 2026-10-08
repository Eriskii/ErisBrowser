"""Complete authenticated Object.is selection and independent guarded controls."""
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

PROFILE = 'object-is'
CORPUS = runner.ROOT / 'tests/upstream/test262-object-is'
TREE = '04a0f3e25947dbc1100ba4b4138765366d9704b4'


def proof_reader(altered_route=None, mutate=None):
    def read(route):
        raw = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(raw)
            mutate(value)
            return json.dumps(value).encode()
        return raw
    return read


def capture_controls(fake=None):
    _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
    captured = []
    def execute(case, *args):
        captured.append(case)
        result = fake(case) if fake else dict(status='passed')
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)
    with patch.object(runner, 'run_case', side_effect=execute), patch.object(
            runner, 'bounded_process', side_effect=AssertionError('adapter execution forbidden')):
        outcomes = runner.harness_preflight(files, Path('/capture-only'), 3, PROFILE)
    return captured, outcomes


class ObjectIsCorpusTests(unittest.TestCase):
    def test_complete_authenticated_inventory_and_fixed_admission(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline proof replay fetched')):
            manifest, files, cases, fixtures, digest = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (21, 42, []))
        self.assertEqual({name: len(items) for name, items in manifest['directories'].items()}, {'Object/is': 21})
        self.assertEqual(digest, 'cc2d48f90c4001378ddde2a352e12ee936284768a7432a6918cf46770237ed59')
        self.assertEqual(sum(len(raw) for name, raw in files.items() if name.startswith('test/')), 16515)
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [21, 21])
        self.assertTrue(all(c['metadata']['negative'] is None and c['metadata']['flags'] == [] for c in cases))
        self.assertTrue(all(runner.unsupported_reason(c, runner.OBJECT_IS_FEATURES) is None for c in cases))
        self.assertEqual(runner.digest(canonical([(c['id'], c['case_sha256']) for c in cases])),
                         'bb98364490dc817b2f3f99805b44b32a3ba954d3c175c073cdea4d75bcd5b09b')
        self.assertEqual({name[8:] for name in files if name.startswith('harness/')},
                         {'assert.js', 'sta.js', 'propertyHelper.js', 'isConstructor.js'})
        self.assertEqual(manifest['inventory_proof']['subtree_trees'], {'Object/is': TREE})
        self.assertEqual(manifest['inventory_proof']['directory_trees'], {'Object/is': TREE})
        # Preserve the existing runner's order; an early scratch admission draft
        # listed sta/assert in reverse, before any official contract or run.
        self.assertTrue(all([n for n, _ in c['harness']][:2] == ['assert.js', 'sta.js'] for c in cases))

    def test_new_feature_is_profile_scoped_and_43_old_contracts_are_exact(self):
        self.assertEqual(runner.OBJECT_IS_FEATURES, runner.OBJECT_INTEGRITY_FEATURES | {'Object.is'})
        self.assertFalse(runner.OBJECT_IS_FEATURES & {'Proxy', 'BigInt', 'cross-realm', 'SharedArrayBuffer'})
        for name, features in runner.PROFILE_FEATURES.items():
            if name != PROFILE:
                self.assertNotIn('Object.is', features)
        contract = capture_contracts(excluded={'typedarray-to-reversed', 'typedarray-reverse', 'typedarray-fill', 'typedarray-search', 'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', PROFILE})
        self.assertEqual(contract['counts'], dict(profiles=43, cases=19685, preflights=4484))
        self.assertEqual(runner.digest(canonical(contract)),
                         '2555833d1cd7a66a19d02ed295f404060b72472d7af8fbd008517285be5d6cd6')

    def test_exact_reviewed_control_sources_names_modes_and_guards(self):
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (80, 80))
        identities = [(o['name'], c['mode'], runner.digest(c['source']), o['expected'])
                      for c, o in zip(captured[32:], outcomes[32:])]
        self.assertEqual(runner.digest(canonical(identities)),
                         '9beb39c566b2071095e610f6f72b649b234105c3677368d804c614d89adc2af9')
        self.assertEqual(len({(c['id'], c['mode']) for c in captured}), 80)
        by_key = {(o['name'], c['mode']): (c, o) for c, o in zip(captured[32:], outcomes[32:])}
        for (name, mode), (case, outcome) in by_key.items():
            source = case['source']
            self.assertTrue(source.startswith(b"assert.sameValue(typeof Object.is,'function');"))
            self.assertIn(b'assert.sameValue(Object.is(NaN,NaN),true);', source)
            self.assertIn(b'assert.sameValue(Object.is(0,-0),false);', source)
            if name.endswith('-wrong'):
                good, positive = by_key[(name.removesuffix('-wrong') + '-positive', mode)]
                self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                                 source.rsplit(b'assert.sameValue(', 1)[0])
                self.assertNotEqual(good['source'], source)
                self.assertEqual(outcome['prerequisite'], dict(name=positive['name'], mode=mode,
                    case_sha256=good['case_sha256'], source_sha256=runner.digest(good['source']),
                    expected='passed', verified=True))
            if '-native-alias-' in name:
                self.assertIn(b"assert.sameValue(typeof parseInt,'function');", source)
                self.assertIn(b"assert.sameValue(typeof Number.parseInt,'function');", source)
            if '-utf16-' in name:
                self.assertIn(b"var a='\\uD800',b='\\uDC00';", source)
                self.assertNotIn(b"'\\\\uD800'", source)

    def test_common_controls_unchanged_and_incidental_errors_do_not_verify_pairs(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            current = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            common = runner.harness_preflight(files, Path('/fake'), 1)
        self.assertEqual(current[:32], common)
        self.assertEqual(sum(o['verified'] for o in current[32:]), 24)
        self.assertFalse(all(o['verified'] for o in current))
        for kind in ('Test262Error', 'TypeError', 'ReferenceError', 'SyntaxError', 'RangeError'):
            with self.subTest(error=kind), patch.object(runner, 'bounded_process',
                    return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(o['verified'] for o in actual[32:]))
            self.assertTrue(all(o['result']['observation']['error_type'] == kind for o in actual[32:]))

    def test_health_requires_same_mode_positive_and_actual_wrong_assertion(self):
        def fake(case):
            if case['id'].endswith('-wrong') or case['mode'] == 'strict':
                return dict(status='failed', observation={'error_type': 'Test262Error'})
            return dict(status='passed')
        _, outcomes = capture_controls(fake)
        for item in outcomes[32:]:
            self.assertEqual(item['verified'], item['result']['mode'] == 'sloppy')
            if item['expected'] == 'failed':
                self.assertEqual(item['prerequisite']['mode'], item['result']['mode'])
        def wrong_error(case):
            return (dict(status='failed', observation={'error_type': 'TypeError'})
                    if case['id'].endswith('-wrong') else dict(status='passed'))
        _, outcomes = capture_controls(wrong_error)
        self.assertTrue(all(not o['verified'] for o in outcomes[32:] if o['expected'] == 'failed'))
        for status in ('resource', 'timeout', 'adapter-error', 'unsupported'):
            _, outcomes = capture_controls(lambda case: dict(status=status))
            self.assertFalse(any(o['verified'] for o in outcomes[32:]))
            self.assertTrue(all(o['result']['status'] == status for o in outcomes[32:]))

    def test_recursive_proof_rejects_incomplete_substituted_or_unbounded_inventory(self):
        route = f'trees/{TREE}?recursive=1'
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'omitted': lambda d: d['tree'].pop(),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'blob': lambda d: d['tree'][0].update(sha='0' * 40),
            'nonregular': lambda d: d['tree'][0].update(mode='120000'),
            'unhashable-mode': lambda d: d['tree'][0].update(mode=[]),
            'parent-path': lambda d: d['tree'][0].update(path='../outside.js'),
            'orphan': lambda d: d['tree'][0].update(path='missing/source.js'),
            'entry-limit': lambda d: d.update(tree=[d['tree'][0]] * 4097),
            'boolean-size': lambda d: d['tree'][0].update(size=True),
            'source-size': lambda d: d['tree'][0].update(size=importer.MAX_FILE + 1),
            'aggregate-size': lambda d: [e.update(size=importer.MAX_FILE) for e in d['tree']],
        }
        for name, edit in edits.items():
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, edit))
        for directories in ({}, {'Object/is': 20}, {'Object/is': 21, 'Object/is/extra': 0}):
            with patch.dict(importer.PROFILES, {PROFILE: directories}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_ancestor_and_auxiliary_proofs_remain_root_linked(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(proof=entry['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_roundtrip_and_changed_source_helper_legal_rejection(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        def fetch(url):
            if url.startswith(api):
                return proof_reader()(url[len(api):])
            self.assertTrue(url.startswith(raw), url)
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), PROFILE)
            actual, retained, imported, _, _ = runner.load_corpus(Path(temporary), PROFILE)
            self.assertEqual(actual, manifest)
            self.assertEqual(retained, files)
            self.assertEqual([c['case_sha256'] for c in imported], [c['case_sha256'] for c in cases])
            for entry in manifest['inventory_proof']['files']:
                self.assertEqual((Path(temporary) / entry['path']).read_bytes(), (CORPUS / entry['path']).read_bytes())
        for suffix in ('test/built-ins/Object/is/length.js', 'harness/assert.js', 'LICENSE'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(source=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_changed_or_unlisted_bytes(self):
        for name in ('test/built-ins/Object/is/length.js', 'harness/assert.js', 'harness/isConstructor.js',
                     'LICENSE', 'INTERPRETING.md', f'inventory-proof/recursive-{TREE}.json'):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                target = dest / name
                if name.startswith('inventory-proof/'):
                    value = json.loads(target.read_bytes()); value['tree'].pop(); data = json.dumps(value).encode()
                else:
                    data = target.read_bytes() + b'\n'
                target.write_bytes(data)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                entries = manifest['inventory_proof']['files'] if name.startswith('inventory-proof/') else manifest['files']
                next(e for e in entries if e['path'] == name).update(bytes=len(data), sha256=runner.digest(data))
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)
        for mutation in ('extra-helper', 'extra-proof', 'helper-symlink', 'proof-symlink'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                if mutation == 'extra-helper':
                    name = 'harness/unpinned.js'; data = b'// unpinned\n'; (dest / name).write_bytes(data)
                    manifest['files'].append(dict(path=name, upstream_path=name, bytes=len(data), sha256=runner.digest(data)))
                elif mutation == 'extra-proof':
                    manifest['inventory_proof']['files'].append(dict(path='inventory-proof/unrequested.json', bytes=2, sha256=runner.digest(b'{}'), url='unrequested'))
                else:
                    name = 'harness/assert.js' if mutation == 'helper-symlink' else manifest['inventory_proof']['files'][0]['path']
                    target = dest / name; target.unlink(); target.symlink_to(CORPUS / name)
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)
