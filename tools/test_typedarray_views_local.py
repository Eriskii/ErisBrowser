"""Local-runner protocol checks with an inert adapter substitute, never a JS engine."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
import types
import unittest
from unittest import mock

import typedarray_views_local as local


def digest(data):
    return hashlib.sha256(data).hexdigest()


def fingerprint(case):
    # Independent transcription of the public fingerprint schema; no shared
    # runner import, helper invocation, parser or process belongs in these mocks.
    identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                    metadata=case['metadata'], harness=[])
    return digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode())


class LocalViewRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='eris-local-view-protocol-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.corpus = self.root / 'tests/conformance/typedarray-views-local'
        shutil.copytree(local.CORPUS, self.corpus)
        self.tool_dir = self.root / 'tools'
        self.tool_dir.mkdir()
        self.tool = self.tool_dir / 'typedarray_views_local.py'
        self.tool.write_bytes(Path(local.__file__).read_bytes())
        for name in ['test262_conformance.py', 'html_conformance.py', 'import_test262.py']:
            (self.tool_dir / name).write_bytes(b'# inert source binding for protocol test\n')
        self.binary = self.root / 'adapter-placeholder'
        self.binary.write_bytes(b'not an executable; no process is launched\n')
        self.binary_hash = digest(self.binary.read_bytes())
        self.output = self.root / 'result.json'
        self.calls = []
        self.effect = None
        self.fake = types.ModuleType('test262_conformance')
        self.fake.case_fingerprint = fingerprint
        self.fake.run_case = self.run_case
        for patcher in [mock.patch.object(local, 'ROOT', self.root),
                        mock.patch.object(local, 'CORPUS', self.corpus),
                        mock.patch.object(local, '__file__', str(self.tool)),
                        mock.patch.dict(sys.modules, {'test262_conformance': self.fake})]:
            patcher.start()
            self.addCleanup(patcher.stop)

    def run_case(self, case, binary, timeout):
        self.assertEqual(binary, self.binary)
        self.assertEqual(timeout, 3)
        self.calls.append(case)
        wrong = '-wrong#' in case['id']
        result = {key: case[key] for key in ['id', 'file', 'mode', 'case_sha256']}
        result.update(source_sha256=digest(case['source']), expected_negative=None,
                      status='failed' if wrong else 'passed',
                      observation=dict(local.WRONG if wrong else local.COMPLETE))
        if self.effect:
            return self.effect(case, result)
        return result

    def invoke(self, args=None):
        if args is None:
            args = ['--engine', str(self.binary), '--engine-sha256', self.binary_hash,
                    '--output', str(self.output)]
        with mock.patch.object(sys, 'argv', [str(self.tool), *args]), contextlib.redirect_stdout(io.StringIO()) as output:
            code = local.main()
        return code, output.getvalue()

    def report(self):
        return json.loads(self.output.read_text())

    def test_frozen_matrix_keeps_every_mode_and_true_return_guard(self):
        pairs = local.load_matrix(self.fake)
        self.assertEqual(len(pairs), 70)
        self.assertEqual([identity for _, identity in pairs], json.loads((self.corpus / 'matrix.json').read_text()))
        self.assertEqual(sum(row['kind'] == 'case' for _, row in pairs), 46)
        self.assertEqual(sum(row['kind'] == 'control' for _, row in pairs), 24)
        fixture = (self.corpus / 'cases.js').read_bytes()
        for case, row in pairs:
            self.assertEqual(case['metadata']['negative'], None)
            self.assertEqual(case['harness'], [])
            if row['kind'] == 'case':
                name = row['id'].split('#')[0]
                expected = fixture + ('\nif (typedArrayViewCases.' + name + '() !== true) throw new Error("case did not return true");\n').encode()
                self.assertEqual(case['source'], expected)
            self.assertEqual(row['source_sha256'], digest(case['source']))
            self.assertEqual(row['case_sha256'], fingerprint(case))
        self.assertEqual(self.calls, [])

    def test_before_run_tampering_rejects_without_adapter(self):
        for name in ['cases.js', 'controls.json', 'matrix.json']:
            with self.subTest(name=name):
                path = self.corpus / name
                before = path.read_bytes()
                path.write_bytes(before + b' ')
                try:
                    with self.assertRaisesRegex(ValueError, 'frozen fixture changed'):
                        self.invoke()
                finally:
                    path.write_bytes(before)
                self.assertFalse(self.output.exists())
        path = self.corpus / 'manifest.json'
        original = path.read_bytes()
        altered = json.loads(original)
        altered['files'][1] = altered['files'][0]
        path.write_text(json.dumps(altered))
        with self.assertRaisesRegex(ValueError, 'unknown or repeated fixture path'):
            self.invoke()
        path.write_bytes(original)
        self.assertFalse(self.output.exists())
        self.assertEqual(self.calls, [])

    def test_matching_mock_rows_are_complete_and_separately_healthy(self):
        code, _ = self.invoke()
        report = self.report()
        self.assertEqual(code, 0)
        self.assertEqual(len(self.calls), 70)
        self.assertEqual(report['summary'], dict(case_matches=46, case_total=46,
                         control_matches=24, controls_verified=24, control_total=24, runner_errors=0))
        self.assertEqual(report['binding_failures'], [])
        self.assertEqual(report['binary_sha256'], self.binary_hash)
        self.assertEqual(len(report['observed_after']), len(report['inputs']) + 1)
        self.assertEqual(len({row['path'] for row in report['inputs']}), len(report['inputs']))
        self.assertEqual(sum(row['status'] == 'failed' for row in report['controls']), 12)
        self.assertTrue(all(row['verified'] for row in report['controls']))

    def test_missing_method_typeerror_cannot_count_as_wrong_control(self):
        def effect(case, row):
            if '-wrong#' in case['id']:
                row['observation'] = dict(status='exception', phase='runtime', error_type='TypeError', error_identity='TypeError')
            return row
        self.effect = effect
        code, _ = self.invoke()
        report = self.report()
        self.assertEqual(code, 1)
        self.assertEqual(report['summary']['case_matches'], 46)
        self.assertEqual(report['summary']['controls_verified'], 12)
        self.assertTrue(all(not row['verified'] for row in report['controls'] if not row['positive']))
        self.assertEqual(len(self.calls), 70)

    def test_failed_positive_partner_invalidates_only_its_same_mode_pair(self):
        failed_id = 'shared-subarray-positive#sloppy'
        def effect(case, row):
            if case['id'] == failed_id:
                row['status'] = 'failed'
                row['observation'] = dict(status='exception', phase='runtime', error_type='TypeError', error_identity='TypeError')
            return row
        self.effect = effect
        code, _ = self.invoke()
        report = self.report()
        by_id = {row['id']: row for row in report['controls']}
        self.assertEqual(code, 1)
        self.assertTrue(by_id['shared-subarray-wrong#sloppy']['matches_expectation'])
        self.assertFalse(by_id['shared-subarray-wrong#sloppy']['verified'])
        self.assertTrue(by_id['shared-subarray-wrong#strict']['verified'])
        self.assertEqual(report['summary']['control_matches'], 23)
        self.assertEqual(report['summary']['controls_verified'], 22)

    def test_resource_remains_terminal_unhealthy_and_all_rows_are_retained(self):
        def effect(case, row):
            # Even misleading exception fields cannot convert a terminal
            # resource classification into a matched negative control.
            row['status'] = 'resource'
            row['observation'] = dict(local.WRONG)
            row['reason'] = 'retained resource marker'
            return row
        self.effect = effect
        code, _ = self.invoke()
        report = self.report()
        self.assertEqual(code, 1)
        self.assertEqual(report['summary']['case_matches'], 0)
        self.assertEqual(report['summary']['controls_verified'], 0)
        rows = report['cases'] + report['controls']
        self.assertEqual(len(rows), 70)
        self.assertTrue(all(row['status'] == 'resource' and row['reason'] == 'retained resource marker' for row in rows))

    def test_wrong_binary_and_existing_output_stop_before_requests(self):
        with self.assertRaisesRegex(ValueError, 'binary differs'):
            self.invoke(['--engine', str(self.binary), '--engine-sha256', '0' * 64, '--output', str(self.output)])
        self.assertFalse(self.output.exists())
        self.output.write_bytes(b'original result retained\n')
        with self.assertRaises(FileExistsError):
            self.invoke()
        self.assertEqual(self.output.read_bytes(), b'original result retained\n')
        self.assertEqual(self.calls, [])

    def test_late_source_and_binary_drift_retains_observations_but_fails_binding(self):
        def effect(case, row):
            if len(self.calls) == 1:
                path = self.corpus / 'cases.js'
                path.write_bytes(path.read_bytes() + b'\n// drift\n')
                self.binary.write_bytes(b'changed adapter bytes\n')
            return row
        self.effect = effect
        code, _ = self.invoke()
        report = self.report()
        self.assertEqual(code, 2)
        self.assertEqual(len(report['cases']) + len(report['controls']), 70)
        self.assertEqual({row['path'] for row in report['binding_failures']},
                         {str(self.corpus / 'cases.js'), str(self.binary)})
        self.assertEqual(report['binary_sha256'], self.binary_hash)

    def test_runner_exception_and_identity_error_retain_every_later_row(self):
        def effect(case, row):
            if len(self.calls) == 1:
                raise OSError('mock retained failure')
            if len(self.calls) == 2:
                row['case_sha256'] = '0' * 64
            return row
        self.effect = effect
        code, _ = self.invoke()
        report = self.report()
        self.assertEqual(code, 3)
        self.assertEqual(report['summary']['runner_errors'], 2)
        self.assertEqual(len(report['cases']) + len(report['controls']), 70)
        self.assertIn('mock retained failure', report['cases'][0]['reason'])
        self.assertIn('adapter row identity changed', report['cases'][1]['reason'])
        self.assertTrue(all(row['verified'] for row in report['controls']))

    def test_verification_only_has_no_binary_output_or_adapter_request(self):
        code, text = self.invoke(['--verify-only'])
        result = json.loads(text)
        self.assertEqual(code, 0)
        self.assertEqual((result['cases'], result['controls'], result['rows']), (46, 24, 70))
        self.assertFalse(self.output.exists())
        self.assertEqual(self.calls, [])
        with self.assertRaisesRegex(ValueError, 'does not accept execution arguments'):
            self.invoke(['--verify-only', '--engine', str(self.binary)])
        self.assertEqual(self.calls, [])


if __name__ == '__main__':
    unittest.main()
