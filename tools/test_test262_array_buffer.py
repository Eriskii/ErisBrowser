"""Complete pinned ArrayBuffer selection and guarded assertion contracts."""
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


PROFILE = 'array-buffer'
CORPUS = runner.ROOT / 'tests/upstream/test262-array-buffer'
TREE = '86fe659bc58e1293e1ed757a87bdda41ef0bcd86'
DIRECTORIES = {'ArrayBuffer': 28, 'ArrayBuffer/Symbol.species': 4, 'ArrayBuffer/isView': 17, 'ArrayBuffer/prototype': 2, 'ArrayBuffer/prototype/byteLength': 10, 'ArrayBuffer/prototype/detached': 11, 'ArrayBuffer/prototype/immutable': 5, 'ArrayBuffer/prototype/maxByteLength': 11, 'ArrayBuffer/prototype/resizable': 10, 'ArrayBuffer/prototype/resize': 22, 'ArrayBuffer/prototype/slice': 33, 'ArrayBuffer/prototype/sliceToImmutable': 11, 'ArrayBuffer/prototype/transfer': 24, 'ArrayBuffer/prototype/transferToFixedLength': 24, 'ArrayBuffer/prototype/transferToImmutable': 9}


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


class ArrayBufferCorpusTests(unittest.TestCase):
    def test_complete_inventory_and_policy_match_independent_frozen_preparation(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline replay fetched')):
            manifest, files, cases, fixtures, digest = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (221, 442, []))
        self.assertEqual({d: len(names) for d, names in manifest['directories'].items()}, DIRECTORIES)
        self.assertEqual(digest, '030314ffaddefd24b13fdf086cfbf29addb8aa0892d38e9c9a55e84f0bdbfdc2')
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 261335)
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [221, 221])
        self.assertTrue(all(c['metadata']['negative'] is None and c['metadata']['flags'] == [] for c in cases))
        self.assertEqual(runner.digest(canonical([(c['id'], c['case_sha256']) for c in cases])),
                         'bbf1c8c6a2903445ef438b9d878631b601326b7d07377262031055683a875b0a')
        self.assertEqual(runner.ARRAY_BUFFER_FEATURES, runner.ARRAY_CONCAT_FEATURES |
                         {'ArrayBuffer', 'resizable-arraybuffer', 'arraybuffer-transfer',
                          'align-detached-buffer-semantics-with-web-reality'})
        self.assertFalse(runner.ARRAY_BUFFER_FEATURES &
                         {'TypedArray', 'DataView', 'SharedArrayBuffer', 'Proxy', 'cross-realm', 'immutable-arraybuffer', 'BigInt', 'class'})
        self.assertEqual({name[8:] for name in files if name.startswith('harness/')},
                         {'assert.js', 'sta.js', 'compareArray.js', 'propertyHelper.js', 'isConstructor.js',
                          'detachArrayBuffer.js', 'testTypedArray.js'})
        self.assertEqual(len(manifest['inventory_proof']['files']), 6)
        self.assertEqual(sum(x['bytes'] for x in manifest['inventory_proof']['files']), 85888)
        self.assertEqual(manifest['inventory_proof']['subtree_trees'], {'ArrayBuffer': TREE})
        self.assertEqual(len(manifest['inventory_proof']['directory_trees']), 15)
        reasons = [runner.unsupported_reason(c, runner.ARRAY_BUFFER_FEATURES) for c in cases]
        self.assertEqual((reasons.count(None), sum(x is not None for x in reasons)), (312, 130))
        self.assertEqual(sum(bool(x and x.startswith('unimplemented declared features:')) for x in reasons), 112)
        self.assertEqual(reasons.count('Test262 host hooks are not implemented (conservative source check)'), 18)
        self.assertEqual(sum('immutable-arraybuffer' in c['metadata']['features'] for c in cases), 58)

    def test_untagged_prerequisites_huge_allocations_and_original_host_helpers_remain(self):
        _, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for suffix, marker in (
                ('options-non-object.js', b'1n'),
                ('prototype/transfer/from-fixed-to-larger.js', b'new Uint8Array'),
                ('prototype/transferToFixedLength/from-resizable-to-smaller.js', b'new Uint8Array'),
                ('allocation-limit.js', b'7 * 1125899906842624')):
            chosen = [c for c in cases if c['file'] == 'test/built-ins/ArrayBuffer/' + suffix]
            self.assertEqual(len(chosen), 2)
            for case in chosen:
                self.assertIn(marker, case['source'])
                self.assertIsNone(runner.unsupported_reason(case, runner.ARRAY_BUFFER_FEATURES))
        self.assertIn(b'$262.detachArrayBuffer', files['harness/detachArrayBuffer.js'])
        detached = [c for c in cases if c['file'].endswith('/byteLength/detached-buffer.js')]
        self.assertEqual(len(detached), 2)
        for case in detached:
            self.assertEqual(runner.unsupported_reason(case, runner.ARRAY_BUFFER_FEATURES),
                             'Test262 host hooks are not implemented (conservative source check)')

    def test_recursive_inventory_rejects_corruption_and_resource_amplification(self):
        route = f'trees/{TREE}?recursive=1'
        def first_blob(d):
            return next(e for e in d['tree'] if e['type'] == 'blob')
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'missing-source': lambda d: d['tree'].pop(),
            'missing-directory': lambda d: d.update(tree=[e for e in d['tree'] if not e['path'].startswith('prototype/resize')]),
            'replacement-name': lambda d: first_blob(d).update(path='replacement.js'),
            'replacement-blob': lambda d: first_blob(d).update(sha='0' * 40),
            'mode': lambda d: first_blob(d).update(mode='100755'),
            'unhashable-mode': lambda d: first_blob(d).update(mode=[]),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'parent-path': lambda d: first_blob(d).update(path='../outside.js'),
            'orphan': lambda d: first_blob(d).update(path='missing/source.js'),
            'depth': lambda d: first_blob(d).update(path='a/' * 9 + 'source.js'),
            'path-bytes': lambda d: first_blob(d).update(path='a' * 1025 + '.js'),
            'entry-count': lambda d: d.update(tree=[d['tree'][0]] * 4097),
            'boolean-size': lambda d: first_blob(d).update(size=True),
            'source-size': lambda d: first_blob(d).update(size=importer.MAX_FILE + 1),
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
        for directories in ({}, {**DIRECTORIES, 'ArrayBuffer': 27}, {**DIRECTORIES, 'ArrayBuffer/omitted': 0}):
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
        for suffix in ('test/built-ins/ArrayBuffer/allocation-limit.js', 'harness/assert.js', 'LICENSE'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(source=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_changed_or_unpinned_bytes(self):
        for name in ('test/built-ins/ArrayBuffer/allocation-limit.js', 'harness/assert.js', 'harness/detachArrayBuffer.js',
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

    def test_all_41_previous_profile_contracts_are_byte_identical(self):
        contract = capture_contracts(excluded={'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', 'object-is', 'data-view', PROFILE})
        self.assertEqual(contract['counts'], dict(profiles=41, cases=18121, preflights=4092))
        self.assertEqual(runner.digest(canonical(contract)),
                         '7e56c9b8281e5e2944dc0d70d96a8ddeaf6f7769074561e6da53abc65191cceb')

    def test_paired_controls_guard_both_branches_before_error_assertions(self):
        captured, outcomes = capture_controls()
        self.assertEqual(len(captured), 160)
        self.assertEqual(len(outcomes), 160)
        # Independent standard-derived pairs were authored before any engine execution.
        identities = [(o['name'], c['mode'], runner.digest(c['source']), o['expected'])
                      for c, o in zip(captured[32:], outcomes[32:])]
        self.assertEqual(runner.digest(canonical(identities)),
                         '0484c0897d6de251adb26ccdfc4143eb589a1ad15c7afd34a8f61b3e71161799')
        added = captured[32:]
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in added), 64)
        self.assertEqual(sum(c['mode'] == 'strict' for c in added), 64)
        guard_end = b'assert.sameValue(AB.isView($abF),false);'
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
            self.assertIn(b"assert.sameValue(typeof ArrayBuffer,'function','ArrayBuffer availability');", good['source'])
            self.assertIn(b'var $abF=new AB(2),$abR=new AB(2,{maxByteLength:4});', good['source'])
            self.assertIn(b'assert.sameValue(rz.call($abR,3),undefined);', good['source'])
            self.assertIn(b'assert.sameValue(sl.call($abR,1,2).byteLength,1);', good['source'])
            self.assertIn(b'var $abT=tr.call($abR);', good['source'])
            self.assertIn(b'var $abX=tf.call($abT);', good['source'])
            self.assertIn(guard_end, good['source'])
            if b'assert.throws' in good['source']:
                self.assertLess(good['source'].index(guard_end), good['source'].index(b'assert.throws'))
            if 'property-' in good['id']:
                self.assertIn('propertyHelper.js', good['metadata']['includes'])
                self.assertGreater(good['source'].count(b'{restore:true}'), 0)
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
        self.assertEqual(sum(item['verified'] for item in actual[32:]), 64)
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
        self.assertEqual(len(outcomes), 160)
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
