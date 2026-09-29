"""Complete pinned iteration selections and fail-closed control contracts."""
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


PROFILES = ('for-of', 'core-iterators')
CORPORA = {p: runner.ROOT / 'tests/upstream' / runner.corpus_name(p) for p in PROFILES}


def proof_reader(profile, altered_route=None, mutate=None):
    def read(route):
        raw = (CORPORA[profile] / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(raw)
            mutate(value)
            return json.dumps(value).encode()
        return raw
    return read


def capture_controls(profile, fake=None):
    _, files, _, _, _ = runner.load_corpus(CORPORA[profile], profile)
    captured = []
    def execute(case, *args):
        captured.append(case)
        result = (fake(case) if fake else dict(status='passed'))
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)
    with patch.object(runner, 'run_case', side_effect=execute), patch.object(
            runner, 'bounded_process', side_effect=AssertionError('adapter execution forbidden')):
        outcomes = runner.harness_preflight(files, Path('/capture-only'), 3, profile)
    return captured, outcomes


class IterationCorpusTests(unittest.TestCase):
    def test_complete_source_mode_policy_and_helper_inventory(self):
        expected = {
            'for-of': (751, 1442, 189, 1253, 7, 1146879,
                       'c1eea71c09bc3fdc4028c28f90916e20bf7e8d0fb9c1cb834b1ce3dd545c7496'),
            'core-iterators': (76, 144, 106, 38, 15, 87300,
                               '44bcb91d37ca69d53440c40f58b12dce05ec979ae64b15fd8c0a575f79230e09'),
        }
        self.assertEqual(runner.FOR_OF_FEATURES,
                         runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES | {'for-of', 'let', 'const'})
        self.assertEqual(runner.CORE_ITERATOR_FEATURES,
                         runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES | {'Array.prototype.values'})
        for profile, (sources, modes, admitted, excluded, proofs, size, digest) in expected.items():
            with self.subTest(profile=profile), patch.object(importer, 'fetch', side_effect=AssertionError('offline replay fetched')):
                manifest, files, cases, fixtures, actual = runner.load_corpus(CORPORA[profile], profile)
            self.assertEqual((manifest['test_files'], len(cases), fixtures, actual), (sources, modes, [], digest))
            self.assertEqual({d: len(names) for d, names in manifest['directories'].items()}, importer.PROFILES[profile])
            self.assertEqual(sum(len(b) for n, b in files.items() if n.startswith('test/')), size)
            reasons = [runner.unsupported_reason(c, runner.PROFILE_FEATURES[profile]) for c in cases]
            self.assertEqual((reasons.count(None), sum(x is not None for x in reasons)), (admitted, excluded))
            self.assertEqual({n[8:] for n in files if n.startswith('harness/')}, importer.ITERATION_HELPERS[profile])
            self.assertEqual(len(manifest['inventory_proof']['files']), proofs)
            self.assertEqual(manifest['inventory_proof']['subtree_trees'], importer.ITERATION_SUBTREES[profile])
            self.assertEqual(manifest['inventory_proof']['method'], 'complete-root-linked-recursive-git-subtrees')

    def test_modes_negatives_and_untagged_prerequisites_remain_visible(self):
        _, _, cases, _, _ = runner.load_corpus(CORPORA['for-of'], 'for-of')
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict', 'module')], [729, 711, 2])
        self.assertEqual(sum(c['metadata']['negative'] is not None for c in cases), 165)
        selected = [c for c in cases if '/dstr/' in c['file']]
        self.assertEqual(len(selected), 1095)
        self.assertEqual(sum(runner.unsupported_reason(c, runner.FOR_OF_FEATURES) is None for c in selected), 12)
        for name in ('head-let-destructuring.js', 'cptn-decl-itr.js', 'Array.prototype.keys.js', 'Array.prototype.entries.js'):
            matched = [c for c in cases if c['file'].endswith('/' + name)]
            self.assertEqual(len(matched), 2)
            self.assertTrue(all(runner.unsupported_reason(c, runner.FOR_OF_FEATURES) is None for c in matched))
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases], separators=(',', ':')).encode()),
                         '42aeb08b56229cbf7fbd10b123b3acf9b59045a8f6bde7c8b24e310174539ade')
        _, _, cases, _, _ = runner.load_corpus(CORPORA['core-iterators'], 'core-iterators')
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [76, 68])
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases], separators=(',', ':')).encode()),
                         'f2da2b73f556a575b7c9d0b938ebbe23bc36ed52a4a2afb909014977c1200526')

    def test_recursive_proofs_reject_omissions_substitutions_and_hostile_shapes(self):
        for profile in PROFILES:
            tree = next(iter(importer.ITERATION_SUBTREES[profile].values()))
            route = f'trees/{tree}?recursive=1'
            original = json.loads(proof_reader(profile)(route))
            index = max(i for i, row in enumerate(original['tree']) if row['type'] == 'blob')
            edits = {
                'truncated': lambda d: d.update(truncated=True),
                'missing-tail-source': lambda d: d['tree'].pop(index),
                'same-count-name': lambda d: d['tree'][index].update(path='replacement.js'),
                'blob-id': lambda d: d['tree'][index].update(sha='0' * 40),
                'wrong-mode': lambda d: d['tree'][index].update(mode='100755'),
                'unhashable-mode': lambda d: d['tree'][index].update(mode=[]),
                'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][index])),
                'parent-path': lambda d: d['tree'][index].update(path='../outside.js'),
                'backslash-path': lambda d: d['tree'][index].update(path='a\\outside.js'),
                'orphan': lambda d: d['tree'][index].update(path='missing/source.js'),
                'depth': lambda d: d['tree'][index].update(path='a/' * 9 + 'source.js'),
                'path-bytes': lambda d: d['tree'][index].update(path='a' * 1025 + '.js'),
                'entry-count': lambda d: d.update(tree=[d['tree'][index]] * 4097),
                'boolean-size': lambda d: d['tree'][index].update(size=True),
                'oversized-source': lambda d: d['tree'][index].update(size=importer.MAX_FILE + 1),
                'aggregate-source-size': lambda d: [e.update(size=importer.MAX_FILE) for e in d['tree'] if e['type'] == 'blob'],
            }
            for name, edit in edits.items():
                with self.subTest(profile=profile, mutation=name), self.assertRaises(ValueError):
                    importer.iteration_tree_inventory(profile, proof_reader(profile, route, edit))
        route = f"trees/{importer.ITERATION_SUBTREES['for-of']['statements/for-of']}?recursive=1"
        for edit in (lambda d: d['tree'].pop(next(i for i, e in enumerate(d['tree']) if e['type'] == 'tree')),
                     lambda d: next(e for e in d['tree'] if e['type'] == 'tree').update(sha='0' * 40)):
            with self.assertRaises(ValueError):
                importer.iteration_tree_inventory('for-of', proof_reader('for-of', route, edit))

    def test_pinned_ancestors_legal_helpers_and_proof_limits(self):
        for profile in PROFILES:
            manifest, _, _, _, _ = runner.load_corpus(CORPORA[profile], profile)
            for entry in manifest['inventory_proof']['files']:
                route = entry['url'].split('/git/', 1)[1]
                with self.subTest(profile=profile, proof=entry['path']), self.assertRaises(ValueError):
                    importer.iteration_tree_inventory(profile, proof_reader(profile, route, lambda d: d.update(sha='0' * 40)))
            with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
                importer.iteration_tree_inventory(profile, proof_reader(profile))
            with patch.dict(importer.PROFILES, {profile: {}}), self.assertRaises(ValueError):
                importer.iteration_tree_inventory(profile, proof_reader(profile))
        with self.assertRaises(ValueError):
            importer.tree_proof_path(f'trees/{importer.REVISION_TREE}?recursive=1')
        with self.assertRaises(ValueError):
            importer.iteration_tree_inventory('date', lambda r: b'{}')

    def test_import_roundtrip_preserves_original_responses_and_all_bytes(self):
        for profile in PROFILES:
            manifest, files, cases, _, _ = runner.load_corpus(CORPORA[profile], profile)
            raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
            api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
            def fetch(url):
                if url.startswith(api):
                    return proof_reader(profile)(url[len(api):])
                self.assertTrue(url.startswith(raw), url)
                return files[url[len(raw):]]
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
                importer.import_corpus(Path(temporary), profile)
                actual, retained, imported, _, _ = runner.load_corpus(Path(temporary), profile)
                self.assertEqual(actual, manifest)
                self.assertEqual(retained, files)
                self.assertEqual([c['case_sha256'] for c in imported], [c['case_sha256'] for c in cases])
                for entry in manifest['inventory_proof']['files']:
                    self.assertEqual((Path(temporary) / entry['path']).read_bytes(),
                                     (CORPORA[profile] / entry['path']).read_bytes())

    def test_modified_manifest_cannot_bless_changed_sources_helpers_or_proofs(self):
        targets = {
            'for-of': ['test/language/statements/for-of/dstr/const-ary-ptrn-init-err.js',
                       'harness/asyncHelpers.js', 'harness/assert.js', 'LICENSE',
                       'inventory-proof/recursive-592792d58aaf7752ef1b74e0edba13157f698a38.json'],
            'core-iterators': ['test/built-ins/Array/prototype/values/name.js',
                               'harness/testTypedArray.js', 'harness/propertyHelper.js', 'INTERPRETING.md'],
        }
        for profile, names in targets.items():
            for name in names:
                with self.subTest(profile=profile, name=name), tempfile.TemporaryDirectory() as temporary:
                    dest = Path(temporary) / 'corpus'
                    shutil.copytree(CORPORA[profile], dest)
                    target = dest / name
                    if name.startswith('inventory-proof/'):
                        value = json.loads(target.read_bytes()); value['tree'].pop()
                        data = json.dumps(value).encode()
                    else:
                        data = target.read_bytes() + b'\n'
                    target.write_bytes(data)
                    manifest = json.loads((dest / 'manifest.json').read_bytes())
                    entries = manifest['inventory_proof']['files'] if name.startswith('inventory-proof/') else manifest['files']
                    next(e for e in entries if e['path'] == name).update(bytes=len(data), sha256=runner.digest(data))
                    (dest / 'manifest.json').write_text(json.dumps(manifest))
                    with self.assertRaises(ValueError):
                        runner.load_corpus(dest, profile)

    def test_extra_auxiliary_and_symlink_alias_are_rejected(self):
        for mutation in ('extra-helper', 'helper-symlink', 'proof-symlink', 'extra-proof'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPORA['core-iterators'], dest)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                if mutation == 'extra-helper':
                    name = 'harness/unpinned.js'; data = b'// unpinned\n'
                    (dest / name).write_bytes(data)
                    manifest['files'].append(dict(path=name, upstream_path=name, bytes=len(data), sha256=runner.digest(data)))
                elif mutation == 'extra-proof':
                    manifest['inventory_proof']['files'].append(dict(path='inventory-proof/unrequested.json', bytes=2, sha256=runner.digest(b'{}'), url='unrequested'))
                else:
                    name = ('harness/assert.js' if mutation == 'helper-symlink'
                            else manifest['inventory_proof']['files'][0]['path'])
                    target = dest / name; target.unlink(); target.symlink_to(CORPORA['core-iterators'] / name)
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, 'core-iterators')

    def test_guarded_pairs_and_exact_frozen_protocol_sources(self):
        for profile, count in [('for-of', 80), ('core-iterators', 132)]:
            captured, outcomes = capture_controls(profile)
            self.assertEqual(len(captured), count)
            added = captured[32:]
            for good, bad in zip(added[::2], added[1::2]):
                self.assertEqual(good['mode'], bad['mode'])
                self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0], bad['source'].rsplit(b'assert.sameValue(', 1)[0])
                self.assertNotEqual(good['source'], bad['source'])
                if profile == 'for-of':
                    guard = b"assert.sameValue($foGuardCount,2,'successful protocol prerequisite');"
                else:
                    guard = b"assert.sameValue(String.prototype[Symbol.iterator].call('a').next().value,'a');"
                self.assertIn(guard, good['source'])
                if b'assert.throws' in good['source']:
                    self.assertLess(good['source'].index(guard), good['source'].index(b'assert.throws'))
                if 'property-' in good['id']:
                    self.assertIn('propertyHelper.js', good['metadata']['includes'])
                    self.assertGreaterEqual(good['source'].count(b'{restore:true}'), 2)
            for good, bad in zip(outcomes[32::2], outcomes[33::2]):
                self.assertEqual(bad['prerequisite'], dict(name=good['name'], mode=good['result']['mode'],
                    case_sha256=good['result']['case_sha256'], source_sha256=good['result']['source_sha256'],
                    expected='passed', verified=True))
            if profile == 'for-of':
                # Independently frozen local controls; mode/source bytes only,
                # since formal IDs/harness selection are a separate contract.
                self.assertEqual(runner.digest(canonical([(c['mode'], runner.digest(c['source'])) for c in added])),
                                 'e367a2c0924b7a502ffc471300c443ca18cc62ad82efcee66b8f65d731984930')

    def test_unavailable_feature_and_disabled_assertions_fail_closed(self):
        for profile in PROFILES:
            _, files, _, _, _ = runner.load_corpus(CORPORA[profile], profile)
            with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, profile)
                original = runner.harness_preflight(files, Path('/fake'), 1)
            self.assertEqual(actual[:32], original)
            self.assertFalse(all(item['verified'] for item in actual))
            self.assertTrue(all(item['verified'] == (item['expected'] == 'passed') for item in actual[32:]))
            for kind in ('Test262Error', 'TypeError', 'ReferenceError', 'SyntaxError', 'RangeError'):
                with self.subTest(profile=profile, error=kind), patch.object(
                        runner, 'bounded_process', return_value=(0, response('exception', 'runtime', kind), b'')):
                    actual = runner.harness_preflight(files, Path('/fake'), 1, profile)
                self.assertFalse(any(item['verified'] for item in actual[32:]))
                self.assertTrue(all(item['result']['observation']['error_type'] == kind for item in actual[32:]))

    def test_same_mode_partner_and_mismatch_identity_required(self):
        def fake(case):
            if case['id'].endswith('-mismatch') or case['mode'] == 'strict':
                return dict(status='failed', observation={'error_type': 'Test262Error'})
            return dict(status='passed')
        for profile in PROFILES:
            _, outcomes = capture_controls(profile, fake)
            for item in outcomes[32:]:
                self.assertEqual(item['verified'], item['result']['mode'] == 'sloppy')
                if item['expected'] == 'failed':
                    self.assertEqual(item['prerequisite']['mode'], item['result']['mode'])
            def wrong_identity(case):
                return (dict(status='failed', observation={'error_type': 'TypeError'})
                        if case['id'].endswith('-mismatch') else dict(status='passed'))
            _, outcomes = capture_controls(profile, wrong_identity)
            self.assertTrue(all(not item['verified'] for item in outcomes[32:] if item['expected'] == 'failed'))
            for status in ('resource', 'timeout', 'adapter-error', 'unsupported'):
                _, outcomes = capture_controls(profile, lambda case: dict(status=status))
                self.assertFalse(any(item['verified'] for item in outcomes[32:]))
                self.assertTrue(all(item['result']['status'] == status for item in outcomes[32:]))

    def test_all_36_historical_contracts_are_identical(self):
        contract = capture_contracts(excluded=(*PROFILES, 'array-from'))
        self.assertEqual(contract['counts'], dict(profiles=36, cases=16146, preflights=3592))
        self.assertEqual(runner.digest(canonical(contract)),
                         'acd6602c8c218ebe36c9cc245cc0805fc9831598a64cfaaf3fdfb4319f9cce74')


if __name__ == '__main__':
    unittest.main()
