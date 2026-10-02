#!/usr/bin/env python3
"""Bounded real-window native raster checks, before the desktop compositor."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import shutil
import subprocess
import sys
import tempfile
import time

from run import launch_environment

ROOT = Path(__file__).resolve().parent
PROBE = ROOT / 'vulkan-raster-probe'
sys.path.insert(0, str(PROBE))
from run_browser_host import check_success, digest, run_supervised  # noqa: E402
from run_glyph_host import loader_binding  # noqa: E402
from run_host import run_process  # noqa: E402

FIXTURES = ROOT / 'native-raster-fixtures'
FIXTURE_SHA256 = '3bb7764b3db0cbcb3b806ab814a29f8564b8c6607815406dce54b2ff39c550ac'
MAX_BINARY = 128 * 1024 * 1024
MAX_OUTPUT = 2 * 1024 * 1024
SIZE = (1180, 880)
PIXEL_BYTES = 1180 * 880 * 4
CONVERSION_WORK = 148 * 110 * 64
CASES = ('admitted-verified', 'admitted-normal', 'opacity-fallback')
SCENE = re.compile(r'native scene prepared: serial=(\d+) snapshot_generation=(\d+) page_commands=(\d+) images=(\d+) loading=(true|false) phases=(\d+) lowered_commands=(\d+) rounded_masks=(\d+) raster_invocations=(\d+) total_invocations=(\d+) reference=(true|false)')
PRESENTED = re.compile(r'presenter: native shader frame presented; size=(\d+)x(\d+) serial=(\d+) CPU_upload=false reference=(true|false)')
VERIFIED = re.compile(r'presenter: native shader acquired-texture verified; serial=(\d+) generation=(\d+) viewport_revision=(\d+) size=(\d+)x(\d+) compared_bytes=(\d+) exact=true \(before compositor\)')
SUMMARY = re.compile(r'presenter: Vulkan acquired-texture verification passed; route=native-raster frames=(\d+) compared_bytes=(\d+) last_serial=(\d+) \(before compositor\)')
FALLBACK = re.compile(r'presenter: native raster admission fallback; complete CPU upload; reason=(.{1,1024})')
QUOTED = r'"(?:\\.|[^"\\])*"'
ADAPTER = re.compile(r'presenter: Vulkan adapter=' + QUOTED + r' type=(?:DiscreteGpu|IntegratedGpu|VirtualGpu|Other|Cpu) driver=' + QUOTED + r' format=(?:Bgra8Unorm|Rgba8Unorm) color_space=Srgb alpha=Opaque mode=Fifo; raster=native-shaders with complete CPU admission fallback')


def validate_run(stdout: bytes, stderr: bytes, case: str) -> dict:
    if case not in CASES or stdout or len(stderr) > MAX_OUTPUT:
        raise ValueError('case/stdout/output bound mismatch')
    text = stderr.decode('utf-8', errors='strict')
    if not text.endswith('\n'):
        raise ValueError('missing terminal log newline')
    lines = text.splitlines()
    if len(lines) > 4096 or any(len(line) > 16384 for line in lines):
        raise ValueError('native diagnostic bounds')
    scenes, presented, verified, summaries, fallbacks, adapters, diagnostics = {}, [], [], [], [], [], []
    for line in lines:
        if match := SCENE.fullmatch(line):
            serial, generation, commands, images, loading, phases, lowered, rounded, raster, total, reference = match.groups()
            row = dict(zip(('serial', 'snapshot_generation', 'page_commands', 'images', 'phases', 'lowered_commands', 'rounded_masks', 'raster_invocations', 'total_invocations'),
                           map(int, (serial, generation, commands, images, phases, lowered, rounded, raster, total))))
            row.update(loading=loading == 'true', reference=reference == 'true')
            if (summaries or row['serial'] <= 0 or row['serial'] in scenes or row['snapshot_generation'] <= 0
                    or row['loading'] or not 1 <= row['page_commands'] <= 256 or row['images'] != 1
                    or row['phases'] != 3 or not 1 <= row['lowered_commands'] <= 256
                    or not 1 <= row['rounded_masks'] <= row['lowered_commands']
                    or row['raster_invocations'] < CONVERSION_WORK
                    or row['total_invocations'] - row['raster_invocations'] != CONVERSION_WORK
                    or row['total_invocations'] > 4_000_000):
                raise ValueError('loaded-scene identity or admission bounds')
            scenes[row['serial']] = row
        elif match := PRESENTED.fullmatch(line):
            width, height, serial, reference = match.groups()
            if (int(width), int(height)) != SIZE:
                raise ValueError('native presented dimensions')
            serial = int(serial)
            if serial not in scenes or scenes[serial]['reference'] != (reference == 'true') or summaries:
                raise ValueError('presentation must follow its matching scene')
            presented.append({'serial': serial, 'reference': reference == 'true'})
        elif match := VERIFIED.fullmatch(line):
            serial, generation, revision, width, height, count = map(int, match.groups())
            if (width, height) != SIZE or count != PIXEL_BYTES:
                raise ValueError('acquired verification dimensions/bytes')
            if (serial not in scenes or not scenes[serial]['reference']
                    or scenes[serial]['snapshot_generation'] != generation
                    or not any(row['serial'] == serial and row['reference'] for row in presented) or summaries):
                raise ValueError('verification must follow its matching scene and presentation')
            verified.append({'serial': serial, 'generation': generation, 'viewport_revision': revision,
                             'compared_bytes': count})
        elif match := SUMMARY.fullmatch(line):
            frames, count, serial = map(int, match.groups())
            if summaries or len(verified) != 1 or verified[0]['serial'] != serial:
                raise ValueError('summary must follow its matching verification')
            summaries.append({'frames': frames, 'compared_bytes': count, 'last_serial': serial})
        elif match := FALLBACK.fullmatch(line):
            fallbacks.append(match.group(1))
        elif ADAPTER.fullmatch(line):
            adapters.append(line)
        elif (line.startswith(('presenter:', 'native scene', 'paint:', 'eris-browser:', 'Unable to open browser window:', "thread '"))
              or 'panicked at' in line):
            raise ValueError('unrecognized/failure native process diagnostic')
        else:
            diagnostics.append(line)  # Retained verbatim; never interpreted as proof.
    if len(adapters) != 1:
        raise ValueError('one actual Vulkan surface adapter record required')
    if case == 'opacity-fallback':
        if (scenes or presented or verified or summaries
                or not any(re.fullmatch(r'unsupported-opacity at phase 0 original command \d+; size=1180x880', reason) for reason in fallbacks)):
            raise ValueError('opacity requires explicit page-phase whole-CPU admission fallback')
    else:
        if len(presented) != 1 or not scenes:
            raise ValueError('missing unique native presentation or loaded scene')
        first = presented[0]
        scene = scenes.get(first['serial'])
        if scene is None or scene['reference'] != first['reference']:
            raise ValueError('presentation lacks matching loaded-scene serial/reference')
        if case == 'admitted-verified':
            if len(verified) != 1 or len(summaries) != 1 or not first['reference']:
                raise ValueError('exactly one acquired reference verification required')
            row, summary = verified[0], summaries[0]
            if (row['serial'] != first['serial'] or row['generation'] != scene['snapshot_generation']
                    or summary != {'frames': 1, 'compared_bytes': PIXEL_BYTES, 'last_serial': row['serial']}):
                raise ValueError('verification/scene/completion identities disagree')
        elif (verified or summaries or first['reference']
              or any(row['reference'] for row in scenes.values())):
            raise ValueError('normal native run must have no CPU reference or verification')
    return {'case': case, 'adapter': adapters[0], 'prepared_scenes': list(scenes.values()),
            'native_presentations': presented, 'acquired_verifications': verified,
            'verification_summaries': summaries, 'fallback_reasons': fallbacks,
            'other_diagnostic_lines': diagnostics,
            'acquired_texture_compared_bytes': sum(row['compared_bytes'] for row in verified),
            'reference': 'original Canvas differential' if verified else 'none',
            'scope': 'native acquired texture before compositor; fallback is admission evidence without pixel verification; no performance claim'}


def fixture_binding() -> dict:
    frozen = FIXTURES / 'freeze.json'
    if digest(frozen, 65536) != FIXTURE_SHA256:
        raise ValueError('fixed fixture manifest changed')
    ledger = json.loads(frozen.read_bytes())
    if {row['path'] for row in ledger['files']} != {'README.md', 'admitted.html', 'opacity.html', 'two-pixels.png'}:
        raise ValueError('fixture inventory mismatch')
    for row in ledger['files']:
        path = FIXTURES / row['path']
        if path.is_symlink() or digest(path, 65536) != row['sha256'] or path.stat().st_size != row['bytes']:
            raise ValueError('fixed fixture bytes changed')
    return {'directory': str(FIXTURES), 'freeze_sha256': FIXTURE_SHA256, 'files': ledger['files']}


def source_binding() -> dict:
    paths = [Path(__file__).resolve(), ROOT / 'run.py'] + [PROBE / name for name in (
        'run_browser_host.py', 'run_glyph_host.py', 'run_host.py', 'bridge_supervisor.py',
        'browser_protocol.py', 'glyph_protocol.py')]
    return {str(path): digest(path, 1024 * 1024) for path in paths}


def environment(loader: Path) -> dict:
    if not sys.platform.startswith('linux') or not loader.is_dir():
        raise ValueError('native host requires Linux and an existing explicit loader directory')
    env = launch_environment()
    env['LD_LIBRARY_PATH'] = str(loader.resolve()) + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else '')
    for key in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME', 'CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS'):
        env.pop(key, None)
    return env



# These helpers run only inside the existing isolated subreaper. The browser is
# their direct child; no other thread/SIGCHLD handler reaps it during control.
WAIT_FLAGS = os.WEXITED | os.WNOHANG | os.WNOWAIT
CONTROL_BYTES = 256 * 1024
CONTROL_COMMANDS = 48


def child_identity(pid: int) -> dict:
    with open(f'/proc/{pid}/stat', 'rb') as source:
        raw = source.read(4097)
    if len(raw) > 4096:
        raise ValueError('owned child stat exceeds bound')
    fields = raw[raw.rfind(b')') + 2:].split()
    if len(fields) < 20 or int(fields[1]) != os.getpid():
        raise ValueError('browser is not the direct owned child')
    return {'pid': pid, 'parent_pid': int(fields[1]), 'process_group': int(fields[2]),
            'start_ticks': int(fields[19])}


def ensure_live_child(identity: dict) -> None:
    # Even if it exits immediately after this check, the unreaped PID cannot be
    # reused before the following pid: selector reaches the compositor.
    if os.waitid(os.P_PID, identity['pid'], WAIT_FLAGS) is not None:
        raise ValueError('owned browser exited before window control completed')
    if child_identity(identity['pid']) != identity:
        raise ValueError('owned browser identity changed')


def controller_call(controller: Path, args: list[str], env: dict, timeout: float):
    # Reuse the existing bounded pipe reader and kill-before-reap helper. Client
    # queries are temporary: unrelated window metadata is never published.
    with tempfile.TemporaryDirectory(prefix='eris-native-controller-') as directory:
        result = run_process(controller, args, 'control', Path(directory), env,
                             timeout=timeout, output_limit=CONTROL_BYTES)
    return result


def configure_owned(identity: dict, controller: Path, env: dict, record: dict, persist) -> None:
    deadline = time.monotonic() + 3.0
    pid = identity['pid']

    def invoke(args):
        ensure_live_child(identity)
        remaining = deadline - time.monotonic()
        if remaining < 0.05 or len(record['commands']) >= CONTROL_COMMANDS:
            raise ValueError('owned-window control deadline/command bound')
        result = controller_call(controller, args, env, min(0.5, remaining))
        query = args == ['-j', 'clients']
        row = {'argv': [str(controller), *args], 'status': result.record['status'],
               'returncode': result.record['returncode'], 'stdout_bytes': len(result.stdout),
               'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
               'stderr_bytes': len(result.stderr), 'stderr_sha256': hashlib.sha256(result.stderr).hexdigest()}
        # Dispatch responses are retained exactly; query output is filtered below.
        if not query and len(result.stdout) <= 4096:
            row['stdout'] = result.stdout.decode('utf-8', errors='strict')
        if len(result.stderr) <= 4096:
            row['stderr'] = result.stderr.decode('utf-8', errors='strict')
        record['commands'].append(row)
        persist()
        if (result.record['status'] != 'exited' or result.record['returncode'] != 0
                or result.stderr or (not query and len(result.stdout) > 4096)):
            raise ValueError('bounded window controller failed')
        ensure_live_child(identity)
        return result.stdout, row

    def own_client():
        stdout, row = invoke(['-j', 'clients'])
        clients = json.loads(stdout)
        if not isinstance(clients, list) or len(clients) > 256 or any(not isinstance(c, dict) for c in clients):
            raise ValueError('compositor client inventory exceeds bound or is malformed')
        matches = [c for c in clients if type(c.get('pid')) is int and c['pid'] == pid]
        if len(matches) > 1:
            raise ValueError('multiple windows for the owned browser')
        if not matches:
            row['owned_client'] = None
            persist()
            return None
        client = matches[0]
        size = client.get('size')
        if (not isinstance(size, list) or len(size) != 2
                or any(type(v) is not int or not 1 <= v <= 16384 for v in size)
                or type(client.get('floating')) is not bool or client.get('mapped') is not True):
            raise ValueError('owned compositor client metadata is invalid')
        row['owned_client'] = {'pid': pid, 'size': size, 'floating': client['floating'], 'mapped': True}
        persist()
        return row['owned_client']

    client = None
    while client is None:
        client = own_client()
        if client is None:
            time.sleep(0.025)
    invoke(['dispatch', 'setfloating', f'pid:{pid}'])
    # Recheck the same owned PID after the first state-changing command.
    if own_client() is None:
        raise ValueError('owned compositor window disappeared')
    invoke(['dispatch', 'resizewindowpixel', f'exact {SIZE[0]} {SIZE[1]},pid:{pid}'])
    while True:
        client = own_client()
        if client and client['floating'] and tuple(client['size']) == SIZE:
            record['controlled_size'] = list(SIZE)
            record['control_complete'] = True
            persist()
            return
        time.sleep(0.025)


def owned_launch(command: list[str], receipt: Path, controller: Path, controller_sha: str,
                 binary_sha: str, window_seconds: float) -> int:
    if receipt.exists() or not receipt.parent.is_dir() or not command:
        raise ValueError('owned launch needs fresh receipt and explicit browser command')
    if (not math.isfinite(window_seconds) or not 2 <= window_seconds <= 20
            or digest(controller, MAX_BINARY) != controller_sha
            or digest(Path(command[0]), MAX_BINARY) != binary_sha):
        raise ValueError('owned launch inputs changed')
    record = {'schema': 1, 'success': False, 'control_complete': False, 'commands': [],
              'browser_command': command, 'binary_sha256': binary_sha,
              'controller': str(controller), 'controller_sha256': controller_sha,
              'identity': None, 'browser_returncode': None, 'browser_reaped': False,
              'scope': 'only owned unreaped child PID; descendants remain outer subreaper responsibility'}
    child = None
    started = time.monotonic()

    def persist():
        temp = receipt.with_suffix('.tmp')
        temp.write_text(json.dumps(record, indent=2) + '\n')
        temp.replace(receipt)

    persist()
    try:
        # Inherit the wrapper group: the outer supervisor can kill the wrapper
        # and browser together, then adopt/clean independently grouped workers.
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, close_fds=True)
        record['identity'] = child_identity(child.pid)
        persist()
        configure_owned(record['identity'], controller, dict(os.environ), record, persist)
        while os.waitid(os.P_PID, child.pid, WAIT_FLAGS) is None:
            if time.monotonic() - started > window_seconds + 5:
                raise ValueError('owned browser exit deadline')
            time.sleep(0.025)
        # No further compositor calls after reaping.
        record['browser_returncode'] = child.wait(timeout=2)
        record['browser_reaped'] = True
        record['success'] = record['control_complete'] and record['browser_returncode'] == 0
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        if child is not None and not record['browser_reaped']:
            try:
                os.waitid(os.P_PID, child.pid, WAIT_FLAGS)  # Ownership proof before signal.
                os.kill(child.pid, signal.SIGKILL)
                record['browser_returncode'] = child.wait(timeout=2)
                record['browser_reaped'] = True
            except (OSError, subprocess.SubprocessError) as error:
                record['cleanup_error'] = f'{type(error).__name__}: {error}'[:4096]
        try:
            if digest(controller, MAX_BINARY) != controller_sha or digest(Path(command[0]), MAX_BINARY) != binary_sha:
                raise ValueError('owned launch inputs changed after execution')
        except (OSError, ValueError) as error:
            record['success'] = False
            record['postcheck_error'] = str(error)[:4096]
        persist()
    return 0 if record['success'] else 1


def run_host(binary: Path, output: Path, loader: Path, timeout: float = 30, window_seconds: float = 6,
             hyprland_size: str | None = None) -> dict:
    if (not math.isfinite(timeout) or not math.isfinite(window_seconds)
            or not 2 <= window_seconds <= 20 or not window_seconds + 10 <= timeout <= 120):
        raise ValueError('window/host deadlines outside bounded range')
    if output.exists() or output.resolve().is_relative_to(FIXTURES.resolve()):
        raise ValueError('output must be new and outside frozen fixtures')
    if hyprland_size not in (None, '1180x880'):
        raise ValueError('only the fixed 1180x880 owned-window request is admitted')
    binary, loader = binary.resolve(), loader.resolve()
    binary_sha = digest(binary, MAX_BINARY)
    fixtures, sources = fixture_binding(), source_binding()
    env = environment(loader)
    loader_record = loader_binding(loader, env)
    controller = None
    if hyprland_size:
        located = shutil.which('hyprctl', path=env.get('PATH', os.defpath))
        if not located:
            raise ValueError('explicit owned-window sizing requires hyprctl')
        path = Path(located).resolve()
        controller = {'path': str(path), 'sha256': digest(path, MAX_BINARY)}
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'binary': str(binary), 'binary_sha256': binary_sha,
              'fixtures': fixtures, 'host_source_bindings': sources, 'loader': loader_record,
              'display_environment': {key: env.get(key) for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'XDG_SESSION_TYPE')},
              'hyprland_size': hyprland_size, 'controller': controller,
              'window_seconds': window_seconds, 'timeout_seconds': timeout, 'runs': [],
              'scope': 'three separate native-window cases, no retries; acquired verification is Canvas-differential and before compositor'}

    def persist():
        temp = output / 'host-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n')
        temp.replace(output / 'host-results.json')

    def unchanged():
        if controller and digest(Path(controller['path']), MAX_BINARY) != controller['sha256']:
            raise ValueError('window controller changed')
        if (digest(binary, MAX_BINARY) != binary_sha or fixture_binding() != fixtures
                or source_binding() != sources or loader_binding(loader, env) != loader_record):
            raise ValueError('frozen binary/fixture/host/loader binding changed')

    persist()
    try:
        for case in CASES:
            unchanged()
            html = 'opacity.html' if case == 'opacity-fallback' else 'admitted.html'
            command = [str(binary), str((FIXTURES / html).resolve()), '--presenter=vulkan', '--raster=gpu',
                       '--no-scripts', '--exit-after', str(window_seconds)]
            if case == 'admitted-verified':
                command += ['--vulkan-verify-frames', '1']
            control_receipt = output / f'{case}.window-control.json'
            if controller:
                command = [sys.executable, str(Path(__file__).resolve()), '--owned-launch',
                           '--receipt', str(control_receipt.resolve()), '--controller', controller['path'],
                           '--controller-sha256', controller['sha256'], '--binary-sha256', binary_sha,
                           '--window-seconds', str(window_seconds), '--allow-experimental-gpu', '--', *command]
            record, stdout = run_supervised(command, output, case, env, timeout)
            result['runs'].append(record)
            persist()
            if controller and control_receipt.is_file():
                record['window_control_receipt_sha256'] = digest(control_receipt, 1024 * 1024)
                record['window_control'] = json.loads(control_receipt.read_bytes())
                persist()
            check_success(record)
            if controller:
                control = record.get('window_control', {})
                if (not control.get('success') or not control.get('control_complete')
                        or not control.get('browser_reaped') or control.get('browser_returncode') != 0
                        or control.get('controlled_size') != list(SIZE)
                        or control.get('binary_sha256') != binary_sha
                        or control.get('controller_sha256') != controller['sha256']):
                    raise ValueError('owned-window control was not completed')
            if ((output / f'{case}.supervisor.stderr.log').stat().st_size
                    or record.get('capture_gate_required') or record.get('capture_gate_granted')):
                raise ValueError('unexpected supervisor diagnostic or worker grant')
            stderr_path = output / f'{case}.stderr.log'
            if digest(stderr_path, MAX_OUTPUT) != record['stderr_sha256']:
                raise ValueError('raw native stderr changed')
            stderr = stderr_path.read_bytes()
            if len(stdout) + len(stderr) > MAX_OUTPUT or hashlib.sha256(stdout).hexdigest() != record['stdout_sha256']:
                raise ValueError('raw native output binding mismatch')
            record['validation'] = validate_run(stdout, stderr, case)
            unchanged()
            persist()
        unchanged()
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        try:
            unchanged()
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
            result['success'] = False
            result['postcheck_error'] = f'{type(error).__name__}: {error}'[:4096]
        persist()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--loader-directory', type=Path, required=True)
    parser.add_argument('--timeout', type=float, default=30)
    parser.add_argument('--window-seconds', type=float, default=6)
    parser.add_argument('--hyprland-size', choices=['1180x880'])
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    args = parser.parse_args()
    if not args.allow_experimental_gpu:
        parser.error('actual window/worker/Vulkan execution requires --allow-experimental-gpu')
    previous = {}

    def interrupted(_signum, _frame):
        raise KeyboardInterrupt

    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, interrupted)
        result = run_host(args.binary, args.output, args.loader_directory, args.timeout, args.window_seconds, args.hyprland_size)
    except (OSError, ValueError) as error:
        parser.exit(1, f'{error}\n')
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    print(json.dumps({'success': result['success'], 'completed_cases': len(result['runs']),
                      'results': str(args.output / 'host-results.json')}))
    return 0 if result['success'] else 1


def owned_main() -> int:
    parser = argparse.ArgumentParser(description='Internal owned-child window controller')
    parser.add_argument('--receipt', type=Path, required=True)
    parser.add_argument('--controller', type=Path, required=True)
    parser.add_argument('--controller-sha256', required=True)
    parser.add_argument('--binary-sha256', required=True)
    parser.add_argument('--window-seconds', type=float, required=True)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args(sys.argv[2:])
    if not args.allow_experimental_gpu:
        parser.error('owned browser execution requires --allow-experimental-gpu')
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    def interrupted(_signum, _frame):
        raise KeyboardInterrupt

    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, interrupted)
    return owned_launch(command, args.receipt, args.controller, args.controller_sha256,
                        args.binary_sha256, args.window_seconds)


if __name__ == '__main__':
    raise SystemExit(owned_main() if sys.argv[1:2] == ['--owned-launch'] else main())
