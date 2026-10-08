"""Complete pinned DataView selection and guarded assertion contracts."""
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


PROFILE = 'data-view'
CORPUS = runner.ROOT / 'tests/upstream/test262-data-view'
TREE = 'ee1337a290051a188aacccb7cfbcaa16609efe64'
DIRECTORIES = {
    'DataView': 62, 'DataView/prototype': 1, 'DataView/prototype/buffer': 11,
    'DataView/prototype/byteLength': 14, 'DataView/prototype/byteOffset': 13,
    'DataView/prototype/getBigInt64': 21, 'DataView/prototype/getBigUint64': 21,
    'DataView/prototype/getFloat16': 21, 'DataView/prototype/getFloat32': 21,
    'DataView/prototype/getFloat64': 21, 'DataView/prototype/getInt16': 18,
    'DataView/prototype/getInt32': 28, 'DataView/prototype/getInt8': 17,
    'DataView/prototype/getUint16': 18, 'DataView/prototype/getUint32': 18,
    'DataView/prototype/getUint8': 17, 'DataView/prototype/setBigInt64': 24,
    'DataView/prototype/setBigUint64': 3, 'DataView/prototype/setFloat16': 24,
    'DataView/prototype/setFloat32': 24, 'DataView/prototype/setFloat64': 24,
    'DataView/prototype/setInt16': 24, 'DataView/prototype/setInt32': 24,
    'DataView/prototype/setInt8': 22, 'DataView/prototype/setUint16': 24,
    'DataView/prototype/setUint32': 24, 'DataView/prototype/setUint8': 22,
}


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


class DataViewCorpusTests(unittest.TestCase):
    def test_complete_inventory_and_policy_match_independent_preparation(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline replay fetched')):
            manifest, files, cases, fixtures, digest = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (561, 1122, []))
        self.assertEqual({d: len(names) for d, names in manifest['directories'].items()}, DIRECTORIES)
        self.assertEqual(digest, '09922664d5b4e38e74515e2c46b032dc574b0e950d4003be600fad843777fd0a')
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 693530)
        self.assertEqual([sum(c['mode'] == mode for c in cases) for mode in ('sloppy', 'strict')], [561, 561])
        self.assertTrue(all(c['metadata']['negative'] is None and c['metadata']['flags'] == [] for c in cases))
        self.assertEqual(runner.digest(canonical([(c['id'], c['case_sha256']) for c in cases])),
                         '5ee25ae346a538f6f80b24c36ea5f850f1bc8e1767f192e69b8ad47fc3e35b89')
        self.assertEqual(runner.DATA_VIEW_FEATURES, runner.ARRAY_BUFFER_FEATURES | {
            'DataView', 'DataView.prototype.getFloat32', 'DataView.prototype.getFloat64',
            'DataView.prototype.getInt16', 'DataView.prototype.getInt32',
            'DataView.prototype.getInt8', 'DataView.prototype.getUint16',
            'DataView.prototype.getUint32', 'DataView.prototype.setUint8', 'Float16Array'})
        self.assertFalse(runner.DATA_VIEW_FEATURES & {
            'TypedArray', 'SharedArrayBuffer', 'Proxy', 'cross-realm', 'immutable-arraybuffer', 'BigInt', 'class'})
        self.assertEqual({name[8:] for name in files if name.startswith('harness/')}, {
            'assert.js', 'sta.js', 'compareArray.js', 'propertyHelper.js', 'isConstructor.js',
            'detachArrayBuffer.js', 'byteConversionValues.js'})
        self.assertEqual(len(manifest['inventory_proof']['files']), 6)
        self.assertEqual(sum(x['bytes'] for x in manifest['inventory_proof']['files']), 174345)
        self.assertEqual(manifest['inventory_proof']['subtree_trees'], {'DataView': TREE})
        self.assertEqual(len(manifest['inventory_proof']['directory_trees']), 27)
        reasons = [runner.unsupported_reason(c, runner.DATA_VIEW_FEATURES) for c in cases]
        self.assertEqual((reasons.count(None), sum(x is not None for x in reasons)), (706, 416))
        self.assertEqual(sum(bool(x and x.startswith('unimplemented declared features:')) for x in reasons), 278)
        self.assertEqual(reasons.count('Test262 host hooks are not implemented (conservative source check)'), 138)

    def test_float16_tag_and_untagged_bigint_prerequisites_remain_visible(self):
        _, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        half = [c for c in cases if 'Float16Array' in c['metadata']['features']]
        self.assertEqual(len(half), 88)
        self.assertEqual({c['file'].split('/')[-2] for c in half}, {'getFloat16', 'setFloat16'})
        for c in half:
            # Strip metadata/comments for the API-reference check; the tag is retained.
            body = c['source'].split(b'---*/', 1)[1]
            self.assertNotIn(b'Float16Array', body)
            self.assertNotIn(b'Math.f16round', body)
        half_reasons = [runner.unsupported_reason(c, runner.DATA_VIEW_FEATURES) for c in half]
        self.assertEqual(half_reasons.count(None), 70)
        self.assertEqual(half_reasons.count(
            'Test262 host hooks are not implemented (conservative source check)'), 14)
        self.assertEqual(half_reasons.count('unimplemented declared features: Int8Array'), 4)
        admitted_big = [c for c in cases if any('/'+method+'/' in c['file'] for method in (
            'getBigInt64', 'getBigUint64', 'setBigInt64', 'setBigUint64'))
            and runner.unsupported_reason(c, runner.DATA_VIEW_FEATURES) is None]
        self.assertEqual(len(admitted_big), 12)
        self.assertEqual({c['file'].split('/prototype/')[1] for c in admitted_big}, {
            'getBigInt64/resizable-buffer.js', 'getBigUint64/resizable-buffer.js',
            'setBigInt64/resizable-buffer.js', 'setBigUint64/resizable-buffer.js',
            'getBigUint64/not-a-constructor.js', 'setBigUint64/not-a-constructor.js'})
        self.assertTrue(all('BigInt' not in c['metadata']['features'] for c in admitted_big))
        self.assertIn(b'$262.detachArrayBuffer', files['harness/detachArrayBuffer.js'])
        self.assertTrue(any('$262' in c['source'].decode() for c in cases))

    def test_all_42_previous_profile_contracts_are_byte_identical(self):
        contract = capture_contracts(excluded={'typedarray-reverse', 'typedarray-fill', 'typedarray-search', 'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', 'object-is', PROFILE})
        self.assertEqual(contract['counts'], dict(profiles=42, cases=18563, preflights=4252))
        self.assertEqual(runner.digest(canonical(contract)),
                         'a30b99cc99f9c001b52cbcccf3ea522a4a290cac4938f6f9f6a9cd3718f415a6')

    def test_paired_controls_guard_both_branches_before_error_assertions(self):
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (232, 232))
        identities = [(o['name'], c['mode'], runner.digest(c['source']), o['expected'])
                      for c, o in zip(captured[32:], outcomes[32:])]
        self.assertEqual(runner.digest(canonical(identities)),
                         'b5aa03940b0f97b41f4e548ccceb93edc18a7101acad12e6cfed860d8df52121')
        self.assertEqual([sum(c['mode'] == mode for c in captured[32:])
                          for mode in ('sloppy', 'strict')], [100, 100])
        guard_end = b'assert.sameValue(ArrayBuffer.isView(v),true);'
        for good, bad in zip(captured[32::2], captured[33::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
            for text in (b"assert.sameValue(typeof DataView,'function');",
                         b'var b=new ArrayBuffer(32),v=new DataView(b);',
                         b'v.setUint8(0,91);assert.sameValue(v.getUint8(0),91);',
                         b'v.setUint16(28,258);assert.sameValue(v.getUint16(28),258);', guard_end):
                self.assertIn(text, good['source'])
            if b'assert.throws' in good['source']:
                self.assertLess(good['source'].index(guard_end), good['source'].index(b'assert.throws'))
            if 'property-' in good['id']:
                self.assertIn('propertyHelper.js', good['metadata']['includes'])
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
        self.assertEqual(sum(item['verified'] for item in actual[32:]), 100)
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
        self.assertEqual(len(outcomes), 232)
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

    def test_recursive_inventory_rejects_corruption_and_resource_amplification(self):
        route = f'trees/{TREE}?recursive=1'
        def first_blob(d):
            return next(e for e in d['tree'] if e['type'] == 'blob')
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'missing-source': lambda d: d['tree'].pop(),
            'missing-directory': lambda d: d.update(tree=[e for e in d['tree'] if not e['path'].startswith('prototype/getFloat16')]),
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
        for directories in ({}, {**DIRECTORIES, 'DataView': 61}, {**DIRECTORIES, 'DataView/omitted': 0}):
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
        for suffix in ('test/built-ins/DataView/constructor.js', 'harness/assert.js', 'LICENSE'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(source=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_changed_or_unpinned_bytes(self):
        for name in ('test/built-ins/DataView/constructor.js', 'harness/assert.js', 'harness/detachArrayBuffer.js',
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
