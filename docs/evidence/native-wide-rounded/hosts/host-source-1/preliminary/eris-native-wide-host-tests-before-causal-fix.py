"""Synthetic wide-window tests; no renderer, worker, compositor or GPU launch."""
from contextlib import ExitStack
import io
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_raster_wide_host as wide
import native_raster_host as base
from test_native_raster_host import ADAPTER


def request(path='/admitted.html', port=12345):
    return f'GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n'.encode()


def bodies():
    return {path: (base.FIXTURES / name).read_bytes() for path, (name, _) in wide.ROUTES.items()}


def population(case='wide-verified'):
    prefix = (ADAPTER + '\npresenter: native raster admission fallback; complete CPU upload; reason=' + wide.PHYSICAL + '\n').encode()
    event = {'prefix_bytes': len(prefix), 'prefix_sha256': wide.sha(prefix),
             'html_sha256': wide.sha(bodies()['/admitted.html']), 'physical_size': list(wide.SIZE)}
    marker = (f"native wide gate: document released for owned pid=71 size=1280x880 prefix_bytes={len(prefix)} "
              f"prefix_sha256={wide.sha(prefix)} html_sha256={event['html_sha256']}\n").encode()
    reference = 'true' if case == 'wide-verified' else 'false'
    scene = ('native scene prepared: serial=4 snapshot_generation=1 page_commands=6 images=1 loading=false '
             'phases=3 lowered_commands=150 rounded_masks=3 raster_invocations=2200000 total_invocations=3326400 '
             f'reference={reference}\n').encode()
    presentation = f'presenter: native shader frame presented; size=1280x880 serial=4 CPU_upload=false reference={reference}\n'.encode()
    suffix = b''
    if case == 'wide-verified':
        suffix = (f'presenter: native shader acquired-texture verified; serial=4 generation=1 viewport_revision=2 size=1280x880 compared_bytes={wide.PIXEL_BYTES} exact=true (before compositor)\n'
                  f'presenter: Vulkan acquired-texture verification passed; route=native-raster frames=1 compared_bytes={wide.PIXEL_BYTES} last_serial=4 (before compositor)\n').encode()
    control = {'success': True, 'control_complete': True, 'controlled_size': list(wide.SIZE),
               'browser_returncode': 0, 'browser_reaped': True, 'identity': {'pid': 71},
               'release': event, 'accepted_connections': 2, 'requests': []}
    for path, body in bodies().items():
        response = wide.response_for(path, bodies())
        control['requests'].append({'path': path, 'method': 'GET', 'header_bytes': len(request(path)),
                                    'header_sha256': wide.sha(request(path)), 'body_bytes': len(body),
                                    'body_sha256': wide.sha(body), 'response_complete': True,
                                    'response_bytes': len(response), 'response_sha256': wide.sha(response)})
    return prefix + marker + scene + presentation + suffix, control


class WideProtocolTests(unittest.TestCase):
    def test_verified_and_normal_use_exact_wide_population_without_predicted_op_count(self):
        for case in wide.CASES:
            data, control = population(case)
            result = wide.validate_run(b'', data, case, control)
            self.assertEqual(result['acquired_texture_compared_bytes'], wide.PIXEL_BYTES if case == 'wide-verified' else 0)
            self.assertEqual(result['prepared_scenes'][0]['rounded_masks'], 3)
            result = wide.validate_run(b'', data.replace(b'lowered_commands=150', b'lowered_commands=151'), case, control)
            self.assertEqual(result['prepared_scenes'][0]['lowered_commands'], 151)

    def test_native_proof_cannot_precede_release_or_physical_loading_confirmation(self):
        data, control = population()
        lines = data.splitlines(keepends=True)
        for a, b in [(2, 3), (3, 4), (4, 5), (5, 6)]:
            changed = lines.copy()
            changed[a], changed[b] = changed[b], changed[a]
            with self.subTest(pair=(a, b)), self.assertRaises(ValueError):
                wide.validate_run(b'', b''.join(changed), 'wide-verified', control)
        with self.assertRaises(ValueError):
            wide.validate_run(b'', data.replace(b'current loaded page; size=1280x880', b'current loaded page; size=1279x880'), 'wide-verified', control)

    def test_bounds_quota_size_masks_and_serials_are_exact(self):
        data, control = population()
        changes = [(b'rounded_masks=3', b'rounded_masks=2'), (b'page_commands=6', b'page_commands=5'),
                   (b'images=1', b'images=0'), (b'phases=3', b'phases=4'),
                   (b'lowered_commands=150', b'lowered_commands=257'),
                   (b'total_invocations=3326400', b'total_invocations=3326401'),
                   (b'compared_bytes=4505600', b'compared_bytes=4153600'), (b'frames=1', b'frames=2'),
                   (b'size=1280x880 serial=4', b'size=1180x880 serial=4'),
                   (b'last_serial=4', b'last_serial=5'), (b'loading=false', b'loading=true')]
        for before, after in changes:
            with self.subTest(before=before), self.assertRaises(ValueError):
                wide.validate_run(b'', data.replace(before, after), 'wide-verified', control)

    def test_gate_and_response_binding_failures_cannot_pass(self):
        data, original = population()
        for change in [lambda r: r.update(control_complete=False), lambda r: r.update(controlled_size=[1180, 880]),
                       lambda r: r['release'].update(prefix_sha256='0' * 64),
                       lambda r: r['requests'][0].update(body_sha256='0' * 64),
                       lambda r: r['requests'][1].update(response_complete=False),
                       lambda r: r.update(accepted_connections=9), lambda r: r.update(error='failure')]:
            control = json.loads(json.dumps(original))
            change(control)
            with self.assertRaises(ValueError):
                wide.validate_run(b'', data, 'wide-verified', control)
        with self.assertRaises(ValueError):
            wide.validate_run(b'', data, 'wide-normal', original)


class HttpBoundsTests(unittest.TestCase):
    def test_only_exact_get_host_routes_and_bodyless_headers_are_accepted(self):
        for path in wide.ROUTES:
            self.assertEqual(wide.parse_request(request(path), 12345), path)
        for raw in [request('/../admitted.html'), request('/admitted.html?x=1'), request().replace(b'GET ', b'POST '),
                    request(port=9), request().replace(b'Connection: close', b'Host: attacker'),
                    request().replace(b'Connection: close', b'Content-Length: 1'),
                    request().replace(b'Connection: close', b'Transfer-Encoding: chunked'),
                    request() + b'extra', b'X' * (wide.MAX_HEADER + 1)]:
            with self.assertRaises(ValueError):
                wide.parse_request(raw, 12345)

    def test_response_body_is_exact_frozen_bytes_with_explicit_length_and_close(self):
        for path, body in bodies().items():
            header, actual = wide.response_for(path, bodies()).split(b'\r\n\r\n', 1)
            self.assertEqual(actual, body)
            self.assertIn(f'Content-Length: {len(body)}'.encode(), header)
            self.assertIn(b'Connection: close', header)


class ScheduledSelector:
    def __init__(self, schedule):
        self.schedule = list(schedule)
        self.entries = {}
    def register(self, obj, events, data):
        if obj in self.entries:
            raise KeyError('duplicate register')
        self.entries[obj] = (events, data)
    def modify(self, obj, events, data):
        if obj not in self.entries:
            raise KeyError('modify after unregister')
        self.entries[obj] = (events, data)
    def unregister(self, obj):
        del self.entries[obj]
    def select(self, _timeout):
        if not self.schedule:
            raise ValueError('synthetic event schedule exhausted')
        label = self.schedule.pop(0)
        found = next((obj for obj, (_, data) in self.entries.items() if data == label), None)
        if found is not None and label == 'http' and self.entries[found][0] == wide.selectors.EVENT_WRITE:
            self.schedule.insert(0, 'http')  # Keep servicing deliberately partial writes.
        return [] if found is None else [(SimpleNamespace(fileobj=found, fd=found.fileno(), data=label), 0)]
    def close(self):
        self.entries.clear()


class HttpReleaseTests(unittest.TestCase):
    def scenario(self, physical=True, controlled=True, first_native=False, bad_header=None, connections=0):
        read_fd, write_fd = os.pipe()
        startup = ('presenter: native raster admission fallback; complete CPU upload; reason=' +
                   (wide.PHYSICAL if physical else 'native scene awaits a current loaded page; size=1275x764') + '\n').encode()
        if first_native:
            startup += b'presenter: native shader frame presented; size=1280x880 serial=4 CPU_upload=false reference=false\n'
        os.write(write_fd, startup)
        os.close(write_fd)
        stream = os.fdopen(read_fd, 'rb')
        self.addCleanup(stream.close)
        class Connection:
            def __init__(self, fd):
                self.fd, self.incoming, self.outgoing = fd, bytearray(), bytearray()
                self.closed = False
            def fileno(self): return self.fd
            def setblocking(self, _value): pass
            def close(self): self.closed = True
            def recv(self, size):
                data = bytes(self.incoming[:size])
                del self.incoming[:size]
                return data
            def send(self, data):
                if self.closed: raise OSError('closed fake connection')
                count = min(17, len(data))
                self.outgoing.extend(data[:count])
                return count
        servers = [Connection(-81), Connection(-82)]
        pairs = [(server, SimpleNamespace(recv=lambda n, server=server: bytes(server.outgoing[:n]))) for server in servers]
        servers[0].incoming.extend(bad_header if bad_header is not None else request())
        servers[1].incoming.extend(request('/two-pixels.png'))
        pending = servers.copy()
        class Listener:
            def fileno(self): return -71
            def accept(self): return pending.pop(0), ('127.0.0.1', 123)
        listener = Listener()
        selector = ScheduledSelector(['listener', 'http', 'stderr', 'http', 'listener', 'http', 'http', 'stderr'])
        record = {'port': 12345, 'identity': {'pid': 71}, 'accepted_connections': connections,
                  'requests': [], 'control_complete': controlled, 'controlled_size': list(wide.SIZE)}
        emitted = []
        context = ExitStack()
        context.enter_context(patch.object(wide.selectors, 'DefaultSelector', return_value=selector))
        context.enter_context(patch.object(base, 'ensure_live_child'))
        context.enter_context(patch.object(wide.os, 'waitid', return_value=object()))
        return stream, listener, pairs, record, emitted, context, startup

    def test_document_response_waits_for_both_signals_then_serves_exact_fixed_bodies(self):
        stream, listener, pairs, record, emitted, context, startup = self.scenario()
        with context:
            wide.serve_and_relay(SimpleNamespace(pid=71, stderr=stream), listener, record, bodies(), lambda: None,
                                 wide.time.monotonic() + 2, emitted.append)
        self.assertEqual(record['release']['prefix_sha256'], wide.sha(startup))
        self.assertEqual([r['path'] for r in record['requests']], list(wide.ROUTES))
        for (_, client), path in zip(pairs, wide.ROUTES):
            self.assertEqual(client.recv(4096), wide.response_for(path, bodies()))
        self.assertTrue(all(r['response_complete'] for r in record['requests']))
        self.assertIn(b'native wide gate:', b''.join(emitted))

    def test_missing_physical_or_compositor_confirmation_never_releases_document(self):
        for kwargs in [{'physical': False}, {'controlled': False}]:
            stream, listener, _pairs, record, emitted, context, _ = self.scenario(**kwargs)
            with context, self.assertRaises(ValueError):
                wide.serve_and_relay(SimpleNamespace(pid=71, stderr=stream), listener, record, bodies(), lambda: None,
                                     wide.time.monotonic() + 2, emitted.append)
            self.assertNotIn('release', record)
            self.assertNotIn(b'native wide gate:', b''.join(emitted))

    def test_early_native_frame_connection_cap_and_oversized_header_refuse(self):
        for kwargs in [{'first_native': True}, {'connections': wide.MAX_CONNECTIONS},
                       {'bad_header': b'X' * (wide.MAX_HEADER + 1)}]:
            stream, listener, _pairs, record, emitted, context, _ = self.scenario(**kwargs)
            with context, self.assertRaises(ValueError):
                wide.serve_and_relay(SimpleNamespace(pid=71, stderr=stream), listener, record, bodies(), lambda: None,
                                     wide.time.monotonic() + 2, emitted.append)
            self.assertFalse(all(r['response_complete'] for r in record['requests']) and len(record['requests']) == 2)


class WideHostTests(unittest.TestCase):
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
        self.stack.enter_context(patch.object(wide.shutil, 'which', return_value=str(self.controller)))
        self.stack.enter_context(patch.object(base, 'environment', return_value={}))
        self.calls = []
        self.failure = None
        self.stack.enter_context(patch.object(base, 'run_supervised', side_effect=self.supervised))
    def supervised(self, command, output, name, env, timeout):
        self.calls.append(command)
        raw, control = population(name)
        control.update(binary_sha256=base.digest(self.binary, base.MAX_BINARY),
                       controller_sha256=base.digest(self.controller, base.MAX_BINARY), fixture_freeze=base.FIXTURE_SHA256)
        Path(command[command.index('--receipt') + 1]).write_text(json.dumps(control))
        (output / f'{name}.stderr.log').write_bytes(raw)
        (output / f'{name}.supervisor.stderr.log').write_bytes(b'')
        result = {'status': 'exited', 'returncode': 0, 'cleanup': {'complete': True, 'descendants': 0},
                  'stdout_sha256': wide.sha(b''), 'stderr_sha256': wide.sha(raw)}
        if self.failure:
            result.update(self.failure)
        return result, b''
    def test_two_separate_cases_bind_resources_and_verify_flag(self):
        result = wide.run_host(self.binary, self.root / 'out', self.loader)
        self.assertTrue(result['success'], result.get('error'))
        self.assertEqual(len(self.calls), 2)
        self.assertIn('--verify', self.calls[0])
        self.assertNotIn('--verify', self.calls[1])
        self.assertEqual(result['window_seconds'], 6)
        self.assertEqual(result['timeout_seconds'], 30)
    def test_cleanup_failure_stops_before_second_window(self):
        self.failure = {'cleanup': {'complete': False, 'descendants': 1}}
        result = wide.run_host(self.binary, self.root / 'out', self.loader)
        self.assertFalse(result['success'])
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(json.loads((self.root / 'out/wide-host-results.json').read_text()), result)
    def test_configuration_failure_preserves_pipe_evidence_and_owned_kill_before_reap(self):
        read_fd, write_fd = os.pipe()
        os.write(write_fd, b'original wide startup diagnostic\n')
        os.close(write_fd)
        stream = os.fdopen(read_fd, 'rb')
        child = SimpleNamespace(pid=71, stderr=stream)
        events = []
        child.wait = lambda timeout: events.append('reap') or -9
        captured = io.BytesIO()
        fake_listener = SimpleNamespace(bind=lambda _: None, listen=lambda _: None, setblocking=lambda _: None,
                                        getsockname=lambda: ('127.0.0.1', 12345), close=lambda: None)
        receipt = self.root / 'launch.json'
        with patch.object(base, 'digest', return_value='bound'), patch.object(wide.socket, 'socket', return_value=fake_listener), \
                patch.object(wide.subprocess, 'Popen', return_value=child), patch.object(base, 'child_identity', return_value={'pid': 71}), \
                patch.object(base, 'configure_owned', side_effect=ValueError('controller failed')), \
                patch.object(wide.os, 'waitid', side_effect=lambda *_: events.append('owned')), \
                patch.object(wide.os, 'kill', side_effect=lambda *_: events.append('kill')), \
                patch.object(wide.sys, 'stderr', SimpleNamespace(buffer=captured)), \
                patch.object(base, 'fixture_binding', return_value={'freeze_sha256': base.FIXTURE_SHA256}):
            code = wide.launch_wide(self.binary, receipt, self.controller, 'bound', 'bound', 6, True)
        self.assertEqual(code, 1)
        self.assertEqual(events, ['owned', 'kill', 'reap'])
        self.assertEqual(captured.getvalue(), b'original wide startup diagnostic\n')
        self.assertFalse(json.loads(receipt.read_text())['success'])


if __name__ == '__main__':
    unittest.main()
