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


PROFILE = 'date'
CORPUS = runner.ROOT / 'tests/upstream/test262-date'


class DateCorpusTests(unittest.TestCase):
    def test_complete_recursive_inventory_modes_helpers_and_exact_policy(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline replay fetched')):
            manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         importer.DATE_DIRECTORIES)
        self.assertEqual((len(manifest['directories']), manifest['test_files'], len(cases), fixtures),
                         (51, 594, 1188, []))
        self.assertEqual(manifest_hash, '41195dabf14aba17fadbe6e1a9412f6ac70b660f6cd73508621874e2f94c19c2')
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases],
                                                  separators=(',', ':')).encode()),
                         'd4d054267aa2d73510caa8b1594e4ee45f78ecef1a97c77d985cd598038a384e')
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 594)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 594)
        self.assertTrue(all(c['metadata']['negative'] is None and not c['metadata']['flags'] for c in cases))
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 678607)
        self.assertEqual(sum(map(len, files.values())), 726353)
        proof = manifest['inventory_proof']
        self.assertEqual((proof['method'], proof['subtree'], proof['subtree_tree']),
                         ('complete-root-linked-recursive-git-subtree', 'test/built-ins/Date', importer.DATE_TREE))
        self.assertEqual((len(proof['files']), sum(item['bytes'] for item in proof['files'])), (6, 182419))
        self.assertEqual(len(proof['directory_trees']), 51)
        self.assertEqual({name for name in files if name.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js', 'harness/propertyHelper.js',
            'harness/isConstructor.js', 'harness/assertRelativeDateMs.js', 'harness/dateConstants.js'})
        self.assertEqual(runner.DATE_FEATURES, runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES)
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.DATE_FEATURES)]
        self.assertEqual(len(excluded), 22)
        self.assertTrue(all(set(c['metadata']['features']) & {'cross-realm', 'Temporal'} for c in excluded))
        # Untagged for-of and missing Date prerequisites are retained as executable cases.
        for suffix in ('Date/year-zero.js', 'Date/parse/year-zero.js'):
            selected = [c for c in cases if c['file'].endswith(suffix)]
            self.assertEqual(len(selected), 2)
            self.assertTrue(all(runner.unsupported_reason(c, runner.DATE_FEATURES) is None for c in selected))

    def test_recursive_proof_rejects_truncation_omissions_replacements_and_paths(self):
        route = f'trees/{importer.DATE_TREE}?recursive=1'
        original = json.loads((CORPUS / importer.tree_proof_path(route)).read_bytes())
        blob_index = next(i for i, item in enumerate(original['tree']) if item['type'] == 'blob' and '/' in item['path'])
        tree_index = next(i for i, item in enumerate(original['tree']) if item['type'] == 'tree')
        mutations = {
            'truncated-response': lambda d: d.update(truncated=True),
            'missing-nested-source': lambda d: d['tree'].pop(blob_index),
            'same-count-source-replacement': lambda d: d['tree'][blob_index].update(path='replacement.js'),
            'changed-subtree-hash': lambda d: d['tree'][tree_index].update(sha='0' * 40),
            'omitted-directory': lambda d: d['tree'].pop(tree_index),
            'extra-directory': lambda d: d['tree'].append(dict(path='extra', mode='040000', type='tree', sha='0' * 40)),
            'duplicate-path': lambda d: d['tree'].append(copy.deepcopy(d['tree'][blob_index])),
            'parent-path': lambda d: d['tree'][blob_index].update(path='../escape.js'),
            'overdeep-path': lambda d: d['tree'][blob_index].update(path='a/' * 8 + 'test.js'),
            'overlong-path': lambda d: d['tree'][blob_index].update(path='a' * 1025 + '.js'),
            'too-many-entries': lambda d: d.update(tree=d['tree'] * 7),
            'oversized-source': lambda d: d['tree'][blob_index].update(size=importer.MAX_FILE + 1),
            'boolean-source-size': lambda d: d['tree'][blob_index].update(size=True),
            'aggregate-source-limit': lambda d: [item.update(size=4096) for item in d['tree'] if item['type'] == 'blob'],
        }
        for name, mutate in mutations.items():
            def read(actual):
                raw = (CORPUS / importer.tree_proof_path(actual)).read_bytes()
                if actual == route:
                    value = json.loads(raw)
                    mutate(value)
                    return json.dumps(value).encode()
                return raw
            with self.subTest(mutation=name), self.assertRaises(ValueError):
                importer.date_tree_inventory(read)

    def test_root_links_proof_bytes_and_explicit_selection_are_fail_closed(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            def read(route):
                path = importer.tree_proof_path(route)
                raw = (CORPUS / path).read_bytes()
                if path == entry['path']:
                    value = json.loads(raw)
                    value['sha'] = '0' * 40
                    return json.dumps(value).encode()
                return raw
            with self.subTest(proof=entry['path']), self.assertRaises(ValueError):
                importer.date_tree_inventory(read)
        def read(route):
            return (CORPUS / importer.tree_proof_path(route)).read_bytes()
        with patch.object(importer, 'DATE_DIRECTORIES', {'Date': 594}), self.assertRaises(ValueError):
            importer.date_tree_inventory(read)
        with patch.object(importer, 'MAX_FILE', 1024), self.assertRaises(ValueError):
            importer.date_tree_inventory(read)
        with self.assertRaises(ValueError):
            importer.tree_proof_path(f'trees/{importer.REVISION_TREE}?recursive=1')

    def test_import_roundtrip_retains_original_proof_and_all_source_bytes(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'
        def fetch(url):
            self.assertNotIn('/contents/', url)
            if url.startswith(api):
                return (CORPUS / importer.tree_proof_path(url[len(api):])).read_bytes()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), PROFILE)
            actual, retained, imported, _, _ = runner.load_corpus(Path(temporary), PROFILE)
            self.assertEqual(actual, manifest)
            self.assertEqual(retained, files)
            self.assertEqual([c['case_sha256'] for c in imported], [c['case_sha256'] for c in cases])

    def test_updated_manifest_cannot_bless_changed_source_helper_or_proof(self):
        mutations = ['test/built-ins/Date/prototype/getTime/name.js', 'harness/assertRelativeDateMs.js',
                     'harness/propertyHelper.js', 'LICENSE', 'inventory-proof/recursive-' + importer.DATE_TREE + '.json']
        for name in mutations:
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, destination)
                target = destination / name
                if name.startswith('inventory-proof/'):
                    value = json.loads(target.read_bytes())
                    value['tree'].pop()
                    data = json.dumps(value).encode()
                else:
                    data = target.read_bytes() + b'\n'
                target.write_bytes(data)
                manifest = json.loads((destination / 'manifest.json').read_bytes())
                entries = manifest['inventory_proof']['files'] if name.startswith('inventory-proof/') else manifest['files']
                entry = next(item for item in entries if item['path'] == name)
                entry.update(bytes=len(data), sha256=runner.digest(data))
                (destination / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(destination, PROFILE)

    def test_guarded_control_pairs_have_identical_setup_and_bound_positive_partners(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                        source_sha256=runner.digest(case['source']))
        with patch.object(runner, 'run_case', side_effect=capture):
            outcomes = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(len(captured), 340)
        added = captured[32:]
        self.assertEqual(sum(c['mode'] == 'strict' for c in added), 154)
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
            self.assertTrue(good['source'].startswith(b"assert.sameValue(typeof Date,'function');"))
            guard = b"assert.sameValue(new Date(0).toISOString(),'1970-01-01T00:00:00.000Z');"
            self.assertIn(guard, good['source'])
            if b'assert.throws' in good['source']:
                self.assertLess(good['source'].index(guard), good['source'].index(b'assert.throws'))
            if 'property-' in good['id']:
                self.assertIn('propertyHelper.js', good['metadata']['includes'])
                self.assertGreaterEqual(good['source'].count(b'{restore:true}'), 3)
        for good, bad in zip(outcomes[32::2], outcomes[33::2]):
            self.assertEqual(bad['prerequisite'], dict(name=good['name'], mode=good['result']['mode'],
                case_sha256=good['result']['case_sha256'], source_sha256=good['result']['source_sha256'],
                expected='passed', verified=True))

    def test_missing_feature_incidental_error_and_disabled_assertions_cannot_verify_controls(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(files, Path('/fake'), 1)
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(actual[:32], core)
        self.assertEqual(sum(item['verified'] for item in actual[32:]), 154)
        self.assertFalse(all(item['verified'] for item in actual))
        for kind in ('TypeError', 'RangeError', 'ReferenceError', 'SyntaxError', 'Test262Error'):
            with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(item['verified'] for item in actual[32:]), kind)
            for item in actual[32:]:
                if item['expected'] == 'failed':
                    self.assertFalse(item['prerequisite']['verified'])

    def test_all_35_prior_source_policy_case_and_control_contracts_unchanged(self):
        # Iteration profiles postdate this unchanged historical snapshot.
        contract = capture_contracts(excluded={'typedarray-to-reversed', 'typedarray-reverse', 'typedarray-fill', 'typedarray-search', 'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', 'object-is', 'data-view', 'array-buffer', 'array-concat', 'array-splice', 'array-from', 'for-of', 'core-iterators', PROFILE})
        self.assertEqual(contract['counts'], dict(profiles=35, cases=14958, preflights=3252))
        self.assertEqual(runner.digest(canonical(contract)),
                         'e6a81c9fa4863df4f9797833c184f6ef212a547340ba758af160de44902f2574')


if __name__ == '__main__':
    unittest.main()
