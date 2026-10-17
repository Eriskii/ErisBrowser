"""Try loader admission/authentication controls; no adapter or network execution."""
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner

PROFILE = 'promise-try'
CORPUS = importer.ROOT / 'tests/upstream/test262-promise-try'
ORACLE = Path(__file__).with_name('test262-promise-try-import-oracle.json')
ORACLE_SHA = '3ae368d1bc5f8839a49d2115152b2f0f284aa567649424c2ded86870f1f48da2'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def replace_file(directory, manifest, path, data, *, proof=False):
    target = directory / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    rows = manifest['inventory_proof']['files'] if proof else manifest['files']
    row = next(r for r in rows if r['path'] == path)
    row.update(bytes=len(data), sha256=digest(data))


def write_manifest(directory, manifest):
    (directory / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


class PromiseTryLoaderTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.manifest = self.oracle['expected_manifest']
        for owner, name in ((importer, 'fetch'), (runner, 'bounded_process')):
            guard = patch.object(owner, name,
                                 side_effect=AssertionError('network or adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_actual_loader_admits_all_30_host_identities_and_exact_helper_bytes(self):
        manifest, files, cases, fixtures, manifest_sha = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual(manifest, self.manifest)
        self.assertEqual(manifest_sha, self.oracle['manifest_sha256'])
        self.assertEqual(fixtures, [])
        self.assertEqual(set(files), {row['path'] for row in self.manifest['files']})
        self.assertEqual(len(files), 25)
        for row in self.manifest['files']:
            self.assertEqual((len(files[row['path']]), digest(files[row['path']])),
                             (row['bytes'], row['sha256']), row['path'])
        self.assertEqual([dict(id=c['id'], case_sha256=c['case_sha256']) for c in cases],
                         self.oracle['case_identities'])
        self.assertEqual(len(cases), 30)
        for case in cases:
            meta = self.oracle['metadata'][case['file']]
            self.assertEqual(case['metadata'], meta)
            self.assertEqual(case['source'], files[case['file']])
            self.assertIs(type(case['completion_policy']), int)
            self.assertEqual(case['completion_policy'], int('async' in meta['flags']))
            self.assertEqual(case['harness'], [(name, files['harness/' + name]) for name in
                self.oracle['helper_order'][case['file']]])
            self.assertIn(case['mode'], ('sloppy', 'strict'))
        self.assertEqual({c['file'] for c in cases}, set(self.oracle['metadata']))
        self.assertEqual(sum(c['completion_policy'] == 1 for c in cases), 8)
        self.assertEqual(sum('class' in c['metadata']['features'] for c in cases), 6)
        self.assertTrue(all('promiseHelper.js' not in [n for n, _ in c['harness']] for c in cases))
        self.assertIn('harness/promiseHelper.js', files)
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], {'Reflect.construct', 'promise-try', 'class'})

    def test_auxiliary_authentication_refuses_self_consistent_edits_omissions_and_extras(self):
        auxiliary = sorted(self.manifest['inventory_proof']['auxiliary_blobs'])
        self.assertEqual(len(auxiliary), 10)
        for name in auxiliary:
            for operation in ('edit-and-rehash', 'delete-and-unlist'):
                with self.subTest(path=name, operation=operation), tempfile.TemporaryDirectory() as temporary:
                    directory = Path(temporary) / 'corpus'
                    shutil.copytree(CORPUS, directory)
                    manifest = json.loads((directory / 'manifest.json').read_bytes())
                    if operation == 'edit-and-rehash':
                        replace_file(directory, manifest, name, (directory / name).read_bytes() + b'\n')
                    else:
                        (directory / name).unlink()
                        manifest['files'] = [r for r in manifest['files'] if r['path'] != name]
                    write_manifest(directory, manifest)
                    with self.assertRaises(ValueError):
                        runner.load_corpus(directory, PROFILE)
        for name in ('extra.txt', 'harness/extra.js'):
            with self.subTest(extra=name), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, directory)
                manifest = json.loads((directory / 'manifest.json').read_bytes())
                data = b'// unbound auxiliary\n'
                (directory / name).write_bytes(data)
                manifest['files'].append(dict(path=name, upstream_path=name,
                                              bytes=len(data), sha256=digest(data)))
                write_manifest(directory, manifest)
                with self.assertRaises(ValueError):
                    runner.load_corpus(directory, PROFILE)
        # Rewriting the declared auxiliary authority cannot replace its root-linked tree.
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / 'corpus'
            shutil.copytree(CORPUS, directory)
            manifest = json.loads((directory / 'manifest.json').read_bytes())
            name = 'harness/propertyHelper.js'
            data = (directory / name).read_bytes() + b'\n'
            replace_file(directory, manifest, name, data)
            manifest['inventory_proof']['auxiliary_blobs'][name] = hashlib.sha1(
                b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            write_manifest(directory, manifest)
            with self.assertRaises(ValueError):
                runner.load_corpus(directory, PROFILE)

    def test_sources_and_all_proofs_remain_bound_after_manifest_rewrites(self):
        for row in self.manifest['inventory_proof']['files']:
            with self.subTest(proof=row['path']), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, directory)
                manifest = json.loads((directory / 'manifest.json').read_bytes())
                document = json.loads((directory / row['path']).read_bytes())
                document['sha'] = '0' * 40
                replace_file(directory, manifest, row['path'], json.dumps(document).encode(), proof=True)
                write_manifest(directory, manifest)
                with self.assertRaises(ValueError):
                    runner.load_corpus(directory, PROFILE)
        for operation in ('edit-source-and-rehash', 'delete-source-and-recount',
                          'delete-source-and-patch-count', 'unlisted-source', 'delete-proof-and-unlist',
                          'extra-proof', 'proof-symlink'):
            with self.subTest(operation=operation), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, directory)
                manifest = json.loads((directory / 'manifest.json').read_bytes())
                name = 'test/built-ins/Promise/try/args.js'
                if operation == 'edit-source-and-rehash':
                    replace_file(directory, manifest, name, (directory / name).read_bytes() + b'\n')
                elif operation in ('delete-source-and-recount', 'delete-source-and-patch-count'):
                    (directory / name).unlink()
                    manifest['files'] = [r for r in manifest['files'] if r['path'] != name]
                    manifest['directories']['Promise/try'].remove('args.js')
                    manifest['test_files'] -= 1
                elif operation == 'unlisted-source':
                    (directory / 'test/built-ins/Promise/try/unlisted.js').write_bytes(b'1;\n')
                elif operation == 'delete-proof-and-unlist':
                    proof = manifest['inventory_proof']['files'].pop()
                    (directory / proof['path']).unlink()
                elif operation == 'proof-symlink':
                    proof = manifest['inventory_proof']['files'][0]
                    target = directory / proof['path']
                    saved = Path(temporary) / 'aliased-proof.json'
                    saved.write_bytes(target.read_bytes())
                    target.unlink()
                    target.symlink_to(saved)
                else:
                    path = 'inventory-proof/tree-' + '0' * 40 + '.json'
                    data = b'{}'
                    (directory / path).write_bytes(data)
                    manifest['inventory_proof']['files'].append(dict(path=path, bytes=len(data),
                        sha256=digest(data), url='https://api.github.com/repos/tc39/test262/git/trees/' + '0' * 40))
                write_manifest(directory, manifest)
                if operation == 'delete-source-and-patch-count':
                    narrowed = {'Promise/try': 14}
                    with patch.dict(importer.PROFILES, {PROFILE: narrowed}), \
                            patch.dict(runner.PROFILES, {PROFILE: narrowed}), \
                            patch.object(importer, 'PROMISE_TRY_DIRECTORIES', narrowed), \
                            self.assertRaises(ValueError):
                        runner.load_corpus(directory, PROFILE)
                else:
                    with self.assertRaises(ValueError):
                        runner.load_corpus(directory, PROFILE)


if __name__ == '__main__':
    unittest.main()
