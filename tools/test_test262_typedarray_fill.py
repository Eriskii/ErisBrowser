"""Pinned fill inputs and rejection of false semantic control health.

Root runs these after the frozen offline import. Network and adapter execution
are mocked; no test can fall back to a real engine or fetch.
"""
import contextlib
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner

PROFILE = 'typedarray-fill'
CORPUS = runner.ROOT / 'tests/upstream/test262-typedarray-fill'
CONTROLS = Path(__file__).with_name('typedarray-fill-controls.json')
ERROR = dict(status='exception', phase='runtime', error_type='Error', error_identity='Error')
PAIRS = ['ordered-clamped-range', 'ten-number-kinds', 'tracking-captured-bounds',
         'invalid-and-abrupt-order', 'metadata-saved-identity']
DIRECTORIES = {'TypedArray/prototype/fill': 34, 'TypedArray/prototype/fill/BigInt': 18}
FEATURES = {'TypedArray', 'ArrayBuffer', 'Symbol', 'Reflect.construct',
            'arrow-function', 'resizable-arraybuffer'}


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
        result = fake(case) if fake else (
            dict(status='failed', observation=ERROR.copy())
            if case['id'].endswith('-wrong') else dict(status='passed'))
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)

    with patch.object(runner, 'run_case', side_effect=execute):
        outcomes = runner.harness_preflight(files, Path('/capture-only-no-adapter'), 3, PROFILE)
    return captured, outcomes


class TypedArrayFillTests(unittest.TestCase):
    def setUp(self):
        for owner, name in [(runner, 'bounded_process'), (importer, 'fetch')]:
            guard = patch.object(owner, name, side_effect=AssertionError('execution/network forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_complete_method_roster_overlap_helpers_and_explicit_policy(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), len(files), fixtures), (52, 104, 64, []))
        self.assertEqual({k: len(v) for k, v in manifest['directories'].items()}, DIRECTORIES)
        self.assertEqual([sum(c['mode'] == m for c in cases) for m in ['sloppy', 'strict']], [52, 52])
        self.assertEqual(len(manifest['inventory_proof']['files']), 8)
        self.assertEqual({p[8:] for p in files if p.startswith('harness/')},
                         {'assert.js', 'sta.js', 'propertyHelper.js', 'isConstructor.js',
                          'testTypedArray.js', 'detachArrayBuffer.js', 'resizableArrayBufferUtils.js',
                          'nans.js', 'byteConversionValues.js', 'compareArray.js'})
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES)
        self.assertNotIn('BigInt', FEATURES)
        self.assertNotIn('immutable-arraybuffer', FEATURES)
        reasons = [runner.unsupported_reason(c, FEATURES, PROFILE) for c in cases]
        self.assertEqual(reasons.count(None), 54)
        self.assertEqual(reasons.count('unimplemented declared features: BigInt'), 36)
        self.assertEqual(reasons.count('unimplemented declared features: immutable-arraybuffer'), 2)
        self.assertEqual(reasons.count('Test262 host hooks are not implemented (conservative source check)'), 8)
        self.assertEqual(reasons.count('TypedArray fill policy excludes the complete resizableArrayBufferUtils helper'), 4)
        self.assertEqual(sum(runner.unsupported_reason(c, FEATURES) is None for c in cases), 58)
        self.assertEqual(sum('/BigInt/' in c['file'] for c in cases), 36)
        self.assertEqual(len(runner.PROFILES), 50)

    def test_controls_retain_frozen_bytes_modes_and_same_mode_positive_links(self):
        raw = CONTROLS.read_bytes()
        self.assertEqual(runner.digest(raw), '8e7235a2e834a592417cd3f9a562159edb49f948bda1a728432c6a47134059ba')
        frozen = json.loads(raw)['controls']
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (52, 52))
        self.assertEqual([c['pair'] for c in frozen if c['positive']], PAIRS)
        expected = [('typedarray-fill-' + c['name'], mode, c)
                    for c in frozen for mode in c['modes']]
        for (case, outcome), (name, mode, held) in zip(zip(captured[32:], outcomes[32:]), expected):
            self.assertEqual((outcome['name'], case['mode'], case['source']),
                             (name, mode, held['source'].encode()))
            self.assertEqual([n for n, _ in case['harness']], ['assert.js', 'sta.js'])
            self.assertTrue(outcome['verified'])
            if not held['positive']:
                self.assertEqual(outcome['prerequisite']['name'], 'typedarray-fill-' + held['pair'] + '-positive')
                self.assertEqual(outcome['prerequisite']['mode'], mode)
                self.assertTrue(outcome['prerequisite']['verified'])

    def test_missing_methods_incidental_errors_and_terminal_refusals_never_verify(self):
        for error in ['TypeError', 'ReferenceError', 'SyntaxError', 'Test262Error', 'Error']:
            _, outcomes = capture_controls(lambda c: dict(status='failed', observation=dict(ERROR, error_type=error, error_identity=error)))
            self.assertFalse(any(o['verified'] for o in outcomes[32:]), error)
        for status in ['resource', 'timeout', 'adapter-error', 'unsupported', 'harness-error']:
            _, outcomes = capture_controls(lambda c: dict(status=status))
            self.assertFalse(any(o['verified'] for o in outcomes[32:]), status)

    def test_exact_runtime_error_identity_and_mode_are_both_required(self):
        def mode_cut(case):
            if case['id'].endswith('-wrong') or case['mode'] == 'strict':
                return dict(status='failed', observation=ERROR.copy())
            return dict(status='passed')
        _, outcomes = capture_controls(mode_cut)
        self.assertTrue(all(o['verified'] == (o['result']['mode'] == 'sloppy') for o in outcomes[32:]))
        for missing in ERROR:
            observation = {k: v for k, v in ERROR.items() if k != missing}
            _, outcomes = capture_controls(lambda c: dict(status='failed', observation=observation)
                                           if c['id'].endswith('-wrong') else dict(status='passed'))
            self.assertFalse(any(o['verified'] for o in outcomes[32:] if o['expected'] == 'failed'), missing)
        _, outcomes = capture_controls(lambda c: dict(status='passed'))
        self.assertEqual(sum(o['verified'] for o in outcomes[32:]), 10)

    def test_common_controls_and_protocol_failures_remain_separate(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, current = capture_controls()
        with patch.object(runner, 'run_case', return_value=dict(status='passed')):
            common = runner.harness_preflight(files, Path('/capture-only'), 3)
        self.assertEqual([(o['name'], o['expected']) for o in current[:32]],
                         [(o['name'], o['expected']) for o in common])
        for returned in [(0, b'', b''), (0, b'ERJR2', b''), (0, b'null', b''), (7, b'', b'failed')]:
            with self.subTest(returned=returned), patch.object(runner, 'bounded_process', return_value=returned):
                results = runner.harness_preflight(files, Path('/fake'), 3, PROFILE)
            self.assertEqual(len(results), 52)
            self.assertTrue(all(not r['verified'] and r['result']['status'] == 'adapter-error' for r in results))

    def test_every_proof_and_rehashed_source_or_helper_tamper_is_rejected(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(route=route), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        names = ['test/built-ins/TypedArray/prototype/fill/length.js',
                 'test/built-ins/TypedArray/prototype/fill/BigInt/detached-buffer.js',
                 'test/built-ins/TypedArray/prototype/fill/immutable-buffer.js',
                 'harness/testTypedArray.js', 'harness/resizableArrayBufferUtils.js', 'LICENSE']
        for name in names:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                destination = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, destination)
                data = (destination / name).read_bytes() + b'\n'
                (destination / name).write_bytes(data)
                changed = json.loads((destination / 'manifest.json').read_bytes())
                next(r for r in changed['files'] if r['path'] == name).update(bytes=len(data), sha256=runner.digest(data))
                (destination / 'manifest.json').write_text(json.dumps(changed))
                with self.assertRaises(ValueError):
                    runner.load_corpus(destination, PROFILE)

    def test_full_roster_requires_unselected_ancestors_and_excluded_bigint_entries(self):
        ancestor = 'trees/37185873f6a6ea631208a82eb48c6c007216dcdd'
        recursive = 'trees/5eaac187670caf1fa1256e78d7ae5780de16c47d?recursive=1'
        for route, name in [(ancestor, 'constructor.js'), (recursive, 'BigInt')]:
            with self.subTest(route=route), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d:
                    d['tree'].__setitem__(slice(None), [r for r in d['tree'] if r['path'] != name])))
        _, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        key = 'test/built-ins/TypedArray/prototype/fill/BigInt/return-abrupt-from-this-out-of-bounds.js'
        self.assertIn(key, files)
        retained = [c for c in cases if c['file'] == key]
        self.assertEqual(len(retained), 2)
        self.assertTrue(all(runner.unsupported_reason(c, FEATURES, PROFILE) ==
                            'unimplemented declared features: BigInt' for c in retained))

    def test_serialized_helper_policy_and_resource_health_are_bound(self):
        # Mock observations exercise report/baseline policy, never an adapter.
        for profile, scheduled in [(PROFILE, 54), ('typedarray-search', 158), ('typedarray-views', 110)]:
            def observed(case, binary, timeout, features, selected):
                reason = runner.unsupported_reason(case, features, selected)
                return dict(id=case['id'], mode=case['mode'],
                            case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']),
                            status='unsupported' if reason else 'resource')
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                binary, report, baseline = [directory / n for n in ['fake-adapter', 'report.json', 'baseline.json']]
                binary.write_bytes(b'inert never executed')
                argv = ['test262_conformance.py', '--profile', profile, '--binary', str(binary),
                        '--output', str(report), '--record-baseline', str(baseline), '--jobs', '1']
                with patch.object(runner.sys, 'argv', argv), patch.object(runner, 'run_case', side_effect=observed), patch.object(runner, 'harness_preflight', return_value=[dict(verified=True)]), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.main(), 1)
                actual = json.loads(report.read_bytes())
                expected = dict(format=2, supported_features=sorted(runner.PROFILE_FEATURES[profile]),
                                negative_intrinsic_errors=sorted(runner.INTRINSIC_ERRORS), strict=True,
                                modules=False, async_completion=False, host_hooks=False,
                                timeout_seconds=3.0, excluded_harness=['resizableArrayBufferUtils.js'])
                self.assertEqual(actual['runner_policy'], expected)
                self.assertEqual(actual['runner_policy_sha256'], runner.digest(json.dumps(expected, sort_keys=True).encode()))
                self.assertEqual(actual['counts']['resource'], scheduled)
                self.assertTrue(actual['harness_preflight_passed'])
                self.assertFalse(baseline.exists())

    def test_exact_offline_import_and_auxiliary_corruption_rejection(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'

        def fetch(url):
            if url.startswith(api):
                return proof_reader()(url[len(api):])
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]

        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), PROFILE)
            actual, retained, imported, _, _ = runner.load_corpus(Path(temporary), PROFILE)
            self.assertEqual((actual, retained), (manifest, files))
            self.assertEqual([(c['id'], c['case_sha256']) for c in imported], [(c['id'], c['case_sha256']) for c in cases])
        for suffix in ['harness/testTypedArray.js', 'harness/detachArrayBuffer.js', 'harness/isConstructor.js', 'LICENSE']:
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=lambda u: fetch(u) + (b'\n' if u.endswith(suffix) else b'')), self.assertRaises(ValueError):
                importer.import_corpus(Path(temporary), PROFILE)


if __name__ == '__main__':
    unittest.main()
