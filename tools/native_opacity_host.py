#!/usr/bin/env python3
"""Two bounded owned-window opacity checks using the existing native host."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import subprocess
import sys
import time

import native_raster_host as base
import native_raster_wide_host as wide
from native_raster_followup import drain_stderr

ROOT = Path(__file__).resolve().parents[1]
HTML = ROOT / 'examples/vulkan-opacity.html'
PNG = ROOT / 'examples/vulkan-opacity.png'
SIZE = wide.SIZE
PIXEL_BYTES = 1280 * 880 * 4
QUOTA = 4
SCRATCH = 262144
CASES = ('opacity-click-verified', 'opacity-normal')
OPACITY = re.compile(r'native opacity prepared: serial=([1-9][0-9]*) groups=([0-9]+) scratch_bytes=([0-9]+) opacity_sum=([0-9]+)')
LOADING = re.compile(r'native scene awaits a current loaded page; size=([1-9][0-9]*)x([1-9][0-9]*)')
WORKER = re.compile(r'\[page\] (?:Page process [1-9][0-9]*: Landlock ABI 6 and seccomp: no direct resource or socket access|Resource broker [1-9][0-9]*: committed URL and fetch policy enforced outside renderer)')


def sha(data):
    return hashlib.sha256(data).hexdigest()


@contextmanager
def fixture_routes():
    # Local to this helper process. The inherited server still serves exactly
    # two fixed GET routes, with its original gate, bounds and cleanup.
    original = wide.ROUTES
    wide.ROUTES = {'/admitted.html': ('vulkan-opacity.html', 'text/html; charset=utf-8'),
                   '/vulkan-opacity.png': ('vulkan-opacity.png', 'image/png')}
    try:
        yield
    finally:
        wide.ROUTES = original


class Observation:
    """Pure bounded log classification; no browser/controller operation."""
    def __init__(self, verified):
        self.wants_reference = verified
        self.scenes, self.groups = {}, {}
        self.presented, self.verified, self.summaries, self.adapters = [], [], [], []
        self.releases = []

    def line(self, text):
        if match := base.SCENE.fullmatch(text):
            serial, generation, commands, images, loading, phases, lowered, rounded, raster, total, reference = match.groups()
            serial, generation, commands, images, phases, lowered, rounded, raster, total = map(
                int, (serial, generation, commands, images, phases, lowered, rounded, raster, total))
            if (self.summaries or serial <= max(self.scenes, default=0) or generation != 1
                    or not self.releases or loading != 'false' or images != 1 or phases != 3
                    or not 1 <= commands <= 256 or not 1 <= lowered <= 256
                    or not 1 <= rounded <= lowered or raster < wide.CONVERSION_WORK
                    or total - raster != wide.CONVERSION_WORK or total > 4_000_000
                    or (reference == 'true' and not self.wants_reference)
                    or (reference != 'true' and self.wants_reference and len(self.verified) < QUOTA)):
                raise ValueError('opacity scene identity/admission/reference mismatch')
            self.scenes[serial] = {'generation': generation, 'reference': reference == 'true'}
        elif match := OPACITY.fullmatch(text):
            serial, groups, scratch, numerator = map(int, match.groups())
            if (self.summaries or serial not in self.scenes or serial in self.groups
                    or groups != 2 or scratch != SCRATCH or numerator not in (256, 320)
                    or (not self.wants_reference and numerator != 256)):
                raise ValueError('opacity group route or phase mismatch')
            self.groups[serial] = numerator
        elif match := base.PRESENTED.fullmatch(text):
            width, height, serial, reference = match.groups()
            serial = int(serial)
            if (self.summaries or self.presented or (int(width), int(height)) != SIZE
                    or self.groups.get(serial) != 256
                    or (reference == 'true') != self.wants_reference):
                raise ValueError('initial native opacity presentation mismatch')
            self.presented.append(serial)
        elif match := base.VERIFIED.fullmatch(text):
            serial, generation, revision, width, height, count = map(int, match.groups())
            scene = self.scenes.get(serial)
            if (not self.wants_reference or self.summaries or len(self.verified) >= QUOTA
                    or not scene or not scene['reference'] or self.groups.get(serial) not in (256, 320)
                    or generation != scene['generation'] or revision <= 0
                    or (width, height) != SIZE or count != PIXEL_BYTES or not self.presented
                    or (self.verified and revision != self.verified[0]['viewport_revision'])
                    or (self.verified and serial <= self.verified[-1]['serial'])):
                raise ValueError('acquired opacity verification identity mismatch')
            self.verified.append({'serial': serial, 'generation': generation,
                                  'viewport_revision': revision, 'opacity_sum': self.groups[serial],
                                  'compared_bytes': count})
            return self.verified[-1]
        elif match := base.SUMMARY.fullmatch(text):
            frames, count, serial = map(int, match.groups())
            if (not self.wants_reference or self.summaries or len(self.verified) != QUOTA
                    or (frames, count, serial) != (QUOTA, QUOTA * PIXEL_BYTES, self.verified[-1]['serial'])):
                raise ValueError('opacity verification completion mismatch')
            self.summaries.append({'frames': frames, 'compared_bytes': count, 'last_serial': serial})
        elif match := wide.RELEASE.fullmatch(text):
            if self.releases or self.scenes:
                raise ValueError('opacity document release order')
            self.releases.append(match.groups())
        elif base.ADAPTER.fullmatch(text):
            self.adapters.append(text)
        elif match := base.FALLBACK.fullmatch(text):
            loading = LOADING.fullmatch(match[1])
            if (self.scenes or loading is None
                    or any(int(value) > 16384 for value in loading.groups())
                    or (self.releases and match[1] != wide.PHYSICAL)):
                raise ValueError('unexpected whole-CPU opacity fallback')
        elif WORKER.fullmatch(text):
            pass  # Retained ordinary confinement diagnostics, not pixel proof.
        elif (text.startswith(('presenter:', 'native scene', 'native opacity', 'native zoom', 'native wide gate:',
                              'paint:', 'eris-browser:', '[page]', 'Unable to open browser window:', "thread '"))
              or 'panicked at' in text):
            raise ValueError('unknown/failure opacity diagnostic')
        return None

    def complete(self):
        if len(self.adapters) != 1 or len(self.releases) != 1 or len(self.presented) != 1:
            raise ValueError('missing unique adapter/release/native presentation')
        if set(self.scenes) != set(self.groups):
            raise ValueError('every prepared scene requires matching group diagnostics')
        if self.wants_reference:
            phases = [row['opacity_sum'] for row in self.verified]
            if (len(phases) != QUOTA or len(self.summaries) != 1 or phases[0] != 256
                    or 320 not in phases or 256 in phases[phases.index(320):]):
                raise ValueError('four exact frames must include ordered before/after click states')
        elif self.verified or self.summaries:
            raise ValueError('normal opacity run cannot contain CPU-reference verification')
        return {'prepared_serials': list(self.scenes), 'opacity_sum_by_serial': self.groups,
                'presented_serials': self.presented, 'verified': self.verified,
                'compared_bytes': sum(row['compared_bytes'] for row in self.verified),
                'summary': self.summaries, 'scratch_bytes': SCRATCH,
                'scope': ('acquired texture before compositor; two opacity states among four serials, no performance claim'
                          if self.wants_reference else
                          'native presentation without CPU reference; initial opacity state, no performance claim')}


def control(identity, controller, record, persist, args):
    base.ensure_live_child(identity)
    if len(record['commands']) >= base.CONTROL_COMMANDS:
        raise ValueError('inherited controller command bound')
    result = base.controller_call(controller, args, dict(os.environ), 0.5)
    row = {'argv': [str(controller), *args], 'status': result.record['status'],
           'returncode': result.record['returncode'], 'stdout_bytes': len(result.stdout),
           'stdout_sha256': sha(result.stdout), 'stderr_bytes': len(result.stderr),
           'stderr_sha256': sha(result.stderr)}
    query = args == ['-j', 'clients']
    if not query and len(result.stdout) <= 4096:
        row['stdout'] = result.stdout.decode('utf-8', errors='strict')
    if len(result.stderr) <= 4096:
        row['stderr'] = result.stderr.decode('utf-8', errors='strict')
    record['commands'].append(row)
    persist()
    if (row['status'] != 'exited' or row['returncode'] != 0 or result.stderr
            or (not query and result.stdout.strip() != b'ok')):
        raise ValueError('owned opacity controller failed')
    base.ensure_live_child(identity)
    return result.stdout, row


def own_geometry(identity, controller, record, persist):
    raw, row = control(identity, controller, record, persist, ['-j', 'clients'])
    clients = json.loads(raw)
    if not isinstance(clients, list) or len(clients) > 256 or any(not isinstance(c, dict) for c in clients):
        raise ValueError('bounded compositor client list required')
    matches = [c for c in clients if type(c.get('pid')) is int and c['pid'] == identity['pid']]
    if len(matches) != 1:
        raise ValueError('one owned mapped client required')
    client = matches[0]
    at = client.get('at')
    if (client.get('size') != list(SIZE) or client.get('mapped') is not True
            or client.get('floating') is not True or not isinstance(at, list) or len(at) != 2
            or any(type(v) is not int or not -32768 <= v <= 32768 for v in at)):
        raise ValueError('owned client geometry mismatch')
    row['owned_client'] = {'pid': identity['pid'], 'at': at, 'size': list(SIZE), 'mapped': True, 'floating': True}
    persist()
    return row['owned_client']


def launch(binary, binary_sha, controller, controller_sha, receipt, seconds, verified):
    if (receipt.exists() or not receipt.parent.is_dir() or not 2 <= seconds <= 20
            or not math.isfinite(seconds) or base.digest(binary, base.MAX_BINARY) != binary_sha
            or base.digest(controller, base.MAX_BINARY) != controller_sha):
        raise ValueError('fresh receipt and immutable owned launch inputs required')
    bodies = {'/admitted.html': HTML.read_bytes(), '/vulkan-opacity.png': PNG.read_bytes()}
    if any(len(body) > 65536 for body in bodies.values()):
        raise ValueError('fixed opacity fixture byte bound')
    fixture_hashes = {path: sha(body) for path, body in bodies.items()}
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [], 'requests': [],
              'accepted_connections': 0, 'identity': None, 'browser_reaped': False, 'browser_returncode': None,
              'binary_sha256': binary_sha, 'controller_sha256': controller_sha,
              'fixtures': fixture_hashes, 'escapes': [], 'verified_mode': verified}
    child, listener = None, None
    retained = bytearray()
    observation = Observation(verified)
    deadline = time.monotonic() + seconds + 5

    def persist():
        temporary = receipt.with_suffix('.tmp')
        temporary.write_text(json.dumps(record, indent=2) + '\n')
        temporary.replace(receipt)

    def event(kind, checked, key):
        if time.monotonic() >= deadline:
            raise ValueError('original opacity control deadline expired')
        before = {'after_serial': checked['serial'], 'prefix_bytes': len(retained),
                  'prefix_sha256': sha(retained), 'dispatch_complete': False}
        if kind == 'click':
            geometry = own_geometry(record['identity'], controller, record, persist)
            if geometry != record['click_geometry']:
                raise ValueError('owned window moved before click')
            record['click'] = before
        else:
            if len(record['escapes']) >= QUOTA - 1:
                raise ValueError('opacity redraw bound')
            record['escapes'].append(before)
        persist()
        _, command = control(record['identity'], controller, record, persist,
                             ['dispatch', 'sendshortcut', f',{key},pid:{child.pid}'])
        before.update(command=command, dispatch_complete=True)
        persist()

    def emit(data):
        if len(retained) + len(data) > base.MAX_OUTPUT:
            raise ValueError('inherited opacity relay output bound')
        retained.extend(data)
        sys.stderr.buffer.write(data)
        sys.stderr.buffer.flush()
        if not data.endswith(b'\n'):
            return  # Evidence-only teardown bytes still retained.
        for line in data.decode('utf-8', errors='strict').splitlines():
            checked = observation.line(line)
            if checked and verified:
                if 'click' not in record:
                    if checked['opacity_sum'] != 256:
                        raise ValueError('click prerequisite must be initial verified group scene')
                    event('click', checked, 'mouse:272')
                elif checked['opacity_sum'] == 320 and len(observation.verified) < QUOTA:
                    event('escape', checked, 'Escape')

    persist()
    try:
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(('127.0.0.1', 0))
        listener.listen(wide.MAX_ACTIVE_CONNECTIONS)
        listener.setblocking(False)
        record['port'] = listener.getsockname()[1]
        command = [str(binary), f'http://127.0.0.1:{record["port"]}/admitted.html',
                   '--presenter=vulkan', '--raster=gpu', '--exit-after', str(seconds)]
        if verified:
            command += ['--vulkan-verify-frames', str(QUOTA)]
        record['browser_command'] = command
        persist()
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE, close_fds=True)
        record['identity'] = base.child_identity(child.pid)
        persist()
        base.configure_owned(record['identity'], controller, dict(os.environ), record, persist, requested_size=SIZE)
        record['control_complete'] = False
        persist()
        geometry = own_geometry(record['identity'], controller, record, persist)
        control(record['identity'], controller, record, persist, ['dispatch', 'focuswindow', f'pid:{child.pid}'])
        x, y = geometry['at']
        control(record['identity'], controller, record, persist, ['dispatch', 'movecursor', f'{x + 20} {y + 96}'])
        record['click_geometry'] = own_geometry(record['identity'], controller, record, persist)
        if record['click_geometry'] != geometry:
            raise ValueError('owned window geometry changed during pointer preparation')
        record['control_complete'] = True
        persist()
        with fixture_routes():
            wide.serve_and_relay(child, listener, record, bodies, persist, deadline, emit,
                                 forbidden_patterns=(OPACITY,))
        record['browser_returncode'] = child.wait(timeout=2)
        record['browser_reaped'] = True
        record['observations'] = observation.complete()
        if verified and not record.get('click', {}).get('dispatch_complete'):
            raise ValueError('verified run lacks actual owned mouse click')
        record['success'] = record['browser_returncode'] == 0
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, subprocess.SubprocessError, KeyboardInterrupt) as error:
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
                # Do not trigger new control operations during teardown.
                def drain(data):
                    if len(retained) + len(data) > base.MAX_OUTPUT:
                        raise ValueError('opacity teardown output bound')
                    retained.extend(data)
                    sys.stderr.buffer.write(data)
                    sys.stderr.buffer.flush()
                drain_stderr(child.stderr, drain)
            except (OSError, ValueError) as error:
                record['success'] = False
                record['stderr_drain_error'] = str(error)[:4096]
            child.stderr.close()
        try:
            if (base.digest(binary, base.MAX_BINARY) != binary_sha
                    or base.digest(controller, base.MAX_BINARY) != controller_sha
                    or {path: sha(file.read_bytes()) for path, file in
                        [('/admitted.html', HTML), ('/vulkan-opacity.png', PNG)]} != fixture_hashes):
                raise ValueError('opacity launch input changed')
        except (OSError, ValueError) as error:
            record['success'] = False
            record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def validate(stdout, stderr, verified, record, controller):
    if stdout or len(stderr) > base.MAX_OUTPUT or not stderr.endswith(b'\n'):
        raise ValueError('opacity raw output framing/bound')
    if (not record.get('success') or not record.get('control_complete') or not record.get('browser_reaped')
            or record.get('browser_returncode') != 0 or record.get('controlled_size') != list(SIZE)
            or record.get('verified_mode') is not verified
            or any(key in record for key in ('error', 'cleanup_error', 'postcheck_error', 'stderr_drain_error'))):
        raise ValueError('opacity owned control/cleanup failed')
    state = Observation(verified)
    offset = 0
    physical = False
    lines = stderr.decode('utf-8', errors='strict').splitlines(keepends=True)
    if len(lines) > 4096 or any(len(line) > 16384 for line in lines):
        raise ValueError('inherited opacity diagnostic bounds')
    for raw in lines:
        text = raw.rstrip('\n')
        if match := base.FALLBACK.fullmatch(text):
            physical = match[1] == wide.PHYSICAL
        if match := wide.RELEASE.fullmatch(text):
            if (not physical or int(match[1]) != record['identity']['pid'] or int(match[2]) != offset
                    or match[3] != sha(stderr[:offset]) or match[4] != sha(HTML.read_bytes())):
                raise ValueError('delayed opacity document release binding')
            if record.get('release') != {'prefix_bytes': offset, 'prefix_sha256': match[3],
                                        'html_sha256': match[4], 'physical_size': list(SIZE)}:
                raise ValueError('retained opacity release receipt mismatch')
        state.line(text)
        offset += len(raw.encode())
    checked = state.complete()
    if checked != record.get('observations'):
        # JSON object keys turn integer serials into strings in retained records.
        if json.loads(json.dumps(checked)) != record.get('observations'):
            raise ValueError('live and retained opacity observations differ')
    events = ([record.get('click', {})] + record.get('escapes', [])) if verified else []
    shortcuts = [row for row in record['commands'] if row.get('argv', [])[1:3] == ['dispatch', 'sendshortcut']]
    if len(record['commands']) > base.CONTROL_COMMANDS or len(shortcuts) != len(events):
        raise ValueError('exact owned input command population required')
    if not verified and ('click' in record or record.get('escapes')):
        raise ValueError('normal reference-free run must have no synthetic redraws/click')
    previous_prefix = 0
    for index, event in enumerate(events):
        key = 'mouse:272' if index == 0 else 'Escape'
        prefix = event.get('prefix_bytes')
        command = event.get('command', {})
        if (type(prefix) is not int or not previous_prefix < prefix <= len(stderr)
                or event.get('prefix_sha256') != sha(stderr[:prefix])
                or not event.get('dispatch_complete')
                or command.get('argv') != [str(controller), 'dispatch', 'sendshortcut', f',{key},pid:{record["identity"]["pid"]}']
                or command.get('status') != 'exited' or command.get('returncode') != 0
                or command.get('stdout', '').strip() != 'ok' or command.get('stderr') != ''
                or command.get('stdout_bytes') != len(command.get('stdout', '').encode())
                or command.get('stdout_sha256') != sha(command.get('stdout', '').encode())
                or command.get('stderr_bytes') != 0 or command.get('stderr_sha256') != sha(b'')
                or record['commands'].count(command) != 1):
            raise ValueError('owned input event binding')
        last = base.VERIFIED.fullmatch(stderr[:prefix].decode().splitlines()[-1])
        if not last or int(last[1]) != event.get('after_serial'):
            raise ValueError('input was not gated by its retained verified frame')
        expected = 256 if index == 0 else 320
        if state.groups.get(event['after_serial']) != expected:
            raise ValueError('input prerequisite belongs to the wrong opacity state')
        if index == 0:
            if event['after_serial'] != state.verified[0]['serial']:
                raise ValueError('click must follow first acquired verification')
            before = Observation(True)
            for line in stderr[:prefix].decode().splitlines():
                before.line(line)
            if any(value != 256 for value in before.groups.values()):
                raise ValueError('changed opacity appeared before actual click')
        previous_prefix = prefix
    bodies = {'/admitted.html': HTML.read_bytes(), '/vulkan-opacity.png': PNG.read_bytes()}
    requests = record.get('requests', [])
    if (record.get('fixtures') != {path: sha(body) for path, body in bodies.items()}
            or len(requests) != 2 or [row.get('path') for row in requests] != list(bodies)
            or not 1 <= record.get('accepted_connections', 0) <= wide.MAX_CONNECTIONS):
        raise ValueError('opacity fixed HTTP input inventory mismatch')
    with fixture_routes():
        for request in requests:
            body = bodies[request['path']]
            response = wide.response_for(request['path'], bodies)
            if (request.get('method') != 'GET' or not 1 <= request.get('header_bytes', 0) <= wide.MAX_HEADER
                    or request.get('body_bytes') != len(body) or request.get('body_sha256') != sha(body)
                    or not request.get('response_complete') or request.get('response_bytes') != len(response)
                    or request.get('response_sha256') != sha(response)):
                raise ValueError('opacity HTTP response bytes/completion mismatch')
    return checked


def run_host(binary, binary_sha, output, loader, timeout, seconds):
    if (output.exists() or output.is_symlink() or not math.isfinite(seconds) or not 2 <= seconds <= 20
            or not math.isfinite(timeout) or not seconds + 10 <= timeout <= 120):
        raise ValueError('fresh output and inherited bounded lifetimes required')
    binary, loader = binary.resolve(), loader.resolve()
    env = base.environment(loader)
    located = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not located:
        raise ValueError('owned opacity check requires hyprctl')
    controller = Path(located).resolve()
    def bindings():
        return {'binary': base.digest(binary, base.MAX_BINARY), 'controller': base.digest(controller, base.MAX_BINARY),
                'base_sources': base.source_binding(), 'loader': base.loader_binding(loader, env),
                'files': {str(p): base.digest(p, 1024 * 1024) for p in
                          (Path(__file__), Path(wide.__file__), ROOT / 'tools/native_raster_followup.py', HTML, PNG)}}
    frozen = bindings()
    if frozen['binary'] != binary_sha:
        raise ValueError('explicit frozen opacity binary mismatch')
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'bindings': frozen, 'runs': [],
              'scope': 'one actual-click four-frame Canvas differential, one separate reference-free native run; before compositor'}
    def persist():
        temporary = output / 'host-results.json.tmp'
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(output / 'host-results.json')
    persist()
    try:
        for case in CASES:
            if bindings() != frozen:
                raise ValueError('opacity input drift before run')
            verified = case == CASES[0]
            receipt = output / f'{case}.window-control.json'
            command = [sys.executable, str(Path(__file__).resolve()), '--owned-launch', '--binary', str(binary),
                       '--binary-sha256', binary_sha, '--controller', str(controller),
                       '--controller-sha256', frozen['controller'], '--receipt', str(receipt.resolve()),
                       '--window-seconds', str(seconds), '--allow-experimental-gpu']
            if verified:
                command.append('--verified')
            run, stdout = base.run_supervised(command, output, case, env, timeout)
            result['runs'].append(run)
            persist()
            base.check_success(run)
            run['control_sha256'] = base.digest(receipt, 1024 * 1024)
            control_record = json.loads(receipt.read_bytes())
            if (control_record.get('binary_sha256') != binary_sha
                    or control_record.get('controller_sha256') != frozen['controller']
                    or (output / f'{case}.supervisor.stderr.log').stat().st_size
                    or run.get('capture_gate_required') or run.get('capture_gate_granted')):
                raise ValueError('opacity process/source supervision mismatch')
            stderr_path = output / f'{case}.stderr.log'
            if base.digest(stderr_path, base.MAX_OUTPUT) != run['stderr_sha256'] or sha(stdout) != run['stdout_sha256']:
                raise ValueError('raw opacity output changed')
            run['validation'] = validate(stdout, stderr_path.read_bytes(), verified, control_record, controller)
            if bindings() != frozen:
                raise ValueError('opacity input drift after run')
            persist()
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        try:
            if bindings() != frozen:
                raise ValueError('opacity final input drift')
        except (OSError, ValueError, RuntimeError) as error:
            result['success'] = False
            result['postcheck_error'] = str(error)[:4096]
        persist()
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--owned-launch', action='store_true')
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=30)
    parser.add_argument('--window-seconds', type=float, default=10)
    parser.add_argument('--controller', type=Path)
    parser.add_argument('--controller-sha256')
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--verified', action='store_true')
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    args = parser.parse_args()
    if not args.allow_experimental_gpu:
        parser.error('actual window/worker/Vulkan execution requires --allow-experimental-gpu')
    def interrupted(_signal, _frame):
        raise KeyboardInterrupt
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    if args.owned_launch:
        if args.controller is None or args.controller_sha256 is None or args.receipt is None:
            parser.error('owned launch needs explicit controller/hash and receipt')
        return launch(args.binary, args.binary_sha256, args.controller, args.controller_sha256,
                      args.receipt, args.window_seconds, args.verified)
    if args.output is None or args.loader_directory is None:
        parser.error('host requires fresh output and explicit loader directory')
    result = run_host(args.binary, args.binary_sha256, args.output, args.loader_directory,
                      args.timeout, args.window_seconds)
    print(json.dumps({'success': result['success'], 'completed_cases': len(result['runs']),
                      'results': str(args.output / 'host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
