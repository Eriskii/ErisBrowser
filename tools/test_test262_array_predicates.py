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


PROFILE = 'array-predicates'
CORPUS = runner.ROOT / 'tests/upstream/test262-array-predicates'


class ArrayPredicateCorpusTests(unittest.TestCase):
    def test_complete_inventory_matches_frozen_before_source_mode_harness_identities(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline proof replay fetched')):
            manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'Array/prototype/every': 218, 'Array/prototype/some': 219})
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (437, 867, []))
        self.assertEqual(manifest_hash, '67cf3fa8c3b006412ee76f9f62e060cd0c938d4c129e8a3ac28d7aa95e2b16a9')
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases],
                                                  separators=(',', ':')).encode()),
                         '0e64b5a2366481dbbc5e5149b420f4052a965ef97f443c182f98f3b3843b8c10')
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 437)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 430)
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 283468)
        self.assertEqual(sum(map(len, files.values())), 350063)
        self.assertEqual(len(manifest['inventory_proof']['files']), 8)
        self.assertEqual(sum(entry['bytes'] for entry in manifest['inventory_proof']['files']), 140701)
        self.assertEqual({name for name in files if name.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js',
            'harness/testTypedArray.js', 'harness/resizableArrayBufferUtils.js'})
        self.assertEqual(runner.ARRAY_PREDICATE_FEATURES, runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES)
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.ARRAY_PREDICATE_FEATURES)]
        self.assertEqual(len(excluded), 16)
        self.assertTrue(all('resizable-arraybuffer' in c['metadata']['features'] for c in excluded))

    def test_complete_tree_proof_rejects_truncation_and_same_count_replacement(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        route = 'trees/' + manifest['inventory_proof']['directory_trees']['Array/prototype/every']
        for mutate in (lambda d: d.update(truncated=True), lambda d: d['tree'].pop(),
                       lambda d: d['tree'][0].update(path='replacement.js')):
            def read(actual):
                raw = (CORPUS / importer.tree_proof_path(actual)).read_bytes()
                if actual == route:
                    data = json.loads(raw)
                    mutate(data)
                    return json.dumps(data).encode()
                return raw
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                importer.git_tree_inventory(importer.ARRAY_PREDICATE_DIRECTORIES, 'test/built-ins', read)

    def test_import_roundtrip_requires_verified_complete_git_trees(self):
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

    def test_pairs_guard_actual_calls_and_differ_only_in_final_assertion(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(len(captured), 128)
        added = captured[32:]
        for case in added:
            source = case['source']
            self.assertTrue(source.startswith(b'var predicate=Array.prototype.'))
            self.assertIn(b"assert.sameValue(typeof predicate,'function');", source)
            self.assertIn(b'assert.sameValue(smokeCalls,2);', source)
            self.assertLess(source.index(b'assert.sameValue(typeof predicate'), source.index(b'predicate.call([1]'))
            if b'assert.throws' in source:
                self.assertLess(source.index(b'assert.sameValue(smokeCalls,2);'), source.index(b'assert.throws'))
            if 'property-metadata' in case['id']:
                self.assertEqual(source.count(b'{restore:true}'), 3)
                self.assertIn('propertyHelper.js', case['metadata']['includes'])
            if 'no-species' in case['id']:
                self.assertIn(b'Symbol.species', source)
                self.assertIn(b"Object.defineProperty(a,'constructor'", source)
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
        self.assertEqual(sum(c['mode'] == 'strict' for c in added), 48)
        self.assertEqual(sum('-every-' in c['id'] for c in added), 48)

    def test_controls_fail_closed_for_disabled_assertions_and_incidental_errors(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(files, Path('/fake'), 1)
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(actual[:32], core)
        self.assertEqual(sum(c['verified'] for c in actual[32:]), 48)
        self.assertFalse(all(c['verified'] for c in actual))
        for kind in ('TypeError', 'RangeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(c['verified'] for c in actual[32:]), kind)
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'Test262Error'), b'')):
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertTrue(all(c['verified'] == (c['expected'] == 'failed') for c in actual[32:]))
        self.assertFalse(all(c['verified'] for c in actual))

    def test_all_32_prior_contracts_and_policies_are_unchanged(self):
        retained = {}
        for name in runner.PROFILES:
            if name in {PROFILE, 'object-integrity'}:
                continue
            _, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(name), name)
            captured = []
            def capture(case, *args):
                captured.append(dict(name=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                                     source_sha256=runner.digest(case['source'])))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=capture):
                runner.harness_preflight(files, Path('/fake'), 3, name)
            retained[name] = dict(manifest_sha256=manifest_hash, cases={c['id']: c['case_sha256'] for c in cases},
                                 preflights=captured, fixtures=fixtures, features=sorted(runner.PROFILE_FEATURES[name]))
        self.assertEqual(len(retained), 32)
        self.assertEqual(sum(len(v['cases']) for v in retained.values()), 13437)
        self.assertEqual(sum(len(v['preflights']) for v in retained.values()), 2612)
        self.assertEqual(runner.digest(json.dumps(retained, sort_keys=True, separators=(',', ':')).encode()),
                         '6ea90ab6bd16182bee8c21e24da87ed0bb0fa3fea1a04edd12566d23a233ec4e')
