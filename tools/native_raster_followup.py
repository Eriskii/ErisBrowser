#!/usr/bin/env python3
"""Two explicit native-window follow-ups; no changes to the passing base suite."""
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
import subprocess
import sys
import time

import native_raster_host as base

CASES = ('reload-after-native', 'opacity-verification-refused')
PAGE = re.compile(r'\[page\] Page process ([1-9][0-9]*): Landlock ABI ([1-9][0-9]*) and seccomp: no direct resource or socket access')
BROKER = re.compile(r'\[page\] Resource broker ([1-9][0-9]*): committed URL and fetch policy enforced outside renderer')
MARKER = re.compile(r'native follow-up: reload requested for owned pid=([1-9][0-9]*) after serial=([1-9][0-9]*)')
INCOMPLETE = b'eris-browser: Vulkan verification incomplete: 0/1 frames\n'


def identities(data: bytes) -> tuple[set[int], set[int]]:
    lines = data.decode('utf-8', errors='strict').splitlines()
    return ({int(m[1]) for line in lines if (m := PAGE.fullmatch(line))},
            {int(m[1]) for line in lines if (m := BROKER.fullmatch(line))})


def reload_prefix(data: bytes) -> dict:
    checked = base.validate_run(b'', data, 'admitted-normal')
    presentation = checked['native_presentations'][0]
    scenes = {row['serial']: row for row in checked['prepared_scenes']}
    pages, brokers = identities(data)
    if (scenes[presentation['serial']]['snapshot_generation'] != 1
            or len(pages) != 1 or len(brokers) != 1 or pages & brokers):
        raise ValueError('reload needs one confined page/broker and generation 1 before native presentation')
    return {'serial': presentation['serial'], 'page_pid': next(iter(pages)),
            'broker_pid': next(iter(brokers)), 'prefix_bytes': len(data),
            'prefix_sha256': hashlib.sha256(data).hexdigest()}


def validate_followup(stdout: bytes, stderr: bytes, case: str, control: dict) -> dict:
    if stdout or case not in CASES or len(stderr) > base.MAX_OUTPUT:
        raise ValueError('follow-up case/output bounds')
    if (not control.get('control_complete') or not control.get('browser_reaped')
            or control.get('controlled_size') != list(base.SIZE)
            or any(k in control for k in ('error', 'cleanup_error', 'postcheck_error', 'stderr_drain_error'))):
        raise ValueError('owned-window control/cleanup was not completed')
    if case == 'opacity-verification-refused':
        if (control.get('browser_returncode') != 1 or control.get('success')
                or stderr.count(INCOMPLETE) != 1):
            raise ValueError('opacity must explicitly fail the 0/1 native verification quota')
        # Exactly one recognized terminal error is removed for the existing
        # strict no-native-record fallback validator; every other byte remains.
        if not stderr.endswith(INCOMPLETE):
            raise ValueError('incomplete verification must be the terminal diagnostic')
        checked = base.validate_run(stdout, stderr[:-len(INCOMPLETE)], 'opacity-fallback')
        return {'case': case, 'expected_browser_exit': 1, 'native_verified_frames': 0,
                'incomplete_quota': '0/1', 'validation': checked}
    if not control.get('success') or control.get('browser_returncode') != 0:
        raise ValueError('reload browser must exit successfully')
    lines = stderr.splitlines(keepends=True)
    markers = [(i, MARKER.fullmatch(line.decode('utf-8').rstrip('\n'))) for i, line in enumerate(lines)
               if line.startswith(b'native follow-up:')]
    if len(markers) != 1 or not markers[0][1]:
        raise ValueError('exactly one recognized reload marker required')
    at, marker = markers[0]
    before, after = b''.join(lines[:at]), b''.join(lines[at + 1:])
    prefix = reload_prefix(before)
    event = control.get('reload', {})
    if (int(marker[1]) != control['identity']['pid'] or int(marker[2]) != prefix['serial']
            or any(event.get(key) != value for key, value in prefix.items())
            or not event.get('dispatch_complete')):
        raise ValueError('reload marker and owned dispatch receipt disagree')
    checked = base.validate_run(stdout, before + after, 'admitted-normal')
    old_pages, old_brokers = identities(before)
    new_pages, new_brokers = identities(after)
    fresh_pages, fresh_brokers = new_pages - old_pages, new_brokers - old_brokers
    if (len(fresh_pages) != 1 or len(fresh_brokers) != 1
            or fresh_pages & (old_pages | old_brokers | fresh_brokers)
            or fresh_brokers & (old_pages | old_brokers)):
        raise ValueError('reload must create a distinct confined page and broker after native presentation')
    generation2 = []
    page_seen = broker_seen = False
    for line in after.decode().splitlines():
        if match := PAGE.fullmatch(line):
            page_seen |= int(match[1]) in fresh_pages
        if match := BROKER.fullmatch(line):
            broker_seen |= int(match[1]) in fresh_brokers
        if match := base.SCENE.fullmatch(line):
            if int(match[2]) == 2:
                if not page_seen or not broker_seen or int(match[1]) <= prefix['serial']:
                    raise ValueError('generation 2 scene must follow fresh worker/broker diagnostics')
                generation2.append(int(match[1]))
            elif int(match[2]) != 1:
                raise ValueError('unexpected navigation generation')
    if not generation2:
        raise ValueError('missing loaded generation 2 native preparation')
    return {'case': case, 'expected_browser_exit': 0, 'initial': prefix,
            'reloaded_page_pid': next(iter(fresh_pages)), 'reloaded_broker_pid': next(iter(fresh_brokers)),
            'generation_2_prepared_serials': generation2, 'validation': checked,
            'scope': 'new confined worker after actual native presentation; generation 2 preparation, no separate second-presentation or pixel-verification claim'}


def request_reload(identity: dict, controller: Path, env: dict, record: dict,
                   persist, prefix: bytes, emit) -> None:
    event = reload_prefix(prefix)
    if 'reload' in record:
        raise ValueError('reload already requested')
    base.ensure_live_child(identity)
    record['reload'] = event
    marker = f"native follow-up: reload requested for owned pid={identity['pid']} after serial={event['serial']}\n".encode()
    emit(marker)
    persist()
    args = ['dispatch', 'sendshortcut', f"CTRL,r,pid:{identity['pid']}"]
    result = base.controller_call(controller, args, env, 0.5)
    event['command'] = {'argv': [str(controller), *args], 'status': result.record['status'],
                        'returncode': result.record['returncode'],
                        'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
                        'stderr_sha256': hashlib.sha256(result.stderr).hexdigest(),
                        'stdout_bytes': len(result.stdout), 'stderr_bytes': len(result.stderr)}
    if len(result.stdout) <= 4096 and len(result.stderr) <= 4096:
        event['command'].update(stdout=result.stdout.decode('utf-8'), stderr=result.stderr.decode('utf-8'))
    persist()
    if (result.record['status'] != 'exited' or result.record['returncode'] != 0
            or result.stdout.strip() != b'ok' or result.stderr):
        raise ValueError('owned reload dispatch failed')
    base.ensure_live_child(identity)
    event['dispatch_complete'] = True
    persist()


def relay_until_exit(child, controller: Path, record: dict, persist, deadline: float, emit) -> None:
    """Relay exact browser bytes; only a separately labeled marker is added."""
    stream = child.stderr
    os.set_blocking(stream.fileno(), False)
    retained = bytearray()
    pending = bytearray()
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(stream, selectors.EVENT_READ)
            while selector.get_map():
                if time.monotonic() >= deadline:
                    raise ValueError('original owned-browser deadline expired')
                for key, _ in selector.select(0.025):
                    chunk = os.read(key.fd, 16384)
                    if not chunk:
                        selector.unregister(stream)
                        if pending:
                            raise ValueError('unterminated browser diagnostic')
                        continue
                    if len(retained) + len(pending) + len(chunk) > base.MAX_OUTPUT - 256:
                        room = max(0, base.MAX_OUTPUT - 256 - len(retained) - len(pending))
                        pending.extend(chunk[:room])
                        raise ValueError('bounded browser stderr relay exceeded')
                    pending.extend(chunk)
                    while (end := pending.find(b'\n')) != -1:
                        line = bytes(pending[:end + 1])
                        del pending[:end + 1]
                        retained.extend(line)
                        emit(line)
                        if base.PRESENTED.fullmatch(line.decode('utf-8').rstrip('\n')) and 'reload' not in record:
                            request_reload(record['identity'], controller, dict(os.environ), record,
                                           persist, bytes(retained), emit)
                    if len(pending) > 16384:
                        raise ValueError('browser diagnostic line exceeded bound')
    finally:
        # Preserve bytes already removed from the pipe even on parse/controller
        # failure. Teardown drains the remaining pipe without interpreting it.
        if pending:
            emit(bytes(pending))
            pending.clear()
    if 'reload' not in record:
        raise ValueError('browser never presented a native frame before reload deadline')
    while os.waitid(os.P_PID, child.pid, base.WAIT_FLAGS) is None:
        if time.monotonic() >= deadline:
            raise ValueError('original owned-browser exit deadline expired')
        time.sleep(0.025)



def drain_stderr(stream, emit, seconds: float = 2.0) -> None:
    """Bounded evidence-only drain; never parses or triggers compositor actions."""
    deadline = time.monotonic() + seconds
    os.set_blocking(stream.fileno(), False)
    with selectors.DefaultSelector() as selector:
        selector.register(stream, selectors.EVENT_READ)
        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ValueError('bounded stderr teardown drain did not reach EOF')
            for key, _ in selector.select(min(remaining, 0.025)):
                chunk = os.read(key.fd, 16384)
                if not chunk:
                    selector.unregister(stream)
                else:
                    emit(chunk)


def reload_launch(command: list[str], receipt: Path, controller: Path, controller_sha: str,
                  binary_sha: str, window_seconds: float) -> int:
    if (receipt.exists() or not receipt.parent.is_dir() or not command
            or not math.isfinite(window_seconds) or not 2 <= window_seconds <= 20
            or base.digest(controller, base.MAX_BINARY) != controller_sha
            or base.digest(Path(command[0]), base.MAX_BINARY) != binary_sha):
        raise ValueError('fresh reload receipt and immutable explicit inputs required')
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [],
              'browser_command': command, 'binary_sha256': binary_sha, 'controller_sha256': controller_sha,
              'identity': None, 'browser_returncode': None, 'browser_reaped': False}
    child = None
    deadline = time.monotonic() + window_seconds + 5

    def persist():
        temporary = receipt.with_suffix('.tmp')
        temporary.write_text(json.dumps(record, indent=2) + '\n')
        temporary.replace(receipt)

    emitted_bytes = 0

    def emit(data):
        nonlocal emitted_bytes
        retained = min(len(data), base.MAX_OUTPUT - emitted_bytes)
        sys.stderr.buffer.write(data[:retained])
        sys.stderr.buffer.flush()
        emitted_bytes += retained
        if retained != len(data):
            raise ValueError('bounded relay output exceeded')

    persist()
    try:
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE, close_fds=True)
        record['identity'] = base.child_identity(child.pid)
        persist()
        base.configure_owned(record['identity'], controller, dict(os.environ), record, persist)
        relay_until_exit(child, controller, record, persist, deadline, emit)
        record['browser_returncode'] = child.wait(timeout=2)
        record['browser_reaped'] = True
        record['success'] = record['browser_returncode'] == 0 and record['reload']['dispatch_complete']
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
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
            if (base.digest(controller, base.MAX_BINARY) != controller_sha
                    or base.digest(Path(command[0]), base.MAX_BINARY) != binary_sha):
                raise ValueError('reload launch inputs changed')
        except (OSError, ValueError) as error:
            record['success'] = False
            record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def run_followups(binary: Path, output: Path, loader: Path, timeout: float = 30,
                  window_seconds: float = 6) -> dict:
    if (not math.isfinite(timeout) or not math.isfinite(window_seconds)
            or not 2 <= window_seconds <= 20 or not window_seconds + 10 <= timeout <= 120
            or output.exists() or output.resolve().is_relative_to(base.FIXTURES.resolve())):
        raise ValueError('fresh output and unchanged bounded deadlines required')
    binary, loader = binary.resolve(), loader.resolve()
    env = base.environment(loader)
    found = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not found:
        raise ValueError('explicit owned-window follow-ups require hyprctl')
    controller = Path(found).resolve()
    def bindings():
        return {'binary': base.digest(binary, base.MAX_BINARY), 'controller': base.digest(controller, base.MAX_BINARY),
                'fixtures': base.fixture_binding(), 'base_sources': base.source_binding(),
                'followup_source': base.digest(Path(__file__), 1024 * 1024),
                'loader': base.loader_binding(loader, env)}
    frozen = bindings()
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'bindings': frozen, 'binary': str(binary),
              'controller': str(controller), 'timeout_seconds': timeout, 'window_seconds': window_seconds, 'runs': []}
    def persist():
        temp = output / 'followup-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n')
        temp.replace(output / 'followup-results.json')
    def unchanged():
        if bindings() != frozen:
            raise ValueError('follow-up immutable inputs changed')
    persist()
    try:
        for case in CASES:
            unchanged()
            control = output / f'{case}.window-control.json'
            opacity = case == 'opacity-verification-refused'
            command = [str(binary), str(base.FIXTURES / ('opacity.html' if opacity else 'admitted.html')),
                       '--presenter=vulkan', '--raster=gpu', '--no-scripts', '--exit-after', str(window_seconds)]
            if opacity:
                command += ['--vulkan-verify-frames', '1']
            wrapper = Path(base.__file__) if opacity else Path(__file__)
            command = [sys.executable, str(wrapper.resolve()), '--owned-launch', '--receipt', str(control.resolve()),
                       '--controller', str(controller), '--controller-sha256', frozen['controller'],
                       '--binary-sha256', frozen['binary'], '--window-seconds', str(window_seconds),
                       '--allow-experimental-gpu', '--', *command]
            record, stdout = base.run_supervised(command, output, case, env, timeout)
            result['runs'].append(record)
            persist()
            if control.is_file():
                record['control_sha256'] = base.digest(control, 1024 * 1024)
                record['control'] = json.loads(control.read_bytes())
                persist()
            expected_exit = 1 if opacity else 0
            if (record.get('status') != 'exited' or record.get('returncode') != expected_exit
                    or not record['cleanup']['complete'] or record['cleanup']['descendants'] != 0
                    or record.get('capture_gate_required') or record.get('capture_gate_granted')
                    or (output / f'{case}.supervisor.stderr.log').stat().st_size):
                raise ValueError('follow-up process status/cleanup disagrees with expected outcome')
            stderr = output / f'{case}.stderr.log'
            if (base.digest(stderr, base.MAX_OUTPUT) != record['stderr_sha256']
                    or hashlib.sha256(stdout).hexdigest() != record['stdout_sha256']):
                raise ValueError('follow-up raw output identity changed')
            details = record['control']
            if details.get('binary_sha256') != frozen['binary'] or details.get('controller_sha256') != frozen['controller']:
                raise ValueError('owned wrapper identity disagrees')
            record['validation'] = validate_followup(stdout, stderr.read_bytes(), case, details)
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
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    parser.add_argument('--window-seconds', type=float, default=6)
    if owned:
        parser.add_argument('--receipt', type=Path, required=True)
        parser.add_argument('--controller', type=Path, required=True)
        parser.add_argument('--controller-sha256', required=True)
        parser.add_argument('--binary-sha256', required=True)
        parser.add_argument('command', nargs=argparse.REMAINDER)
    else:
        parser.add_argument('--binary', type=Path, required=True)
        parser.add_argument('--output', type=Path, required=True)
        parser.add_argument('--loader-directory', type=Path, required=True)
        parser.add_argument('--timeout', type=float, default=30)
    args = parser.parse_args(sys.argv[2:] if owned else None)
    if not args.allow_experimental_gpu:
        parser.error('actual window follow-ups require --allow-experimental-gpu')
    def interrupted(_signal, _frame):
        raise KeyboardInterrupt
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    if owned:
        command = args.command[1:] if args.command[:1] == ['--'] else args.command
        return reload_launch(command, args.receipt, args.controller, args.controller_sha256,
                             args.binary_sha256, args.window_seconds)
    result = run_followups(args.binary, args.output, args.loader_directory, args.timeout, args.window_seconds)
    print(json.dumps({'success': result['success'], 'results': str(args.output / 'followup-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
