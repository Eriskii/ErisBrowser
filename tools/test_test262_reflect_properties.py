"""Frozen Reflect roster, immutable inputs and false-health rejection.

All process and network calls are mocked. The project owner runs these tests
only after importing the independently frozen corpus.
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

PROFILE = 'reflect-properties'
CORPUS = runner.ROOT / 'tests/upstream/test262-reflect-properties'
CONTROLS = Path(__file__).with_name('reflect-properties-controls.json')
ERROR = dict(status='exception', phase='runtime', error_type='Error', error_identity='Error')
PAIR_NAMES = ['literal-units', 'get-receiver', 'set-receiver-accessor',
              'typed-target-receiver', 'resizable-prevention']
DIRECTORIES = {'Reflect': 3, 'Reflect/deleteProperty': 11, 'Reflect/get': 11,
               'Reflect/getOwnPropertyDescriptor': 13, 'Reflect/getPrototypeOf': 10,
               'Reflect/has': 10, 'Reflect/isExtensible': 8,
               'Reflect/preventExtensions': 10, 'Reflect/set': 18,
               'Reflect/setPrototypeOf': 14}
FEATURES = {'Reflect', 'Reflect.set', 'Reflect.setPrototypeOf', 'Reflect.construct',
            'Symbol', 'Symbol.toStringTag', 'arrow-function'}


def proof_reader(altered_route=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(data)
            mutate(value)
            return json.dumps(value).encode()
        return data
    return read


def capture_controls(fake=None):
    _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
    captured = []

    def execute(case, *args):
        captured.append(case)
        result = fake(case) if fake else (
            {'status': 'failed', 'observation': ERROR.copy()}
            if case['id'].endswith('-wrong') else {'status': 'passed'})
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)

    with patch.object(runner, 'run_case', side_effect=execute):
        outcomes = runner.harness_preflight(files, Path('/capture-only-no-adapter'), 3, PROFILE)
    return captured, outcomes


class ReflectPropertyTests(unittest.TestCase):
    def setUp(self):
        for owner, name in [(runner, 'bounded_process'), (importer, 'fetch')]:
            guard = patch.object(owner, name, side_effect=AssertionError('execution/network forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_full_roster_modes_helpers_and_explicit_proxy_policy(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), len(files), fixtures), (108, 216, 115, []))
        self.assertEqual({k: len(v) for k, v in manifest['directories'].items()}, DIRECTORIES)
        self.assertEqual([sum(c['mode'] == m for c in cases) for m in ['sloppy', 'strict']], [108, 108])
        self.assertEqual(len(manifest['inventory_proof']['files']), 14)
        self.assertEqual({p[8:] for p in files if p.startswith('harness/')},
                         {'assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js'})
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES)
        self.assertNotIn('Proxy', FEATURES)
        reasons = [runner.unsupported_reason(c, FEATURES) for c in cases]
        self.assertEqual((reasons.count(None), reasons.count('unimplemented declared features: Proxy')),
                         (200, 16))
        self.assertTrue(all((r is not None) == ('Proxy' in c['metadata']['features'])
                            for c, r in zip(cases, reasons)))
        self.assertEqual(len(set(runner.PROFILES) - {'typedarray-search', 'typedarray-views'}), 47)

    def test_controls_retain_all_frozen_sources_modes_and_positive_links(self):
        raw = CONTROLS.read_bytes()
        self.assertEqual(runner.digest(raw), 'c29babfc2d1f8db802e2cbdde1dfbd95038023cbe1a1704a34caaab4914dcf13')
        frozen = json.loads(raw)['controls']
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (52, 52))
        feature = list(zip(captured[32:], outcomes[32:]))
        expected = [('reflect-properties-' + c['name'], mode, c)
                    for c in frozen for mode in c['modes']]
        self.assertEqual([c['pair'] for c in frozen if c['positive']], PAIR_NAMES)
        for (case, outcome), (name, mode, held) in zip(feature, expected):
            self.assertEqual((outcome['name'], case['mode'], case['source']),
                             (name, mode, held['source'].encode()))
            self.assertEqual([n for n, _ in case['harness']], ['assert.js', 'sta.js'])
            self.assertTrue(outcome['verified'])
            if not held['positive']:
                self.assertEqual(outcome['prerequisite']['name'], 'reflect-properties-' + held['pair'] + '-positive')
                self.assertEqual(outcome['prerequisite']['mode'], mode)
                self.assertTrue(outcome['prerequisite']['verified'])

    def test_missing_methods_incidental_errors_and_bad_statuses_never_verify(self):
        for error in ['TypeError', 'ReferenceError', 'SyntaxError', 'Test262Error', 'Error']:
            _, outcomes = capture_controls(lambda c: dict(status='failed', observation=dict(ERROR, error_type=error, error_identity=error)))
            self.assertFalse(any(o['verified'] for o in outcomes[32:]), error)
        for status in ['resource', 'timeout', 'adapter-error', 'unsupported', 'harness-error']:
            _, outcomes = capture_controls(lambda c: dict(status=status))
            self.assertFalse(any(o['verified'] for o in outcomes[32:]), status)

    def test_intrinsic_error_identity_and_same_mode_positive_are_both_required(self):
        def mode_cut(c):
            if c['id'].endswith('-wrong') or c['mode'] == 'strict':
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

    def test_common_controls_and_existing_adapter_supervision_are_unchanged(self):
        _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        captured, current = capture_controls()
        with patch.object(runner, 'run_case', return_value=dict(status='passed')):
            common = runner.harness_preflight(files, Path('/capture-only'), 3)
        self.assertEqual([(o['name'], o['expected']) for o in current[:32]],
                         [(o['name'], o['expected']) for o in common])
        for returned in [(0, b'', b''), (0, b'ERJR2', b''), (0, b'null', b''), (7, b'', b'failed')]:
            with self.subTest(returned=returned), patch.object(runner, 'bounded_process', return_value=returned):
                result = runner.harness_preflight(files, Path('/fake'), 3, PROFILE)
            self.assertEqual(len(result), 52)
            self.assertTrue(all(not o['verified'] and o['result']['status'] == 'adapter-error' for o in result))

    def test_all_tree_proofs_and_rehashed_sources_fail_closed(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(route=route), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        names = ['test/built-ins/Reflect/get/length.js', 'test/built-ins/Reflect/prop-desc.js',
                 'harness/assert.js', 'harness/isConstructor.js', 'LICENSE', 'INTERPRETING.md']
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
        for suffix in ['harness/assert.js', 'harness/propertyHelper.js', 'harness/isConstructor.js', 'LICENSE']:
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=lambda u: fetch(u) + (b'\n' if u.endswith(suffix) else b'')), self.assertRaises(ValueError):
                importer.import_corpus(Path(temporary), PROFILE)


if __name__ == '__main__':
    unittest.main()
