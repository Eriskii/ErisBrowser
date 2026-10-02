"""Fail-closed orchestration tests using fake receipts; no child/worker/GPU launch."""
from __future__ import annotations

from contextlib import ExitStack
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest import mock

import run_worker_text_host as host

SOURCES = ('run_worker_text_host.py', 'worker_text_protocol.py', 'browser_protocol.py',
           'run_browser_host.py', 'run_glyph_host.py', 'glyph_protocol.py',
           'run_host.py', 'bridge_supervisor.py')
ADAPTERS = ['synthetic-adapter-0', 'synthetic-adapter-1']


class WorkerTextHostTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.binary = self.directory / 'checker'
        self.browser = self.directory / 'browser'
        # These are deliberately nonexecutable text bytes, never launched.
        self.binary.write_bytes(b'fake-checker-identity')
        self.browser.write_bytes(b'fake-browser-identity')
        self.fixtures = self.directory / 'fixtures'
        shutil.copytree(host.ROOT / 'worker-text-fixtures', self.fixtures)
        self.sources = self.directory / 'sources'
        self.sources.mkdir()
        for name in SOURCES:
            shutil.copyfile(host.ROOT / name, self.sources / name)
        self.calls = []
        self.mutate = None
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(mock.patch.object(host, 'ROOT', self.sources))
        self.supervisor = self.stack.enter_context(mock.patch.object(host, 'run_supervised', side_effect=self.fake_supervisor))
        self.listing = self.stack.enter_context(mock.patch.object(host.protocol, 'validate_listing', return_value=ADAPTERS))
        self.cpu = self.stack.enter_context(mock.patch.object(host.protocol, 'validate_cpu', return_value={'synthetic': 'cpu', 'gpu_comparisons': 0}))
        self.gpu = self.stack.enter_context(mock.patch.object(host.protocol, 'validate_run', return_value={'synthetic': 'gpu'}))
        # load_fixtures/digest/check_success remain real, so mutations exercise
        # exact actual source/fixture/executable checks, not a mocked identity.

    def fake_supervisor(self, command, output, name, environment, timeout, *, worker_text_gate=False):
        self.calls.append({'name': name, 'command': command, 'gate': worker_text_gate,
                           'environment': environment.copy(), 'timeout': timeout})
        (output / f'{name}.supervisor.stderr.log').write_bytes(b'')
        record = {'name': name, 'status': 'exited', 'returncode': 0,
                  'cleanup': {'complete': True, 'descendants': 0},
                  'stderr_sha256': hashlib.sha256(b'').hexdigest(),
                  'capture_gate_required': worker_text_gate,
                  'capture_gate_granted': worker_text_gate}
        if self.mutate:
            self.mutate(name, record, output)
        return record, b'synthetic protocol bytes\n'

    def run_case(self, *, cpu=False, suffix='result'):
        output = self.directory / suffix
        result = host.run_host(self.binary, self.browser, output, self.fixtures,
                               timeout=1.0, cpu_check=cpu)
        self.assertEqual(json.loads((output / 'host-results.json').read_text()), result)
        return result

    def test_cpu_phase_has_no_enumeration_or_gpu_grant(self):
        with mock.patch.dict('os.environ', {'PYTHONOPTIMIZE': '1', 'PYTHONPATH': '/fake', 'PYTHONHOME': '/fake'}):
            result = self.run_case(cpu=True)
        self.assertTrue(result['success'])
        self.assertEqual(result['adapter_count'], 0)
        self.assertEqual([(c['name'], c['gate']) for c in self.calls], [('cpu-check', False)])
        command = self.calls[0]['command']
        self.assertEqual(command, [str(self.binary), '--cpu-check', '--browser', str(self.browser), '--fixtures', str(self.fixtures)])
        self.assertFalse(any(k in self.calls[0]['environment'] for k in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME')))
        self.cpu.assert_called_once()
        self.listing.assert_not_called()
        self.gpu.assert_not_called()

    def test_gpu_enumeration_ungated_and_every_capture_gated(self):
        result = self.run_case()
        self.assertTrue(result['success'])
        self.assertEqual(result['adapter_count'], 2)
        self.assertEqual([(c['name'], c['gate']) for c in self.calls],
                         [('enumeration', False), ('adapter-0', True), ('adapter-1', True)])
        self.assertEqual(self.calls[0]['command'], [str(self.binary), '--list'])
        for i in range(2):
            self.assertEqual(self.calls[i + 1]['command'], [str(self.binary), '--adapter', str(i), '--browser', str(self.browser), '--fixtures', str(self.fixtures)])
        self.assertEqual(self.gpu.call_count, 2)
        self.cpu.assert_not_called()

    def test_first_process_or_cleanup_failure_stops_following_adapters(self):
        failures = [dict(status='timeout', returncode=None), dict(returncode=1),
                    dict(cleanup={'complete': False, 'descendants': 0}),
                    dict(cleanup={'complete': True, 'descendants': 1})]
        for index, failure in enumerate(failures):
            with self.subTest(failure=failure):
                self.calls.clear()
                self.mutate = lambda name, record, _output: record.update(failure) if name == 'adapter-0' else None
                result = self.run_case(suffix=f'failure-{index}')
                self.assertFalse(result['success'])
                self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
                self.assertEqual(len(result['runs']), 2)
                self.assertIn('cleanup failed', result['error'])

    def test_missing_gpu_grant_and_unexpected_cpu_grant_fail_closed(self):
        self.mutate = lambda name, record, _output: record.update(capture_gate_granted=False) if name == 'adapter-0' else None
        result = self.run_case()
        self.assertFalse(result['success'])
        self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
        self.assertIn('grant disagrees', result['error'])
        self.calls.clear()
        self.mutate = lambda _name, record, _output: record.update(capture_gate_required=True, capture_gate_granted=True)
        result = self.run_case(cpu=True, suffix='cpu-bad-gate')
        self.assertFalse(result['success'])
        self.assertEqual([c['name'] for c in self.calls], ['cpu-check'])
        self.cpu.assert_not_called()

    def test_enumeration_gate_or_stderr_failure_prevents_any_adapter(self):
        for index, failure in enumerate([{'capture_gate_required': True}, {'stderr_sha256': hashlib.sha256(b'noise').hexdigest()}]):
            with self.subTest(failure=failure):
                self.calls.clear()
                self.mutate = lambda _name, record, _output: record.update(failure)
                result = self.run_case(suffix=f'bad-enumeration-{index}')
                self.assertFalse(result['success'])
                self.assertEqual([c['name'] for c in self.calls], ['enumeration'])
        self.listing.assert_not_called()
        self.gpu.assert_not_called()

    def test_checker_or_supervisor_stderr_stops_before_later_adapter(self):
        for index, supervisor in enumerate([False, True]):
            with self.subTest(supervisor=supervisor):
                self.calls.clear()
                def noise(name, record, output):
                    if name == 'adapter-0':
                        if supervisor:
                            (output / f'{name}.supervisor.stderr.log').write_bytes(b'unexpected supervisor output')
                        else:
                            record['stderr_sha256'] = hashlib.sha256(b'unexpected checker output').hexdigest()
                self.mutate = noise
                result = self.run_case(suffix=f'stderr-{index}')
                self.assertFalse(result['success'])
                self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
                self.assertIn('stderr must be empty', result['error'])

    def test_postexecution_binary_browser_fixture_and_source_mutations_fail(self):
        paths = [self.binary, self.browser, self.fixtures / 'html/worker-text-flow-unicode.html',
                 self.sources / 'run_worker_text_host.py']
        for index, path in enumerate(paths):
            with self.subTest(path=path.name):
                before = path.read_bytes()
                self.calls.clear()
                def mutate(name, _record, _output):
                    if name == 'adapter-0':
                        path.write_bytes(before + b'changed-after-run')
                self.mutate = mutate
                try:
                    result = self.run_case(suffix=f'mutation-{index}')
                    self.assertFalse(result['success'])
                    self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
                    self.assertIn('changed', result['error'])
                finally:
                    path.write_bytes(before)

    def test_protocol_failure_and_interruption_stop_future_execution(self):
        self.gpu.side_effect = ValueError('synthetic protocol mismatch')
        result = self.run_case()
        self.assertFalse(result['success'])
        self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
        self.assertIn('protocol mismatch', result['error'])
        self.gpu.side_effect = None
        self.calls.clear()
        def interrupt(name, _record, _output):
            if name == 'adapter-0':
                raise KeyboardInterrupt
        self.mutate = interrupt
        result = self.run_case(suffix='interrupted')
        self.assertFalse(result['success'])
        self.assertEqual([c['name'] for c in self.calls], ['enumeration', 'adapter-0'])
        self.assertIn('KeyboardInterrupt', result['error'])
        # Actual child cleanup is owned/tested by the unchanged supervisor;
        # this mock verifies only that orchestration cannot start another one.

    def test_invalid_timeout_or_existing_output_never_launches(self):
        for value in (float('nan'), 0, 120.01):
            with self.subTest(timeout=value), self.assertRaises(ValueError):
                host.run_host(self.binary, self.browser, self.directory / 'unused', self.fixtures, timeout=value)
        output = self.directory / 'existing'; output.mkdir()
        (output / 'retained').write_bytes(b'original')
        with self.assertRaises(ValueError):
            host.run_host(self.binary, self.browser, output, self.fixtures)
        self.assertEqual((output / 'retained').read_bytes(), b'original')
        self.supervisor.assert_not_called()


if __name__ == '__main__':
    unittest.main()
