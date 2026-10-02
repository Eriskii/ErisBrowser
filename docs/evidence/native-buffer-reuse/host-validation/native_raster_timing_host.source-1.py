#!/usr/bin/env python3
"""Paired CPU/native or baseline/candidate native host durations for one scene.

Correctness checks are separate processes. Durations bracket host work through
present/retirement; they are neither GPU timestamps nor display latency. The
listener/URL and paint identity remain identical across all runs. The native A/B
mode binds each executable to its own pinned release manifest and source tree.
"""
from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import statistics
import struct
import subprocess
import sys
import time

import native_raster_host as base
import native_raster_wide_host as wide
from native_raster_followup import drain_stderr
from bridge_supervisor import validate_listener_fds

ROOT = Path(__file__).resolve().parents[1]
WAITING = re.compile(r'native benchmark waiting: size=([1-9][0-9]*)x([1-9][0-9]*) loaded=false')
RELEASE = re.compile(r'native timing gate: document released for owned pid=([1-9][0-9]*) size=1280x880 prefix_bytes=(\d+) prefix_sha256=([0-9a-f]{64}) html_sha256=([0-9a-f]{64})')
CPU_VERIFIED = re.compile(r'presenter: Vulkan verified serial=(\d+) generation=(\d+) viewport_revision=(\d+) size=(\d+)x(\d+) compared_bytes=(\d+) exact=true \(before compositor\)')
SUMMARY = re.compile(r'presenter: Vulkan acquired-texture verification passed; route=(cpu-upload|native-raster) frames=(\d+) compared_bytes=(\d+) last_serial=(\d+) \(before compositor\)')
ADAPTER = re.compile(r'presenter: Vulkan adapter=' + base.QUOTED + r' type=(?:DiscreteGpu|IntegratedGpu|VirtualGpu|Other|Cpu) driver=' + base.QUOTED + r' format=(Bgra8Unorm|Rgba8Unorm) color_space=Srgb alpha=Opaque mode=Fifo; raster=(CPU upload|native-shaders with complete CPU admission fallback)')
ROUTES = {'cpu': 'cpu-upload', 'gpu': 'native-raster'}
NS = ('ui_prepare_ns', 'queue_ns', 'acquire_ns', 'encode_upload_ns', 'submit_ns',
      'completion_wait_ns', 'cleanup_ns', 'present_call_ns', 'owner_total_ns', 'prepare_to_present_ns')
REPORT_KEYS = set(('schema kind mode route requested_frames completed_frames accepted_frames scene_hex '
                   'width height generation viewport_revision scale_factor source_load_ms edit_sequence '
                   'diagnostics checked failure owner_failure owner_init_ns samples').split())
SAMPLE_KEYS = set(('id generation viewport_revision serial width height route outcome submission configured').split()) | set(NS)
MAX_REPORT = 256 * 1024
MAX_SCENE = 64 * 1024


def integer(value, minimum=0, maximum=(1 << 64) - 1):
    if type(value) is not int or not minimum <= value <= maximum:
        raise ValueError('bounded integer required')
    return value


def strict_json(data: bytes, cap=MAX_REPORT):
    if len(data) > cap:
        raise ValueError('JSON byte bound')
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate JSON key')
            result[key] = value
        return result
    def bad_constant(_):
        raise ValueError('nonfinite JSON number')
    try:
        return json.loads(data, object_pairs_hook=pairs, parse_constant=bad_constant)
    except (RecursionError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError('malformed bounded JSON') from error


class IdentityReader:
    def __init__(self, data):
        self.data, self.at = data, 0
    def take(self, count):
        if count < 0 or count > len(self.data) - self.at:
            raise ValueError('truncated scene identity')
        value = self.data[self.at:self.at + count]
        self.at += count
        return value
    def u32(self):
        return int.from_bytes(self.take(4), 'little')
    def boolean(self):
        value = self.take(1)[0]
        if value not in (0, 1):
            raise ValueError('noncanonical scene boolean')
        return bool(value)
    def scalar(self):
        value = struct.unpack('<f', self.take(4))[0]
        if not math.isfinite(value):
            raise ValueError('nonfinite scene scalar')
        return value
    def blob(self):
        return self.take(self.u32())
    def text(self):
        try:
            return self.blob().decode('utf-8', errors='strict')
        except UnicodeError as error:
            raise ValueError('scene text is not UTF-8') from error
    def rect(self):
        value = [self.scalar() for _ in range(4)]
        if min(value[2:]) < 0:
            raise ValueError('negative scene extent')
        return value


def parse_identity(scene_hex: str, url: str) -> dict:
    if (type(scene_hex) is not str or not 2 <= len(scene_hex) <= MAX_SCENE * 2
            or len(scene_hex) % 2 or re.fullmatch('[0-9a-f]+', scene_hex) is None):
        raise ValueError('complete lowercase scene hex required')
    data = bytes.fromhex(scene_hex)
    reader = IdentityReader(data)
    if reader.take(5) != b'EBNI\x01' or (reader.u32(), reader.u32()) != wide.SIZE:
        raise ValueError('scene version or viewport mismatch')
    scroll, zoom = reader.scalar(), reader.scalar()
    title, address = reader.text(), reader.text()
    focused = reader.boolean()
    value = reader.text()
    caret, anchor = reader.u32(), reader.u32()
    # Selection indices are UTF-8 byte offsets in the browser.
    selected = address if focused else value
    selected_bytes = selected.encode()
    if any(offset > len(selected_bytes) or
           (offset < len(selected_bytes) and selected_bytes[offset] & 0xc0 == 0x80)
           for offset in (caret, anchor)):
        raise ValueError('scene selection is not normalized UTF-8')
    has_focus = reader.boolean()
    if has_focus:
        reader.u32()
        if reader.boolean():
            reader.rect()
            reader.boolean()
    height = reader.scalar()
    status = reader.text()
    if (scroll != 0 or zoom != 1 or title != 'N' or address != url or height < 0
            or status != 'Native timing scene' or has_focus):
        raise ValueError('unexpected loaded fixture paint inputs')
    count = reader.u32()
    if count > 256:
        raise ValueError('scene command bound')
    stack, image_keys = [], []
    for _ in range(count):
        tag = reader.take(1)[0]
        if tag in (0, 2, 4):
            if len(stack) >= 128:
                raise ValueError('scene scope bound')
            stack.append(tag)
        elif tag in (1, 3, 5):
            if not stack or stack.pop() != tag - 1:
                raise ValueError('scene scope mismatch')
        if tag == 0:
            reader.rect()
        elif tag in (1, 2, 3, 5):
            pass
        elif tag == 4:
            if not 0 <= reader.scalar() <= 1:
                raise ValueError('scene opacity')
        elif tag == 6:
            reader.rect()
            reader.take(4)
            if reader.scalar() < 0:
                raise ValueError('negative scene radius')
        elif tag == 7:
            reader.scalar(); reader.scalar(); reader.text()
            if reader.scalar() <= 0:
                raise ValueError('nonpositive scene font size')
            reader.take(4)
            for _ in range(3):
                reader.boolean()
        elif tag == 8:
            reader.rect()
            image_keys.append(reader.text())
        elif tag == 9:
            values = [reader.scalar() for _ in range(5)]
            if values[-1] < 0:
                raise ValueError('negative scene line width')
            reader.take(4)
        else:
            raise ValueError('unknown scene command')
    if stack:
        raise ValueError('unclosed scene scope')
    image_count = reader.u32()
    if image_count > 256:
        raise ValueError('scene image bound')
    images, prior_key = [], None
    for index in range(image_count):
        key = reader.text()
        key_bytes = key.encode()
        if prior_key is not None and key_bytes <= prior_key:
            raise ValueError('scene image keys not uniquely sorted')
        prior_key = key_bytes
        alias, width, height = reader.u32(), reader.u32(), reader.u32()
        rgba = reader.blob()
        if (alias > index or not width or not height or width * height * 4 != len(rgba)
                or (alias < index and (images[alias]['alias'] != alias
                                      or images[alias]['payload'] != (width, height, rgba)))):
            raise ValueError('scene image payload/alias mismatch')
        images.append({'key': key, 'alias': alias, 'payload': (width, height, rgba)})
    if (reader.at != len(data) or count != 6 or image_count != 1
            or len(image_keys) != 1 or image_keys[0] != images[0]['key']
            or images[0]['payload'][:2] != (2, 1)):
        raise ValueError('complete frozen loaded scene inventory mismatch')
    return {'bytes': len(data), 'sha256': wide.sha(data), 'commands': count,
            'images': image_count, 'status': status, 'title': title, 'url': address}


def validate_report(data: bytes, route: str, mode: str, frames: int, url: str) -> dict:
    if route not in ROUTES or mode not in ('check', 'measure') or not data.endswith(b'\n'):
        raise ValueError('report route/mode/framing')
    report = strict_json(data)
    if type(report) is not dict or set(report) != REPORT_KEYS:
        raise ValueError('report schema keys')
    expected = 1 if mode == 'check' else integer(frames, 2, 128)
    if (type(report['schema']) is not int or report['schema'] != 1 or report['kind'] != 'native-benchmark'
            or report['route'] != ROUTES[route] or report['mode'] != mode
            or integer(report['requested_frames']) != expected
            or integer(report['completed_frames']) != expected
            or (integer(report['width']), integer(report['height'])) != wide.SIZE
            or report['failure'] is not None or report['owner_failure'] is not None):
        raise ValueError('unsuccessful/incomplete benchmark report')
    for name in ('generation', 'viewport_revision'):
        integer(report[name], 1)
    integer(report['edit_sequence'])
    integer(report['diagnostics'])
    scale = report['scale_factor']
    if type(scale) not in (int, float) or not math.isfinite(scale) or scale <= 0:
        raise ValueError('report scale factor')
    load = report['source_load_ms']
    if load is not None and (type(load) not in (int, float) or not math.isfinite(load) or load < 0):
        raise ValueError('report load metadata')
    identity = parse_identity(report['scene_hex'], url)
    if mode == 'check':
        if (report['checked'] is not True or integer(report['accepted_frames']) != 0
                or report['samples'] != [] or report['owner_init_ns'] is not None):
            raise ValueError('check must contain retirement proof without timing records')
    else:
        if (report['checked'] is not False or integer(report['accepted_frames']) != expected
                or type(report['samples']) is not list or len(report['samples']) != expected):
            raise ValueError('timing sample population mismatch')
        integer(report['owner_init_ns'])
        previous_serial = 0
        for index, sample in enumerate(report['samples'], 1):
            if type(sample) is not dict or set(sample) != SAMPLE_KEYS:
                raise ValueError('timing sample keys')
            if (integer(sample['id']) != index or sample['route'] != ROUTES[route]
                    or sample['outcome'] != 'presented' or sample['submission'] != 'draws'
                    or type(sample['configured']) is not bool
                    or integer(sample['serial'], 1) <= previous_serial
                    or any(integer(sample[key]) != report[key] for key in
                           ('generation', 'viewport_revision', 'width', 'height'))):
                raise ValueError('timing sample identity/order/outcome')
            previous_serial = sample['serial']
            for key in NS:
                if key == 'cleanup_ns' and route == 'cpu':
                    if sample[key] is not None:
                        raise ValueError('CPU upload has no cleanup timing bracket')
                else:
                    integer(sample[key])
    return {'report': report, 'identity': identity}


def validate_control(control: dict, port: int, bodies: dict) -> None:
    if (control.get('success') is not True or control.get('control_complete') is not True
            or control.get('browser_reaped') is not True or control.get('browser_returncode') != 0
            or control.get('controlled_size') != list(wide.SIZE) or control.get('port') != port
            or any(key in control for key in ('error', 'cleanup_error', 'postcheck_error', 'stderr_drain_error'))):
        raise ValueError('timing window ownership/control/cleanup failed')
    integer(control['identity']['pid'], 1)
    requests = control.get('requests')
    if (type(requests) is not list or len(requests) != 2
            or [r['path'] for r in requests] != list(wide.ROUTES)):
        raise ValueError('fixed timing HTTP request inventory')
    integer(control['accepted_connections'], 2, wide.MAX_CONNECTIONS)
    for row in requests:
        body = bodies[row['path']]
        response = wide.response_for(row['path'], bodies)
        if (row['method'] != 'GET' or row['body_bytes'] != len(body)
                or row['body_sha256'] != wide.sha(body) or row['response_complete'] is not True
                or row['response_bytes'] != len(response) or row['response_sha256'] != wide.sha(response)
                or re.fullmatch('[0-9a-f]{64}', row['header_sha256']) is None):
            raise ValueError('timing HTTP response binding')
        integer(row['header_bytes'], 1, wide.MAX_HEADER)


def validate_stderr(data: bytes, control: dict, report: dict, port: int, bodies: dict) -> dict:
    validate_control(control, port, bodies)
    if len(data) > base.MAX_OUTPUT or not data.endswith(b'\n'):
        raise ValueError('timing stderr framing/bound')
    try:
        lines = data.decode('utf-8').splitlines(keepends=True)
    except UnicodeError as error:
        raise ValueError('timing stderr encoding') from error
    if len(lines) > 4096 or any(len(line.encode()) > 16384 for line in lines):
        raise ValueError('timing diagnostic bounds')
    prefix, released, physical = bytearray(), False, False
    verified, summary, presented, adapters, other = [], [], [], [], []
    check = report['mode'] == 'check'
    for raw in lines:
        line = raw.rstrip('\n')
        if match := WAITING.fullmatch(line):
            if released:
                raise ValueError('loading resumed after timing gate')
            physical = tuple(map(int, match.groups())) == wide.SIZE
        elif match := RELEASE.fullmatch(line):
            event = {'prefix_bytes': len(prefix), 'prefix_sha256': wide.sha(prefix),
                     'html_sha256': wide.sha(bodies['/admitted.html']), 'physical_size': list(wide.SIZE)}
            if (released or not physical or int(match[1]) != control['identity']['pid']
                    or int(match[2]) != len(prefix) or match[3] != event['prefix_sha256']
                    or match[4] != event['html_sha256'] or control.get('release') != event):
                raise ValueError('timing release causal/physical binding')
            released = True
        elif match := ADAPTER.fullmatch(line):
            expected = 'CPU upload' if report['route'] == 'cpu-upload' else 'native-shaders with complete CPU admission fallback'
            if match[2] != expected or adapters or summary:
                raise ValueError('timing adapter route/count mismatch')
            adapters.append({'line': line, 'format': match[1]})
        elif match := base.PRESENTED.fullmatch(line):
            width, height, serial, reference = match.groups()
            if (not released or not check or report['route'] != 'native-raster' or presented
                    or summary or reference != 'true' or (int(width), int(height)) != wide.SIZE):
                raise ValueError('unexpected native timing presentation diagnostic')
            presented.append(int(serial))
        elif (match := CPU_VERIFIED.fullmatch(line)) or (match := base.VERIFIED.fullmatch(line)):
            cpu = CPU_VERIFIED.fullmatch(line) is not None
            serial, generation, revision, width, height, count = map(int, match.groups())
            if (not released or not check or verified or summary or serial <= 0
                    or cpu != (report['route'] == 'cpu-upload') or (width, height) != wide.SIZE
                    or count != wide.PIXEL_BYTES or generation != report['generation']
                    or revision != report['viewport_revision'] or (not cpu and presented != [serial])):
                raise ValueError('verification must match one loaded check frame')
            verified.append(serial)
        elif match := SUMMARY.fullmatch(line):
            route, frames, count, serial = match.groups()
            if (not check or not released or summary or verified != [int(serial)]
                    or route != report['route'] or int(frames) != 1 or int(count) != wide.PIXEL_BYTES):
                raise ValueError('check verification summary mismatch')
            summary.append(int(serial))
        elif (line.startswith(('native benchmark', 'native timing gate:', 'presenter:', 'native scene',
                               'paint:', 'eris-browser:', 'Unable to open browser window:', "thread '"))
              or 'panicked at' in line):
            raise ValueError('unknown/failure timing diagnostic')
        else:
            other.append(line)
        prefix.extend(raw.encode())
    if not released or len(adapters) != 1 or (check and (len(verified) != 1 or summary != verified)):
        raise ValueError('timing gate/adapter/check population mismatch')
    if not check and (verified or summary or presented):
        raise ValueError('measured process performed verification')
    return {'adapter': adapters[0], 'verified_frames': len(verified),
            'compared_bytes': wide.PIXEL_BYTES if verified else 0, 'other_diagnostics': other}


def distributions(values):
    if not values:
        raise ValueError('empty timing population')
    ordered = sorted(values)
    return {'count': len(values), 'median_ns': statistics.median(ordered),
            'p95_nearest_rank_ns': ordered[math.ceil(len(ordered) * .95) - 1]}


def sample_statistics(reports, metrics):
    if not reports:
        raise ValueError('missing paired timing population')
    return {metric: {
        'first_frame': distributions([r['samples'][0][metric] for r in reports]),
        'steady_frames': distributions([s[metric] for r in reports for s in r['samples'][1:]])}
        for metric in metrics}


def statistics_for(reports):
    result = {}
    for route in ROUTES.values():
        selected = [r for r in reports if r['mode'] == 'measure' and r['route'] == route]
        if not selected:
            raise ValueError('missing paired timing route')
        result[route] = sample_statistics(selected, ('prepare_to_present_ns', 'owner_total_ns'))
    return result


def fixture_data(directory: Path) -> dict:
    frozen = directory / 'freeze.json'
    if frozen.is_symlink() or base.digest(frozen, 65536) != base.FIXTURE_SHA256:
        raise ValueError('timing fixture freeze changed')
    for row in strict_json(frozen.read_bytes(), 65536)['files']:
        path = directory / row['path']
        if (path.is_symlink() or not path.resolve().is_relative_to(directory.resolve())
                or path.stat().st_size != row['bytes'] or base.digest(path, 65536) != row['sha256']):
            raise ValueError('timing copied fixture changed')
    return {route: (directory / name).read_bytes() for route, (name, _) in wide.ROUTES.items()}


def source_binding() -> list[dict]:
    paths = {ROOT / name for name in ('Cargo.toml', 'Cargo.lock', 'crates/raster-core/Cargo.toml',
                                     'crates/raster-core/Cargo.lock') if (ROOT / name).exists()}
    for parent in ('src', 'crates/raster-core/src', 'assets'):
        paths.update(path for path in (ROOT / parent).rglob('*') if path.is_file())
    paths.update(path for path in (ROOT / 'tools').glob('*.py'))
    paths.update(path for path in (ROOT / 'tools/vulkan-raster-probe').glob('*.py'))
    if (ROOT / 'build.rs').exists():
        paths.add(ROOT / 'build.rs')
    if len(paths) > 1024:
        raise ValueError('source manifest count bound')
    rows, total = [], 0
    for path in sorted(paths):
        if path.is_symlink():
            raise ValueError('source manifest does not follow symlinks')
        size = path.stat().st_size
        total += size
        if total > 64 * 1024 * 1024:
            raise ValueError('source manifest byte bound')
        rows.append({'path': str(path.relative_to(ROOT)), 'bytes': size, 'sha256': base.digest(path, 8 * 1024 * 1024)})
    return rows


def compiled_sources(root: Path) -> list[dict]:
    """Complete release-input scope, independently rooted for each executable."""
    if root.is_symlink() or not root.is_dir():
        raise ValueError('compiled source root must be a real directory')
    root = root.resolve()
    paths = []
    def add(path):
        if path.is_symlink() or not path.is_file():
            raise ValueError('compiled inputs must be regular files without symlinks')
        if len(paths) >= 1024:
            raise ValueError('compiled source file bound')
        paths.append(path)
    for name in ('Cargo.toml', 'Cargo.lock', 'crates/raster-core/Cargo.toml', 'crates/raster-core/Cargo.lock'):
        path = root
        for part in Path(name).parts:
            path /= part
            if path.is_symlink():
                raise ValueError('compiled source path contains symlink')
        add(path)
    if (root / 'build.rs').exists() or (root / 'build.rs').is_symlink():
        add(root / 'build.rs')
    entries = 0
    for name in ('src', 'crates/raster-core/src', 'assets'):
        parent = root / name
        if parent.is_symlink() or not parent.is_dir():
            raise ValueError('compiled source directory missing or symlinked')
        pending = [(parent, 0)]
        while pending:
            directory, depth = pending.pop()
            if depth > 32:
                raise ValueError('compiled source directory depth bound')
            with os.scandir(directory) as scan:
                for entry in scan:
                    entries += 1
                    if entries > 4096 or entry.is_symlink():
                        raise ValueError('compiled source entry bound or symlink')
                    if entry.is_dir(follow_symlinks=False):
                        pending.append((Path(entry.path), depth + 1))
                    else:
                        add(Path(entry.path))
    rows, total = [], 0
    for path in sorted(paths):
        size = path.stat().st_size
        total += size
        if total > 64 * 1024 * 1024:
            raise ValueError('compiled source byte bound')
        rows.append({'path': path.relative_to(root).as_posix(), 'bytes': size,
                     'sha256': base.digest(path, 8 * 1024 * 1024)})
    return rows


def compiled_binding(binary: Path, manifest: Path, pin: str, source_root: Path) -> dict:
    """No stale-source exception: every listed byte must match its own source root."""
    if (not isinstance(pin, str) or re.fullmatch('[0-9a-f]{64}', pin) is None
            or manifest.is_symlink() or not manifest.is_file()
            or base.digest(manifest, MAX_REPORT) != pin):
        raise ValueError('compiled manifest pin mismatch')
    raw = manifest.read_bytes()
    if wide.sha(raw) != pin:
        raise ValueError('compiled manifest changed during read')
    record = strict_json(raw)
    keys = {'binary', 'bytes', 'sha256', 'profile', 'toolchain', 'features', 'source_files'}
    if (type(record) is not dict or set(record) != keys or record['profile'] != 'release'
            or record['features'] != ['vulkan-raster'] or type(record['toolchain']) is not str
            or re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?', record['toolchain']) is None
            or len(record['toolchain']) > 64 or type(record['binary']) is not str
            or not record['binary'] or len(record['binary']) > 4096
            or type(record['sha256']) is not str or re.fullmatch('[0-9a-f]{64}', record['sha256']) is None):
        raise ValueError('compiled release manifest schema/profile/features')
    integer(record['bytes'], 1, base.MAX_BINARY)
    if (not binary.is_file() or binary.stat().st_size != record['bytes']
            or base.digest(binary, base.MAX_BINARY) != record['sha256']):
        raise ValueError('compiled executable size/hash mismatch')
    rows = record['source_files']
    if type(rows) is not list or not 1 <= len(rows) <= 1024:
        raise ValueError('compiled manifest source count')
    seen = set()
    for row in rows:
        if type(row) is not dict or set(row) != {'path', 'bytes', 'sha256'}:
            raise ValueError('compiled source row schema')
        name = row['path']
        if (type(name) is not str or not name or len(name) > 512 or '\\' in name or '\0' in name
                or Path(name).is_absolute() or any(part in ('', '.', '..') for part in name.split('/'))
                or name in seen or type(row['sha256']) is not str
                or re.fullmatch('[0-9a-f]{64}', row['sha256']) is None):
            raise ValueError('compiled source path/hash/duplicate')
        seen.add(name)
        integer(row['bytes'], 0, 8 * 1024 * 1024)
    observed = compiled_sources(source_root)
    if sorted(rows, key=lambda row: row['path']) != observed:
        raise ValueError('compiled source inventory/bytes mismatch')
    return {'binary': str(binary.resolve()), 'source_directory': str(source_root.resolve()),
            'manifest': str(manifest.resolve()), 'manifest_bytes': len(raw), 'manifest_sha256': pin,
            'release': record}


def launch_owned(args) -> int:
    if (args.receipt.exists() or not args.receipt.parent.is_dir()
            or not math.isfinite(args.window_seconds) or not 2 <= args.window_seconds <= 20
            or base.digest(args.binary, base.MAX_BINARY) != args.binary_sha256
            or base.digest(args.controller, base.MAX_BINARY) != args.controller_sha256
            or args.route not in ROUTES or args.mode not in ('check', 'measure')):
        raise ValueError('fresh timing receipt and immutable launch inputs required')
    integer(args.frames, 2, 128)
    integer(args.port, 1, 65535)
    validate_listener_fds((args.listener_fd,))
    bodies = fixture_data(args.fixtures)
    listener = socket.socket(fileno=args.listener_fd)
    if listener.getsockname() != ('127.0.0.1', args.port):
        listener.close()
        raise ValueError('timing inherited listener port mismatch')
    listener.setblocking(False)
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [], 'requests': [],
              'accepted_connections': 0, 'identity': None, 'browser_reaped': False, 'browser_returncode': None,
              'binary_sha256': args.binary_sha256, 'controller_sha256': args.controller_sha256,
              'fixture_freeze': base.FIXTURE_SHA256, 'port': args.port, 'route': args.route, 'mode': args.mode}
    child, emitted = None, 0
    deadline = time.monotonic() + args.window_seconds + 5
    def persist():
        temp = args.receipt.with_suffix('.tmp')
        temp.write_text(json.dumps(record, indent=2) + '\n')
        temp.replace(args.receipt)
    def emit(data):
        nonlocal emitted
        keep = min(len(data), base.MAX_OUTPUT - emitted)
        sys.stderr.buffer.write(data[:keep]); sys.stderr.buffer.flush()
        emitted += keep
        if keep != len(data):
            raise ValueError('timing relay output limit')
    persist()
    try:
        url = f'http://127.0.0.1:{args.port}/admitted.html'
        command = [str(args.binary), url, '--presenter=vulkan', f'--raster={args.route}', '--no-scripts',
                   '--exit-after', str(args.window_seconds)]
        command += ['--benchmark-native-check'] if args.mode == 'check' else ['--benchmark-native', str(args.frames)]
        record['browser_command'] = command
        persist()
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE, close_fds=True)
        record['identity'] = base.child_identity(child.pid)
        persist()
        base.configure_owned(record['identity'], args.controller, dict(os.environ), record, persist,
                             requested_size=wide.SIZE)
        wide.serve_and_relay(child, listener, record, bodies, persist, deadline, emit,
                             waiting_pattern=WAITING, marker_prefix='native timing gate',
                             forbidden_patterns=(CPU_VERIFIED, SUMMARY))
        record['browser_returncode'] = child.wait(timeout=2)
        record['browser_reaped'] = True
        record['success'] = record['browser_returncode'] == 0
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        listener.close()
        if child is not None:
            if not record['browser_reaped']:
                try:
                    os.waitid(os.P_PID, child.pid, base.WAIT_FLAGS)
                    os.kill(child.pid, signal.SIGKILL)
                    record['browser_returncode'] = child.wait(timeout=2)
                    record['browser_reaped'] = True
                except (OSError, subprocess.SubprocessError) as error:
                    record['cleanup_error'] = str(error)[:4096]
            try:
                drain_stderr(child.stderr, emit)
            except (OSError, ValueError) as error:
                record['success'] = False
                record['stderr_drain_error'] = str(error)[:4096]
            child.stderr.close()
        try:
            if (base.digest(args.binary, base.MAX_BINARY) != args.binary_sha256
                    or base.digest(args.controller, base.MAX_BINARY) != args.controller_sha256
                    or fixture_data(args.fixtures) != bodies):
                raise ValueError('timing launch inputs changed')
        except (OSError, ValueError) as error:
            record['success'] = False
            record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def sequence(repeats: int):
    integer(repeats, 1, 8)
    result = [('check-cpu', 'cpu', 'check'), ('check-gpu', 'gpu', 'check')]
    for index in range(repeats):
        for route in (('cpu', 'gpu') if index % 2 == 0 else ('gpu', 'cpu')):
            result.append((f'measure-{index + 1}-{route}', route, 'measure'))
    return result


def comparison_sequence(comparison: str, frames: int, repeats: int):
    if comparison == 'cpu-native':
        return [(name, route, mode, None) for name, route, mode in sequence(repeats)]
    if comparison != 'native-ab' or integer(frames) != 16 or integer(repeats) != 3:
        raise ValueError('native-ab requires exactly three pairs of sixteen frames')
    runs = [('check-baseline', 'gpu', 'check', 'baseline'), ('check-candidate', 'gpu', 'check', 'candidate')]
    for index in range(3):
        for variant in (('baseline', 'candidate') if index % 2 == 0 else ('candidate', 'baseline')):
            runs.append((f'measure-{index + 1}-{variant}', 'gpu', 'measure', variant))
    return runs


def run_host(binary: Path, output: Path, loader: Path, timeout: float, window_seconds: float,
             frames=16, repeats=3, expected_binary_sha256=None, *, comparison='cpu-native',
             candidate_manifest=None, candidate_manifest_sha256=None, baseline_binary=None,
             baseline_manifest=None, baseline_manifest_sha256=None, baseline_source=None) -> dict:
    integer(frames, 2, 128)
    runs = comparison_sequence(comparison, frames, repeats)
    comparison_inputs = (candidate_manifest, candidate_manifest_sha256, baseline_binary,
                         baseline_manifest, baseline_manifest_sha256, baseline_source)
    if ((comparison == 'native-ab' and any(value is None for value in comparison_inputs))
            or (comparison == 'cpu-native' and any(value is not None for value in comparison_inputs))):
        raise ValueError('native-ab requires both manifests, pins and baseline source/binary; default mode takes none')
    if (output.exists() or not math.isfinite(timeout) or not math.isfinite(window_seconds)
            or not 2 <= window_seconds <= 20 or not window_seconds + 10 <= timeout <= 120
            or re.fullmatch('[0-9a-f]{64}', expected_binary_sha256 or '') is None):
        raise ValueError('new output, bounded lifetime and frozen binary SHA required')
    binary, output = binary.resolve(), output.resolve()
    env = base.environment(loader)
    found = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not found:
        raise ValueError('timing requires owned compositor control')
    controller = Path(found).resolve()
    def bindings():
        bound = {'binary': base.digest(binary, base.MAX_BINARY), 'controller': base.digest(controller, base.MAX_BINARY),
                 'fixtures': base.fixture_binding(), 'source_files': source_binding(),
                 'loader': base.loader_binding(loader, env)}
        if comparison == 'native-ab':
            bound['variants'] = {
                'baseline': compiled_binding(baseline_binary, baseline_manifest, baseline_manifest_sha256, baseline_source),
                'candidate': compiled_binding(binary, candidate_manifest, candidate_manifest_sha256, ROOT)}
        return bound
    frozen = bindings()
    if frozen['binary'] != expected_binary_sha256:
        raise ValueError('release binary does not match supplied frozen SHA')
    output.mkdir(parents=True)
    copied = output / 'fixtures'
    shutil.copytree(base.FIXTURES, copied)
    if comparison == 'native-ab':
        for variant, manifest in (('baseline', baseline_manifest), ('candidate', candidate_manifest)):
            shutil.copyfile(manifest, output / f'{variant}-release.json')
    bodies = fixture_data(copied)
    result = {'schema': 1, 'success': False, 'runs': [], 'bindings': frozen, 'binary': str(binary),
              'controller': str(controller), 'frames': frames, 'repeats': repeats,
              'timeout_seconds': timeout, 'window_seconds': window_seconds,
              'scope': 'host durations before compositor; separate correctness checks; no GPU timestamps',
              'cache_scope': 'fresh process per run; first loaded frame can follow startup font work; native mask session rebuilt each frame',
              'percentile': 'nearest rank, pooled complete steady samples; first frame kept separately'}
    if comparison == 'native-ab':
        result['comparison'] = comparison
        result['comparison_scope'] = 'two independently bound native executables; one static scene, no hidden warmups or allocation-policy toggle'
    listener = None
    def persist():
        temp = output / 'timing-host-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n')
        temp.replace(output / 'timing-host-results.json')
    def unchanged():
        if bindings() != frozen or fixture_data(copied) != bodies:
            raise ValueError('timing source/fixture/executable inputs changed')
        if comparison == 'native-ab':
            for variant, bound in frozen['variants'].items():
                if base.digest(output / f'{variant}-release.json', MAX_REPORT) != bound['manifest_sha256']:
                    raise ValueError('copied compiled manifest changed')
    persist()
    try:
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(('127.0.0.1', 0))
        listener.listen(wide.MAX_ACTIVE_CONNECTIONS)
        listener.setblocking(False)
        port = listener.getsockname()[1]
        url = f'http://127.0.0.1:{port}/admitted.html'
        result.update(port=port, url=url)
        scene, scale, adapter = None, None, None
        accepted_reports = []
        for name, route, mode, variant in runs:
            unchanged()
            run_binary, run_hash = str(binary), frozen['binary']
            if variant is not None:
                bound = frozen['variants'][variant]
                run_binary, run_hash = bound['binary'], bound['release']['sha256']
            receipt = output / f'{name}.window-control.json'
            command = [sys.executable, str(Path(__file__).resolve()), '--owned-launch', '--binary', run_binary,
                       '--receipt', str(receipt), '--controller', str(controller), '--controller-sha256', frozen['controller'],
                       '--binary-sha256', run_hash, '--listener-fd', str(listener.fileno()), '--port', str(port),
                       '--fixtures', str(copied), '--route', route, '--mode', mode, '--frames', str(frames),
                       '--window-seconds', str(window_seconds), '--allow-experimental-gpu']
            attempt = {'name': name, 'route': route, 'mode': mode, 'command': command, 'success': False}
            if variant is not None:
                attempt['variant'] = variant
            result['runs'].append(attempt)
            persist()
            record, stdout = base.run_supervised(command, output, name, env, timeout, pass_fds=(listener.fileno(),))
            attempt['supervision'] = record
            persist()
            if receipt.is_file():
                attempt['control_sha256'] = base.digest(receipt, 1024 * 1024)
                attempt['control'] = strict_json(receipt.read_bytes(), 1024 * 1024)
                persist()
            base.check_success(record)
            if ((output / f'{name}.supervisor.stderr.log').stat().st_size
                    or record.get('capture_gate_required') or record.get('capture_gate_granted')):
                raise ValueError('unexpected timing supervisor diagnostics/gate')
            stderr_path = output / f'{name}.stderr.log'
            if base.digest(stderr_path, base.MAX_OUTPUT) != record['stderr_sha256'] or wide.sha(stdout) != record['stdout_sha256']:
                raise ValueError('timing raw output binding mismatch')
            control = attempt['control']
            if (control.get('binary_sha256') != run_hash or control.get('controller_sha256') != frozen['controller']
                    or control.get('fixture_freeze') != base.FIXTURE_SHA256 or control.get('route') != route
                    or control.get('mode') != mode):
                raise ValueError('timing wrapper inputs mismatch')
            validated = validate_report(stdout, route, mode, frames, url)
            report = validated['report']
            diagnostics = validate_stderr(stderr_path.read_bytes(), control, report, port, bodies)
            # Keep exact bytes, not just their hash, as the cross-process equality test.
            current = bytes.fromhex(report['scene_hex'])
            adapter_key = diagnostics['adapter']['line'].split('; raster=')[0]
            if scene is None:
                scene, scale, adapter = current, report['scale_factor'], adapter_key
                (output / 'scene.ebni').write_bytes(scene)
            elif current != scene or report['scale_factor'] != scale or adapter_key != adapter:
                raise ValueError('paired scene bytes, scale or Vulkan adapter changed')
            attempt['validation'] = {'identity': validated['identity'], 'diagnostics': diagnostics}
            attempt['report'] = report
            if mode == 'measure':
                attempt['statistics'] = {metric: {
                    'first_frame_ns': report['samples'][0][metric],
                    'steady_frames': distributions([sample[metric] for sample in report['samples'][1:]])}
                    for metric in ('prepare_to_present_ns', 'owner_total_ns')}
            accepted_reports.append(report)
            try:
                extra, _ = listener.accept()
            except BlockingIOError:
                pass
            else:
                extra.close()
                raise ValueError('HTTP connection survived its case boundary')
            unchanged()
            attempt['success'] = True
            persist()
        if comparison == 'native-ab':
            result['statistics'], result['phase_statistics'], result['initialization_statistics'] = {}, {}, {}
            for variant in ('baseline', 'candidate'):
                selected = [run['report'] for run in result['runs'] if run['variant'] == variant and run['mode'] == 'measure']
                result['statistics'][variant] = sample_statistics(selected, ('prepare_to_present_ns', 'owner_total_ns'))
                result['phase_statistics'][variant] = sample_statistics(selected, NS[:-2])
                result['initialization_statistics'][variant] = distributions([r['owner_init_ns'] for r in selected])
        else:
            result['statistics'] = statistics_for(accepted_reports)
        result['scene_sha256'] = wide.sha(scene)
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        if listener is not None:
            listener.close()
        try:
            unchanged()
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
            result['success'] = False
            result['postcheck_error'] = str(error)[:4096]
        persist()
    return result


def main() -> int:
    owned = sys.argv[1:2] == ['--owned-launch']
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--window-seconds', type=float, default=20)
    parser.add_argument('--frames', type=int, default=16)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    if owned:
        parser.add_argument('--receipt', type=Path, required=True)
        parser.add_argument('--controller', type=Path, required=True)
        parser.add_argument('--controller-sha256', required=True)
        parser.add_argument('--listener-fd', type=int, required=True)
        parser.add_argument('--port', type=int, required=True)
        parser.add_argument('--fixtures', type=Path, required=True)
        parser.add_argument('--route', choices=ROUTES, required=True)
        parser.add_argument('--mode', choices=('check', 'measure'), required=True)
    else:
        parser.add_argument('--output', type=Path, required=True)
        parser.add_argument('--loader-directory', type=Path, required=True)
        parser.add_argument('--timeout', type=float, default=30)
        parser.add_argument('--repeats', type=int, default=3)
        parser.add_argument('--comparison', choices=('cpu-native', 'native-ab'), default='cpu-native')
        parser.add_argument('--candidate-manifest', type=Path)
        parser.add_argument('--candidate-manifest-sha256')
        parser.add_argument('--baseline-binary', type=Path)
        parser.add_argument('--baseline-manifest', type=Path)
        parser.add_argument('--baseline-manifest-sha256')
        parser.add_argument('--baseline-source', type=Path)
    args = parser.parse_args(sys.argv[2:] if owned else None)
    if not args.allow_experimental_gpu:
        parser.error('actual paired windows require --allow-experimental-gpu')
    def interrupted(_sig, _frame):
        raise KeyboardInterrupt
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    try:
        if owned:
            return launch_owned(args)
        result = run_host(args.binary, args.output, args.loader_directory, args.timeout, args.window_seconds,
                          args.frames, args.repeats, args.binary_sha256, comparison=args.comparison,
                          candidate_manifest=args.candidate_manifest, candidate_manifest_sha256=args.candidate_manifest_sha256,
                          baseline_binary=args.baseline_binary, baseline_manifest=args.baseline_manifest,
                          baseline_manifest_sha256=args.baseline_manifest_sha256, baseline_source=args.baseline_source)
    except (OSError, ValueError) as error:
        parser.exit(1, f'{error}\n')
    print(json.dumps({'success': result['success'], 'results': str(args.output / 'timing-host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
