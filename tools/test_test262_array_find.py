import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner
from test_test262_conformance import response
from test_test262_object_integrity import canonical, capture_contracts


PROFILE = 'array-find'
CORPUS = runner.ROOT / 'tests/upstream/test262-array-find'


class ArrayFindCorpusTests(unittest.TestCase):
    def test_complete_inventory_matches_frozen_before_case_identities(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline proof replay fetched')):
            manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()}, {
            'Array/prototype/find': 23, 'Array/prototype/findIndex': 23,
            'Array/prototype/findLast': 24, 'Array/prototype/findLastIndex': 24})
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (94, 180, []))
        self.assertEqual(manifest_hash, '09fba3be727b98e9fcdc54343abd7725f1392244ff1bdec1c0afc3e30b5c0eba')
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases],
                                                  separators=(',', ':')).encode()),
                         'fcfbb3348fd6f25f52219d46886e4e58b5c8e654f8bffbb963f4ef5e1a82ff24')
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 90)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 90)
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 111737)
        self.assertEqual(sum(map(len, files.values())), 178332)
        self.assertEqual(len(manifest['inventory_proof']['files']), 10)
        self.assertEqual(sum(entry['bytes'] for entry in manifest['inventory_proof']['files']), 66085)
        self.assertEqual({name for name in files if name.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js',
            'harness/testTypedArray.js', 'harness/resizableArrayBufferUtils.js'})
        self.assertEqual(runner.ARRAY_FIND_FEATURES, runner.ARRAY_PREDICATE_FEATURES | {'array-find-from-last'})
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.ARRAY_FIND_FEATURES)]
        self.assertEqual(len(excluded), 32)
        self.assertTrue(all('resizable-arraybuffer' in c['metadata']['features'] for c in excluded))
        self.assertEqual(sum('TypedArray' in c['metadata']['features'] for c in excluded), 8)
        self.assertEqual(sum('array-find-from-last' in c['metadata']['features'] for c in cases), 76)

    def test_complete_proofs_reject_truncation_omission_replacement_and_blob_change(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for tree in manifest['inventory_proof']['directory_trees'].values():
            route = 'trees/' + tree
            for mutate in (lambda d: d.update(truncated=True), lambda d: d['tree'].pop(),
                           lambda d: d['tree'][0].update(path='replacement.js'),
                           lambda d: d['tree'][0].update(sha='0' * 40)):
                def read(actual):
                    raw = (CORPUS / importer.tree_proof_path(actual)).read_bytes()
                    if actual == route:
                        value = json.loads(raw)
                        mutate(value)
                        return json.dumps(value).encode()
                    return raw
                with self.subTest(tree=tree, mutation=mutate), self.assertRaises(ValueError):
                    importer.git_tree_inventory(importer.ARRAY_FIND_DIRECTORIES, 'test/built-ins', read)

    def test_import_roundtrip_preserves_complete_verified_tree_selection(self):
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
        def corrupt_source(url):
            value = fetch(url)
            return value + b'\n' if '/test/built-ins/' in url else value
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=corrupt_source), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
            importer.import_corpus(Path(temporary), PROFILE)

    def test_pairs_have_success_guards_and_identical_setup(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(len(captured), 288)
        added = captured[32:]
        for case in added:
            source = case['source']
            self.assertTrue(source.startswith(b'var find=Array.prototype.'))
            self.assertIn(b"assert.sameValue(typeof find,'function');", source)
            guard_end = b'assert.sameValue(smokeCalls,2);'
            self.assertIn(guard_end, source)
            if b'assert.throws' in source:
                self.assertLess(source.index(guard_end), source.index(b'assert.throws'))
            if 'property-metadata' in case['id']:
                self.assertEqual(source.count(b'{restore:true}'), 3)
                self.assertIn('propertyHelper.js', case['metadata']['includes'])
            if '-sloppy-thisarg' in case['id']:
                self.assertIn(b'Function(', source)
            if '-strict-thisarg' in case['id']:
                self.assertEqual(source.count(b"'use strict';"), 3)
            if '-captured-value' in case['id']:
                self.assertIn(b'a[0]=after;return true;', source)
                self.assertIn(b'assert.sameValue(seen,before);', source)
            self.assertLess(len(source), 2048)
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
        for method in ('find', 'findIndex', 'findLast', 'findLastIndex'):
            selected = [c for c in added if ('array-find-' + method + '-') in c['id']]
            self.assertEqual(len(selected), 64)
            self.assertEqual(sum(c['mode'] == 'strict' for c in selected), 32)

    def test_disabled_assertions_and_incidental_errors_cannot_verify_profile(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(files, Path('/fake'), 1)
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(actual[:32], core)
        self.assertEqual(sum(c['verified'] for c in actual[32:]), 128)
        self.assertFalse(all(c['verified'] for c in actual))
        for kind in ('TypeError', 'RangeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(c['verified'] for c in actual[32:]), kind)
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'Test262Error'), b'')):
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertTrue(all(c['verified'] == (c['expected'] == 'failed') for c in actual[32:]))
        self.assertFalse(all(c['verified'] for c in actual))

    def test_all_34_prior_source_policy_exclusion_and_control_contracts_unchanged(self):
        # Iteration profiles postdate this unchanged historical snapshot.
        contract = capture_contracts(excluded={'typedarray-to-reversed', 'typedarray-reverse', 'typedarray-fill', 'typedarray-search', 'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', 'object-is', 'data-view', 'array-buffer', 'array-concat', 'array-splice', 'array-from', 'for-of', 'core-iterators', PROFILE, 'date'})
        self.assertEqual(contract['counts'], dict(profiles=34, cases=14778, preflights=2964))
        self.assertEqual(runner.digest(canonical(contract)),
                         '5ecae3add38adb9f66b21557c8bfef771f6e510390ed1d537c1dac964a58444a')
