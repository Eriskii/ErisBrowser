#!/usr/bin/env python3
"""Delayed local-document admission for two bounded 1280x880 native windows."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import selectors
import shutil
import signal
import socket
import subprocess
import sys
import time

import native_raster_host as base
from native_raster_followup import drain_stderr

SIZE = (1280, 880)
PIXEL_BYTES = 1280 * 880 * 4
CONVERSION_WORK = 160 * 110 * 64
MAX_HEADER = 8192
MAX_CONNECTIONS = 8
MAX_ACTIVE_CONNECTIONS = 4
CASES = ('wide-verified', 'wide-normal')
PHYSICAL = 'native scene awaits a current loaded page; size=1280x880'
RELEASE = re.compile(r'native wide gate: document released for owned pid=([1-9][0-9]*) size=1280x880 prefix_bytes=(\d+) prefix_sha256=([0-9a-f]{64}) html_sha256=([0-9a-f]{64})')
ROUTES = {'/admitted.html': ('admitted.html', 'text/html; charset=utf-8'),
          '/two-pixels.png': ('two-pixels.png', 'image/png')}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_request(header: bytes, port: int) -> str:
    if len(header) > MAX_HEADER or not header.endswith(b'\r\n\r\n'):
        raise ValueError('HTTP header bound or framing')
    lines = header[:-4].decode('ascii', errors='strict').split('\r\n')
    request = lines[0].split(' ')
    if len(request) != 3 or request[0] != 'GET' or request[1] not in ROUTES or request[2] not in ('HTTP/1.0', 'HTTP/1.1'):
        raise ValueError('only the two fixed GET paths are served')
    fields = {}
    if len(lines) > 65:
        raise ValueError('HTTP header count bound')
    for line in lines[1:]:
        if ':' not in line:
            raise ValueError('malformed HTTP header')
        key, value = line.split(':', 1)
        key = key.lower()
        if not re.fullmatch(r'[a-z0-9-]+', key) or key in fields:
            raise ValueError('malformed or duplicate HTTP header')
        fields[key] = value.strip()
    if (fields.get('host') != f'127.0.0.1:{port}' or 'transfer-encoding' in fields
            or fields.get('content-length', '0') != '0'):
        raise ValueError('unexpected host or request body')
    return request[1]


def response_for(path: str, bodies: dict[str, bytes]) -> bytes:
    body = bodies[path]
    kind = ROUTES[path][1]
    return (f'HTTP/1.1 200 OK\r\nContent-Type: {kind}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n'.encode() + body)


def serve_and_relay(child, listener, record: dict, bodies: dict[str, bytes], persist,
                    deadline: float, emit) -> None:
    """One selector, fixed routes, fixed admission gate, one original deadline."""
    stream = child.stderr
    os.set_blocking(stream.fileno(), False)
    browser_prefix = bytearray()
    pending = bytearray()
    connections = {}
    physical = False
    released = False
    stderr_open = True
    selector = selectors.DefaultSelector()
    selector.register(listener, selectors.EVENT_READ, 'listener')
    selector.register(stream, selectors.EVENT_READ, 'stderr')

    def close_connection(connection):
        selector.unregister(connection)
        connection.close()
        del connections[connection]

    def drain_browser():
        nonlocal physical, stderr_open
        # Before opening the gate, consume every already-readable byte. A proof
        # following the size line in the same read must remain pre-release.
        while stderr_open:
            if time.monotonic() >= deadline:
                raise ValueError('original wide-window lifetime expired')
            try:
                chunk = os.read(stream.fileno(), 16384)
            except BlockingIOError:
                return
            if not chunk:
                selector.unregister(stream)
                stderr_open = False
                if pending:
                    raise ValueError('unterminated browser diagnostic')
                return
            if len(browser_prefix) + len(pending) + len(chunk) > base.MAX_OUTPUT - 512:
                room = max(0, base.MAX_OUTPUT - 512 - len(browser_prefix) - len(pending))
                pending.extend(chunk[:room])
                raise ValueError('wide browser stderr bound')
            pending.extend(chunk)
            while (end := pending.find(b'\n')) != -1:
                line = bytes(pending[:end + 1])
                del pending[:end + 1]
                browser_prefix.extend(line)
                emit(line)
                text = line.decode('utf-8').rstrip('\n')
                if not released and (base.SCENE.fullmatch(text) or base.PRESENTED.fullmatch(text)
                                     or base.VERIFIED.fullmatch(text) or base.SUMMARY.fullmatch(text)):
                    raise ValueError('native proof preceded delayed-document release')
                if match := base.FALLBACK.fullmatch(text):
                    physical = match[1] == PHYSICAL
            if len(pending) > 16384:
                raise ValueError('wide browser diagnostic line bound')

    def release_document():
        nonlocal released
        if not stderr_open or pending or not physical or not record.get('control_complete') or released:
            return
        waiting = [(c, s) for c, s in connections.items() if s.get('path') == '/admitted.html']
        if not waiting:
            return
        base.ensure_live_child(record['identity'])
        prefix = bytes(browser_prefix)
        event = {'prefix_bytes': len(prefix), 'prefix_sha256': sha(prefix),
                 'html_sha256': sha(bodies['/admitted.html']), 'physical_size': list(SIZE)}
        marker = (f"native wide gate: document released for owned pid={record['identity']['pid']} size=1280x880 "
                  f"prefix_bytes={event['prefix_bytes']} prefix_sha256={event['prefix_sha256']} html_sha256={event['html_sha256']}\n").encode()
        record['release'] = event
        emit(marker)
        released = True
        persist()
        connection, state = waiting[0]
        state['response'] = response_for('/admitted.html', bodies)
        if state.pop('unregistered', False):
            selector.register(connection, selectors.EVENT_WRITE, 'http')
        else:
            selector.modify(connection, selectors.EVENT_WRITE, 'http')

    try:
        while stderr_open or connections:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ValueError('original wide-window lifetime expired')
            for key, _ in selector.select(min(remaining, 0.025)):
                if key.data == 'listener':
                    connection, peer = listener.accept()
                    connection.setblocking(False)
                    if (peer[0] != '127.0.0.1' or record['accepted_connections'] >= MAX_CONNECTIONS
                            or len(connections) >= MAX_ACTIVE_CONNECTIONS):
                        connection.close()
                        raise ValueError('owned HTTP connection bound')
                    record['accepted_connections'] += 1
                    connections[connection] = {'header': bytearray(), 'offset': 0}
                    selector.register(connection, selectors.EVENT_READ, 'http')
                    persist()
                elif key.data == 'stderr':
                    drain_browser()
                else:
                    connection = key.fileobj
                    state = connections[connection]
                    if 'response' in state:
                        try:
                            sent = connection.send(state['response'][state['offset']:])
                        except BlockingIOError:
                            continue
                        if sent <= 0:
                            raise ValueError('HTTP response write stopped')
                        state['offset'] += sent
                        if state['offset'] == len(state['response']):
                            state['request']['response_complete'] = True
                            state['request']['response_bytes'] = state['offset']
                            state['request']['response_sha256'] = sha(state['response'])
                            close_connection(connection)
                            persist()
                    else:
                        try:
                            chunk = connection.recv(MAX_HEADER + 1 - len(state['header']))
                        except BlockingIOError:
                            continue
                        if not chunk:
                            raise ValueError('HTTP request ended before its bounded header')
                        state['header'].extend(chunk)
                        if len(state['header']) > MAX_HEADER:
                            raise ValueError('HTTP request header exceeded bound')
                        end = state['header'].find(b'\r\n\r\n')
                        if end != -1:
                            if end + 4 != len(state['header']):
                                raise ValueError('HTTP request body/pipelining is unsupported')
                            path = parse_request(bytes(state['header']), record['port'])
                            if any(r['path'] == path for r in record['requests']) or len(record['requests']) >= len(ROUTES):
                                raise ValueError('duplicate or excess HTTP request')
                            if path != '/admitted.html' and not released:
                                raise ValueError('image requested before document release')
                            request = {'path': path, 'method': 'GET', 'header_bytes': len(state['header']),
                                       'header_sha256': sha(bytes(state['header'])), 'body_bytes': len(bodies[path]),
                                       'body_sha256': sha(bodies[path]), 'response_complete': False}
                            state.update(path=path, request=request)
                            record['requests'].append(request)
                            state['header'].clear()
                            persist()
                            if path == '/admitted.html':
                                # A parsed request waits without read/write readiness.
                                selector.unregister(connection)
                                state['unregistered'] = True
                            else:
                                state['response'] = response_for(path, bodies)
                                selector.modify(connection, selectors.EVENT_WRITE, 'http')
            if not released:
                drain_browser()
                release_document()
            if not stderr_open and connections:
                raise ValueError('browser stderr ended with incomplete HTTP responses')
        if not released or len(record['requests']) != 2 or not all(r['response_complete'] for r in record['requests']):
            raise ValueError('document gate and both fixed responses must complete')
        while os.waitid(os.P_PID, child.pid, base.WAIT_FLAGS) is None:
            if time.monotonic() >= deadline:
                raise ValueError('original browser exit deadline expired')
            time.sleep(0.025)
    finally:
        try:
            if pending:
                emit(bytes(pending))
        finally:
            for connection in connections:
                connection.close()
            selector.close()


def launch_wide(binary: Path, receipt: Path, controller: Path, controller_sha: str,
                binary_sha: str, window_seconds: float, verify: bool) -> int:
    if (receipt.exists() or not receipt.parent.is_dir() or not math.isfinite(window_seconds)
            or not 2 <= window_seconds <= 20 or base.digest(binary, base.MAX_BINARY) != binary_sha
            or base.digest(controller, base.MAX_BINARY) != controller_sha):
        raise ValueError('fresh wide receipt and immutable launch inputs required')
    fixtures = base.fixture_binding()
    bodies = {route: (base.FIXTURES / name).read_bytes() for route, (name, _) in ROUTES.items()}
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [], 'requests': [],
              'accepted_connections': 0, 'identity': None, 'browser_reaped': False, 'browser_returncode': None,
              'binary_sha256': binary_sha, 'controller_sha256': controller_sha, 'fixture_freeze': fixtures['freeze_sha256'],
              'limits': {'header_bytes': MAX_HEADER, 'connections': MAX_CONNECTIONS,
                         'active_connections': MAX_ACTIVE_CONNECTIONS, 'requests': len(ROUTES)}}
    child = None
    listener = None
    deadline = time.monotonic() + window_seconds + 5
    emitted = 0
    def emit(data):
        nonlocal emitted
        keep = min(len(data), base.MAX_OUTPUT - emitted)
        sys.stderr.buffer.write(data[:keep])
        sys.stderr.buffer.flush()
        emitted += keep
        if keep != len(data):
            raise ValueError('wide relay output limit')
    def persist():
        temp = receipt.with_suffix('.tmp')
        temp.write_text(json.dumps(record, indent=2) + '\n')
        temp.replace(receipt)
    persist()
    try:
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(('127.0.0.1', 0))
        listener.listen(MAX_ACTIVE_CONNECTIONS)
        listener.setblocking(False)
        record['port'] = listener.getsockname()[1]
        url = f"http://127.0.0.1:{record['port']}/admitted.html"
        command = [str(binary), url, '--presenter=vulkan', '--raster=gpu', '--no-scripts', '--exit-after', str(window_seconds)]
        if verify:
            command += ['--vulkan-verify-frames', '1']
        record['browser_command'] = command
        persist()
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE, close_fds=True)
        record['identity'] = base.child_identity(child.pid)
        persist()
        base.configure_owned(record['identity'], controller, dict(os.environ), record, persist, requested_size=SIZE)
        serve_and_relay(child, listener, record, bodies, persist, deadline, emit)
        record['browser_returncode'] = child.wait(timeout=2)
        record['browser_reaped'] = True
        record['success'] = record['browser_returncode'] == 0
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        if listener is not None:
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
            if (base.digest(binary, base.MAX_BINARY) != binary_sha or base.fixture_binding() != fixtures
                    or base.digest(controller, base.MAX_BINARY) != controller_sha):
                raise ValueError('wide launch inputs changed')
        except (OSError, ValueError) as error:
            record['success'] = False
            record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def validate_run(stdout: bytes, stderr: bytes, case: str, control: dict) -> dict:
    if stdout or case not in CASES or len(stderr) > base.MAX_OUTPUT or not stderr.endswith(b'\n'):
        raise ValueError('wide case/raw output bound')
    if (not control.get('success') or not control.get('control_complete') or not control.get('browser_reaped')
            or control.get('browser_returncode') != 0 or control.get('controlled_size') != list(SIZE)
            or any(key in control for key in ('error', 'cleanup_error', 'postcheck_error', 'stderr_drain_error'))):
        raise ValueError('wide owned control/cleanup failed')
    lines = stderr.decode('utf-8').splitlines(keepends=True)
    if len(lines) > 4096 or any(len(line) > 16384 for line in lines):
        raise ValueError('wide diagnostic limits')
    scenes, presented, verified, summaries, adapters, fallbacks, other = {}, [], [], [], [], [], []
    release = None
    prefix = bytearray()
    physical = False
    for raw in lines:
        line = raw.rstrip('\n')
        if match := RELEASE.fullmatch(line):
            event = control.get('release', {})
            if (release is not None or not physical or int(match[1]) != control['identity']['pid']
                    or int(match[2]) != len(prefix) or match[3] != sha(prefix)
                    or event != {'prefix_bytes': len(prefix), 'prefix_sha256': sha(prefix),
                                 'html_sha256': match[4], 'physical_size': list(SIZE)}
                    or match[4] != sha((base.FIXTURES / 'admitted.html').read_bytes())):
                raise ValueError('wide release lacked both controlled size and physical loading proof')
            release = event
        elif match := base.SCENE.fullmatch(line):
            serial, generation, commands, images, loading, phases, lowered, rounded, raster, total, reference = match.groups()
            row = dict(zip(('serial', 'snapshot_generation', 'page_commands', 'images', 'phases', 'lowered_commands', 'rounded_masks', 'raster_invocations', 'total_invocations'),
                           map(int, (serial, generation, commands, images, phases, lowered, rounded, raster, total))))
            row.update(loading=loading == 'true', reference=reference == 'true')
            if (release is None or summaries or row['serial'] <= 0 or row['serial'] in scenes
                    or row['snapshot_generation'] != 1 or row['page_commands'] != 6 or row['images'] != 1
                    or row['loading'] or row['phases'] != 3 or not 1 <= row['lowered_commands'] <= 256
                    or row['rounded_masks'] != 3 or row['raster_invocations'] < CONVERSION_WORK
                    or row['total_invocations'] - row['raster_invocations'] != CONVERSION_WORK
                    or row['total_invocations'] > 4_000_000):
                raise ValueError('wide loaded-scene admission/ordering mismatch')
            scenes[row['serial']] = row
        elif match := base.PRESENTED.fullmatch(line):
            width, height, serial, reference = match.groups()
            serial = int(serial)
            if (release is None or summaries or (int(width), int(height)) != SIZE
                    or serial not in scenes or scenes[serial]['reference'] != (reference == 'true')):
                raise ValueError('wide presentation lacked its released scene')
            presented.append({'serial': serial, 'reference': reference == 'true'})
        elif match := base.VERIFIED.fullmatch(line):
            serial, generation, revision, width, height, count = map(int, match.groups())
            if (release is None or summaries or (width, height) != SIZE or count != PIXEL_BYTES
                    or serial not in scenes or generation != scenes[serial]['snapshot_generation']
                    or not any(row['serial'] == serial and row['reference'] for row in presented)):
                raise ValueError('wide acquired verification lacked its presentation')
            verified.append({'serial': serial, 'generation': generation, 'viewport_revision': revision, 'compared_bytes': count})
        elif match := base.SUMMARY.fullmatch(line):
            frames, count, serial = map(int, match.groups())
            if summaries or len(verified) != 1 or verified[0]['serial'] != serial:
                raise ValueError('wide completion lacks verification')
            summaries.append({'frames': frames, 'compared_bytes': count, 'last_serial': serial})
        elif match := base.FALLBACK.fullmatch(line):
            fallbacks.append(match[1])
            physical = match[1] == PHYSICAL
        elif base.ADAPTER.fullmatch(line):
            adapters.append(line)
        elif (line.startswith(('native wide gate:', 'presenter:', 'native scene', 'paint:', 'eris-browser:', 'Unable to open browser window:', "thread '"))
              or 'panicked at' in line):
            raise ValueError('unknown/failure wide diagnostic')
        else:
            other.append(line)
        prefix.extend(raw.encode())
    if release is None or len(adapters) != 1 or len(presented) != 1:
        raise ValueError('wide release/adapter/presentation population mismatch')
    first = presented[0]
    if case == 'wide-verified':
        if (not first['reference'] or len(verified) != 1 or verified[0]['serial'] != first['serial']
                or summaries != [{'frames': 1, 'compared_bytes': PIXEL_BYTES, 'last_serial': first['serial']}]):
            raise ValueError('wide quota must verify exactly the first native frame')
    elif verified or summaries or any(row['reference'] for row in scenes.values()):
        raise ValueError('normal wide frame must not request CPU reference/readback')
    requests = control.get('requests', [])
    if (len(requests) != 2 or [r['path'] for r in requests] != list(ROUTES)
            or not 1 <= control.get('accepted_connections', 0) <= MAX_CONNECTIONS):
        raise ValueError('wide HTTP request inventory mismatch')
    for request in requests:
        body = (base.FIXTURES / ROUTES[request['path']][0]).read_bytes()
        response = response_for(request['path'], {request['path']: body})
        if (request['method'] != 'GET' or not 1 <= request['header_bytes'] <= MAX_HEADER
                or request['body_bytes'] != len(body) or request['body_sha256'] != sha(body)
                or not request['response_complete'] or request['response_bytes'] != len(response)
                or request['response_sha256'] != sha(response)):
            raise ValueError('wide HTTP response did not preserve frozen bytes')
    return {'case': case, 'size': list(SIZE), 'release': release, 'adapter': adapters[0],
            'prepared_scenes': list(scenes.values()), 'native_presentations': presented,
            'acquired_verifications': verified, 'verification_summaries': summaries,
            'acquired_texture_compared_bytes': sum(v['compared_bytes'] for v in verified),
            'fallback_reasons': fallbacks, 'other_diagnostic_lines': other,
            'scope': 'frozen HTML/PNG over a gated owned loopback origin; Canvas differential before compositor; no performance claim'}


def run_host(binary: Path, output: Path, loader: Path, timeout: float = 30, window_seconds: float = 6) -> dict:
    if (not math.isfinite(timeout) or not math.isfinite(window_seconds) or not 2 <= window_seconds <= 20
            or not window_seconds + 10 <= timeout <= 120 or output.exists()
            or output.resolve().is_relative_to(base.FIXTURES.resolve())):
        raise ValueError('fresh output and bounded lifetime required')
    binary, loader = binary.resolve(), loader.resolve()
    env = base.environment(loader)
    found = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not found:
        raise ValueError('owned wide window requires hyprctl')
    controller = Path(found).resolve()
    def bindings():
        return {'binary': base.digest(binary, base.MAX_BINARY), 'controller': base.digest(controller, base.MAX_BINARY),
                'fixtures': base.fixture_binding(), 'base_sources': base.source_binding(),
                'wide_host': base.digest(Path(__file__), 1024 * 1024),
                'drain_source': base.digest(Path(__file__).with_name('native_raster_followup.py'), 1024 * 1024),
                'loader': base.loader_binding(loader, env)}
    frozen = bindings()
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'binary': str(binary), 'controller': str(controller),
              'bindings': frozen, 'runs': [], 'timeout_seconds': timeout, 'window_seconds': window_seconds}
    def persist():
        temp = output / 'wide-host-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n')
        temp.replace(output / 'wide-host-results.json')
    def unchanged():
        if bindings() != frozen:
            raise ValueError('wide host inputs changed')
    persist()
    try:
        for case in CASES:
            unchanged()
            receipt = output / f'{case}.window-control.json'
            command = [sys.executable, str(Path(__file__).resolve()), '--owned-launch', '--binary', str(binary),
                       '--receipt', str(receipt.resolve()), '--controller', str(controller),
                       '--controller-sha256', frozen['controller'], '--binary-sha256', frozen['binary'],
                       '--window-seconds', str(window_seconds), '--allow-experimental-gpu']
            if case == 'wide-verified':
                command.append('--verify')
            record, stdout = base.run_supervised(command, output, case, env, timeout)
            result['runs'].append(record)
            persist()
            if receipt.is_file():
                record['control_sha256'] = base.digest(receipt, 1024 * 1024)
                record['control'] = json.loads(receipt.read_bytes())
                persist()
            base.check_success(record)
            if ((output / f'{case}.supervisor.stderr.log').stat().st_size
                    or record.get('capture_gate_required') or record.get('capture_gate_granted')):
                raise ValueError('unexpected supervisor diagnostics/grant')
            stderr = output / f'{case}.stderr.log'
            if base.digest(stderr, base.MAX_OUTPUT) != record['stderr_sha256'] or sha(stdout) != record['stdout_sha256']:
                raise ValueError('wide raw output changed')
            control = record['control']
            if (control.get('binary_sha256') != frozen['binary'] or control.get('controller_sha256') != frozen['controller']
                    or control.get('fixture_freeze') != frozen['fixtures']['freeze_sha256']):
                raise ValueError('owned wide input identity changed')
            record['validation'] = validate_run(stdout, stderr.read_bytes(), case, control)
            unchanged()
            persist()
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
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
    parser.add_argument('--window-seconds', type=float, default=6)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    if owned:
        parser.add_argument('--receipt', type=Path, required=True)
        parser.add_argument('--controller', type=Path, required=True)
        parser.add_argument('--controller-sha256', required=True)
        parser.add_argument('--binary-sha256', required=True)
        parser.add_argument('--verify', action='store_true')
    else:
        parser.add_argument('--output', type=Path, required=True)
        parser.add_argument('--loader-directory', type=Path, required=True)
        parser.add_argument('--timeout', type=float, default=30)
    args = parser.parse_args(sys.argv[2:] if owned else None)
    if not args.allow_experimental_gpu:
        parser.error('actual wide window execution requires --allow-experimental-gpu')
    def interrupted(_sig, _frame):
        raise KeyboardInterrupt
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    if owned:
        return launch_wide(args.binary, args.receipt, args.controller, args.controller_sha256,
                           args.binary_sha256, args.window_seconds, args.verify)
    result = run_host(args.binary, args.output, args.loader_directory, args.timeout, args.window_seconds)
    print(json.dumps({'success': result['success'], 'results': str(args.output / 'wide-host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
