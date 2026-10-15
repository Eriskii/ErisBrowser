"""Frozen AggregateError corpus controls; replay bytes only, no engine or network."""
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

PROFILE = 'aggregate-error'
CORPUS = importer.ROOT / 'tests/upstream/test262-aggregate-error'
ORACLE = Path(__file__).with_name('aggregate-error-importer-oracle.json')
ORACLE_SHA = 'e15f624222ba10f31b36b901c8b6b2e1177f20e1ed2fc011187c3edd8feb4a93'
DIRECTORIES = {'AggregateError': 19, 'AggregateError/prototype': 6}
HELPERS = {'assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js',
           'isConstructor.js', 'promiseHelper.js'}
ROOT_TREE = '29aa43922ae5a34467878bf9f498ffc4a89bdf67'
PROTOTYPE_TREE = 'f1b375f2e936e2329af2c4307260ab896efd7daf'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def proof_reader(changed=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == changed:
            document = json.loads(data)
            mutate(document)
            return json.dumps(document).encode()
        return data
    return read


class AggregateErrorImporterTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.manifest = self.oracle['expected_manifest']
        self.assertEqual(importer.REVISION, self.manifest['revision'])
        self.assertEqual('https://github.com/' + importer.REPOSITORY,
                         self.manifest['repository'])
        guard = patch.object(importer, 'fetch', side_effect=AssertionError('network forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def cached_fetch(self, url):
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        self.assertNotIn('/contents/', url)
        if url.startswith(api):
            self.assertIn(url, {row['url'] for row in self.manifest['inventory_proof']['files']})
            return proof_reader()(url[len(api):])
        self.assertTrue(url.startswith(raw), url)
        path = url[len(raw):]
        self.assertIn(path, {row['path'] for row in self.manifest['files']})
        return (CORPUS / path).read_bytes()

    def test_complete_41_file_replay_matches_frozen_manifest_and_every_byte(self):
        self.assertEqual(importer.AGGREGATE_ERROR_DIRECTORIES, DIRECTORIES)
        self.assertEqual(importer.AGGREGATE_ERROR_HELPERS, HELPERS)
        self.assertEqual(importer.PROFILES[PROFILE], DIRECTORIES)
        self.assertEqual(importer.PROFILE_ROOTS[PROFILE], 'test/built-ins')
        self.assertIn(PROFILE, importer.TREE_PROFILES)
        self.assertEqual(importer.corpus_name(PROFILE), 'test262-aggregate-error')
        manifest_bytes = (CORPUS / 'manifest.json').read_bytes()
        self.assertEqual(digest(manifest_bytes), self.oracle['manifest_sha256'])
        self.assertEqual(json.loads(manifest_bytes), self.manifest)
        self.assertEqual((self.manifest['test_files'], len(self.manifest['files']),
                          len(self.manifest['inventory_proof']['files'])), (25, 33, 7))
        expected_names = {row['path'] for row in self.oracle['all_files']}
        actual_names = {p.relative_to(CORPUS).as_posix() for p in CORPUS.rglob('*') if p.is_file()}
        self.assertEqual(actual_names, expected_names)
        self.assertEqual((len(expected_names), sum(row['bytes'] for row in self.oracle['all_files'])),
                         (41, 124695))
        for row in self.oracle['all_files']:
            data = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(data), digest(data)), (row['bytes'], row['sha256']), row['path'])
        listings, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        self.assertEqual({name: sorted(row['name'] for row in rows) for name, rows in listings.items()},
                         self.manifest['directories'])
        self.assertEqual(description, {k: v for k, v in self.manifest['inventory_proof'].items()
                                       if k != 'files'})
        self.assertEqual(set(proof), {row['path'] for row in self.manifest['inventory_proof']['files']})
        for path, data in proof.items():
            self.assertEqual(data, (CORPUS / path).read_bytes(), path)
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary) / 'corpus'
            importer.import_corpus(output, PROFILE)
            self.assertEqual({p.relative_to(output).as_posix() for p in output.rglob('*') if p.is_file()},
                             expected_names)
            for name in sorted(expected_names):
                self.assertEqual((output / name).read_bytes(), (CORPUS / name).read_bytes(), name)

    def test_all_50_metadata_identities_retain_cross_realm_and_untagged_proxy_sources(self):
        helpers = {'assert.js', 'sta.js'}
        identities, cross_realm, proxy = [], [], []
        source_bytes = 0
        for path, expected in self.oracle['metadata'].items():
            data = (CORPUS / path).read_bytes()
            metadata = importer.parse_metadata(data.decode('utf-8'))
            self.assertEqual(metadata, expected, path)
            self.assertEqual(metadata['flags'], [])
            self.assertEqual(metadata['locale'], [])
            self.assertIsNone(metadata['negative'])
            self.assertNotIn('timeout', metadata)
            source_bytes += len(data)
            helpers.update(metadata['includes'])
            names = list(dict.fromkeys(['assert.js', 'sta.js'] + metadata['includes']))
            harness = [dict(name=name, sha256=digest((CORPUS / 'harness' / name).read_bytes()))
                       for name in names]
            for mode in ('sloppy', 'strict'):
                identity = dict(source_sha256=digest(data), mode=mode, metadata=metadata, harness=harness)
                identities.append(dict(id=path + ':' + mode,
                                       case_sha256=digest(json.dumps(identity, sort_keys=True,
                                                                    ensure_ascii=True).encode())))
            if 'cross-realm' in metadata['features']:
                cross_realm.append(path)
                self.assertIn(b'$262.createRealm()', data)
            if b'new Proxy(' in data:
                proxy.append(path)
                self.assertNotIn('Proxy', metadata['features'])
        self.assertEqual((len(self.oracle['metadata']), source_bytes), (25, 28499))
        self.assertEqual(helpers, HELPERS)
        self.assertEqual(identities, self.oracle['case_identities'])
        self.assertEqual(len({row['id'] for row in identities}), 50)
        self.assertEqual(cross_realm, ['test/built-ins/AggregateError/proto-from-ctor-realm.js'])
        self.assertEqual(proxy, ['test/built-ins/AggregateError/newtarget-proto-custom.js',
                                'test/built-ins/AggregateError/newtarget-proto-fallback.js'])
        self.assertEqual(sum(row['id'].rsplit(':', 1)[0] in cross_realm for row in identities), 2)
        self.assertEqual(sum(row['id'].rsplit(':', 1)[0] in proxy for row in identities), 4)

    def test_all_proofs_and_both_complete_trees_refuse_corruption_or_descendant_omission(self):
        for row in self.manifest['inventory_proof']['files']:
            route = row['url'].split('/git/', 1)[1]
            with self.subTest(proof=row['path']), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        edits = {
            'truncated': lambda d: d.update(truncated=True),
            'omitted': lambda d: d['tree'].pop(),
            'changed-blob': lambda d: d['tree'][0].update(sha='0' * 40),
            'renamed': lambda d: d['tree'][0].update(path='replacement.js'),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'unsafe': lambda d: d['tree'][0].update(path='../outside.js'),
        }
        for tree in (ROOT_TREE, PROTOTYPE_TREE):
            for name, edit in edits.items():
                with self.subTest(tree=tree, mutation=name), self.assertRaises(ValueError):
                    importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + tree, edit))
        def omit_prototype(document):
            self.assertEqual([row['path'] for row in document['tree'] if row['type'] == 'tree'],
                             ['prototype'])
            document['tree'] = [row for row in document['tree'] if row['path'] != 'prototype']
        with self.subTest(mutation='omitted-prototype-child'), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + ROOT_TREE, omit_prototype))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_missing_extra_wrong_count_or_wrong_root_selections_are_refused(self):
        selections = [
            {}, {'AggregateError': 19}, {'AggregateError/prototype': 6},
            dict(DIRECTORIES, AggregateError=18),
            dict(DIRECTORIES, **{'AggregateError/prototype': 5}),
            dict(DIRECTORIES, **{'AggregateError/missing': 1}),
        ]
        for selected in selections:
            with self.subTest(selected=selected), patch.dict(importer.PROFILES, {PROFILE: selected}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())
        with patch.dict(importer.PROFILE_ROOTS, {PROFILE: 'test/language'}), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_every_test_helper_and_legal_blob_mutation_is_refused_before_publication(self):
        names = [row['path'] for row in self.manifest['files']]
        self.assertEqual((len(names), sum(p.startswith('test/') for p in names),
                          sum(p.startswith('harness/') for p in names)), (33, 25, 6))
        for name in names:
            def changed(url):
                data = self.cached_fetch(url)
                return data + b'\n' if url.endswith('/' + name) else data
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=changed):
                output = Path(temporary) / 'corpus'
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(output, PROFILE)
                self.assertFalse(output.exists())

    def test_requested_helper_closure_refuses_every_missing_helper_and_one_extra(self):
        self.assertEqual(importer.AGGREGATE_ERROR_HELPERS, HELPERS)
        selections = [HELPERS - {name} for name in sorted(HELPERS)]
        selections.append(HELPERS | {'doneprintHandle.js'})
        for selected in selections:
            with self.subTest(helpers=sorted(selected)), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=self.cached_fetch), patch.object(
                    importer, 'AGGREGATE_ERROR_HELPERS', selected):
                output = Path(temporary) / 'corpus'
                with self.assertRaisesRegex(ValueError, 'harness|helper'):
                    importer.import_corpus(output, PROFILE)
                self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
