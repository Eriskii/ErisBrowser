#!/usr/bin/env python3
"""Check real browser snapshots with the optional offscreen Vulkan bridge."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys

import browser_protocol
from run_host import child_environment

ROOT = Path(__file__).resolve().parent
ORACLE_SHA256 = 'ec652b547096f37db93433d5347c78873cef2eae63a03ced622013aa746b88e1'
MAX_BINARY = 128 * 1024 * 1024
MAX_OUTPUT = 2 * 1024 * 1024


def digest(path: Path, cap: int) -> str:
    if not path.is_file() or path.stat().st_size > cap:
        raise ValueError(f'file missing, nonregular or over bound: {path.name}')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def check_oracle(directory: Path) -> dict:
    frozen = directory / 'freeze.json'
    if digest(frozen, 65536) != ORACLE_SHA256:
        raise ValueError('independent oracle ledger changed')
    manifest = json.loads(frozen.read_text())
    if len(manifest['files']) != 29:
        raise ValueError('independent oracle inventory changed')
    for row in manifest['files']:
        path = (directory / row['path']).resolve()
        if not path.is_relative_to(directory.resolve()):
            raise ValueError('oracle path escapes its directory')
        if digest(path, 1024 * 1024) != row['sha256'] or path.stat().st_size != row['bytes']:
            raise ValueError(f'independent oracle input changed: {row["path"]}')
    return {'freeze_sha256': ORACLE_SHA256, 'files': len(manifest['files'])}


def run_supervised(command: list[str], output: Path, name: str, env: dict,
                   timeout: float, capture_gate: bool = False,
                   worker_text_gate: bool = False) -> tuple[dict, bytes]:
    if capture_gate and worker_text_gate:
        raise ValueError('capture gate modes are mutually exclusive')
    receipt = output / f'{name}.json'
    launcher_error = output / f'{name}.supervisor.stderr.log'
    supervisor = [sys.executable, str(ROOT / 'bridge_supervisor.py'), '--timeout',
                  str(timeout), '--output-limit', str(MAX_OUTPUT), '--result',
                  str(receipt), *(['--capture-gate'] if capture_gate else []),
                  *(['--worker-text-gate'] if worker_text_gate else []), '--', *command]
    failure = None
    with launcher_error.open('xb') as errors:
        proc = subprocess.Popen(supervisor, stdin=subprocess.DEVNULL,
                                stdout=subprocess.DEVNULL, stderr=errors, env=env)
        try:
            proc.wait(timeout=timeout + 15)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as exc:
            failure = type(exc).__name__
            # Give the subreaper its own cleanup deadline. Killing that supervisor
            # would lose adoption ownership, so even a later receipt is not used
            # to permit another checker following an outer interruption.
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
                try:
                    proc.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
    if failure or not receipt.is_file() or receipt.stat().st_size > 65536:
        record = {'status': 'supervisor_error', 'error': failure or 'missing or oversized receipt',
                  'cleanup': {'complete': False}, 'name': name}
        (output / f'{name}.outer-error.json').write_text(json.dumps(record, indent=2) + '\n')
        return record, b''
    record = json.loads(receipt.read_text())
    record['name'] = name
    stdout = receipt.with_suffix('.stdout.log')
    stderr = receipt.with_suffix('.stderr.log')
    if (digest(stdout, MAX_OUTPUT) != record['stdout_sha256'] or
            digest(stderr, MAX_OUTPUT) != record['stderr_sha256'] or
            stdout.stat().st_size + stderr.stat().st_size != record['captured_bytes'] or
            record['captured_bytes'] > MAX_OUTPUT):
        raise ValueError('supervisor output binding mismatch')
    if proc.returncode != (0 if record['status'] == 'exited' and record['returncode'] == 0 else 1):
        raise ValueError('supervisor exit and receipt disagree')
    return record, stdout.read_bytes()


def check_success(record: dict) -> None:
    if (record['status'] != 'exited' or record['returncode'] != 0 or
            not record['cleanup']['complete'] or record['cleanup']['descendants'] != 0):
        raise ValueError(f'checker or owned descendant cleanup failed: {record["status"]}')


def run_host(binary: Path, browser: Path, output: Path, oracle: Path,
             loader: Path | None, timeout: float) -> dict:
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError('timeout must be finite and in 0.05..120 seconds')
    if output.exists():
        raise ValueError('output directory must be new; evidence is never overwritten')
    binary, browser, oracle = binary.resolve(), browser.resolve(), oracle.resolve()
    bindings = {str(binary): digest(binary, MAX_BINARY), str(browser): digest(browser, MAX_BINARY)}
    oracle_binding = check_oracle(oracle)
    env = child_environment(loader)
    output.mkdir(parents=True)
    summary = {'schema': 1, 'success': False, 'runs': [], 'adapter_count': 0,
               'binary_bindings': bindings, 'oracle': oracle_binding,
               'scope': 'offscreen snapshot bridge; no native window or performance claim'}

    def persist():
        temp = output / 'host-results.json.tmp'
        temp.write_text(json.dumps(summary, indent=2) + '\n')
        temp.replace(output / 'host-results.json')

    def unchanged():
        for path, expected in bindings.items():
            if digest(Path(path), MAX_BINARY) != expected:
                raise ValueError('frozen executable bytes changed')
        check_oracle(oracle)

    try:
        unchanged()
        record, stdout = run_supervised([str(binary), '--list'], output, 'enumeration', env, timeout)
        summary['runs'].append(record)
        check_success(record)
        adapters = browser_protocol.validate_listing(stdout)
        summary['adapter_count'] = len(adapters)
        persist()
        for index in range(len(adapters)):
            unchanged()
            command = [str(binary), '--adapter', str(index), '--browser', str(browser),
                       '--fixtures', str(oracle)]
            record, stdout = run_supervised(command, output, f'adapter-{index}', env, timeout,
                                           capture_gate=True)
            summary['runs'].append(record)
            persist()
            # Fail closed for *any* failed phase, including incomplete cleanup;
            # no later worker or GPU execution is attempted by this host.
            check_success(record)
            if not record['capture_gate_required'] or not record['capture_gate_granted']:
                raise ValueError('supervisor did not grant the worker-to-Vulkan transition')
            record['validation'] = browser_protocol.validate_run(stdout, adapters, index, oracle)
            unchanged()
            persist()
        summary['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, KeyboardInterrupt) as exc:
        summary['error'] = f'{type(exc).__name__}: {exc}'[:4096]
    finally:
        persist()
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--browser', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path, default=ROOT / 'browser-fixtures/oracle')
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    args = parser.parse_args()
    if not args.allow_experimental_gpu:
        parser.error('actual Vulkan execution requires --allow-experimental-gpu')
    try:
        result = run_host(args.binary, args.browser, args.output_dir, args.fixtures,
                          args.loader_directory, args.timeout)
    except (OSError, ValueError) as exc:
        parser.exit(1, f'{exc}\n')
    print(json.dumps({'success': result['success'], 'adapter_count': result['adapter_count'],
                      'results': str(args.output_dir / 'host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
