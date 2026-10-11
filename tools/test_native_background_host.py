"""Independent synthetic protocol oracles; these tests never launch a process."""
import copy
import hashlib
import json
from pathlib import Path
import struct
import unittest

import native_background_host as host


PORT = 47123
URL = f'http://127.0.0.1:{PORT}/admitted.html'
KEY = b'eris-css-background:0'
RGBA = bytes.fromhex('ff0000ff00ff00ff0000ffff00ffffffff00ffffffff00ff000000ffff8000ff')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def identity():
    """Encode the frozen literal fields independently of the host reader."""
    data, offsets = bytearray(), {}
    def write(name, value):
        offsets[name] = len(data)
        data.extend(value)
    def scalar(name, *values):
        write(name, struct.pack('<' + 'f' * len(values), *values))
    def count(name, value):
        write(name, struct.pack('<I', value))
    def text(name, value):
        value = value.encode() if isinstance(value, str) else value
        count(name + '_length', len(value)); write(name, value)
    write('magic', b'EBNI\x01')
    count('width', 1280); count('height', 880)
    scalar('scroll', 0); scalar('zoom', 1)
    text('title', 'Native background window'); text('address', URL)
    write('address_focus', b'\0'); text('input', '')
    count('caret', 0); count('anchor', 0); write('focused_node', b'\0')
    scalar('content_height', 779); text('status', 'Native timing scene')
    count('commands', 4)
    write('canvas_tag', b'\x06'); scalar('canvas_rect', 0, 0, 1280, 779)
    write('canvas_color', b'\xff' * 4); scalar('canvas_radius', 0)
    for name, x, kind in [('color', 10, 0), ('image', 26, 1), ('gradient', 42, 2)]:
        write(name + '_tag', b'\x0d'); scalar(name + '_rect', x, 10, 4, 2)
        scalar(name + '_radii', *([0] * 8)); write(name + '_cutout', b'\0')
        write(name + '_paint', bytes([kind]))
        if kind == 0:
            write('fill_color', bytes.fromhex('00ff00ff'))
        else:
            write(name + '_repeat', b'\0'); scalar(name + '_tile', x + 1, 11, 4, 2)
            if kind == 1:
                text('image_key', KEY)
            else:
                write('current_color', bytes.fromhex('ff0000ff')); scalar('stop_scale', 1)
                write('direction', b'\0'); write('angle', struct.pack('<d', 90))
                count('stop_count', 2)
                write('stop0_color', b'\x01'); write('stop0_present', b'\x01')
                write('stop0_position', struct.pack('<dd', 0, 50))
                write('stop1_color', b'\0\0\0\xff\xff'); write('stop1_present', b'\x01')
                write('stop1_position', struct.pack('<dd', 0, 50))
    count('images', 1); text('store_key', KEY)
    count('alias', 0); count('image_width', 4); count('image_height', 2)
    text('rgba', RGBA)
    return bytes(data), offsets


def records(case='admitted'):
    bodies = {'/admitted.html': b'<html>frozen test document</html>', '/two-pixels.png': b'frozen PNG bytes'}
    requests = []
    for path, body in bodies.items():
        kind = 'text/html; charset=utf-8' if path.endswith('.html') else 'image/png'
        response = (f'HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n'.encode() + body)
        requests.append(dict(path=path, method='GET', body_bytes=len(body), body_sha256=digest(body),
                             header_bytes=64, header_sha256='a' * 64, response_complete=True,
                             response_bytes=len(response), response_sha256=digest(response)))
    control = dict(success=True, control_complete=True, browser_reaped=True, browser_returncode=0,
                   controlled_size=[1280, 880], port=PORT, identity=dict(pid=100, parent_pid=99, process_group=99, start_ticks=1000),
                   accepted_connections=2, requests=requests)
    adapter = 'presenter: Vulkan adapter="fixture adapter" type=Cpu driver="fixture driver" format=Bgra8Unorm color_space=Srgb alpha=Opaque mode=Fifo; raster=native-shaders with complete CPU admission fallback\n'
    waiting = ('native benchmark waiting: size=1280x880 loaded=false\n' if case == 'admitted' else
               'presenter: native raster admission fallback; complete CPU upload; reason=native scene awaits a current loaded page; size=1280x880\n')
    prefix = (adapter + waiting).encode()
    confinement = (b'[page] Page process 101: Landlock ABI 6 and seccomp: no direct resource or socket access\n' +
                   b'[page] Resource broker 102: committed URL and fetch policy enforced outside renderer\n')
    control['release'] = dict(prefix_bytes=len(prefix), prefix_sha256=digest(prefix), html_sha256=digest(bodies['/admitted.html']), physical_size=[1280, 880])
    release = (f'native background gate: document released for owned pid=100 size=1280x880 prefix_bytes={len(prefix)} prefix_sha256={digest(prefix)} html_sha256={digest(bodies["/admitted.html"])}\n').encode()
    stats = b'native boxes prepared: serial=7 generation=1 viewport_revision=3 box_rectangles=1 box_masks=0 box_background_images=1 box_gradients=1 box_loop_work=159 generated_gradient_sources=1 generated_gradient_bytes=12\n'
    tail = (stats + b'presenter: native shader frame presented; size=1280x880 serial=7 CPU_upload=false reference=true\n' +
            b'presenter: native shader acquired-texture verified; serial=7 generation=1 viewport_revision=3 size=1280x880 compared_bytes=4505600 exact=true (before compositor)\n' +
            b'presenter: Vulkan acquired-texture verification passed; route=native-raster frames=1 compared_bytes=4505600 last_serial=7 (before compositor)\n')
    if case == 'rounded':
        tail = b'presenter: native raster admission fallback; complete CPU upload; reason=unsupported-box-decoration at phase 0 original command 2; size=1280x880\n'
    report = dict(schema=1, kind='native-benchmark', mode='check', route='native-raster', requested_frames=1,
                  completed_frames=1, accepted_frames=0, scene_hex=identity()[0].hex(), width=1280, height=880,
                  generation=1, viewport_revision=3, scale_factor=1, source_load_ms=1.5, edit_sequence=0,
                  diagnostics=2, checked=True, failure=None, owner_failure=None, owner_init_ns=None, samples=[])
    return report, prefix + release + confinement + tail, control, bodies


class BackgroundHostTests(unittest.TestCase):
    def validate(self, report=None, stderr=None, control=None, bodies=None, case='admitted'):
        old = records(case)
        values = [old[i] if value is None else value for i, value in enumerate((report, stderr, control, bodies))]
        stdout = json.dumps(values[0]).encode() + b'\n' if case == 'admitted' else b''
        return host.validate_run(stdout, values[1], values[2], PORT, values[3], case)

    def test_complete_literal_identity_and_acquired_check(self):
        parsed = host.parse_identity(identity()[0].hex(), URL)
        self.assertEqual((parsed['commands'], parsed['images'], parsed['rgba_bytes']), (4, 1, 32))
        result = self.validate()
        self.assertEqual((result['verified_frames'], result['compared_bytes'], result['box_loop_work']), (1, 4505600, 159))

    def test_every_identity_truncation_extra_byte_and_oversize_is_rejected(self):
        data, _ = identity()
        for size in range(len(data)):
            with self.subTest(size=size), self.assertRaises(ValueError):
                host.parse_identity(data[:size].hex(), URL)
        for value in (data.hex() + '00', data.hex().upper(), '0', 'gg', '00' * 65537):
            with self.subTest(value=value[:20]), self.assertRaises(ValueError):
                host.parse_identity(value, URL)

    def test_every_semantic_field_is_bound_including_unused_channels(self):
        data, offsets = identity()
        for field, at in offsets.items():
            changed = bytearray(data); changed[at] ^= 1
            with self.subTest(field=field), self.assertRaises(ValueError):
                host.parse_identity(changed.hex(), URL)
        for field, raw in [('scroll', struct.pack('<f', -0.0)), ('gradient_radii', struct.pack('<f', float('nan'))),
                           ('stop_scale', struct.pack('<f', float('inf'))), ('angle', struct.pack('<d', float('nan'))),
                           ('commands', b'\xff' * 4), ('rgba_length', b'\xff' * 4)]:
            changed = bytearray(data); at = offsets[field]; changed[at:at + len(raw)] = raw
            with self.subTest(field=field), self.assertRaises(ValueError):
                host.parse_identity(changed.hex(), URL)

    def test_report_is_check_only_complete_and_current(self):
        report, _, _, _ = records()
        for field, value in [('checked', False), ('completed_frames', 0), ('requested_frames', 2),
                             ('accepted_frames', 1), ('mode', 'measure'), ('samples', [{}]), ('owner_init_ns', 0),
                             ('failure', 'failed'), ('owner_failure', 'failed'), ('diagnostics', 0), ('diagnostics', 1), ('diagnostics', 3),
                             ('edit_sequence', 1), ('scale_factor', 2), ('generation', 2), ('viewport_revision', 4),
                             ('schema', True), ('width', True), ('source_load_ms', float('nan'))]:
            changed = copy.deepcopy(report); changed[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(report=changed)

    def test_stats_and_frame_records_cannot_be_missing_forged_or_reordered(self):
        _, stderr, _, _ = records()
        for old, new in [(b'box_masks=0', b'box_masks=1'), (b'box_gradients=1', b'box_gradients=0'),
                         (b'box_loop_work=159', b'box_loop_work=158'), (b'generated_gradient_bytes=12', b'generated_gradient_bytes=16'),
                         (b'serial=7', b'serial=8'), (b'compared_bytes=4505600', b'compared_bytes=4505599'),
                         (b'exact=true', b'exact=false'), (b'reference=true', b'reference=false')]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                self.validate(stderr=stderr.replace(old, new, 1))
        lines = stderr.splitlines(keepends=True)
        for index in range(5, len(lines)):
            for replacement in ([], [lines[index], lines[index]]):
                with self.subTest(index=index, copies=len(replacement)), self.assertRaises(ValueError):
                    self.validate(stderr=b''.join(lines[:index] + replacement + lines[index + 1:]))
        with self.assertRaises(ValueError):
            self.validate(stderr=b''.join(lines[:5] + [lines[6], lines[5]] + lines[7:]))

    def test_confinement_release_http_and_cleanup_are_required(self):
        _, stderr, control, _ = records()
        for old, new in [(b'Page process 101', b'Page process 102'), (b'Landlock ABI 6', b'Landlock ABI 0'),
                         (b'[page] Resource broker', b'[page] unknown broker'), (b'owned pid=100', b'owned pid=99'),
                         (b'prefix_sha256=', b'prefix_sha256=0')]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                self.validate(stderr=stderr.replace(old, new))
        lines = stderr.splitlines(keepends=True)
        # Even a self-consistent prefix hash cannot put a loaded snapshot before the gate.
        early = lines[:2] + lines[3:5]
        changed = copy.deepcopy(control)
        changed['release'].update(prefix_bytes=len(b''.join(early)), prefix_sha256=digest(b''.join(early)))
        marker = (f'native background gate: document released for owned pid=100 size=1280x880 prefix_bytes={len(b"".join(early))} prefix_sha256={digest(b"".join(early))} html_sha256={changed["release"]["html_sha256"]}\n').encode()
        with self.assertRaises(ValueError):
            self.validate(stderr=b''.join(early) + marker + b''.join(lines[5:]), control=changed)
        for field, value in [('success', False), ('control_complete', False), ('browser_reaped', False),
                             ('browser_returncode', 1), ('controlled_size', [1180, 880]), ('error', 'failed')]:
            changed = copy.deepcopy(control); changed[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(control=changed)
        for field, value in [('response_complete', False), ('body_sha256', '0' * 64), ('response_bytes', 0), ('path', '/other.png')]:
            changed = copy.deepcopy(control); changed['requests'][1][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.validate(control=changed)

    def test_negative_is_exact_fallback_and_never_pixel_success(self):
        result = self.validate(case='rounded')
        self.assertEqual((result['verified_frames'], result['compared_bytes']), (0, 0))
        _, stderr, _, _ = records('rounded')
        lines = stderr.splitlines(keepends=True)
        with self.assertRaises(ValueError):
            self.validate(stderr=b''.join(lines[:3] + lines[5:] + lines[3:5]), case='rounded')
        for old, new in [(b'original command 2', b'original command 1'), (b'phase 0', b'phase 1'),
                         (b'unsupported-box-decoration', b'planner-budget')]:
            with self.subTest(old=old), self.assertRaises(ValueError):
                self.validate(stderr=stderr.replace(old, new), case='rounded')
        for extra in [b'native boxes prepared: serial=7 generation=1 viewport_revision=3 box_rectangles=1 box_masks=0 box_background_images=1 box_gradients=1 box_loop_work=159 generated_gradient_sources=1 generated_gradient_bytes=12\n',
                      b'presenter: native shader frame presented; size=1280x880 serial=7 CPU_upload=false reference=false\n']:
            with self.assertRaises(ValueError):
                self.validate(stderr=stderr + extra, case='rounded')

    def test_malformed_or_unknown_proof_is_fatal(self):
        _, stderr, _, _ = records()
        for value in (stderr[:-1], stderr + b'\xff\n', stderr + b'presenter: unrecognized proof\n',
                      stderr + b'[page] resource failed\n', stderr + b"thread 'main' panicked at bad input\n",
                      stderr + b'x' * 16385 + b'\n', stderr + b'x\n' * 4097):
            with self.assertRaises(ValueError):
                self.validate(stderr=value)
        report, stderr, control, bodies = records()
        for raw in (json.dumps(report).encode(), b'{"schema":1,"schema":1}\n', b'{}\n', b'x' * 262145):
            with self.assertRaises(ValueError):
                host.validate_run(raw, stderr, control, PORT, bodies, 'admitted')



# Frozen before the Xlib control implementation. These fakes do not load Xlib,
# connect to X11, or launch a subprocess.
import ctypes
from unittest import mock


class XFunction:
    def __init__(self, call):
        self.call = call
    def __call__(self, *args):
        return self.call(*args)


class FakeX11:
    def __init__(self, clients=(11,), owners=None):
        self.clients = list(clients)
        self.owners = dict({11: 100} if owners is None else owners)
        self.buffers = []
        self.freed = 0
        self.closed = 0
        self.resizes = []
        self.reads = []
        self.owner_reads = {}
        self.change_before = False
        self.change_after = False
        self.bad_property = None
        self.final_size = (1280, 880)
        self.geometry_calls = 0
        self.XOpenDisplay = XFunction(lambda name: 123)
        self.XCloseDisplay = XFunction(self.close)
        self.XDefaultRootWindow = XFunction(lambda display: 1)
        self.XInternAtom = XFunction(lambda display, name, only: {b'_NET_CLIENT_LIST': 2, b'_NET_WM_PID': 3}[name])
        self.XSetErrorHandler = XFunction(lambda callback: None)
        self.XSync = XFunction(lambda display, discard: 0)
        self.XFree = XFunction(self.free)
        self.XGetWindowProperty = XFunction(self.property)
        self.XResizeWindow = XFunction(self.resize)
        self.XGetGeometry = XFunction(self.geometry)
    def close(self, display):
        self.closed += 1
        return 0
    def free(self, pointer):
        self.freed += 1
        return 0
    @staticmethod
    def put(pointer, kind, value):
        ctypes.cast(pointer, ctypes.POINTER(kind))[0] = value
    def property(self, display, window, atom, offset, length, delete, requested, actual, fmt, count, after, output):
        self.reads.append((window, atom, length))
        assert offset == 0 and delete == 0
        if atom == 2:
            values, kind = self.clients, 33
            assert window == 1 and length == 256 and requested == 33
        else:
            assert atom == 3 and length == 1 and requested == 6
            self.owner_reads[window] = self.owner_reads.get(window, 0) + 1
            owner = self.owners.get(window)
            if window == 11 and ((self.change_before and self.owner_reads[window] >= 2) or (self.change_after and self.resizes)):
                owner = 999
            values, kind = ([] if owner is None else [owner]), (0 if owner is None else 6)
        buffer = (ctypes.c_ulong * max(1, len(values)))(*values)
        self.buffers.append(buffer)
        self.put(actual, ctypes.c_ulong, kind)
        self.put(fmt, ctypes.c_int, 32 if kind else 0)
        self.put(count, ctypes.c_ulong, len(values))
        self.put(after, ctypes.c_ulong, 0)
        ctypes.cast(output, ctypes.POINTER(ctypes.POINTER(ctypes.c_ubyte)))[0] = ctypes.cast(buffer, ctypes.POINTER(ctypes.c_ubyte))
        if atom == 2 and self.bad_property:
            field, value = self.bad_property
            pointer, typ = {'format': (fmt, ctypes.c_int), 'type': (actual, ctypes.c_ulong), 'count': (count, ctypes.c_ulong), 'after': (after, ctypes.c_ulong)}[field]
            self.put(pointer, typ, value)
        return 0
    def resize(self, display, window, width, height):
        self.resizes.append((window, width, height))
        return 1
    def geometry(self, display, window, root, x, y, width, height, border, depth):
        self.geometry_calls += 1
        size = self.final_size if self.resizes else (1920, 1320)
        for pointer, kind, value in [(root, ctypes.c_ulong, 1), (x, ctypes.c_int, 0), (y, ctypes.c_int, 0), (width, ctypes.c_uint, size[0]), (height, ctypes.c_uint, size[1]), (border, ctypes.c_uint, 0), (depth, ctypes.c_uint, 24)]:
            self.put(pointer, kind, value)
        return 1


class X11PhysicalControlTests(unittest.TestCase):
    def run_fake(self, lib):
        with mock.patch.object(host.time, 'sleep'):
            return host.resize_x11(lib, 100)

    def test_unique_owned_window_exact_physical_size_and_release(self):
        lib = FakeX11((10, 11, 12), {10: 9, 11: 100})
        result = self.run_fake(lib)
        self.assertEqual(result['pid'], 100)
        self.assertEqual(result['window'], 11)
        self.assertEqual(result['pid_before'], result['pid_after'])
        self.assertEqual(result['pid_after'], 100)
        self.assertEqual(result['initial_size'], [1920, 1320])
        self.assertEqual(result['requested_size'], [1280, 880])
        self.assertEqual(result['observed_size'], [1280, 880])
        self.assertEqual(lib.resizes, [(11, 1280, 880)])
        self.assertEqual(lib.closed, 1)
        self.assertEqual(lib.freed, len(lib.reads))
        self.assertEqual(lib.geometry_calls, 2)
        self.assertEqual(len(lib.XGetWindowProperty.argtypes), 12)
        self.assertEqual(lib.XResizeWindow.restype, ctypes.c_int)

    def test_bounded_client_list_and_duplicate_or_absent_pid_refuse_before_resize(self):
        for clients, owners in [((), {}), ((10, 11), {10: 100, 11: 100}), ((10,), {10: 999})]:
            lib = FakeX11(clients, owners)
            with self.assertRaises(ValueError): self.run_fake(lib)
            self.assertEqual(lib.resizes, [])
            self.assertEqual(lib.closed, 1)
        for bad in [('format', 8), ('type', 6), ('count', 257), ('after', 4)]:
            lib = FakeX11(); lib.bad_property = bad
            with self.assertRaises(ValueError): self.run_fake(lib)
            self.assertEqual(lib.resizes, [])
            self.assertEqual(lib.freed, 1)
            self.assertEqual(lib.closed, 1)
        lib = FakeX11(tuple(range(1, 257)), {11: 100})
        self.run_fake(lib)
        self.assertEqual(len(lib.reads), 259)  # client list + 256 PID reads + two rechecks
        self.assertEqual(lib.resizes, [(11, 1280, 880)])

    def test_pid_recheck_before_and_after_configuration(self):
        for field, expected_resizes in [('change_before', []), ('change_after', [(11, 1280, 880)])]:
            lib = FakeX11(); setattr(lib, field, True)
            with self.assertRaises(ValueError): self.run_fake(lib)
            self.assertEqual(lib.resizes, expected_resizes)
            self.assertEqual(lib.closed, 1)
            self.assertEqual(lib.freed, len(lib.reads))

    def test_failed_physical_confirmation_is_bounded_and_closes_display(self):
        lib = FakeX11(); lib.final_size = (1279, 880)
        with self.assertRaises(ValueError): self.run_fake(lib)
        self.assertEqual(lib.resizes, [(11, 1280, 880)])
        self.assertEqual(lib.geometry_calls, 33)  # initial plus at most32 confirmations
        self.assertEqual(lib.closed, 1)
        lib = FakeX11(); lib.XOpenDisplay = XFunction(lambda name: None)
        with self.assertRaises(ValueError): self.run_fake(lib)
        self.assertEqual(lib.closed, 0)
        self.assertEqual(lib.resizes, [])

    def test_parent_control_refuses_expired_or_full_command_ledger(self):
        identity = dict(pid=100, parent_pid=99, process_group=99, start_ticks=1000)
        library = dict(path='/fixed/libX11.so.6.4.0', bytes=10, sha256='a' * 64)
        for commands, deadline in [([], 0), ([{}] * host.base.CONTROL_COMMANDS, 10)]:
            record = dict(identity=identity, commands=commands, control_complete=True, controlled_size=[1280, 880])
            with mock.patch.object(host.time, 'monotonic', return_value=1), mock.patch.object(host.base, 'ensure_live_child') as live, mock.patch.object(host.base, 'controller_call') as call:
                with self.assertRaises(ValueError):
                    host.configure_x11_owned(identity, library, {}, record, lambda: None, deadline)
                call.assert_not_called()
                self.assertFalse(record['control_complete'])



class X11OuterJoinTests(unittest.TestCase):
    def test_frozen_library_owned_pid_and_physical_receipt_are_bound(self):
        library = dict(path='/fixed/libX11.so.6.4.0', bytes=10, sha256='a' * 64)
        physical = dict(library=library, pid=100, window=11, pid_before=100, pid_after=100,
                        client_count=3, property_reads=6, geometry_reads=2,
                        initial_size=[1920, 1320], requested_size=[1280, 880], observed_size=[1280, 880],
                        scope='unique X11 PID-property match; parent retains unreaped browser identity')
        stdout = json.dumps(physical, sort_keys=True) + '\n'
        argv = [host.sys.executable, str(Path(host.__file__).resolve()), '--x11-resize-child', '--owned-pid', '100',
                '--x11-library', library['path'], '--x11-library-sha256', library['sha256'], '--allow-experimental-gpu']
        row = dict(argv=argv, status='exited', returncode=0, stdout=stdout,
                   stdout_bytes=len(stdout.encode()), stdout_sha256=digest(stdout.encode()),
                   stderr='', stderr_bytes=0, stderr_sha256=digest(b''))
        control = dict(identity=dict(pid=100), x11_library=library, control_size_space='x11-physical',
                       logical_controlled_size_before_physical=[1280, 880], controlled_size=[1280, 880],
                       x11_physical_control=physical, commands=[row])
        host.validate_x11_control(control, library)
        host.validate_x11_control(dict(x11_library=None), None)
        for key in ('x11_library', 'control_size_space', 'x11_physical_control'):
            changed = copy.deepcopy(control); del changed[key]
            with self.assertRaises(ValueError): host.validate_x11_control(changed, library)
        for key, value in [('library', {**library, 'sha256': 'b' * 64}), ('pid', 101), ('pid_before', 101),
                           ('pid_after', 101), ('window', 0), ('observed_size', [1279, 880]),
                           ('requested_size', [1920, 1320]), ('client_count', 257), ('property_reads', 7),
                           ('geometry_reads', 34), ('scope', 'kernel ownership')]:
            changed = copy.deepcopy(control); changed['x11_physical_control'][key] = value
            with self.assertRaises(ValueError): host.validate_x11_control(changed, library)
        for key, value in [('stdout_sha256', 'b' * 64), ('status', 'timeout'), ('stderr_bytes', 1)]:
            changed = copy.deepcopy(control); changed['commands'][0][key] = value
            with self.assertRaises(ValueError): host.validate_x11_control(changed, library)
        changed = copy.deepcopy(control); changed['commands'].append(copy.deepcopy(row))
        with self.assertRaises(ValueError): host.validate_x11_control(changed, library)
        with self.assertRaises(ValueError): host.validate_x11_control(control, None)



class UnitMonitorControlTests(unittest.TestCase):
    def monitor(self, **updates):
        result = dict(id=1, name='unit', width=2560, height=1440, scale=1.0, transform=0,
                      disabled=False, activeWorkspace=dict(id=1, name='1'))
        result.update(updates)
        return result

    def test_explicit_enabled_unit_monitor_selection_is_bounded(self):
        expected = dict(id=1, name='unit', width=2560, height=1440, scale=1.0, transform=0,
                        disabled=False, active_workspace=1)
        self.assertEqual(host.select_unit_monitor([self.monitor(id=0, scale=1.5), self.monitor()], 1), expected)
        for rows in [[], [self.monitor(), self.monitor()], [self.monitor(id=0)], [self.monitor()] * 17,
                     [self.monitor(disabled=True)], [self.monitor(transform=1)], [self.monitor(scale=1.5)],
                     [self.monitor(width=1279)], [self.monitor(height=879)], [self.monitor(activeWorkspace=dict(id=0))]]:
            with self.assertRaises(ValueError): host.select_unit_monitor(rows, 1)
        for selected in (-1, 256, True):
            with self.assertRaises(ValueError): host.select_unit_monitor([self.monitor()], selected)

    def run_control(self, client_pid=100, final_monitor=1):
        from types import SimpleNamespace
        identity = dict(pid=100, parent_pid=99, process_group=99, start_ticks=1000)
        before = dict(pid=client_pid, monitor=0, workspace=dict(id=2), size=[1280, 880], floating=True, mapped=True)
        after = dict(before, monitor=final_monitor, workspace=dict(id=1))
        clients = iter([[before], [after], [after]])
        calls = []
        def call(controller, args, env, timeout):
            calls.append(args)
            self.assertLessEqual(timeout, 0.5)
            if args == ['-j', 'monitors']:
                data = json.dumps([self.monitor()]).encode()
            elif args == ['-j', 'clients']:
                data = json.dumps(next(clients, [after])).encode()
            else:
                self.assertIn(args, [['dispatch', 'movetoworkspacesilent', '1,pid:100'],
                                     ['dispatch', 'resizewindowpixel', 'exact 1280 880,pid:100']])
                data = b'ok\n'
            return SimpleNamespace(record=dict(status='exited', returncode=0), stdout=data, stderr=b'')
        record = dict(identity=identity, commands=[], control_complete=True, controlled_size=[1280, 880])
        with mock.patch.object(host.time, 'monotonic', return_value=1), mock.patch.object(host.time, 'sleep'), mock.patch.object(host.base, 'ensure_live_child'), mock.patch.object(host.base, 'controller_call', side_effect=call):
            host.configure_unit_monitor(identity, Path('/fixed/hyprctl'), {}, record, lambda: None, 4, 1)
        return record, calls

    def test_owned_pid_only_move_and_exact_fixed_resize_receipt(self):
        record, calls = self.run_control()
        self.assertEqual(len(calls), 7)
        self.assertEqual(calls[2], ['dispatch', 'movetoworkspacesilent', '1,pid:100'])
        self.assertEqual(calls[4], ['dispatch', 'resizewindowpixel', 'exact 1280 880,pid:100'])
        proof = record['unit_monitor_control']
        self.assertEqual(proof['requested_id'], 1)
        self.assertEqual(proof['owned_before']['pid'], 100)
        self.assertEqual(proof['owned_after'], dict(pid=100, monitor=1, workspace=1, size=[1280, 880], floating=True, mapped=True))
        self.assertEqual(len([r for r in record['commands'] if 'selected_monitor' in r]), 2)
        self.assertEqual(len([r for r in record['commands'] if 'owned_client' in r]), 3)
        host.validate_unit_monitor_control(record, 1)
        for key, value in [('requested_id', 0), ('owned_after', {**proof['owned_after'], 'pid': 101})]:
            changed = copy.deepcopy(record); changed['unit_monitor_control'][key] = value
            with self.assertRaises(ValueError): host.validate_unit_monitor_control(changed, 1)
        with self.assertRaises(ValueError): host.validate_unit_monitor_control(record, None)
        host.validate_unit_monitor_control(dict(commands=[]), None)

    def test_wrong_pid_wrong_monitor_and_expired_budget_refuse(self):
        with self.assertRaises(ValueError): self.run_control(client_pid=101)
        with self.assertRaises(ValueError): self.run_control(final_monitor=0)
        for commands, deadline in [([], 0), ([{}] * host.base.CONTROL_COMMANDS, 4)]:
            record = dict(commands=commands, control_complete=True)
            with mock.patch.object(host.time, 'monotonic', return_value=1), mock.patch.object(host.base, 'ensure_live_child'), mock.patch.object(host.base, 'controller_call') as call:
                with self.assertRaises(ValueError):
                    host.configure_unit_monitor(dict(pid=100), Path('/fixed/hyprctl'), {}, record, lambda: None, deadline, 1)
                call.assert_not_called()
                self.assertFalse(record['control_complete'])



class UnitMonitorTranscriptOrderTests(unittest.TestCase):
    def test_stage_order_and_dispatch_raw_hashes_are_required(self):
        original, _ = UnitMonitorControlTests().run_control()
        for order in ([0, 2, 4, 1, 3, 5, 6], [0, 1, 2, 4, 3, 5, 6], [0, 1, 2, 3, 4, 6, 5]):
            changed = copy.deepcopy(original)
            changed['commands'] = [changed['commands'][index] for index in order]
            with self.assertRaises(ValueError): host.validate_unit_monitor_control(changed, 1)
        for key, value in [('stdout', 'different\n'), ('stdout_bytes', 99), ('stdout_sha256', 'b' * 64)]:
            changed = copy.deepcopy(original); changed['commands'][2][key] = value
            with self.assertRaises(ValueError): host.validate_unit_monitor_control(changed, 1)
        changed = copy.deepcopy(original)
        changed['commands'][3]['owned_client']['monitor'] = 0
        with self.assertRaises(ValueError): host.validate_unit_monitor_control(changed, 1)


class DefaultWindowDurationTests(unittest.TestCase):
    def test_default_and_reparsed_window_duration_have_identical_argv_text(self):
        common = ['native_background_host.py', '--allow-experimental-gpu',
                  '--binary', '/fixed/browser', '--binary-sha256', 'a' * 64]
        top = ['--output', '/fixed/output', '--loader-directory', '/fixed/loader']
        owned = ['--owned-launch', '--receipt', '/fixed/receipt',
                 '--controller', '/fixed/hyprctl', '--controller-sha256', 'b' * 64,
                 '--listener-fd', '9', '--port', '47123', '--case', 'admitted']
        for mode, suffix in [('host', top), ('owned', owned)]:
            for duration in ([], ['--window-seconds', '8'], ['--window-seconds', '8.0']):
                with self.subTest(mode=mode, duration=duration):
                    with mock.patch.object(host.sys, 'argv', common + suffix + duration), \
                         mock.patch.object(host.signal, 'signal'), \
                         mock.patch.object(host, 'run_host', return_value={'success': True}) as run, \
                         mock.patch.object(host, 'launch_owned', return_value=0) as launch, \
                         mock.patch('builtins.print'):
                        self.assertEqual(host.main(), 0)
                    selected, inactive = (run, launch) if mode == 'host' else (launch, run)
                    selected.assert_called_once()
                    inactive.assert_not_called()
                    value = selected.call_args.args[0].window_seconds
                    self.assertIs(type(value), float)
                    self.assertEqual(value, 8.0)
                    self.assertEqual(['--exit-after', str(value)], ['--exit-after', '8.0'])


if __name__ == '__main__':
    unittest.main()
