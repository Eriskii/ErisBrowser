#!/usr/bin/env python3
"""Three bounded owned-window zoom checks; reuses the existing wide host."""
from __future__ import annotations
import argparse
from contextlib import contextmanager
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import sys

TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import native_raster_host as base
import native_raster_wide_host as wide

CASES = {'zoom-in-verified': ('equal', 1066192077, True),
         'zoom-out-verified': ('minus', 1063675494, True),
         'zoom-in-normal': ('equal', 1066192077, False)}
ZOOM = re.compile(r'native zoom prepared: serial=([1-9][0-9]*) zoom_bits=([0-9]+)')


def waiting_reason(case):
    return f'native scene awaits a current loaded page; zoom_bits={CASES[case][1]}; size=1280x880'


@contextmanager
def case_scope(case, *, control=False):
    """Process-local substitutions only; original helpers and files stay exact."""
    key, bits, _ = CASES[case]
    old_reason, configure = wide.PHYSICAL, base.configure_owned
    wide.PHYSICAL = waiting_reason(case)
    def configure_zoom(identity, controller, env, record, persist, **kwargs):
        configure(identity, controller, env, record, persist, **kwargs)
        record['control_complete'] = False
        record['zoom'] = {'case': case, 'zoom_bits': bits, 'dispatch_complete': False}
        persist()
        base.ensure_live_child(identity)
        if len(record['commands']) >= base.CONTROL_COMMANDS:
            raise ValueError('zoom controller command bound')
        args = ['dispatch', 'sendshortcut', f'CTRL,{key},pid:{identity["pid"]}']
        result = base.controller_call(controller, args, env, 0.5)
        command = {'argv': [str(controller), *args], 'status': result.record['status'],
                   'returncode': result.record['returncode'],
                   'stdout_bytes': len(result.stdout), 'stdout_sha256': wide.sha(result.stdout),
                   'stderr_bytes': len(result.stderr), 'stderr_sha256': wide.sha(result.stderr)}
        if len(result.stdout) <= 4096 and len(result.stderr) <= 4096:
            command.update(stdout=result.stdout.decode('utf-8'), stderr=result.stderr.decode('utf-8'))
        record['commands'].append(command)
        record['zoom']['command'] = command
        persist()
        if (result.record['status'] != 'exited' or result.record['returncode'] != 0
                or result.stdout.strip() != b'ok' or result.stderr):
            raise ValueError('owned zoom shortcut dispatch failed')
        base.ensure_live_child(identity)
        record['zoom']['dispatch_complete'] = True
        record['control_complete'] = True
        persist()
    if control:
        base.configure_owned = configure_zoom
    try:
        yield
    finally:
        wide.PHYSICAL, base.configure_owned = old_reason, configure


def validate_zoom(stdout, stderr, case, control, controller):
    key, bits, verify = CASES[case]
    event = control.get('zoom', {})
    command = event.get('command', {})
    if (event.get('case') != case or event.get('zoom_bits') != bits
            or event.get('dispatch_complete') is not True
            or command.get('argv') != [str(controller), 'dispatch', 'sendshortcut',
                                       f'CTRL,{key},pid:{control["identity"]["pid"]}']
            or command.get('status') != 'exited' or command.get('returncode') != 0
            or command.get('stdout', '').strip() != 'ok' or command.get('stderr') != ''
            or command.get('stdout_bytes') != len(command.get('stdout', '').encode())
            or command.get('stdout_sha256') != wide.sha(command.get('stdout', '').encode())
            or command.get('stderr_bytes') != 0 or command.get('stderr_sha256') != wide.sha(b'')
            or control.get('commands', []).count(command) != 1
            or sum(row.get('argv', [])[1:3] == ['dispatch', 'sendshortcut']
                   for row in control.get('commands', [])) != 1
            or len(control.get('commands', [])) > base.CONTROL_COMMANDS):
        raise ValueError('zoom shortcut identity/completion binding')
    with case_scope(case):
        checked = wide.validate_run(stdout, stderr, 'wide-verified' if verify else 'wide-normal', control)
    released, scenes, zooms = False, set(), {}
    for line in stderr.decode('utf-8').splitlines():
        if wide.RELEASE.fullmatch(line):
            released = True
        elif match := base.SCENE.fullmatch(line):
            scenes.add(int(match[1]))
        elif match := ZOOM.fullmatch(line):
            serial, observed = map(int, match.groups())
            if not released or serial not in scenes or serial in zooms or observed != bits:
                raise ValueError('zoom state must follow its released scene and match requested bits')
            zooms[serial] = observed
        elif match := base.PRESENTED.fullmatch(line):
            if int(match[3]) not in zooms:
                raise ValueError('presentation preceded matching prepared zoom state')
        elif line.startswith('native zoom'):
            raise ValueError('unknown zoom diagnostic')
    if set(zooms) != scenes or not scenes:
        raise ValueError('every retained prepared scene requires exact zoom proof')
    checked.update(case=case, requested_zoom_bits=bits, observed_zoom_by_serial=zooms,
                   shortcut=command, verification_requested=verify)
    return checked


def run_host(binary, binary_sha, output, loader, timeout, seconds):
    if (output.exists() or output.is_symlink() or not math.isfinite(seconds)
            or not 2 <= seconds <= 20 or not math.isfinite(timeout)
            or not seconds + 10 <= timeout <= 120
            or output.resolve().is_relative_to(base.FIXTURES.resolve())):
        raise ValueError('fresh output and inherited bounded lifetimes required')
    binary, loader = binary.resolve(), loader.resolve()
    env = base.environment(loader)
    located = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
    if not located:
        raise ValueError('owned zoom control requires hyprctl')
    controller = Path(located).resolve()
    def bindings():
        return {'binary': base.digest(binary, base.MAX_BINARY),
                'controller': base.digest(controller, base.MAX_BINARY),
                'fixtures': base.fixture_binding(), 'base_sources': base.source_binding(),
                'wrapper': base.digest(Path(__file__), 1024 * 1024),
                'wide_host': base.digest(Path(wide.__file__), 1024 * 1024),
                'drain_source': base.digest(TOOLS / 'native_raster_followup.py', 1024 * 1024),
                'loader': base.loader_binding(loader, env)}
    frozen = bindings()
    if frozen['binary'] != binary_sha:
        raise ValueError('explicit frozen binary hash mismatch')
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'bindings': frozen, 'binary': str(binary),
              'controller': str(controller), 'timeout_seconds': timeout, 'window_seconds': seconds,
              'runs': [], 'scope': 'three native zoom windows; two Canvas acquired-texture comparisons before compositor and one normal frame; no performance claim'}
    def persist():
        temporary = output / 'zoom-host-results.json.tmp'
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(output / 'zoom-host-results.json')
    def unchanged():
        if bindings() != frozen:
            raise ValueError('zoom binary/fixture/helper/loader changed')
    persist()
    try:
        for case in CASES:
            unchanged()
            receipt = output / f'{case}.window-control.json'
            argv = [sys.executable, str(Path(__file__).resolve()), '--owned-launch', '--case', case,
                    '--binary', str(binary), '--binary-sha256', binary_sha, '--receipt', str(receipt.resolve()),
                    '--controller', str(controller), '--controller-sha256', frozen['controller'],
                    '--window-seconds', str(seconds), '--allow-experimental-gpu']
            record, stdout = base.run_supervised(argv, output, case, env, timeout)
            result['runs'].append(record)
            persist()
            if receipt.is_file():
                record['control_sha256'] = base.digest(receipt, 1024 * 1024)
                record['control'] = json.loads(receipt.read_bytes())
                persist()
            base.check_success(record)
            if ((output / f'{case}.supervisor.stderr.log').stat().st_size
                    or record.get('capture_gate_required') or record.get('capture_gate_granted')):
                raise ValueError('unexpected supervisor diagnostics or GPU grant')
            stderr = output / f'{case}.stderr.log'
            if base.digest(stderr, base.MAX_OUTPUT) != record['stderr_sha256'] or wide.sha(stdout) != record['stdout_sha256']:
                raise ValueError('zoom raw output binding')
            control = record['control']
            if (control.get('binary_sha256') != binary_sha or control.get('controller_sha256') != frozen['controller']
                    or control.get('fixture_freeze') != frozen['fixtures']['freeze_sha256']):
                raise ValueError('owned launch input identity mismatch')
            record['validation'] = validate_zoom(stdout, stderr.read_bytes(), case, control, controller)
            if record['validation']['adapter'] != result['runs'][0]['validation']['adapter']:
                raise ValueError('adapter changed between zoom cases')
            unchanged()
            persist()
        result['acquired_texture_compared_bytes'] = sum(row['validation']['acquired_texture_compared_bytes'] for row in result['runs'])
        if result['acquired_texture_compared_bytes'] != 2 * wide.PIXEL_BYTES:
            raise ValueError('two complete acquired frames required')
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


def main():
    owned = sys.argv[1:2] == ['--owned-launch']
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--binary-sha256', required=True)
    p.add_argument('--window-seconds', type=float, default=8)
    p.add_argument('--allow-experimental-gpu', action='store_true')
    if owned:
        p.add_argument('--case', choices=CASES, required=True)
        p.add_argument('--receipt', type=Path, required=True)
        p.add_argument('--controller', type=Path, required=True)
        p.add_argument('--controller-sha256', required=True)
    else:
        p.add_argument('--output', type=Path, required=True)
        p.add_argument('--loader-directory', type=Path, required=True)
        p.add_argument('--timeout', type=float, default=30)
    a = p.parse_args(sys.argv[2:] if owned else None)
    if not a.allow_experimental_gpu:
        p.error('actual native windows require --allow-experimental-gpu')
    def interrupted(_signal, _frame):
        raise KeyboardInterrupt
    for s in (signal.SIGINT, signal.SIGTERM):
        signal.signal(s, interrupted)
    if owned:
        with case_scope(a.case, control=True):
            return wide.launch_wide(a.binary, a.receipt, a.controller, a.controller_sha256,
                                    a.binary_sha256, a.window_seconds, CASES[a.case][2])
    result = run_host(a.binary, a.binary_sha256, a.output, a.loader_directory, a.timeout, a.window_seconds)
    print(json.dumps({'success': result['success'], 'results': str(a.output / 'zoom-host-results.json')}))
    return 0 if result['success'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
