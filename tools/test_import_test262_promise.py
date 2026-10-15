"""Frozen Promise corpus integrity controls; no JS engine or network execution."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer

PROFILE = 'promise-methods-async'
CORPUS = importer.ROOT / 'tests/upstream/test262-promise-methods-async'
ORACLE = Path(__file__).with_name('promise-methods-importer-oracle.json')
PROMISE_TREE = '176043820c3a0729f2da2a047303a984cca036c6'
EXPECTED_DIRECTORIES = {
    'Promise': 58, 'Promise/Symbol.species': 5, 'Promise/prototype': 6,
    'Promise/prototype/catch': 14, 'Promise/prototype/finally': 29,
    'Promise/prototype/then': 75, 'Promise/reject': 15, 'Promise/resolve': 30,
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def proof_reader(altered_route=None, mutate=None, corpus=CORPUS):
    def read(route):
        raw = (corpus / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(raw)
            mutate(value)
            return json.dumps(value).encode()
        return raw
    return read


class PromiseImporter(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), 'd72a5e98e96d2162f1243d79fa93e1aecc4a21554ba54a82529cd7f42f41e7fe')
        self.oracle = json.loads(ORACLE.read_bytes())
        self.expected = self.oracle['expected_manifest']
        guard = patch.object(importer, 'fetch', side_effect=AssertionError('network forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def cached_fetch(self, url):
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        self.assertNotIn('/contents/', url)
        if url.startswith(api):
            return proof_reader()(url[len(api):])
        self.assertTrue(url.startswith(raw), url)
        path = url[len(raw):]
        self.assertIn(path, {row['path'] for row in self.expected['files']})
        return (CORPUS / path).read_bytes()

    def test_exact_complete_profile_manifest_and_every_bound_file(self):
        self.assertEqual(importer.PROFILES[PROFILE], EXPECTED_DIRECTORIES)
        self.assertEqual(importer.PROFILE_ROOTS[PROFILE], 'test/built-ins')
        self.assertIn(PROFILE, importer.TREE_PROFILES)
        self.assertEqual(importer.corpus_name(PROFILE), 'test262-promise-methods-async')
        manifest = json.loads((CORPUS / 'manifest.json').read_bytes())
        self.assertEqual(manifest, self.expected)
        self.assertEqual((manifest['test_files'], len(manifest['files']),
                          len(manifest['inventory_proof']['files'])), (232, 240, 13))
        self.assertEqual(sum(row['bytes'] for row in manifest['inventory_proof']['files']), 86344)
        for row in manifest['files'] + manifest['inventory_proof']['files']:
            data = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(data), digest(data)), (row['bytes'], row['sha256']), row['path'])
        listings, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        self.assertEqual({name: sorted(row['name'] for row in rows) for name, rows in listings.items()},
                         manifest['directories'])
        self.assertEqual(description, {k: v for k, v in manifest['inventory_proof'].items() if k != 'files'})
        self.assertEqual(set(proof), {row['path'] for row in manifest['inventory_proof']['files']})
        self.assertEqual(set(description['auxiliary_blobs']), {
            'LICENSE', 'INTERPRETING.md', 'harness/assert.js', 'harness/sta.js',
            'harness/doneprintHandle.js', 'harness/isConstructor.js',
            'harness/promiseHelper.js', 'harness/propertyHelper.js'})

    def test_async_class_and_cross_realm_inputs_are_retained_without_outcome_filtering(self):
        counts = dict(async_files=0, sync_files=0, async_modes=0, sync_modes=0,
                      class_files=0, cross_realm_files=0, bytes=0)
        helpers = {'assert.js', 'sta.js'}
        for row in self.expected['files']:
            if not row['path'].startswith('test/'):
                continue
            self.assertNotIn('_FIXTURE', Path(row['path']).name)
            data = (CORPUS / row['path']).read_bytes()
            metadata = importer.parse_metadata(data.decode('utf-8'))
            self.assertIsNone(metadata['negative'])
            flags, features = metadata['flags'], metadata['features']
            modes = 1 if 'noStrict' in flags or 'onlyStrict' in flags else 2
            kind = 'async' if 'async' in flags else 'sync'
            counts[kind + '_files'] += 1
            counts[kind + '_modes'] += modes
            counts['class_files'] += 'class' in features
            counts['cross_realm_files'] += 'cross-realm' in features
            counts['bytes'] += len(data)
            helpers.update(metadata['includes'])
            if kind == 'async':
                helpers.add('doneprintHandle.js')
        self.assertEqual(counts, dict(async_files=113, sync_files=119, async_modes=222,
                                     sync_modes=236, class_files=7, cross_realm_files=1, bytes=236187))
        self.assertEqual(helpers, {'assert.js', 'sta.js', 'doneprintHandle.js',
                                   'isConstructor.js', 'promiseHelper.js', 'propertyHelper.js'})

    def test_each_proof_and_complete_selected_directory_is_authenticated(self):
        for row in self.expected['inventory_proof']['files']:
            route = row['url'].split('/git/', 1)[1]
            with self.subTest(path=row['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'omitted': lambda d: d['tree'].pop(),
            'replacement': lambda d: d['tree'][0].update(sha='0' * 40),
            'renamed': lambda d: d['tree'][0].update(path='replacement.js'),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'unsafe': lambda d: d['tree'][0].update(path='../outside.js'),
        }
        for name, edit in edits.items():
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + PROMISE_TREE, edit))
        for directories in ({}, dict(EXPECTED_DIRECTORIES, Promise=57),
                            dict(EXPECTED_DIRECTORIES, **{'Promise/all': 98})):
            with patch.dict(importer.PROFILES, {PROFILE: directories}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_reproduces_manifest_and_original_proof_source_bytes(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, PROFILE)
            self.assertEqual(json.loads((output / 'manifest.json').read_bytes()), self.expected)
            for row in self.expected['files'] + self.expected['inventory_proof']['files']:
                self.assertEqual((output / row['path']).read_bytes(), (CORPUS / row['path']).read_bytes())

    def test_raw_source_and_all_eight_auxiliary_blob_mutations_are_refused_before_output(self):
        names = ['test/built-ins/Promise/name.js'] + sorted(self.expected['inventory_proof']['auxiliary_blobs'])
        for name in names:
            def changed(url):
                data = self.cached_fetch(url)
                return data + b'\n' if url.endswith('/' + name) else data
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=changed):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), PROFILE)
                self.assertEqual(list(Path(temporary).iterdir()), [])
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch), patch.object(
                importer, 'PROMISE_METHOD_HELPERS', {'assert.js', 'sta.js'}), self.assertRaisesRegex(ValueError, 'harness'):
            importer.import_corpus(Path(temporary), PROFILE)

    def test_all_prior_profile_tables_and_nonrecursive_proof_description_remain_exact(self):
        old = {'profiles': {k: v for k, v in importer.PROFILES.items() if k != PROFILE},
               'roots': {k: v for k, v in importer.PROFILE_ROOTS.items() if k != PROFILE},
               'tree_profiles': sorted(importer.TREE_PROFILES - {PROFILE})}
        self.assertEqual(len(old['profiles']), 53)
        self.assertEqual(digest(canonical(old)), 'e51b4b6882c1163b10fa3eb84321fac4b923acadc8a29a41b648c0d399a502e6')
        corpus = importer.ROOT / 'tests/upstream/test262-array-find'
        manifest = json.loads((corpus / 'manifest.json').read_bytes())
        listings, proof, description = importer.profile_tree_inventory('array-find', proof_reader(corpus=corpus))
        self.assertEqual(description, {k: v for k, v in manifest['inventory_proof'].items() if k != 'files'})
        self.assertEqual({name: sorted(row['name'] for row in rows) for name, rows in listings.items()},
                         manifest['directories'])
        for row in manifest['inventory_proof']['files']:
            self.assertEqual(proof[row['path']], (corpus / row['path']).read_bytes())


if __name__ == '__main__':
    unittest.main()
