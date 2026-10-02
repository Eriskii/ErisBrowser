"""Focused synthetic follow-up checks; no window/controller/worker execution."""
from contextlib import ExitStack
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_raster_followup as follow
import native_raster_host as base
from test_native_raster_host import synthetic

PAGE1 = b'[page] Page process 201: Landlock ABI 6 and seccomp: no direct resource or socket access\n'
BROKER1 = b'[page] Resource broker 202: committed URL and fetch policy enforced outside renderer\n'
PREFIX = synthetic('admitted-normal').replace(b'[page] synthetic retained worker diagnostic\n', PAGE1 + BROKER1)
MARKER = b'native follow-up: reload requested for owned pid=71 after serial=4\n'
AFTER = (PAGE1.replace(b'201', b'301') + BROKER1.replace(b'202', b'302') +
         next(line for line in PREFIX.splitlines(keepends=True) if line.startswith(b'native scene')).replace(b'serial=4', b'serial=5').replace(b'snapshot_generation=1', b'snapshot_generation=2'))


def control(opacity=False):
    value = {'success': not opacity, 'control_complete': True, 'browser_reaped': True,
             'browser_returncode': 1 if opacity else 0, 'controlled_size': list(base.SIZE),
             'identity': {'pid': 71}}
    if not opacity:
        value['reload'] = {**follow.reload_prefix(PREFIX), 'dispatch_complete': True}
    return value


class FollowupProtocolTests(unittest.TestCase):
    def test_reload_requires_fresh_workers_after_native_presentation(self):
        result = follow.validate_followup(b'', PREFIX + MARKER + AFTER, 'reload-after-native', control())
        self.assertEqual(result['reloaded_page_pid'], 301)
        self.assertEqual(result['reloaded_broker_pid'], 302)
        self.assertEqual(result['generation_2_prepared_serials'], [5])
        self.assertEqual(result['validation']['acquired_texture_compared_bytes'], 0)

    def test_missing_reused_or_early_workers_and_generation_refuse(self):
        bad = [AFTER.replace(b'301', b'201'), AFTER.replace(b'302', b'202'),
               AFTER.replace(b'snapshot_generation=2', b'snapshot_generation=1'),
               b''.join(reversed(AFTER.splitlines(keepends=True))),
               AFTER.replace(b'Landlock ABI 6', b'Landlock ABI 0')]
        for tail in bad:
            with self.subTest(tail=tail), self.assertRaises(ValueError):
                follow.validate_followup(b'', PREFIX + MARKER + tail, 'reload-after-native', control())
        with self.assertRaises(ValueError):
            follow.validate_followup(b'', PREFIX + AFTER + MARKER, 'reload-after-native', control())

    def test_marker_receipt_and_native_initialization_cannot_be_faked(self):
        for changed in [MARKER.replace(b'pid=71', b'pid=72'), MARKER.replace(b'serial=4', b'serial=5'), b'', MARKER + MARKER]:
            with self.assertRaises(ValueError):
                follow.validate_followup(b'', PREFIX + changed + AFTER, 'reload-after-native', control())
        for key, value in [('dispatch_complete', False), ('prefix_sha256', 'bad'), ('prefix_bytes', 1)]:
            receipt = control()
            receipt['reload'][key] = value
            with self.assertRaises(ValueError):
                follow.validate_followup(b'', PREFIX + MARKER + AFTER, 'reload-after-native', receipt)
        with self.assertRaises(ValueError):
            follow.reload_prefix(PREFIX.replace(next(l for l in PREFIX.splitlines(keepends=True) if l.startswith(b'presenter: native shader')), b''))

    def test_opacity_quota_refusal_is_separate_expected_failure(self):
        stderr = synthetic('opacity-fallback') + follow.INCOMPLETE
        result = follow.validate_followup(b'', stderr, 'opacity-verification-refused', control(True))
        self.assertEqual(result['expected_browser_exit'], 1)
        self.assertEqual(result['native_verified_frames'], 0)
        for data in [stderr.replace(b'0/1', b'1/1'), stderr + b'extra\n', stderr + follow.INCOMPLETE,
                     synthetic('admitted-verified') + follow.INCOMPLETE,
                     synthetic('overdraw-fallback') + follow.INCOMPLETE]:
            with self.assertRaises(ValueError):
                follow.validate_followup(b'', data, 'opacity-verification-refused', control(True))
        with self.assertRaises(ValueError):
            follow.validate_followup(b'', stderr, 'opacity-verification-refused', control(False))

    def test_control_or_terminal_failures_do_not_count_as_expected_opacity(self):
        for key, value in [('error', 'controller failed'), ('cleanup_error', 'not reaped'),
                           ('postcheck_error', 'changed'), ('browser_reaped', False), ('controlled_size', [1275, 764])]:
            receipt = control(True)
            receipt[key] = value
            with self.assertRaises(ValueError):
                follow.validate_followup(b'', synthetic('opacity-fallback') + follow.INCOMPLETE,
                                         'opacity-verification-refused', receipt)


class ReloadDispatchTests(unittest.TestCase):
    def test_owned_pid_and_observed_prefix_gate_single_dispatch(self):
        record, emitted = {}, []
        result = SimpleNamespace(record={'status': 'exited', 'returncode': 0}, stdout=b'ok\n', stderr=b'')
        with patch.object(base, 'ensure_live_child') as own, patch.object(base, 'controller_call', return_value=result) as call:
            follow.request_reload({'pid': 71}, Path('/fake/controller'), {}, record, lambda: None, PREFIX, emitted.append)
            self.assertEqual(emitted, [MARKER])
            call.assert_called_once_with(Path('/fake/controller'), ['dispatch', 'sendshortcut', 'CTRL,r,pid:71'], {}, 0.5)
            self.assertEqual(own.call_count, 2)
            self.assertTrue(record['reload']['dispatch_complete'])
            with self.assertRaises(ValueError):
                follow.request_reload({'pid': 71}, Path('/fake/controller'), {}, record, lambda: None, PREFIX, emitted.append)
            self.assertEqual(call.call_count, 1)

    def test_dead_child_and_failed_shortcut_never_claim_reload(self):
        with patch.object(base, 'ensure_live_child', side_effect=ValueError('exit')), patch.object(base, 'controller_call') as call:
            with self.assertRaises(ValueError):
                follow.request_reload({'pid': 71}, Path('/fake/controller'), {}, {}, lambda: None, PREFIX, lambda _: None)
            call.assert_not_called()
        for status, code, stdout, stderr in [('timeout', -9, b'', b''), ('exited', 1, b'', b''),
                                              ('exited', 0, b'unknown dispatcher', b''), ('exited', 0, b'ok', b'error')]:
            record = {}
            result = SimpleNamespace(record={'status': status, 'returncode': code}, stdout=stdout, stderr=stderr)
            with patch.object(base, 'ensure_live_child'), patch.object(base, 'controller_call', return_value=result), self.assertRaises(ValueError):
                follow.request_reload({'pid': 71}, Path('/fake/controller'), {}, record, lambda: None, PREFIX, lambda _: None)
            self.assertNotIn('dispatch_complete', record['reload'])

    def test_live_relay_preserves_all_browser_bytes_and_event_order(self):
        read_fd, write_fd = os.pipe()
        os.write(write_fd, PREFIX + AFTER)
        os.close(write_fd)
        stream = os.fdopen(read_fd, 'rb')
        self.addCleanup(stream.close)
        record = {'identity': {'pid': 71}}
        emitted = []
        response = SimpleNamespace(record={'status': 'exited', 'returncode': 0}, stdout=b'ok\n', stderr=b'')
        with patch.object(base, 'ensure_live_child'), patch.object(base, 'controller_call', return_value=response), \
                patch.object(follow.os, 'waitid', return_value=object()):
            follow.relay_until_exit(SimpleNamespace(pid=71, stderr=stream), Path('/fake/controller'),
                                    record, lambda: None, follow.time.monotonic() + 2, emitted.append)
        self.assertEqual(b''.join(emitted), PREFIX + MARKER + AFTER)
        self.assertTrue(record['reload']['dispatch_complete'])


    def test_relay_failure_retains_pending_bytes_without_a_second_action(self):
        read_fd, write_fd = os.pipe()
        os.write(write_fd, PREFIX + b'pending browser tail')
        os.close(write_fd)
        stream = os.fdopen(read_fd, 'rb')
        self.addCleanup(stream.close)
        emitted = []
        with patch.object(follow, 'request_reload', side_effect=ValueError('controller failed')) as request:
            with self.assertRaises(ValueError):
                follow.relay_until_exit(SimpleNamespace(pid=71, stderr=stream), Path('/fake/controller'),
                                        {'identity': {'pid': 71}}, lambda: None,
                                        follow.time.monotonic() + 2, emitted.append)
        self.assertEqual(b''.join(emitted), PREFIX + b'pending browser tail')
        self.assertEqual(request.call_count, 1)

    def test_configure_failure_drains_browser_bytes_after_owned_kill_before_reap(self):
        read_fd, write_fd = os.pipe()
        os.write(write_fd, b'original startup diagnostic\n')
        os.close(write_fd)
        stream = os.fdopen(read_fd, 'rb')
        child = SimpleNamespace(pid=71, stderr=stream)
        events = []
        child.wait = lambda timeout: events.append('reap') or -9
        captured = io.BytesIO()
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(base, 'digest', return_value='bound'), \
                patch.object(follow.subprocess, 'Popen', return_value=child), \
                patch.object(base, 'child_identity', return_value={'pid': 71}), \
                patch.object(base, 'configure_owned', side_effect=ValueError('wrong window')), \
                patch.object(follow.os, 'waitid', side_effect=lambda *_args: events.append('owned')), \
                patch.object(follow.os, 'kill', side_effect=lambda *_args: events.append('kill')), \
                patch.object(follow.sys, 'stderr', SimpleNamespace(buffer=captured)):
            receipt = Path(directory) / 'receipt.json'
            code = follow.reload_launch(['/fake/browser'], receipt, Path('/fake/controller'), 'bound', 'bound', 6)
            retained = json.loads(receipt.read_bytes())
        self.assertEqual(code, 1)
        self.assertEqual(events, ['owned', 'kill', 'reap'])
        self.assertEqual(captured.getvalue(), b'original startup diagnostic\n')
        self.assertTrue(retained['browser_reaped'])
        self.assertFalse(retained['success'])

    def test_descendant_held_pipe_drain_is_finite(self):
        read_fd, write_fd = os.pipe()
        stream = os.fdopen(read_fd, 'rb')
        self.addCleanup(stream.close)
        self.addCleanup(os.close, write_fd)
        with patch.object(follow.time, 'monotonic', side_effect=[0, 3]), self.assertRaises(ValueError):
            follow.drain_stderr(stream, lambda _: None, seconds=2)


class FollowupHostTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.binary = self.root / 'browser'
        self.binary.write_bytes(b'synthetic binary')
        self.controller = self.root / 'controller'
        self.controller.write_bytes(b'synthetic controller')
        self.loader = self.root / 'loader'
        self.loader.mkdir()
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(patch.object(follow.shutil, 'which', return_value=str(self.controller)))
        self.stack.enter_context(patch.object(base, 'environment', return_value={}))
        self.calls = []
        self.failure = None
        self.stack.enter_context(patch.object(base, 'run_supervised', side_effect=self.supervised))

    def supervised(self, command, output, name, env, timeout):
        self.calls.append(command)
        opacity = name == 'opacity-verification-refused'
        stderr = synthetic('opacity-fallback') + follow.INCOMPLETE if opacity else PREFIX + MARKER + AFTER
        receipt = control(opacity)
        receipt.update(binary_sha256=base.digest(self.binary, base.MAX_BINARY),
                       controller_sha256=base.digest(self.controller, base.MAX_BINARY))
        Path(command[command.index('--receipt') + 1]).write_text(json.dumps(receipt))
        (output / f'{name}.stderr.log').write_bytes(stderr)
        (output / f'{name}.supervisor.stderr.log').write_bytes(b'')
        result = {'status': 'exited', 'returncode': 1 if opacity else 0,
                  'cleanup': {'complete': True, 'descendants': 0},
                  'stdout_sha256': hashlib.sha256(b'').hexdigest(), 'stderr_sha256': hashlib.sha256(stderr).hexdigest()}
        if self.failure:
            result.update(self.failure)
        return result, b''

    def test_separate_cases_keep_explicit_exit_expectations_and_original_deadlines(self):
        result = follow.run_followups(self.binary, self.root / 'out', self.loader)
        self.assertTrue(result['success'], result.get('error'))
        self.assertEqual(len(self.calls), 2)
        self.assertEqual(result['timeout_seconds'], 30)
        self.assertEqual(result['window_seconds'], 6)
        self.assertNotIn('--vulkan-verify-frames', self.calls[0])
        self.assertIn('--vulkan-verify-frames', self.calls[1])
        self.assertEqual([r['returncode'] for r in result['runs']], [0, 1])

    def test_cleanup_failure_stops_before_next_window_and_preserves_error(self):
        self.failure = {'cleanup': {'complete': False, 'descendants': 1}}
        result = follow.run_followups(self.binary, self.root / 'out', self.loader)
        self.assertFalse(result['success'])
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(json.loads((self.root / 'out/followup-results.json').read_text()), result)


if __name__ == '__main__':
    unittest.main()
