#!/usr/bin/env python3
"""One bounded broker-backed acquired-window background check, plus optional fallback.

This reuses the unchanged owned-window controller, fixed two-route server and
isolated subreaper. The new reader accepts only the independently frozen scene.
"""
from __future__ import annotations

import argparse
import ctypes
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import struct
import subprocess
import sys
import time

import native_raster_host as base
import native_raster_wide_host as wide
import native_raster_timing_host as timing
from native_raster_followup import BROKER, PAGE, drain_stderr
from bridge_supervisor import validate_listener_fds

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / 'tools/native-background-fixtures'
FREEZE_SHA = '422b8d83a9481834aab6d70695478034ddc9725705482b0ea7bc8aecc413a4c5'
FILES = ('admitted.html', 'rounded.html', 'two-pixels.png', 'oracles.json', 'freeze.json')
CASES = ('admitted', 'rounded')
STATS = re.compile(r'native boxes prepared: serial=([1-9][0-9]*) generation=([1-9][0-9]*) viewport_revision=([1-9][0-9]*) box_rectangles=([0-9]+) box_masks=([0-9]+) box_background_images=([0-9]+) box_gradients=([0-9]+) box_loop_work=([0-9]+) generated_gradient_sources=([0-9]+) generated_gradient_bytes=([0-9]+)')
RELEASE = re.compile(r'native background gate: document released for owned pid=([1-9][0-9]*) size=1280x880 prefix_bytes=([0-9]+) prefix_sha256=([0-9a-f]{64}) html_sha256=([0-9a-f]{64})')
EXPECTED_STATS = (1, 0, 1, 1, 159, 1, 12)
RGBA = bytes.fromhex('ff0000ff00ff00ff0000ffff00ffffffff00ffffffff00ff000000ffff8000ff')
KEY = 'eris-css-background:0'
FALLBACK = 'unsupported-box-decoration at phase 0 original command 2; size=1280x880'


class FixtureReader(timing.IdentityReader):
    def exact(self, expected):
        if self.take(len(expected)) != expected:
            raise ValueError('scene field differs from frozen literal')

    def values(self, kind, *values):
        # Comparing the finite literal encoding also preserves signed-zero bits.
        self.exact(struct.pack('<' + kind * len(values), *values))

    def string(self, expected):
        if self.text() != expected:
            raise ValueError('scene string differs from frozen literal')

    def shape(self, x):
        self.values('f', x, 10, 4, 2)
        self.values('f', *([0] * 8))
        self.exact(b'\0')  # No cutout, including zero-area cutouts.


def parse_identity(scene_hex, url):
    if (type(scene_hex) is not str or not 2 <= len(scene_hex) <= timing.MAX_SCENE * 2
            or len(scene_hex) % 2 or re.fullmatch('[0-9a-f]+', scene_hex) is None):
        raise ValueError('bounded complete lowercase EBNI hex required')
    data = bytes.fromhex(scene_hex)
    r = FixtureReader(data)
    r.exact(b'EBNI\x01'); r.values('I', 1280, 880); r.values('f', 0, 1)
    r.string('Native background window'); r.string(url)
    r.exact(b'\0'); r.string(''); r.values('I', 0, 0); r.exact(b'\0')
    r.values('f', 779); r.string('Native timing scene'); r.values('I', 4)
    r.exact(b'\x06'); r.values('f', 0, 0, 1280, 779); r.exact(b'\xff' * 4); r.values('f', 0)
    r.exact(b'\x0d'); r.shape(10); r.exact(b'\0\0\xff\0\xff')
    r.exact(b'\x0d'); r.shape(26); r.exact(b'\x01\0')
    r.values('f', 27, 11, 4, 2); r.string(KEY)
    r.exact(b'\x0d'); r.shape(42); r.exact(b'\x02\0')
    r.values('f', 43, 11, 4, 2); r.exact(bytes.fromhex('ff0000ff')); r.values('f', 1)
    r.exact(b'\0'); r.values('d', 90); r.values('I', 2)
    r.exact(b'\x01\x01'); r.values('d', 0, 50)
    r.exact(b'\0\0\0\xff\xff\x01'); r.values('d', 0, 50)
    r.values('I', 1); r.string(KEY); r.values('I', 0, 4, 2)
    r.values('I', 32); r.exact(RGBA)
    if r.at != len(data):
        raise ValueError('trailing scene bytes')
    return {'bytes': len(data), 'sha256': wide.sha(data), 'commands': 4, 'images': 1,
            'rgba_bytes': 32, 'url': url, 'title': 'Native background window'}


def validate_report(data, url):
    if not data.endswith(b'\n'):
        raise ValueError('report framing')
    report = timing.strict_json(data)
    if type(report) is not dict or set(report) != timing.REPORT_KEYS:
        raise ValueError('report schema keys')
    fixed = {'schema': 1, 'requested_frames': 1, 'completed_frames': 1, 'accepted_frames': 0,
             'width': 1280, 'height': 880, 'generation': 1, 'edit_sequence': 0, 'diagnostics': 2}
    for key, value in fixed.items():
        if timing.integer(report[key]) != value:
            raise ValueError('report is not the current complete fixed check')
    timing.integer(report['viewport_revision'], 1)
    if (report['kind'] != 'native-benchmark' or report['mode'] != 'check'
            or report['route'] != 'native-raster' or report['checked'] is not True
            or report['failure'] is not None or report['owner_failure'] is not None
            or report['owner_init_ns'] is not None or report['samples'] != []
            or type(report['scale_factor']) not in (int, float) or report['scale_factor'] != 1):
        raise ValueError('check route, scope, scale or completion mismatch')
    load = report['source_load_ms']
    if load is not None and (type(load) not in (int, float) or not math.isfinite(load) or load < 0):
        raise ValueError('invalid source load metadata')
    return report, parse_identity(report['scene_hex'], url)


def validate_run(stdout, stderr, control, port, bodies, case):
    if case not in CASES:
        raise ValueError('unknown fixed case')
    timing.integer(port, 1, 65535)
    if set(bodies) != set(wide.ROUTES):
        raise ValueError('exact two-route bodies required')
    timing.validate_control(control, port, bodies)
    for name in ('pid', 'parent_pid', 'process_group', 'start_ticks'):
        timing.integer(control['identity'][name], 1)
    if (len(stderr) > base.MAX_OUTPUT or not stderr.endswith(b'\n')):
        raise ValueError('diagnostic byte bound/framing')
    try:
        lines = stderr.decode('utf-8', errors='strict').splitlines(keepends=True)
    except UnicodeError as error:
        raise ValueError('diagnostic UTF-8') from error
    if len(lines) > 4096 or any(len(line.encode()) > 16384 for line in lines):
        raise ValueError('diagnostic line/count bound')
    url = f'http://127.0.0.1:{port}/admitted.html'
    if case == 'admitted':
        report, identity = validate_report(stdout, url)
    elif stdout:
        raise ValueError('ordinary fallback has no stdout report')
    else:
        report, identity = None, None
    prefix = bytearray()
    released, physical = False, False
    adapters, pages, brokers, stats, presentations, verified, summaries, fallbacks, other = [], [], [], [], [], [], [], [], []
    for raw in lines:
        line = raw.rstrip('\n')
        if match := timing.WAITING.fullmatch(line):
            if case != 'admitted' or released:
                raise ValueError('unexpected loaded check restart')
            physical = tuple(map(int, match.groups())) == wide.SIZE
        elif match := RELEASE.fullmatch(line):
            event = dict(prefix_bytes=len(prefix), prefix_sha256=wide.sha(prefix),
                         html_sha256=wide.sha(bodies['/admitted.html']), physical_size=list(wide.SIZE))
            if (released or not physical or int(match[1]) != control['identity']['pid']
                    or int(match[2]) != len(prefix) or match[3] != event['prefix_sha256']
                    or match[4] != event['html_sha256'] or control.get('release') != event):
                raise ValueError('positive document release identity mismatch')
            released = True
        elif match := timing.ADAPTER.fullmatch(line):
            if adapters or summaries or match[2] != 'native-shaders with complete CPU admission fallback':
                raise ValueError('surface adapter/route mismatch')
            adapters.append(line)
        elif match := PAGE.fullmatch(line):
            if not released or pages or stats or summaries:
                raise ValueError('unexpected Page lifecycle')
            pages.append(int(match[1]))
        elif match := BROKER.fullmatch(line):
            if not released or brokers or stats or summaries:
                raise ValueError('unexpected broker lifecycle')
            brokers.append(int(match[1]))
        elif match := STATS.fullmatch(line):
            fields = tuple(timing.integer(int(value)) for value in match.groups())
            if (case != 'admitted' or not released or stats or presentations or verified or summaries
                    or len(pages) != 1 or len(brokers) != 1
                    or fields[1:3] != (report['generation'], report['viewport_revision'])
                    or fields[3:] != EXPECTED_STATS):
                raise ValueError('background admission stats mismatch')
            stats.append(fields)
        elif match := base.PRESENTED.fullmatch(line):
            width, height, serial, reference = match.groups()
            if (case != 'admitted' or not released or not stats or presentations or verified or summaries
                    or (int(width), int(height)) != wide.SIZE or reference != 'true'
                    or int(serial) != stats[0][0]):
                raise ValueError('native presentation identity mismatch')
            presentations.append(int(serial))
        elif match := base.VERIFIED.fullmatch(line):
            serial, generation, revision, width, height, count = map(int, match.groups())
            if (case != 'admitted' or not released or presentations != [serial] or verified or summaries
                    or (serial, generation, revision) != stats[0][:3]
                    or (width, height) != wide.SIZE or count != wide.PIXEL_BYTES):
                raise ValueError('acquired frame identity mismatch')
            verified.append(serial)
        elif match := timing.SUMMARY.fullmatch(line):
            route, frames, count, serial = match.groups()
            if (case != 'admitted' or not released or summaries or verified != [int(serial)]
                    or route != 'native-raster' or int(frames) != 1 or int(count) != wide.PIXEL_BYTES):
                raise ValueError('acquired completion mismatch')
            summaries.append(int(serial))
        elif match := base.FALLBACK.fullmatch(line):
            if not released and match[1].startswith('native scene awaits a current loaded page; size='):
                if case != 'rounded':
                    raise ValueError('unexpected primary native fallback')
                physical = match[1] == wide.PHYSICAL
            elif (case == 'rounded' and released and match[1] == FALLBACK and not fallbacks
                  and len(pages) == len(brokers) == 1):
                fallbacks.append(match[1])
            else:
                raise ValueError('unexpected or duplicate whole-frame fallback')
        elif (line.startswith(('native benchmark', 'native background gate:', 'native boxes', 'presenter:',
                               'native scene', '[page]', 'paint:', 'eris-browser:',
                               'Unable to open browser window:', "thread '")) or 'panicked at' in line):
            raise ValueError('unknown or failing proof diagnostic')
        else:
            other.append(line)
        prefix.extend(raw.encode())
    if (not released or len(adapters) != 1 or len(pages) != 1 or len(brokers) != 1
            or pages[0] == brokers[0] or control['identity']['pid'] in pages + brokers):
        raise ValueError('missing owned release/adapter/confined Page/distinct broker')
    if case == 'admitted' and not (len(stats) == len(presentations) == len(verified) == len(summaries) == 1):
        raise ValueError('incomplete native check population')
    if case == 'rounded' and (fallbacks != [FALLBACK] or stats or presentations or verified or summaries):
        raise ValueError('incomplete exact fallback control')
    return {'case': case, 'identity': identity, 'report': report, 'adapter': adapters[0],
            'page_pid': pages[0], 'broker_pid': brokers[0], 'positive_release': True,
            'verified_frames': len(verified), 'compared_bytes': wide.PIXEL_BYTES if verified else 0,
            'box_loop_work': 159 if stats else 0, 'box_stats': list(stats[0]) if stats else None,
            'fallbacks': fallbacks, 'other_diagnostics': other,
            'scope': 'acquired texture versus Canvas before compositor; no performance claim' if verified
                     else 'exact ordinary CPU fallback; no acquired-pixel claim'}


def fixture_data(directory, case):
    directory = directory.resolve()
    if case not in CASES or {p.name for p in directory.iterdir()} != set(FILES):
        raise ValueError('exact five-file fixture bundle required')
    for name in FILES:
        p = directory / name
        if p.is_symlink() or not p.is_file() or p.stat().st_size > 65536:
            raise ValueError('bounded regular fixture files required')
    if base.digest(directory / 'freeze.json', 65536) != FREEZE_SHA:
        raise ValueError('independent fixture freeze changed')
    freeze = timing.strict_json((directory / 'freeze.json').read_bytes(), 65536)
    if ([row['path'] for row in freeze['files']] != list(FILES[:3])
            or freeze['oracles']['path'] != 'oracles.json'):
        raise ValueError('fixture/oracle roster changed')
    for row in [*freeze['files'], freeze['oracles']]:
        path = directory / row['path']
        if path.stat().st_size != row['bytes'] or base.digest(path, 65536) != row['sha256']:
            raise ValueError('fixture or independent oracle changed')
    document = 'admitted.html' if case == 'admitted' else 'rounded.html'
    return {'/admitted.html': (directory / document).read_bytes(),
            '/two-pixels.png': (directory / 'two-pixels.png').read_bytes()}



def x11_library_binding(env):
    if (not env.get('DISPLAY') or env.get('WAYLAND_DISPLAY') or env.get('WAYLAND_SOCKET')
            or env.get('WINIT_X11_SCALE_FACTOR') != '1'):
        raise ValueError('physical control requires explicit X11 with scale one')
    directories = env.get('LD_LIBRARY_PATH', '').split(':')
    if len(directories) > 64 or any(len(value) > 4096 for value in directories):
        raise ValueError('bounded explicit library search required')
    for value in directories:
        if not value or not Path(value).is_absolute():
            continue
        candidate = Path(value) / 'libX11.so.6'
        if candidate.is_file():
            path = candidate.resolve()
            return {'path': str(path), 'bytes': path.stat().st_size,
                    'sha256': base.digest(path, base.MAX_BINARY)}
    raise ValueError('libX11 must be present in the explicit library path')


def resize_x11(lib, pid):
    """Runs only in the timed helper process. PID is an X11 property attribution.

    The parent separately retains unreaped-child identity. This does not claim
    XRes/kernel ownership, and never enumerates a window tree or reads events.
    """
    timing.integer(pid, 1, 2**31 - 1)
    void, ulong, uint, integer = ctypes.c_void_p, ctypes.c_ulong, ctypes.c_uint, ctypes.c_int
    byte_pointer = ctypes.POINTER(ctypes.c_ubyte)
    signatures = {
        'XOpenDisplay': ([ctypes.c_char_p], void),
        'XCloseDisplay': ([void], integer),
        'XDefaultRootWindow': ([void], ulong),
        'XInternAtom': ([void, ctypes.c_char_p, integer], ulong),
        'XSetErrorHandler': ([void], void),
        'XSync': ([void, integer], integer),
        'XFree': ([void], integer),
        'XGetWindowProperty': ([void, ulong, ulong, ctypes.c_long, ctypes.c_long, integer, ulong,
                               ctypes.POINTER(ulong), ctypes.POINTER(integer), ctypes.POINTER(ulong),
                               ctypes.POINTER(ulong), ctypes.POINTER(byte_pointer)], integer),
        'XResizeWindow': ([void, ulong, uint, uint], integer),
        'XGetGeometry': ([void, ulong, ctypes.POINTER(ulong), ctypes.POINTER(integer),
                         ctypes.POINTER(integer), ctypes.POINTER(uint), ctypes.POINTER(uint),
                         ctypes.POINTER(uint), ctypes.POINTER(uint)], integer),
    }
    for name, (arguments, result) in signatures.items():
        function = getattr(lib, name)
        function.argtypes, function.restype = arguments, result
    errors = []
    @ctypes.CFUNCTYPE(integer, void, void)
    def on_error(_display, _error):
        if not errors:
            errors.append(True)
        return 0
    previous = lib.XSetErrorHandler(ctypes.cast(on_error, void))
    display = None
    property_reads = 0
    geometry_reads = 0
    try:
        display = lib.XOpenDisplay(None)
        if not display:
            raise ValueError('cannot open inherited X11 display')
        root = lib.XDefaultRootWindow(display)
        clients_atom = lib.XInternAtom(display, b'_NET_CLIENT_LIST', 1)
        pid_atom = lib.XInternAtom(display, b'_NET_WM_PID', 1)
        if not root or not clients_atom or not pid_atom:
            raise ValueError('required X11 root/properties unavailable')
        def sync():
            lib.XSync(display, 0)
            if errors:
                raise ValueError('X11 request failed')
        def property_values(window, atom, expected, cap, missing=False):
            nonlocal property_reads
            actual, count, after, fmt = ulong(), ulong(), ulong(), integer()
            pointer = byte_pointer()
            property_reads += 1
            try:
                status = lib.XGetWindowProperty(display, window, atom, 0, cap, 0, expected,
                    ctypes.byref(actual), ctypes.byref(fmt), ctypes.byref(count),
                    ctypes.byref(after), ctypes.byref(pointer))
                if status != 0 or errors:
                    raise ValueError('X11 property read failed')
                if missing and (actual.value, fmt.value, count.value, after.value) == (0, 0, 0, 0):
                    return []
                if (actual.value != expected or fmt.value != 32 or after.value != 0
                        or count.value > cap or (count.value and not pointer)):
                    raise ValueError('bounded typed X11 property required')
                # Xlib expands protocol32 values to native unsigned long slots.
                values = ctypes.cast(pointer, ctypes.POINTER(ulong))
                return [int(values[index]) for index in range(count.value)]
            finally:
                if pointer:
                    lib.XFree(pointer)
        def owner(window):
            values = property_values(window, pid_atom, 6, 1, missing=True)  # XA_CARDINAL
            return values[0] if len(values) == 1 else None
        clients = property_values(root, clients_atom, 33, 256)  # XA_WINDOW
        if len(set(clients)) != len(clients) or any(not 0 < value <= 0xffffffff for value in clients):
            raise ValueError('unique bounded X11 client identifiers required')
        matches = [window for window in clients if owner(window) == pid]
        if len(matches) != 1:
            raise ValueError('exactly one X11 window must report the owned PID')
        window = matches[0]
        def geometry():
            nonlocal geometry_reads
            returned_root, x, y = ulong(), integer(), integer()
            width, height, border, depth = uint(), uint(), uint(), uint()
            geometry_reads += 1
            status = lib.XGetGeometry(display, window, ctypes.byref(returned_root), ctypes.byref(x), ctypes.byref(y),
                ctypes.byref(width), ctypes.byref(height), ctypes.byref(border), ctypes.byref(depth))
            if (not status or errors or returned_root.value != root
                    or not 1 <= width.value <= 16384 or not 1 <= height.value <= 16384
                    or not 1 <= depth.value <= 64):
                raise ValueError('invalid X11 window geometry')
            return [width.value, height.value]
        initial = geometry()
        before = owner(window)
        if before != pid:
            raise ValueError('X11 PID changed before physical resize')
        lib.XResizeWindow(display, window, 1280, 880)
        sync()
        observed = None
        for _ in range(32):
            observed = geometry()
            if observed == [1280, 880]:
                break
            time.sleep(0.01)
            sync()
        if observed != [1280, 880]:
            raise ValueError('X11 physical resize not confirmed')
        after = owner(window)
        if after != pid:
            raise ValueError('X11 PID changed after physical resize')
        return {'pid': pid, 'window': window, 'pid_before': before, 'pid_after': after,
                'client_count': len(clients), 'property_reads': property_reads, 'geometry_reads': geometry_reads,
                'initial_size': initial, 'requested_size': [1280, 880], 'observed_size': observed,
                'scope': 'unique X11 PID-property match; parent retains unreaped browser identity'}
    finally:
        if display:
            lib.XCloseDisplay(display)
        lib.XSetErrorHandler(previous)


def x11_resize_child(args):
    expected = x11_library_binding(dict(os.environ))
    if (args.x11_library is None or str(args.x11_library.resolve()) != expected['path']
            or args.x11_library_sha256 != expected['sha256']):
        raise ValueError('physical helper library binding differs')
    result = resize_x11(ctypes.CDLL(expected['path']), args.owned_pid)
    if x11_library_binding(dict(os.environ)) != expected:
        raise ValueError('X11 library changed during control')
    print(json.dumps({'library': expected, **result}, sort_keys=True))
    return 0


def configure_x11_owned(identity, library, env, record, persist, deadline):
    record['control_complete'] = False
    record['logical_controlled_size_before_physical'] = record.get('controlled_size')
    persist()
    base.ensure_live_child(identity)
    remaining = deadline - time.monotonic()
    if remaining < 0.05 or len(record['commands']) >= base.CONTROL_COMMANDS:
        raise ValueError('owned physical control deadline/command bound')
    if x11_library_binding(env) != library:
        raise ValueError('X11 library changed before helper launch')
    remaining = deadline - time.monotonic()
    if remaining < 0.05:
        raise ValueError('physical control deadline exhausted before helper launch')
    command = [str(Path(__file__).resolve()), '--x11-resize-child', '--owned-pid', str(identity['pid']),
               '--x11-library', library['path'], '--x11-library-sha256', library['sha256'], '--allow-experimental-gpu']
    result = base.controller_call(Path(sys.executable), command, env, min(1.5, remaining))
    row = {'argv': [sys.executable, *command], 'status': result.record['status'],
           'returncode': result.record['returncode'], 'stdout_bytes': len(result.stdout),
           'stdout_sha256': wide.sha(result.stdout), 'stderr_bytes': len(result.stderr),
           'stderr_sha256': wide.sha(result.stderr)}
    if len(result.stdout) <= 4096:
        row['stdout'] = result.stdout.decode('utf-8', errors='strict')
    if len(result.stderr) <= 4096:
        row['stderr'] = result.stderr.decode('utf-8', errors='strict')
    record['commands'].append(row); persist()
    base.ensure_live_child(identity)
    if x11_library_binding(env) != library:
        raise ValueError('X11 library changed after helper control')
    if (result.record['status'] != 'exited' or result.record['returncode'] != 0
            or result.stderr or len(result.stdout) > 4096):
        raise ValueError('bounded X11 physical controller failed')
    checked = timing.strict_json(result.stdout, 4096)
    if (checked['library'] != library or checked['pid'] != identity['pid']
            or checked['pid_before'] != identity['pid'] or checked['pid_after'] != identity['pid']
            or checked['requested_size'] != [1280, 880] or checked['observed_size'] != [1280, 880]
            or not 1 <= checked['client_count'] <= 256 or not 1 <= checked['geometry_reads'] <= 33
            or checked['property_reads'] != checked['client_count'] + 3):
        raise ValueError('physical controller did not confirm the owned fixed size')
    if time.monotonic() >= deadline:
        raise ValueError('physical control exhausted original launch deadline')
    record['x11_physical_control'] = checked
    record['control_size_space'] = 'x11-physical'
    record['controlled_size'] = [1280, 880]
    record['control_complete'] = True
    persist()



def validate_x11_control(control, library):
    """Bind the opt-in control to the selected library, owned PID and raw reply."""
    try:
        physical_keys = {'x11_physical_control', 'control_size_space', 'logical_controlled_size_before_physical'}
        if library is None:
            if control.get('x11_library') is not None or physical_keys & control.keys():
                raise ValueError('physical control reported without opt-in')
            return
        if (control['x11_library'] != library or control['control_size_space'] != 'x11-physical'
                or control['logical_controlled_size_before_physical'] != [1280, 880]
                or control['controlled_size'] != [1280, 880]):
            raise ValueError('physical control library/coordinate binding mismatch')
        pid = timing.integer(control['identity']['pid'], 1, 2**31 - 1)
        data = control['x11_physical_control']
        if set(data) != {'library', 'pid', 'window', 'pid_before', 'pid_after', 'client_count', 'property_reads',
                         'geometry_reads', 'initial_size', 'requested_size', 'observed_size', 'scope'}:
            raise ValueError('complete fixed physical-control reply required')
        timing.integer(data['window'], 1, 0xffffffff)
        clients = timing.integer(data['client_count'], 1, 256)
        timing.integer(data['geometry_reads'], 2, 33)
        timing.integer(data['property_reads'], 4, 259)
        if not isinstance(data['initial_size'], list) or len(data['initial_size']) != 2:
            raise ValueError('physical initial size must have two axes')
        for axis in data['initial_size']:
            timing.integer(axis, 1, 16384)
        for key in ('pid', 'pid_before', 'pid_after'):
            if timing.integer(data[key], 1, 2**31 - 1) != pid:
                raise ValueError('physical-control PID differs from owned browser')
        if (data['library'] != library or data['property_reads'] != clients + 3
                or data['requested_size'] != [1280, 880] or data['observed_size'] != [1280, 880]
                or data['scope'] != 'unique X11 PID-property match; parent retains unreaped browser identity'):
            raise ValueError('physical control reply differs from fixed contract')
        rows = [row for row in control['commands'] if '--x11-resize-child' in row.get('argv', [])]
        if len(rows) != 1:
            raise ValueError('one bounded physical helper command required')
        row = rows[0]
        argv = [sys.executable, str(Path(__file__).resolve()), '--x11-resize-child', '--owned-pid', str(pid),
                '--x11-library', library['path'], '--x11-library-sha256', library['sha256'], '--allow-experimental-gpu']
        stdout = row['stdout'].encode('utf-8')
        if (row['argv'] != argv or row['status'] != 'exited' or row['returncode'] != 0
                or len(stdout) > 4096 or row['stdout_bytes'] != len(stdout) or row['stdout_sha256'] != wide.sha(stdout)
                or row['stderr'] != '' or row['stderr_bytes'] != 0 or row['stderr_sha256'] != wide.sha(b'')
                or timing.strict_json(stdout, 4096) != data):
            raise ValueError('physical-control raw helper output binding mismatch')
    except (KeyError, TypeError, AttributeError, UnicodeError) as error:
        raise ValueError('malformed physical control receipt') from error



def select_unit_monitor(monitors, requested):
    timing.integer(requested, 0, 255)
    if not isinstance(monitors, list) or len(monitors) > 16 or any(not isinstance(row, dict) for row in monitors):
        raise ValueError('bounded monitor inventory required')
    matches = [row for row in monitors if type(row.get('id')) is int and row['id'] == requested]
    if len(matches) != 1:
        raise ValueError('explicit monitor must exist exactly once')
    row = matches[0]
    try:
        width, height = timing.integer(row['width'], 1280, 16384), timing.integer(row['height'], 880, 16384)
        workspace = timing.integer(row['activeWorkspace']['id'], 1, 2**31 - 1)
        if (row['disabled'] is not False or type(row['transform']) is not int or row['transform'] != 0
                or type(row['scale']) not in (int, float) or row['scale'] != 1
                or type(row['name']) is not str or not 1 <= len(row['name']) <= 256):
            raise ValueError('selected monitor must be enabled, untransformed and unit scale')
        return {'id': requested, 'name': row['name'], 'width': width, 'height': height,
                'scale': row['scale'], 'transform': 0, 'disabled': False, 'active_workspace': workspace}
    except (KeyError, TypeError) as error:
        raise ValueError('selected monitor metadata is incomplete') from error


def unit_owned_client(clients, pid):
    if not isinstance(clients, list) or len(clients) > 256 or any(not isinstance(row, dict) for row in clients):
        raise ValueError('bounded client inventory required')
    matches = [row for row in clients if type(row.get('pid')) is int and row['pid'] == pid]
    if len(matches) != 1:
        raise ValueError('unit-monitor control requires one owned client')
    row = matches[0]
    try:
        monitor = timing.integer(row['monitor'], 0, 255)
        workspace = timing.integer(row['workspace']['id'], 1, 2**31 - 1)
        size = row['size']
        if (not isinstance(size, list) or len(size) != 2 or row['floating'] is not True
                or row['mapped'] is not True or row.get('grouped', []) != []):
            raise ValueError('owned floating ungrouped client required')
        for axis in size:
            timing.integer(axis, 1, 16384)
        return {'pid': pid, 'monitor': monitor, 'workspace': workspace, 'size': size, 'floating': True, 'mapped': True}
    except (KeyError, TypeError) as error:
        raise ValueError('owned client metadata is incomplete') from error


def configure_unit_monitor(identity, controller, env, record, persist, deadline, requested):
    record['control_complete'] = False
    persist()
    timing.integer(requested, 0, 255)
    pid = identity['pid']
    def invoke(arguments):
        base.ensure_live_child(identity)
        remaining = deadline - time.monotonic()
        if remaining < 0.05 or len(record['commands']) >= base.CONTROL_COMMANDS:
            raise ValueError('unit-monitor control deadline/command bound')
        result = base.controller_call(controller, arguments, env, min(0.5, remaining))
        row = {'control_stage': 'unit-monitor', 'argv': [str(controller), *arguments],
               'status': result.record['status'], 'returncode': result.record['returncode'],
               'stdout_bytes': len(result.stdout), 'stdout_sha256': wide.sha(result.stdout),
               'stderr_bytes': len(result.stderr), 'stderr_sha256': wide.sha(result.stderr)}
        query = arguments in (['-j', 'monitors'], ['-j', 'clients'])
        if not query and len(result.stdout) <= 4096:
            row['stdout'] = result.stdout.decode('utf-8', errors='strict')
        if len(result.stderr) <= 4096:
            row['stderr'] = result.stderr.decode('utf-8', errors='strict')
        record['commands'].append(row); persist()
        base.ensure_live_child(identity)
        if (row['status'] != 'exited' or row['returncode'] != 0 or result.stderr
                or len(result.stdout) > (base.CONTROL_BYTES if query else 4096)):
            raise ValueError('unit-monitor controller failed')
        return result.stdout, row
    def monitor_query():
        stdout, row = invoke(['-j', 'monitors'])
        selected = select_unit_monitor(timing.strict_json(stdout, base.CONTROL_BYTES), requested)
        row['selected_monitor'] = selected; persist()
        return selected
    def owned_query():
        stdout, row = invoke(['-j', 'clients'])
        owned = unit_owned_client(timing.strict_json(stdout, base.CONTROL_BYTES), pid)
        row['owned_client'] = owned; persist()
        return owned
    monitor = monitor_query()
    before = owned_query()
    invoke(['dispatch', 'movetoworkspacesilent', f'{monitor["active_workspace"]},pid:{pid}'])
    while True:
        moved = owned_query()
        if moved['monitor'] == requested and moved['workspace'] == monitor['active_workspace']:
            break
        time.sleep(0.01)
    invoke(['dispatch', 'resizewindowpixel', f'exact 1280 880,pid:{pid}'])
    while True:
        after = owned_query()
        if (after['monitor'] == requested and after['workspace'] == monitor['active_workspace']
                and after['size'] == [1280, 880]):
            break
        time.sleep(0.01)
    if monitor_query() != monitor or time.monotonic() >= deadline:
        raise ValueError('selected unit monitor changed or control deadline exhausted')
    record['unit_monitor_control'] = {'requested_id': requested, 'monitor': monitor, 'owned_before': before, 'owned_after': after}
    record['controlled_size'] = [1280, 880]
    persist()


def validate_unit_monitor_control(control, requested, controller=None):
    try:
        rows = [row for row in control['commands'] if row.get('control_stage') == 'unit-monitor']
        if requested is None:
            if rows or 'unit_monitor_control' in control:
                raise ValueError('unit-monitor control reported without opt-in')
            return
        timing.integer(requested, 0, 255)
        proof = control['unit_monitor_control']
        if set(proof) != {'requested_id', 'monitor', 'owned_before', 'owned_after'} or proof['requested_id'] != requested:
            raise ValueError('unit-monitor reply does not match explicit selection')
        selected = proof['monitor']
        if select_unit_monitor([{**selected, 'activeWorkspace': {'id': selected['active_workspace']}}], requested) != selected:
            raise ValueError('unit monitor metadata is invalid')
        pid = timing.integer(control['identity']['pid'], 1, 2**31 - 1)
        for key in ('owned_before', 'owned_after'):
            value = proof[key]
            if unit_owned_client([{**value, 'workspace': {'id': value['workspace']}}], pid) != value:
                raise ValueError('owned unit-monitor metadata differs')
        after = proof['owned_after']
        if after['monitor'] != requested or after['workspace'] != selected['active_workspace'] or after['size'] != [1280, 880]:
            raise ValueError('unit-monitor destination or size differs')
        if not 7 <= len(rows) <= base.CONTROL_COMMANDS or len(control['commands']) > base.CONTROL_COMMANDS:
            raise ValueError('bounded unit-monitor command transcript required')
        if controller is None:
            controller = Path(rows[0]['argv'][0])
        operations = []
        monitor_rows, client_rows = [], []
        for row in rows:
            if (row['argv'][0] != str(controller) or row['status'] != 'exited' or row['returncode'] != 0
                    or row['stderr_bytes'] != 0 or row['stderr_sha256'] != wide.sha(b'') or row.get('stderr') != ''):
                raise ValueError('unit-monitor command binding differs')
            arguments = row['argv'][1:]
            if arguments == ['-j', 'monitors']:
                if row['selected_monitor'] != selected:
                    raise ValueError('monitor changed between queries')
                monitor_rows.append(row)
            elif arguments == ['-j', 'clients']:
                value = row['owned_client']
                if unit_owned_client([{**value, 'workspace': {'id': value['workspace']}}], pid) != value:
                    raise ValueError('intermediate client identity differs')
                client_rows.append(row)
            else:
                stdout = row['stdout'].encode('utf-8')
                if len(stdout) > 4096 or row['stdout_bytes'] != len(stdout) or row['stdout_sha256'] != wide.sha(stdout):
                    raise ValueError('unit-monitor dispatch output binding differs')
                operations.append(arguments)
        if (operations != [['dispatch', 'movetoworkspacesilent', f'{selected["active_workspace"]},pid:{pid}'],
                           ['dispatch', 'resizewindowpixel', f'exact 1280 880,pid:{pid}']]
                or len(monitor_rows) != 2 or len(client_rows) < 3
                or rows[0] != monitor_rows[0] or rows[-1] != monitor_rows[-1]
                or client_rows[0]['owned_client'] != proof['owned_before'] or client_rows[-1]['owned_client'] != after):
            raise ValueError('owned unit-monitor query/move/resize sequence differs')
        move_at, resize_at = [index for index, row in enumerate(rows) if row['argv'][1] == 'dispatch']
        if (move_at != 2 or rows[1] != client_rows[0] or resize_at <= move_at + 1 or resize_at >= len(rows) - 2
                or any(row['argv'][1:] != ['-j', 'clients'] for row in rows[move_at + 1:resize_at])
                or any(row['argv'][1:] != ['-j', 'clients'] for row in rows[resize_at + 1:-1])):
            raise ValueError('unit-monitor stage order differs')
        confirmed = rows[resize_at - 1]['owned_client']
        if confirmed['monitor'] != requested or confirmed['workspace'] != selected['active_workspace']:
            raise ValueError('destination was not confirmed before resize')
    except (KeyError, TypeError, AttributeError) as error:
        raise ValueError('malformed unit-monitor control receipt') from error


def launch_owned(args):
    if (args.receipt.exists() or not args.receipt.parent.is_dir() or args.case not in CASES
            or not math.isfinite(args.window_seconds) or not 2 <= args.window_seconds <= 20
            or base.digest(args.binary, base.MAX_BINARY) != args.binary_sha256
            or base.digest(args.controller, base.MAX_BINARY) != args.controller_sha256):
        raise ValueError('fresh receipt, bounded lifetime and immutable inputs required')
    timing.integer(args.port, 1, 65535)
    validate_listener_fds((args.listener_fd,))
    bodies = fixture_data(args.fixtures, args.case)
    listener = socket.socket(fileno=args.listener_fd)
    if listener.getsockname() != ('127.0.0.1', args.port):
        listener.close(); raise ValueError('inherited listener identity mismatch')
    listener.setblocking(False)
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [], 'requests': [],
              'accepted_connections': 0, 'identity': None, 'browser_reaped': False, 'browser_returncode': None,
              'binary_sha256': args.binary_sha256, 'controller_sha256': args.controller_sha256,
              'fixture_freeze': FREEZE_SHA, 'port': args.port, 'case': args.case}
    library = x11_library_binding(dict(os.environ)) if args.x11_physical_size else None
    if library and (str(args.x11_library) != library['path'] or args.x11_library_sha256 != library['sha256']):
        raise ValueError('owned X11 library binding mismatch')
    record['x11_library'] = library
    child, emitted = None, 0
    deadline = time.monotonic() + args.window_seconds + 5
    def persist():
        temp = args.receipt.with_suffix('.tmp')
        temp.write_text(json.dumps(record, indent=2) + '\n'); temp.replace(args.receipt)
    def emit(data):
        nonlocal emitted
        keep = min(len(data), base.MAX_OUTPUT - emitted)
        sys.stderr.buffer.write(data[:keep]); sys.stderr.buffer.flush(); emitted += keep
        if keep != len(data):
            raise ValueError('diagnostic relay bound')
    persist()
    try:
        url = f'http://127.0.0.1:{args.port}/admitted.html'
        command = [str(args.binary), url, '--presenter=vulkan', '--raster=gpu', '--no-scripts',
                   '--exit-after', str(args.window_seconds)]
        if args.case == 'admitted':
            command.append('--benchmark-native-check')
        record['browser_command'] = command; persist()
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE, close_fds=True)
        record['identity'] = base.child_identity(child.pid); persist()
        control_deadline = min(deadline, time.monotonic() + 3.0)
        base.configure_owned(record['identity'], args.controller, dict(os.environ), record, persist,
                             requested_size=wide.SIZE)
        if args.unit_scale_monitor is not None:
            configure_unit_monitor(record['identity'], args.controller, dict(os.environ), record, persist, control_deadline, args.unit_scale_monitor)
        if library:
            configure_x11_owned(record['identity'], library, dict(os.environ), record, persist, control_deadline)
        wide.serve_and_relay(child, listener, record, bodies, persist, deadline, emit,
                             waiting_pattern=timing.WAITING if args.case == 'admitted' else None,
                             marker_prefix='native background gate', forbidden_patterns=(STATS, timing.CPU_VERIFIED, timing.SUMMARY))
        record['browser_returncode'] = child.wait(timeout=2); record['browser_reaped'] = True
        record['success'] = record['browser_returncode'] == 0
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        listener.close()
        if child is not None:
            if not record['browser_reaped']:
                try:
                    os.waitid(os.P_PID, child.pid, base.WAIT_FLAGS)
                    os.kill(child.pid, signal.SIGKILL)
                    record['browser_returncode'] = child.wait(timeout=2); record['browser_reaped'] = True
                except (OSError, subprocess.SubprocessError) as error:
                    record['cleanup_error'] = str(error)[:4096]
            try:
                drain_stderr(child.stderr, emit)
            except (OSError, ValueError) as error:
                record['success'] = False; record['stderr_drain_error'] = str(error)[:4096]
            child.stderr.close()
        try:
            if (base.digest(args.binary, base.MAX_BINARY) != args.binary_sha256
                    or base.digest(args.controller, base.MAX_BINARY) != args.controller_sha256
                    or fixture_data(args.fixtures, args.case) != bodies
                    or (library is not None and x11_library_binding(dict(os.environ)) != library)):
                raise ValueError('owned launch inputs changed')
        except (OSError, ValueError) as error:
            record['success'] = False; record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def source_binding():
    paths = {str(Path(__file__).resolve()), str(Path(wide.__file__).resolve()),
             str(Path(timing.__file__).resolve()), str(ROOT / 'tools/native_raster_followup.py')}
    return {**base.source_binding(), **{path: base.digest(Path(path), 1024 * 1024) for path in sorted(paths)}}


def run_host(args):
    if (args.output.exists() or not math.isfinite(args.timeout) or not math.isfinite(args.window_seconds)
            or not 2 <= args.window_seconds <= 20 or not args.window_seconds + 10 <= args.timeout <= 120
            or re.fullmatch('[0-9a-f]{64}', args.binary_sha256) is None):
        raise ValueError('new output, bounded lifetime and frozen binary SHA required')
    binary, output = args.binary.resolve(), args.output.resolve()
    fixtures = args.fixtures.resolve()
    if output.is_relative_to(fixtures) or fixtures.is_relative_to(output):
        raise ValueError('output must be separate from fixed fixture bundle')
    env = base.environment(args.loader_directory)
    found = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not found:
        raise ValueError('owned Hyprland compositor control is required')
    controller = Path(found).resolve()
    library = x11_library_binding(env) if args.x11_physical_size else None
    fixture_data(fixtures, 'admitted')
    def bindings():
        return {'binary': base.digest(binary, base.MAX_BINARY), 'controller': base.digest(controller, base.MAX_BINARY),
                'fixtures': {name: base.digest(fixtures / name, 65536) for name in FILES},
                'source_files': source_binding(), 'loader': base.loader_binding(args.loader_directory, env),
                'x11_library': x11_library_binding(env) if library else None}
    frozen = bindings()
    if frozen['binary'] != args.binary_sha256:
        raise ValueError('binary does not match supplied frozen SHA')
    output.mkdir(parents=True)
    copied = output / 'fixtures'; copied.mkdir()
    for name in FILES:
        shutil.copyfile(fixtures / name, copied / name)
    result = {'schema': 1, 'success': False, 'runs': [], 'bindings': frozen, 'binary': str(binary),
              'controller': str(controller), 'fixture_freeze': FREEZE_SHA, 'unit_scale_monitor': args.unit_scale_monitor,
              'display_environment': {key: env.get(key) for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'WINIT_X11_SCALE_FACTOR', 'XDG_RUNTIME_DIR', 'XDG_SESSION_TYPE')},
              'scope': 'one acquired native frame versus Canvas before compositor; optional ordinary fallback; no timing claim'}
    def persist():
        temp = output / 'background-host-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n'); temp.replace(output / 'background-host-results.json')
    def unchanged():
        if bindings() != frozen or any(base.digest(copied / name, 65536) != frozen['fixtures'][name] for name in FILES):
            raise ValueError('source/fixture/executable/loader binding changed')
    persist()
    try:
        for case in CASES if args.with_fallback else ('admitted',):
            unchanged()
            bodies = fixture_data(copied, case)
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
                listener.bind(('127.0.0.1', 0)); listener.listen(wide.MAX_ACTIVE_CONNECTIONS); listener.setblocking(False)
                port = listener.getsockname()[1]
                receipt = output / f'{case}.window-control.json'
                command = [sys.executable, str(Path(__file__).resolve()), '--owned-launch', '--binary', str(binary),
                           '--binary-sha256', frozen['binary'], '--receipt', str(receipt), '--controller', str(controller),
                           '--controller-sha256', frozen['controller'], '--listener-fd', str(listener.fileno()),
                           '--port', str(port), '--fixtures', str(copied), '--case', case,
                           '--window-seconds', str(args.window_seconds), '--allow-experimental-gpu']
                if library:
                    command.extend(['--x11-physical-size', '--x11-library', library['path'], '--x11-library-sha256', library['sha256']])
                if args.unit_scale_monitor is not None:
                    command.extend(['--unit-scale-monitor', str(args.unit_scale_monitor)])
                attempt = {'case': case, 'command': command, 'success': False}; result['runs'].append(attempt); persist()
                record, stdout = base.run_supervised(command, output, case, env, args.timeout, pass_fds=(listener.fileno(),))
                attempt['supervision'] = record; persist()
                if receipt.is_file():
                    attempt['control_sha256'] = base.digest(receipt, 1024 * 1024)
                    attempt['control'] = timing.strict_json(receipt.read_bytes(), 1024 * 1024); persist()
                base.check_success(record)
                if ((output / f'{case}.supervisor.stderr.log').stat().st_size
                        or record.get('capture_gate_required') or record.get('capture_gate_granted')):
                    raise ValueError('unexpected supervisor diagnostics or gate')
                stderr_path = output / f'{case}.stderr.log'
                if base.digest(stderr_path, base.MAX_OUTPUT) != record['stderr_sha256'] or wide.sha(stdout) != record['stdout_sha256']:
                    raise ValueError('raw output binding mismatch')
                control = attempt['control']
                expected_command = [str(binary), f'http://127.0.0.1:{port}/admitted.html', '--presenter=vulkan',
                                    '--raster=gpu', '--no-scripts', '--exit-after', str(args.window_seconds)]
                if case == 'admitted':
                    expected_command.append('--benchmark-native-check')
                if (control.get('binary_sha256') != frozen['binary'] or control.get('controller_sha256') != frozen['controller']
                        or control.get('fixture_freeze') != FREEZE_SHA or control.get('case') != case
                        or control.get('browser_command') != expected_command):
                    raise ValueError('owned wrapper input mismatch')
                validate_unit_monitor_control(control, args.unit_scale_monitor, controller)
                validate_x11_control(control, library)
                checked = validate_run(stdout, stderr_path.read_bytes(), control, port, bodies, case)
                attempt['validation'] = checked
                if case == 'admitted':
                    (output / 'scene.ebni').write_bytes(bytes.fromhex(checked['report']['scene_hex']))
                try:
                    extra, _ = listener.accept()
                except BlockingIOError:
                    pass
                else:
                    extra.close(); raise ValueError('HTTP connection survived case boundary')
                unchanged(); attempt['success'] = True; persist()
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        try:
            unchanged()
        except (OSError, ValueError) as error:
            result['success'] = False; result['postcheck_error'] = str(error)[:4096]
        persist()
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--owned-launch', action='store_true')
    parser.add_argument('--binary', type=Path)
    parser.add_argument('--binary-sha256')
    parser.add_argument('--x11-physical-size', action='store_true')
    parser.add_argument('--unit-scale-monitor', type=int)
    parser.add_argument('--x11-resize-child', action='store_true')
    parser.add_argument('--owned-pid', type=int)
    parser.add_argument('--x11-library', type=Path)
    parser.add_argument('--x11-library-sha256')
    parser.add_argument('--fixtures', type=Path, default=FIXTURES)
    parser.add_argument('--window-seconds', type=float, default=8.0)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=30)
    parser.add_argument('--with-fallback', action='store_true')
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--controller', type=Path)
    parser.add_argument('--controller-sha256')
    parser.add_argument('--listener-fd', type=int)
    parser.add_argument('--port', type=int)
    parser.add_argument('--case', choices=CASES)
    args = parser.parse_args()
    if not args.allow_experimental_gpu:
        parser.error('actual owned window/worker/Vulkan execution requires --allow-experimental-gpu')
    if args.unit_scale_monitor is not None and (not args.x11_physical_size or not 0 <= args.unit_scale_monitor <= 255):
        parser.error('unit monitor0..255 requires physical X11 control')
    if args.x11_resize_child:
        if args.owned_launch or args.x11_physical_size or args.owned_pid is None:
            parser.error('physical helper requires only its owned PID and library binding')
        try:
            return x11_resize_child(args)
        except (OSError, ValueError, TypeError, KeyError, AttributeError) as error:
            parser.exit(1, f'{type(error).__name__}: {error}\n')
    required = ('binary', 'binary_sha256') + (('receipt', 'controller', 'controller_sha256', 'listener_fd', 'port', 'case') if args.owned_launch else ('output', 'loader_directory'))
    if args.owned_launch and args.x11_physical_size:
        required += ('x11_library', 'x11_library_sha256')
    if any(getattr(args, key) is None for key in required):
        parser.error('missing required host/owned-launch inputs')
    previous = {}
    def interrupted(_signal, _frame):
        raise KeyboardInterrupt
    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, interrupted)
        if args.owned_launch:
            return launch_owned(args)
        result = run_host(args)
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        parser.exit(1, f'{type(error).__name__}: {error}\n')
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    print(json.dumps({'success': result['success'], 'results': str(args.output / 'background-host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
