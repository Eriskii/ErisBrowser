"""Synthetic identity/protocol tests; never launch a worker, window, or GPU."""
from __future__ import annotations

from contextlib import ExitStack
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_raster_host as host

ADAPTER = ('presenter: Vulkan adapter="synthetic" type=DiscreteGpu driver="test" '
           'format=Bgra8Unorm color_space=Srgb alpha=Opaque mode=Fifo; '
           'raster=native-shaders with complete CPU admission fallback')


def synthetic(case):
    lines = [ADAPTER, '[page] synthetic retained worker diagnostic']
    if case == 'opacity-fallback':
        lines.append('presenter: native raster admission fallback; complete CPU upload; '
                     'reason=unsupported-opacity at phase 0 original command 2; size=1180x880')
    else:
        reference = 'true' if case == 'admitted-verified' else 'false'
        lines += [
            'native scene prepared: serial=4 snapshot_generation=1 page_commands=7 images=1 '
            'loading=false phases=3 lowered_commands=100 rounded_masks=2 '
            f'raster_invocations=1200000 total_invocations=2241920 reference={reference}',
            f'presenter: native shader frame presented; size=1180x880 serial=4 CPU_upload=false reference={reference}',
        ]
        if case == 'admitted-verified':
            lines += [
                'presenter: native shader acquired-texture verified; serial=4 generation=1 '
                f'viewport_revision=2 size=1180x880 compared_bytes={host.PIXEL_BYTES} exact=true (before compositor)',
                'presenter: Vulkan acquired-texture verification passed; route=native-raster '
                f'frames=1 compared_bytes={host.PIXEL_BYTES} last_serial=4 (before compositor)',
            ]
    return ('\n'.join(lines) + '\n').encode()


class NativeWindowProtocolTests(unittest.TestCase):
    def test_separate_verified_normal_and_opacity_populations(self):
        verified = host.validate_run(b'', synthetic('admitted-verified'), 'admitted-verified')
        normal = host.validate_run(b'', synthetic('admitted-normal'), 'admitted-normal')
        fallback = host.validate_run(b'', synthetic('opacity-fallback'), 'opacity-fallback')
        self.assertEqual(verified['acquired_texture_compared_bytes'], host.PIXEL_BYTES)
        self.assertEqual(verified['reference'], 'original Canvas differential')
        self.assertEqual(normal['acquired_texture_compared_bytes'], 0)
        self.assertFalse(normal['prepared_scenes'][0]['reference'])
        self.assertEqual(fallback['acquired_texture_compared_bytes'], 0)
        self.assertEqual(fallback['native_presentations'], [])

    def test_chrome_only_missing_image_or_unjoined_serial_cannot_prove_page(self):
        changes = [(b'loading=false', b'loading=true'), (b'page_commands=7', b'page_commands=0'),
                   (b'images=1', b'images=0'), (b'snapshot_generation=1', b'snapshot_generation=0'),
                   (b'snapshot_generation=1', b'snapshot_generation=2'),
                   (b'size=1180x880 serial=4', b'size=1180x880 serial=5')]
        for before, after in changes:
            with self.subTest(before=before), self.assertRaises(ValueError):
                host.validate_run(b'', synthetic('admitted-verified').replace(before, after), 'admitted-verified')

    def test_native_bounds_dimensions_reference_and_summary_are_exact(self):
        changes = [(b'phases=3', b'phases=4'), (b'lowered_commands=100', b'lowered_commands=257'),
                   (b'rounded_masks=2', b'rounded_masks=0'), (b'total_invocations=2241920', b'total_invocations=2241921'),
                   (b'size=1180x880', b'size=1179x880'), (b'exact=true', b'exact=false'),
                   (b'frames=1', b'frames=2'), (b'last_serial=4', b'last_serial=3'),
                   (b'compared_bytes=4153600', b'compared_bytes=4153599'),
                   (b'reference=true', b'reference=false'), (b'route=native-raster', b'route=cpu-upload')]
        for before, after in changes:
            with self.subTest(before=before), self.assertRaises(ValueError):
                host.validate_run(b'', synthetic('admitted-verified').replace(before, after), 'admitted-verified')

    def test_duplicate_truncated_unknown_or_failure_records_are_rejected(self):
        original = synthetic('admitted-verified')
        lines = original.splitlines(keepends=True)
        candidates = [original[:-1], b'\xff\n', original + lines[0], original + lines[2],
                      original + lines[3], original + lines[4], original + lines[5],
                      original.replace(lines[4], b''), original.replace(lines[5], b''),
                      original + b'presenter: Vulkan device lost\n',
                      original + b'paint: native error\n', original + b"thread 'owner' panicked at x\n"]
        for candidate in candidates:
            with self.subTest(candidate=candidate[-70:]), self.assertRaises(ValueError):
                host.validate_run(b'', candidate, 'admitted-verified')
        with self.assertRaises(ValueError):
            host.validate_run(b'unexpected output', original, 'admitted-verified')

    def test_normal_cannot_claim_reference_and_fallback_cannot_claim_shader_success(self):
        with self.assertRaises(ValueError):
            host.validate_run(b'', synthetic('admitted-verified'), 'admitted-normal')
        with self.assertRaises(ValueError):
            host.validate_run(b'', synthetic('admitted-normal'), 'opacity-fallback')
        for replacement in [b'unsupported-opacity at phase 1', b'viewport-limit at phase 0']:
            with self.assertRaises(ValueError):
                host.validate_run(b'', synthetic('opacity-fallback').replace(b'unsupported-opacity at phase 0', replacement), 'opacity-fallback')

    def test_record_order_is_causal_not_just_a_serial_join(self):
        lines = synthetic('admitted-verified').splitlines(keepends=True)
        for left, right in [(2, 3), (3, 4), (4, 5)]:
            candidate = lines.copy()
            candidate[left], candidate[right] = candidate[right], candidate[left]
            with self.subTest(pair=(left, right)), self.assertRaises(ValueError):
                host.validate_run(b'', b''.join(candidate), 'admitted-verified')
        with self.assertRaises(ValueError):
            host.validate_run(b'', b''.join(lines) + lines[2].replace(b'serial=4', b'serial=5'), 'admitted-verified')

    def test_opacity_requires_controlled_size_but_retains_startup_other_sizes(self):
        good = synthetic('opacity-fallback')
        for suffix in [b'', b'; size=1280x1024', b'; size=0x0']:
            with self.subTest(suffix=suffix), self.assertRaises(ValueError):
                host.validate_run(b'', good.replace(b'; size=1180x880', suffix), 'opacity-fallback')
        startup = b'presenter: native raster admission fallback; complete CPU upload; reason=loading; size=1920x1080\n'
        result = host.validate_run(b'', startup + good, 'opacity-fallback')
        self.assertEqual(len(result['fallback_reasons']), 2)


class NativeWindowHostTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.binary = self.directory / 'browser'
        self.binary.write_bytes(b'nonexecutable synthetic binary identity')
        self.loader = self.directory / 'loader'
        self.loader.mkdir()
        self.fixtures = self.directory / 'fixtures'
        shutil.copytree(host.FIXTURES, self.fixtures)
        self.calls = []
        self.mutate = None
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(patch.object(host, 'FIXTURES', self.fixtures))
        self.stack.enter_context(patch.object(host, 'launch_environment', return_value={
            'DISPLAY': ':synthetic', 'PYTHONPATH': '/untrusted', 'PYTHONHOME': '/untrusted',
            'PYTHONOPTIMIZE': '1', 'MAKEFLAGS': '--jobserver-auth=3,4'}))
        self.launch = self.stack.enter_context(patch.object(host, 'run_supervised', side_effect=self.supervised))

    def supervised(self, command, output, name, env, timeout):
        self.calls.append((name, command, env.copy()))
        stderr = synthetic(name)
        (output / f'{name}.stderr.log').write_bytes(stderr)
        (output / f'{name}.supervisor.stderr.log').write_bytes(b'')
        record = {'name': name, 'command': command, 'status': 'exited', 'returncode': 0,
                  'cleanup': {'complete': True, 'descendants': 0},
                  'capture_gate_required': False, 'capture_gate_granted': False,
                  'stdout_sha256': hashlib.sha256(b'').hexdigest(),
                  'stderr_sha256': hashlib.sha256(stderr).hexdigest()}
        if self.mutate:
            self.mutate(name, record, output)
        return record, b''

    def run_case(self, suffix='result'):
        output = self.directory / suffix
        result = host.run_host(self.binary, output, self.loader)
        self.assertEqual(json.loads((output / 'host-results.json').read_text()), result)
        return result

    def test_all_cases_use_supervision_exact_flags_and_sanitized_environment(self):
        result = self.run_case()
        self.assertTrue(result['success'], result.get('error'))
        self.assertEqual([row[0] for row in self.calls], list(host.CASES))
        for name, command, env in self.calls:
            self.assertIn('--no-scripts', command)
            self.assertIn('--presenter=vulkan', command)
            self.assertIn('--raster=gpu', command)
            self.assertEqual('--vulkan-verify-frames' in command, name == 'admitted-verified')
            self.assertEqual(env['DISPLAY'], ':synthetic')
            self.assertEqual(env['LD_LIBRARY_PATH'], str(self.loader.resolve()))
            self.assertFalse(any(key in env for key in ('PYTHONPATH', 'PYTHONHOME', 'PYTHONOPTIMIZE', 'MAKEFLAGS')))

    def test_process_or_descendant_cleanup_failure_stops_remaining_cases(self):
        failures = [{'status': 'timeout', 'returncode': None}, {'returncode': 1},
                    {'cleanup': {'complete': False, 'descendants': 0}},
                    {'cleanup': {'complete': True, 'descendants': 1}}]
        for index, failure in enumerate(failures):
            self.calls.clear()
            self.mutate = lambda name, record, _output: record.update(failure)
            result = self.run_case(f'failure-{index}')
            self.assertFalse(result['success'])
            self.assertEqual(len(self.calls), 1)

    def test_binary_fixture_loader_or_raw_output_mutation_fails_closed(self):
        mutations = [lambda _n, _r, _o: self.binary.write_bytes(b'changed'),
                     lambda _n, _r, _o: (self.fixtures / 'admitted.html').write_bytes(b'changed'),
                     lambda _n, _r, _o: (self.loader / 'added').write_bytes(b'changed'),
                     lambda n, _r, o: (o / f'{n}.stderr.log').write_bytes(b'changed')]
        original_binary = self.binary.read_bytes()
        original_html = (self.fixtures / 'admitted.html').read_bytes()
        for index, mutation in enumerate(mutations):
            self.calls.clear()
            self.mutate = mutation
            result = self.run_case(f'mutation-{index}')
            self.assertFalse(result['success'])
            self.assertEqual(len(self.calls), 1)
            self.binary.write_bytes(original_binary)
            (self.fixtures / 'admitted.html').write_bytes(original_html)
            (self.loader / 'added').unlink(missing_ok=True)

    def test_supervisor_errors_grants_and_interruption_are_not_success(self):
        mutations = [lambda _n, r, _o: r.update(capture_gate_granted=True),
                     lambda n, _r, o: (o / f'{n}.supervisor.stderr.log').write_bytes(b'error')]
        for index, mutation in enumerate(mutations):
            self.calls.clear()
            self.mutate = mutation
            result = self.run_case(f'supervisor-{index}')
            self.assertFalse(result['success'])
            self.assertEqual(len(self.calls), 1)
        self.launch.side_effect = KeyboardInterrupt
        result = self.run_case('interrupted')
        self.assertFalse(result['success'])
        self.assertIn('KeyboardInterrupt', result['error'])

    def test_owned_sizing_wraps_each_case_and_requires_successful_control_receipt(self):
        controller = self.directory / 'hyprctl'
        controller.write_bytes(b'not executable controller identity')
        def controlled(command, output, name, env, timeout):
            record, stdout = self.supervised(command, output, name, env, timeout)
            receipt = Path(command[command.index('--receipt') + 1])
            receipt.write_text(json.dumps({'success': True, 'control_complete': True,
                                          'browser_reaped': True, 'browser_returncode': 0,
                                          'controlled_size': list(host.SIZE),
                                          'binary_sha256': host.digest(self.binary, host.MAX_BINARY),
                                          'controller_sha256': host.digest(controller, host.MAX_BINARY)}))
            return record, stdout
        self.launch.side_effect = controlled
        with patch.object(host.shutil, 'which', return_value=str(controller)):
            result = host.run_host(self.binary, self.directory / 'controlled', self.loader, hyprland_size='1180x880')
            self.assertTrue(result['success'], result.get('error'))
            self.assertEqual(len(self.calls), 3)
            for _, command, _ in self.calls:
                self.assertIn('--owned-launch', command)
                self.assertIn('--allow-experimental-gpu', command)
            self.launch.side_effect = self.supervised
            result = host.run_host(self.binary, self.directory / 'missing-control', self.loader, hyprland_size='1180x880')
            self.assertFalse(result['success'])
            self.assertIn('owned-window control', result['error'])

    def test_invalid_inputs_and_overwrite_stop_before_launch(self):
        for timeout, seconds in [(float('nan'), 6), (30, float('inf')), (10, 6), (121, 6), (30, 21)]:
            with self.assertRaises(ValueError):
                host.run_host(self.binary, self.directory / 'new', self.loader, timeout, seconds)
        with self.assertRaises(ValueError):
            host.run_host(self.binary, self.directory, self.loader)
        with self.assertRaises(ValueError):
            host.run_host(self.binary, self.fixtures / 'new', self.loader)
        self.launch.assert_not_called()


class OwnedWindowControlTests(unittest.TestCase):
    """Fake controller and owned-child observations only: no subprocesses."""

    def setUp(self):
        self.identity = {'pid': 71, 'parent_pid': 70, 'process_group': 70, 'start_ticks': 99}
        self.record = {'commands': [], 'control_complete': False}
        self.clock = 0.0
        self.calls = []
        self.snapshots = [self.client(False, [1900, 1000]), self.client(True, [1900, 1000]),
                          self.client(True, list(host.SIZE))]
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.live = self.stack.enter_context(patch.object(host, 'ensure_live_child'))
        self.invoke = self.stack.enter_context(patch.object(host, 'controller_call', side_effect=self.controller))
        self.stack.enter_context(patch.object(host.time, 'monotonic', side_effect=lambda: self.clock))
        self.stack.enter_context(patch.object(host.time, 'sleep', side_effect=self.advance))

    def advance(self, seconds):
        self.clock += seconds

    @staticmethod
    def client(floating, size, pid=71):
        return {'pid': pid, 'size': size, 'mapped': True, 'floating': floating,
                'title': 'private window metadata that must not be retained'}

    def controller(self, _controller, args, _env, _timeout):
        self.calls.append(args)
        self.clock += 0.01
        if args == ['-j', 'clients']:
            selected = self.snapshots.pop(0) if len(self.snapshots) > 1 else self.snapshots[0]
            output = json.dumps([self.client(False, [900, 700], pid=999), selected]).encode()
        else:
            output = b'ok\n'
        return SimpleNamespace(record={'status': 'exited', 'returncode': 0}, stdout=output, stderr=b'')

    def configure(self):
        host.configure_owned(self.identity, Path('/fake/hyprctl'), {}, self.record, lambda: None)

    def test_only_owned_pid_dispatches_and_filtered_client_metadata_retained(self):
        self.configure()
        self.assertTrue(self.record['control_complete'])
        self.assertEqual(self.record['controlled_size'], list(host.SIZE))
        dispatches = [args for args in self.calls if args[0] == 'dispatch']
        self.assertEqual(dispatches, [['dispatch', 'setfloating', 'pid:71'],
                                    ['dispatch', 'resizewindowpixel', 'exact 1180 880,pid:71']])
        serialized = json.dumps(self.record)
        self.assertNotIn('private window metadata', serialized)
        for row in self.record['commands']:
            if row['argv'][1:] == ['-j', 'clients']:
                self.assertNotIn('stdout', row)
                self.assertEqual(row['owned_client']['pid'], 71)
        self.assertEqual(self.live.call_count, 2 * len(self.calls))

    def test_mismatched_pid_never_dispatches_and_hits_bounded_deadline(self):
        self.snapshots = [self.client(False, [1900, 1000], pid=333)]
        with self.assertRaises(ValueError):
            self.configure()
        self.assertFalse(self.record['control_complete'])
        self.assertTrue(all(args == ['-j', 'clients'] for args in self.calls))
        self.assertLessEqual(len(self.calls), host.CONTROL_COMMANDS)

    def test_early_browser_exit_and_changed_identity_block_compositor_mutation(self):
        self.live.side_effect = ValueError('owned browser exited')
        with self.assertRaises(ValueError):
            self.configure()
        self.invoke.assert_not_called()

    def test_controller_timeout_nonzero_and_stderr_fail_before_dispatch(self):
        for status, code, stderr in [('timeout', -9, b''), ('output_limit', -9, b''), ('interrupted', -9, b''), ('exited', 1, b''), ('exited', 0, b'failure')]:
            self.record = {'commands': [], 'control_complete': False}
            self.invoke.side_effect = None
            self.invoke.return_value = SimpleNamespace(record={'status': status, 'returncode': code}, stdout=b'[]', stderr=stderr)
            with self.subTest(status=status, code=code, stderr=stderr), self.assertRaises(ValueError):
                self.configure()
            self.assertEqual(len(self.record['commands']), 1)
            self.assertEqual(self.record['commands'][0]['status'], status)

    def test_duplicate_or_malformed_owned_windows_are_rejected(self):
        own = self.client(False, [1180, 880])
        for clients in [[own, own], [{**own, 'size': [0, 880]}], [{**own, 'mapped': False}], [own] * 257]:
            self.record = {'commands': [], 'control_complete': False}
            self.invoke.side_effect = None
            self.invoke.return_value = SimpleNamespace(record={'status': 'exited', 'returncode': 0},
                                                      stdout=json.dumps(clients).encode(), stderr=b'')
            with self.assertRaises(ValueError):
                self.configure()
            self.assertEqual(len(self.record['commands']), 1)

    def test_resize_that_never_reaches_exact_size_fails_closed(self):
        self.snapshots = [self.client(True, [1900, 1000])]
        with self.assertRaises(ValueError):
            self.configure()
        self.assertFalse(self.record['control_complete'])
        self.assertLessEqual(len(self.calls), host.CONTROL_COMMANDS)


class OwnedChildIdentityTests(unittest.TestCase):
    def test_wnowait_exit_or_identity_change_refuses_without_reaping(self):
        identity = {'pid': 71, 'parent_pid': 70, 'process_group': 70, 'start_ticks': 99}
        with patch.object(host.os, 'waitid', return_value=object()) as wait, patch.object(host, 'child_identity') as read:
            with self.assertRaises(ValueError):
                host.ensure_live_child(identity)
            read.assert_not_called()
            wait.assert_called_once_with(host.os.P_PID, 71, host.WAIT_FLAGS)
        with patch.object(host.os, 'waitid', return_value=None), patch.object(host, 'child_identity', return_value={**identity, 'start_ticks': 100}):
            with self.assertRaises(ValueError):
                host.ensure_live_child(identity)

    def test_launcher_failure_signals_owned_child_before_reap_and_preserves_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            receipt = Path(directory) / 'control.json'
            child = SimpleNamespace(pid=71, wait=lambda timeout: -9)
            events = []
            with patch.object(host, 'digest', return_value='bound'), patch.object(host.subprocess, 'Popen', return_value=child), \
                    patch.object(host, 'child_identity', return_value={'pid': 71}), \
                    patch.object(host, 'configure_owned', side_effect=ValueError('mismatched window')), \
                    patch.object(host.os, 'waitid', side_effect=lambda *_a: events.append('owned')), \
                    patch.object(host.os, 'kill', side_effect=lambda *_a: events.append('kill')):
                child.wait = lambda timeout: events.append('reap') or -9
                self.assertEqual(host.owned_launch(['/fake/browser'], receipt, Path('/fake/controller'), 'bound', 'bound', 6), 1)
            self.assertEqual(events, ['owned', 'kill', 'reap'])
            retained = json.loads(receipt.read_bytes())
            self.assertFalse(retained['success'])
            self.assertTrue(retained['browser_reaped'])
            self.assertIn('mismatched window', retained['error'])


if __name__ == '__main__':
    unittest.main()
