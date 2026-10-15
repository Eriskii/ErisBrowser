"""Frozen Promise.withResolvers import controls: retained bytes, no engine or network."""
import contextlib
from collections import Counter
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer

PROFILE = 'promise-with-resolvers'
CORPUS = importer.ROOT / 'tests/upstream/test262-promise-with-resolvers'
ORACLE = Path(__file__).with_name('test262-promise-with-resolvers-import-oracle.json')
ORACLE_SHA = '5b88482eae075e32cf5f0dd4221eb68c14555a7937cc1d964928055ce34b3d56'
DIRECTORIES = {'Promise/withResolvers': 6}
HELPERS = {'assert.js', 'sta.js', 'propertyHelper.js'}
WITH_RESOLVERS_TREE = '062393e615d0cf97a0252e6a93923bd91444a455'


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


class PromiseWithResolversImporterTests(unittest.TestCase):
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

    def test_complete_19_file_replay_matches_frozen_manifest_and_every_byte(self):
        self.assertEqual(importer.PROMISE_WITH_RESOLVERS_DIRECTORIES, DIRECTORIES)
        self.assertEqual(importer.PROMISE_WITH_RESOLVERS_HELPERS, HELPERS)
        self.assertEqual(importer.PROFILES[PROFILE], DIRECTORIES)
        self.assertEqual(importer.PROFILE_ROOTS[PROFILE], 'test/built-ins')
        self.assertIn(PROFILE, importer.TREE_PROFILES)
        self.assertEqual(importer.corpus_name(PROFILE), 'test262-promise-with-resolvers')
        manifest_bytes = (CORPUS / 'manifest.json').read_bytes()
        self.assertEqual(digest(manifest_bytes), self.oracle['manifest_sha256'])
        self.assertEqual(json.loads(manifest_bytes), self.manifest)
        self.assertEqual((self.manifest['test_files'], len(self.manifest['files']),
                          len(self.manifest['inventory_proof']['files'])), (6, 11, 7))
        expected_names = {row['path'] for row in self.oracle['all_files']}
        self.assertEqual({p.relative_to(CORPUS).as_posix() for p in CORPUS.rglob('*') if p.is_file()},
                         expected_names)
        self.assertEqual((len(expected_names), sum(r['bytes'] for r in self.oracle['all_files'])),
                         (19, 101418))
        for row in self.oracle['all_files']:
            data = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(data), digest(data)), (row['bytes'], row['sha256']), row['path'])
        listings, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        self.assertEqual({k: sorted(r['name'] for r in v) for k, v in listings.items()},
                         self.manifest['directories'])
        self.assertEqual(description, {k: v for k, v in self.manifest['inventory_proof'].items()
                                       if k != 'files'})
        self.assertEqual(set(proof), {r['path'] for r in self.manifest['inventory_proof']['files']})
        for path, data in proof.items():
            self.assertEqual(data, (CORPUS / path).read_bytes(), path)
        with tempfile.TemporaryDirectory() as temporary, patch.object(
                importer, 'fetch', side_effect=self.cached_fetch) as fetch, contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary) / 'corpus'
            importer.import_corpus(output, PROFILE)
            expected_urls = {r['url'] for r in self.manifest['inventory_proof']['files']}
            expected_urls.update(f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/' + r['path']
                                 for r in self.manifest['files'])
            self.assertEqual(len(expected_urls), 18)
            self.assertEqual(Counter(call.args[0] for call in fetch.call_args_list), Counter(expected_urls))
            self.assertEqual({p.relative_to(output).as_posix() for p in output.rglob('*') if p.is_file()},
                             expected_names)
            for name in sorted(expected_names):
                self.assertEqual((output / name).read_bytes(), (CORPUS / name).read_bytes(), name)

    def test_all_12_metadata_identities_retain_modes_features_and_helper_order(self):
        helpers, identities = {'assert.js', 'sta.js'}, []
        source_bytes, features, includes = 0, {}, {}
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
            names = ['assert.js', 'sta.js'] + metadata['includes']
            harness = [dict(name=name, sha256=digest((CORPUS / 'harness' / name).read_bytes()))
                       for name in names]
            for mode in ('sloppy', 'strict'):
                identity = dict(source_sha256=digest(data), mode=mode, metadata=metadata,
                                harness=harness)
                self.assertNotIn('completion_policy', identity)
                identities.append(dict(id=path + ':' + mode,
                    case_sha256=digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode())))
            for feature in metadata['features']:
                features[feature] = features.get(feature, 0) + 1
            for name in metadata['includes']:
                includes[name] = includes.get(name, 0) + 1
        self.assertEqual((len(self.oracle['metadata']), source_bytes), (6, 3708))
        self.assertEqual(helpers, HELPERS)
        self.assertEqual(identities, self.oracle['case_identities'])
        self.assertEqual(len({r['id'] for r in identities}), 12)
        self.assertEqual(features, {'promise-with-resolvers': 6, 'class': 1})
        self.assertEqual(includes, {'propertyHelper.js': 1})

    def test_all_proofs_refuse_corruption_and_selected_subtree_has_no_omitted_children(self):
        self.assertEqual(importer.PROFILES[PROFILE], DIRECTORIES)
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
            'new-child': lambda d: d['tree'].append(dict(path='child', type='tree',
                                                        mode='040000', sha='0' * 40)),
        }
        for name, edit in edits.items():
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader('trees/' + WITH_RESOLVERS_TREE, edit))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())
        # Isolate the profile's descendant postcondition after the shared authenticated walker.
        listing, proof, description = importer.profile_tree_inventory(PROFILE, proof_reader())
        altered = dict(proof)
        key = importer.tree_proof_path('trees/' + WITH_RESOLVERS_TREE)
        tree = json.loads(altered[key])
        tree['tree'].append(dict(path='child', type='tree', mode='040000', sha='0' * 40))
        altered[key] = json.dumps(tree).encode()
        with patch.object(importer, 'git_tree_inventory', return_value=(listing, altered, description)), \
                self.assertRaisesRegex(ValueError, 'descendant'):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_frozen_selection_refusals_and_all_59_old_profile_tables_are_preserved(self):
        self.assertEqual(importer.PROMISE_WITH_RESOLVERS_DIRECTORIES, DIRECTORIES)
        for selected in [{}, {'Promise/withResolvers': 5}, {'Promise/withResolvers': 7},
                         {'Promise/resolve': 6}, {'Promise/withResolvers': 6, 'Promise/withResolvers/child': 1}]:
            with self.subTest(selected=selected), patch.dict(importer.PROFILES, {PROFILE: selected}), \
                    self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())
        with patch.dict(importer.PROFILE_ROOTS, {PROFILE: 'test/language'}), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())
        with patch.object(importer, 'PROMISE_WITH_RESOLVERS_DIRECTORIES', {'Promise/withResolvers': 5}), \
                patch.dict(importer.PROFILES, {PROFILE: {'Promise/withResolvers': 5}}), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())
        tables = {'PROFILES': {k: v for k, v in importer.PROFILES.items() if k != PROFILE},
                  'PROFILE_ROOTS': {k: v for k, v in importer.PROFILE_ROOTS.items() if k != PROFILE},
                  'TREE_PROFILES': sorted(importer.TREE_PROFILES - {PROFILE})}
        self.assertEqual(len(tables['PROFILES']), self.oracle['old_profile_count'])
        self.assertEqual(self.oracle['old_profile_count'], 59)
        for name, values in tables.items():
            self.assertEqual(digest(json.dumps(values, sort_keys=True).encode()),
                             self.oracle['old_tables_sha256'][name], name)

    def test_every_source_helper_and_legal_blob_mutation_is_refused_before_publication(self):
        self.assertEqual(importer.PROMISE_WITH_RESOLVERS_HELPERS, HELPERS)
        names = [row['path'] for row in self.manifest['files']]
        self.assertEqual((len(names), sum(p.startswith('test/') for p in names),
                          sum(p.startswith('harness/') for p in names)), (11, 6, 3))
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
        targets = ['test/built-ins/Promise/withResolvers/result.js', 'harness/sta.js',
                   'inventory-proof/commit.json']
        for name in targets:
            url = (next(r['url'] for r in self.manifest['inventory_proof']['files'] if r['path'] == name)
                   if name.startswith('inventory-proof/') else
                   f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/{name}')
            def missing(requested):
                if requested == url:
                    raise FileNotFoundError(name)
                return self.cached_fetch(requested)
            with self.subTest(missing=name), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=missing):
                output = Path(temporary) / 'corpus'
                with self.assertRaises(FileNotFoundError):
                    importer.import_corpus(output, PROFILE)
                self.assertFalse(output.exists())

    def test_requested_helper_closure_refuses_every_missing_helper_and_one_extra(self):
        self.assertEqual(importer.PROMISE_WITH_RESOLVERS_HELPERS, HELPERS)
        selections = [HELPERS - {name} for name in sorted(HELPERS)]
        selections.append(HELPERS | {'compareArray.js'})
        for selected in selections:
            with self.subTest(helpers=sorted(selected)), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=self.cached_fetch), patch.object(
                    importer, 'PROMISE_WITH_RESOLVERS_HELPERS', selected):
                output = Path(temporary) / 'corpus'
                with self.assertRaisesRegex(ValueError, 'harness|helper'):
                    importer.import_corpus(output, PROFILE)
                self.assertFalse(output.exists())


if __name__ == '__main__':
    unittest.main()
