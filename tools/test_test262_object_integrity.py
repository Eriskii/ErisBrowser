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


PROFILE = 'object-integrity'
CORPUS = runner.ROOT / 'tests/upstream/test262-object-integrity'


class ObjectIntegrityCorpusTests(unittest.TestCase):
    def test_complete_inventory_matches_frozen_before_case_identities(self):
        with patch.object(importer, 'fetch', side_effect=AssertionError('offline proof replay fetched')):
            manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'Object/seal': 94, 'Object/freeze': 53, 'Object/isSealed': 33, 'Object/isFrozen': 59})
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (239, 474, []))
        self.assertEqual(manifest_hash, '602023dfe820d591c4a8fc8d83a9e4680012f4b15083f855f84da427539c988d')
        self.assertEqual(runner.digest(json.dumps([(c['id'], c['case_sha256']) for c in cases],
                                                  separators=(',', ':')).encode()),
                         '5721ff7a59f4c97cd2cec713766441262e2cf24d5f5f6a745d8c759fe60ac575')
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 237)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 237)
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(len(data) for name, data in files.items() if name.startswith('test/')), 157276)
        self.assertEqual(sum(map(len, files.values())), 207345)
        self.assertEqual(len(manifest['inventory_proof']['files']), 9)
        self.assertEqual(sum(entry['bytes'] for entry in manifest['inventory_proof']['files']), 95184)
        self.assertEqual({name for name in files if name.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js',
            'harness/resizableArrayBufferUtils.js'})
        self.assertEqual(runner.OBJECT_INTEGRITY_FEATURES, runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES)
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.OBJECT_INTEGRITY_FEATURES)]
        self.assertEqual(len(excluded), 26)
        self.assertTrue(all(set(c['metadata']['features']) & {
            'Proxy', 'resizable-arraybuffer', 'AggregateError', 'FinalizationRegistry',
            'SharedArrayBuffer', 'WeakRef'} for c in excluded))
        for suffix in ('seal-date.js', 'seal-map.js', 'seal-proxy.js', 'seal-asyncfunction.js', 'seal-uint8array.js'):
            selected = [c for c in cases if c['file'].endswith('/' + suffix)]
            self.assertEqual(len(selected), 2)
            self.assertTrue(all(runner.unsupported_reason(c, runner.OBJECT_INTEGRITY_FEATURES) is None for c in selected))

    def test_complete_proofs_reject_truncation_omission_and_replacement(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        route = 'trees/' + manifest['inventory_proof']['directory_trees']['Object/seal']
        for mutate in (lambda d: d.update(truncated=True), lambda d: d['tree'].pop(),
                       lambda d: d['tree'][0].update(path='replacement.js')):
            def read(actual):
                raw = (CORPUS / importer.tree_proof_path(actual)).read_bytes()
                if actual == route:
                    value = json.loads(raw)
                    mutate(value)
                    return json.dumps(value).encode()
                return raw
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                importer.git_tree_inventory(importer.OBJECT_INTEGRITY_DIRECTORIES, 'test/built-ins', read)

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

    def test_pairs_have_success_guards_and_identical_setup(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(len(captured), 224)
        added = captured[32:]
        for case in added:
            source = case['source']
            self.assertTrue(source.startswith(b'var integrity=Object.'))
            self.assertIn(b"assert.sameValue(typeof integrity,'function');", source)
            method = case['id'].split('object-integrity-', 1)[1].split('-', 1)[0]
            guard_end = (b"assert.sameValue(Object.getOwnPropertyDescriptor(smoke,'x').writable," if method in ('seal', 'freeze')
                         else b'assert.sameValue(integrity(emptySmoke),true);')
            self.assertIn(guard_end, source)
            if b'assert.throws' in source:
                self.assertLess(source.index(guard_end), source.index(b'assert.throws'))
            if 'property-metadata' in case['id']:
                self.assertEqual(source.count(b'{restore:true}'), 3)
                self.assertIn('propertyHelper.js', case['metadata']['includes'])
            if '-mapped-arguments' in case['id']:
                self.assertIn(b'Function(', source)
            if '-unmapped-arguments' in case['id']:
                self.assertIn(b"function probe(a){'use strict';", source)
        for good, bad in zip(added[::2], added[1::2]):
            self.assertEqual(good['mode'], bad['mode'])
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
        for method in ('seal', 'freeze', 'isSealed', 'isFrozen'):
            selected = [c for c in added if ('object-integrity-' + method + '-') in c['id']]
            self.assertEqual(len(selected), 48)
            self.assertEqual(sum(c['mode'] == 'strict' for c in selected), 24)

    def test_disabled_assertions_and_incidental_errors_cannot_verify_profile(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(files, Path('/fake'), 1)
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertEqual(actual[:32], core)
        self.assertEqual(sum(c['verified'] for c in actual[32:]), 96)
        self.assertFalse(all(c['verified'] for c in actual))
        for kind in ('TypeError', 'RangeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', kind), b'')):
                actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
            self.assertFalse(any(c['verified'] for c in actual[32:]), kind)
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'Test262Error'), b'')):
            actual = runner.harness_preflight(files, Path('/fake'), 1, PROFILE)
        self.assertTrue(all(c['verified'] == (c['expected'] == 'failed') for c in actual[32:]))
        self.assertFalse(all(c['verified'] for c in actual))

    def test_all_33_prior_source_policy_exclusion_and_control_contracts_unchanged(self):
        # Iteration profiles postdate this unchanged historical snapshot.
        contract = capture_contracts(excluded={'typedarray-reverse', 'typedarray-fill', 'typedarray-search', 'typedarray-views', 'reflect-properties', 'typedarray-foundation', 'object-has-own', 'object-is', 'data-view', 'array-buffer', 'array-concat', 'array-splice', 'array-from', 'for-of', 'core-iterators', PROFILE, 'array-find', 'date'})
        self.assertEqual(contract['counts'], dict(profiles=33, cases=14304, preflights=2740))
        self.assertEqual(runner.digest(canonical(contract)),
                         '36387b1286227b77786f6f55a54f0a84b0eed24a1ec97bb036f814c552b023bd')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode()


def digest(value):
    return runner.digest(value)


def capture_contracts(excluded=()):
    unknown = set(excluded) - set(runner.PROFILES)
    if unknown:
        raise ValueError('cannot exclude unknown profiles: ' + ', '.join(sorted(unknown)))
    profiles = {}
    # A second guard prevents any future direct invocation from escaping capture.
    with patch.object(runner, 'bounded_process', side_effect=AssertionError('actual adapter execution forbidden')):
        for name in sorted(set(runner.PROFILES) - set(excluded)):
            manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT/'tests/upstream'/runner.corpus_name(name), name)
            features = runner.PROFILE_FEATURES[name]
            policy = dict(format=2, supported_features=sorted(features),
                          negative_intrinsic_errors=sorted(runner.INTRINSIC_ERRORS), strict=True,
                          modules=False, async_completion=False, host_hooks=False,
                          timeout_seconds=3.0)
            captured = {}
            def fake(case, binary, timeout, supported_features=runner.SUPPORTED_FEATURES):
                key = (case['id'], case['mode'])
                if key in captured:
                    raise AssertionError('duplicate generated control identity: ' + repr(key))
                captured[key] = dict(id=case['id'], mode=case['mode'],
                    source=case['source'].decode('utf-8'), source_sha256=runner.digest(case['source']),
                    case_sha256=case['case_sha256'], metadata=case['metadata'],
                    harness=[dict(name=n, sha256=runner.digest(data)) for n,data in case['harness']],
                    execution_features=sorted(supported_features), timeout_seconds=timeout,
                    unsupported_reason=runner.unsupported_reason(case, supported_features))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=fake):
                outcomes = runner.harness_preflight(files, Path('/capture-only-no-adapter'), 3.0, name)
            controls = []
            used = set()
            for outcome in outcomes:
                key = ('harness-preflight:' + outcome['name'], outcome['result']['mode'])
                item = dict(captured[key], name=outcome['name'], expected=outcome['expected'])
                if 'prerequisite' in outcome:
                    item['prerequisite'] = {k:outcome['prerequisite'][k] for k in
                        ('name', 'mode', 'case_sha256', 'source_sha256', 'expected')}
                item['source_expected_sha256'] = digest(canonical(dict(
                    source_sha256=item['source_sha256'], mode=item['mode'], expected=item['expected'],
                    prerequisite=item.get('prerequisite'))))
                controls.append(item)
                used.add(key)
            if used != set(captured) or len(controls) != len(captured):
                raise AssertionError('generated control inventory mismatch')
            profiles[name] = dict(
                revision=manifest['revision'], manifest_sha256=manifest_hash,
                corpus_files={path:dict(bytes=len(raw), sha256=runner.digest(raw))
                              for path,raw in sorted(files.items())},
                fixtures=fixtures,
                inventory_proof_files=(manifest.get('inventory_proof') or {}).get('files', []),
                policy=policy, runner_policy_sha256=runner.digest(json.dumps(policy,sort_keys=True).encode()),
                cases={case['id']:dict(file=case['file'], mode=case['mode'],
                    case_sha256=case['case_sha256'], source_sha256=runner.digest(case['source']),
                    metadata=case['metadata'],
                    harness=[dict(name=n, sha256=runner.digest(raw)) for n,raw in case['harness']],
                    unsupported_reason=runner.unsupported_reason(case, features)) for case in cases},
                preflights=controls)
    return dict(format=1, adapter_execution=False,
                counts=dict(profiles=len(profiles), cases=sum(len(p['cases']) for p in profiles.values()),
                            preflights=sum(len(p['preflights']) for p in profiles.values())),
                profiles=profiles)

