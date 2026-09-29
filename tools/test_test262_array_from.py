"""Complete pinned Array.from selection and guarded assertion contracts."""
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


PROFILE = 'array-from'
CORPUS = runner.ROOT / 'tests/upstream/test262-array-from'
TREE = 'e3a97b42c65283fc23a15d65610c2461e6f02715'


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


class ArrayFromCorpusTests(unittest.TestCase):
    def test_complete_inventory_and_policy_match_independent_frozen_preparation(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline replay fetched')):
            manifest, files, cases, fixtures, digest = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (47, 90, []))
        self.assertEqual({d: len(names) for d, names in manifest['directories'].items()}, {'Array/from': 47})
        self.assertEqual(digest, '9fac107410a7e10f895b1c9900c802626f1f3652040877d662c9892b056180d5')
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 50155)
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [45, 45])
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(runner.digest(canonical([(c['id'], c['case_sha256']) for c in cases])),
                         '522e7c54c94e77c6e121dd5ef9e753e5ca4bb020880afc790aeb48bea6412646')
        self.assertEqual(runner.ARRAY_FROM_FEATURES, runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES |
                         {'for-of', 'let', 'const', 'Array.prototype.values'})
        self.assertNotIn('Array.from', runner.ARRAY_FROM_FEATURES)
        self.assertEqual({name[8:] for name in files if name.startswith('harness/')},
                         {'assert.js', 'sta.js', 'compareArray.js', 'propertyHelper.js', 'isConstructor.js'})
        self.assertEqual(len(manifest['inventory_proof']['files']), 7)
        self.assertEqual(sum(x['bytes'] for x in manifest['inventory_proof']['files']), 52346)
        self.assertEqual(manifest['inventory_proof']['subtree_trees'], {'Array/from': TREE})
        self.assertEqual(manifest['inventory_proof']['directory_trees'], {'Array/from': TREE})
        reasons = [runner.unsupported_reason(c, runner.ARRAY_FROM_FEATURES) for c in cases]
        self.assertEqual((reasons.count(None), sum(x is not None for x in reasons)), (86, 4))
        self.assertEqual(reasons.count('unimplemented declared features: generators'), 2)
        self.assertEqual(reasons.count('unimplemented declared features: cross-realm'), 2)

    def test_untagged_dependencies_are_admitted_without_rewriting_sources(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for filename, token, digest in [
            ('items-is-arraybuffer.js', b'new ArrayBuffer', '74a5ca51435c100cf32f22d30bb3511aa64c668f8a97504ea01a929623ab2e13'),
            ('elements-deleted-after.js', b'.splice(', '4da66f66fe87c8c2e1f5874b1d70df861749c6c299029f173465d48faa70686a')]:
            selected = [c for c in cases if c['file'].endswith('/' + filename)]
            self.assertEqual(len(selected), 2)
            for case in selected:
                self.assertIn(token, case['source'])
                self.assertEqual(runner.digest(case['source']), digest)
                self.assertIsNone(runner.unsupported_reason(case, runner.ARRAY_FROM_FEATURES))

    def test_recursive_inventory_rejects_corruption_and_resource_amplification(self):
        route = f'trees/{TREE}?recursive=1'
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'missing-source': lambda d: d['tree'].pop(),
            'replacement-name': lambda d: d['tree'][0].update(path='replacement.js'),
            'replacement-blob': lambda d: d['tree'][0].update(sha='0' * 40),
            'mode': lambda d: d['tree'][0].update(mode='100755'),
            'unhashable-mode': lambda d: d['tree'][0].update(mode=[]),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'parent-path': lambda d: d['tree'][0].update(path='../outside.js'),
            'orphan': lambda d: d['tree'][0].update(path='missing/source.js'),
            'depth': lambda d: d['tree'][0].update(path='a/' * 9 + 'source.js'),
            'path-bytes': lambda d: d['tree'][0].update(path='a' * 1025 + '.js'),
            'entry-count': lambda d: d.update(tree=[d['tree'][0]] * 4097),
            'boolean-size': lambda d: d['tree'][0].update(size=True),
            'source-size': lambda d: d['tree'][0].update(size=importer.MAX_FILE + 1),
            'aggregate-size': lambda d: [e.update(size=importer.MAX_FILE) for e in d['tree']],
        }
        for name, edit in edits.items():
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, edit))

    def test_ancestor_auxiliary_and_complete_directory_set_are_authenticated(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(proof=entry['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())
        for directories in ({}, {'Array/from': 46}, {'Array/from': 47, 'Array/from/omitted': 0}):
            with patch.dict(importer.PROFILES, {PROFILE: directories}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_reproduces_original_proofs_and_source_bytes(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        def fetch(url):
            self.assertNotIn('/contents/', url)
            if url.startswith(api): return proof_reader()(url[len(api):])
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
        for suffix in ('test/built-ins/Array/from/from-array.js', 'harness/assert.js', 'LICENSE'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(source=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_changed_or_unpinned_bytes(self):
        for name in ('test/built-ins/Array/from/from-array.js', 'harness/assert.js', 'harness/isConstructor.js',
                     'LICENSE', 'INTERPRETING.md', f'inventory-proof/recursive-{TREE}.json'):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'; shutil.copytree(CORPUS, dest)
                target = dest / name
                if name.startswith('inventory-proof/'):
                    value = json.loads(target.read_bytes()); value['tree'].pop(); data = json.dumps(value).encode()
                else: data = target.read_bytes() + b'\n'
                target.write_bytes(data)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                entries = manifest['inventory_proof']['files'] if name.startswith('inventory-proof/') else manifest['files']
                next(e for e in entries if e['path'] == name).update(bytes=len(data), sha256=runner.digest(data))
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError): runner.load_corpus(dest, PROFILE)
        for mutation in ('extra-helper', 'extra-proof', 'helper-symlink', 'proof-symlink'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'; shutil.copytree(CORPUS, dest)
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
                with self.assertRaises(ValueError): runner.load_corpus(dest, PROFILE)

    def test_all_38_previous_profile_contracts_are_byte_identical(self):
        contract = capture_contracts(excluded={PROFILE})
        self.assertEqual(contract['counts'], dict(profiles=38, cases=17732, preflights=3804))
        self.assertEqual(runner.digest(canonical(contract)),
                         '6a8b84416da50a0d0210cef1c4f182c83fddbfc9fd02f37f69ff4f19958485df')

    def test_paired_controls_guard_both_branches_before_error_assertions(self):
        captured, outcomes = capture_controls()
        self.assertEqual(len(captured), 96)
        self.assertEqual(len(outcomes), 96)
        # Independent preflights c0409c1e... supplied exact source bytes before execution.
        identities = [(o['name'], c['mode'], runner.digest(c['source']), o['expected'])
                      for c, o in zip(captured[32:], outcomes[32:])]
        self.assertEqual(runner.digest(canonical(identities)),
                         'a152125bbf03f0d6c52ad78df658c16fb9109475cef2e5db0b6ba57f82d9e03c')
        added = captured[32:]
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in added), 32)
        self.assertEqual(sum(c['mode'] == 'strict' for c in added), 32)
        guard_end = b'assert.sameValue($afNext,2);'
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
            self.assertIn(b"assert.sameValue(typeof Array.from,'function','Array.from availability');", good['source'])
            self.assertIn(b'var $afLike=Array.from({0:3,length:1});', good['source'])
            self.assertIn(b'var $afResult=Array.from($afSource);', good['source'])
            self.assertIn(guard_end, good['source'])
            if b'assert.throws' in good['source']:
                self.assertLess(good['source'].index(guard_end), good['source'].index(b'assert.throws'))
            if 'property-metadata' in good['id']:
                self.assertIn('propertyHelper.js', good['metadata']['includes'])
                self.assertEqual(good['source'].count(b'{restore:true}'), 2)
        for good, bad in zip(outcomes[32::2], outcomes[33::2]):
            self.assertEqual(bad['prerequisite'], dict(name=good['name'], mode=good['result']['mode'],
                case_sha256=good['result']['case_sha256'], source_sha256=good['result']['source_sha256'],
                expected='passed', verified=True))

    def test_disabled_assertions_and_incidental_errors_cannot_establish_health(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            core = runner.harness_preflight(files, Path('/fake'), 1)
        self.assertEqual(actual[:32], core)
        self.assertEqual(sum(item['verified'] for item in actual[32:]), 32)
        self.assertFalse(all(item['verified'] for item in actual))
        for kind in ('Test262Error', 'TypeError', 'ReferenceError', 'SyntaxError', 'RangeError'):
            with self.subTest(error=kind), patch.object(runner, 'bounded_process',
                    return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(item['verified'] for item in actual[32:]))
            self.assertTrue(all(item['result']['observation']['error_type'] == kind for item in actual[32:]))

    def test_same_mode_positive_and_exact_wrong_assertion_identity_are_required(self):
        def fake(case):
            if case['id'].endswith('-mismatch') or case['mode'] == 'strict':
                return dict(status='failed', observation={'error_type': 'Test262Error'})
            return dict(status='passed')
        _, outcomes = capture_controls(fake)
        self.assertEqual(len(outcomes), 96)
        for item in outcomes[32:]:
            self.assertEqual(item['verified'], item['result']['mode'] == 'sloppy')
            if item['expected'] == 'failed':
                self.assertEqual(item['prerequisite']['mode'], item['result']['mode'])
        def wrong_identity(case):
            return (dict(status='failed', observation={'error_type': 'TypeError'})
                    if case['id'].endswith('-mismatch') else dict(status='passed'))
        _, outcomes = capture_controls(wrong_identity)
        self.assertTrue(all(not item['verified'] for item in outcomes[32:] if item['expected'] == 'failed'))
        for status in ('resource', 'timeout', 'adapter-error', 'unsupported'):
            _, outcomes = capture_controls(lambda case: dict(status=status))
            self.assertFalse(any(item['verified'] for item in outcomes[32:]))
            self.assertTrue(all(item['result']['status'] == status for item in outcomes[32:]))


if __name__ == '__main__':
    unittest.main()
