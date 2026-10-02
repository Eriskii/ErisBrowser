#!/usr/bin/env python3
"""Compare real worker text snapshots with offscreen Vulkan, under a subreaper."""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import signal

import worker_text_protocol as protocol
from run_browser_host import check_success, digest, run_supervised
from run_glyph_host import EMPTY_SHA256, MAX_BINARY, loader_binding
from run_host import child_environment

ROOT = Path(__file__).resolve().parent


def run_host(binary: Path, browser: Path, output: Path, fixtures: Path,
             loader: Path | None = None, timeout: float = 60.0,
             cpu_check: bool = False) -> dict:
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError('timeout must be finite and in 0.05..120 seconds')
    if output.exists():
        raise ValueError('output directory must be new; evidence is never overwritten')
    binary, browser, fixtures = binary.resolve(), browser.resolve(), fixtures.resolve()
    bindings = {str(path): digest(path, MAX_BINARY) for path in (binary, browser)}
    fixture_binding = protocol.load_fixtures(fixtures)['binding']
    env = child_environment(loader)
    for key in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    loader_record = loader_binding(loader, env)
    source_names = ('run_worker_text_host.py', 'worker_text_protocol.py', 'browser_protocol.py',
                    'run_browser_host.py', 'run_glyph_host.py', 'glyph_protocol.py',
                    'run_host.py', 'bridge_supervisor.py')
    sources = {name: digest(ROOT / name, 1024 * 1024) for name in source_names}
    output.mkdir(parents=True)
    summary = {
        'schema': 1, 'success': False, 'mode': 'cpu-check' if cpu_check else 'gpu',
        'runs': [], 'adapter_count': 0, 'binary_bindings': bindings,
        'fixture_directory': str(fixtures), 'fixture_binding': fixture_binding,
        'loader': loader_record, 'host_source_bindings': sources,
        'timeout_seconds': timeout,
        'scope': 'seven real worker text snapshots; same-snapshot CPU/GPU differential only; '
                 'no independent font golden, native window or performance claim',
    }

    def persist():
        temporary = output / 'host-results.json.tmp'
        temporary.write_text(json.dumps(summary, indent=2) + '\n')
        temporary.replace(output / 'host-results.json')

    def unchanged():
        if any(digest(Path(path), MAX_BINARY) != sha for path, sha in bindings.items()):
            raise ValueError('frozen executable bytes changed')
        if protocol.load_fixtures(fixtures)['binding'] != fixture_binding:
            raise ValueError('worker text fixture binding changed')
        if loader_binding(loader, env) != loader_record:
            raise ValueError('explicit loader binding changed')
        if any(digest(ROOT / name, 1024 * 1024) != sha for name, sha in sources.items()):
            raise ValueError('host or supervisor source changed')

    def execute(name: str, arguments: list[str], gate: bool = False):
        unchanged()
        record, stdout = run_supervised([str(binary), *arguments], output, name, env,
                                        timeout, worker_text_gate=gate)
        summary['runs'].append(record)
        persist()
        check_success(record)
        if (record.get('stderr_sha256') != EMPTY_SHA256 or
                (output / f'{name}.supervisor.stderr.log').stat().st_size):
            raise ValueError('checker and supervisor stderr must be empty')
        if (record.get('capture_gate_required') is not gate or
                record.get('capture_gate_granted') is not gate):
            raise ValueError('worker-to-Vulkan grant disagrees with selected phase')
        unchanged()
        return record, stdout

    persist()
    try:
        common = ['--browser', str(browser), '--fixtures', str(fixtures)]
        if cpu_check:
            record, stdout = execute('cpu-check', ['--cpu-check', *common])
            record['validation'] = protocol.validate_cpu(stdout, fixtures)
        else:
            record, stdout = execute('enumeration', ['--list'])
            adapters = protocol.validate_listing(stdout)
            record['validation'] = {'adapters': adapters}
            summary['adapter_count'] = len(adapters)
            persist()
            for index in range(len(adapters)):
                record, stdout = execute(f'adapter-{index}', ['--adapter', str(index), *common], True)
                record['validation'] = protocol.validate_run(stdout, adapters, index, fixtures)
                persist()
        unchanged()
        summary['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as exc:
        summary['error'] = f'{type(exc).__name__}: {exc}'[:4096]
    finally:
        persist()
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--browser', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path, default=ROOT / 'worker-text-fixtures')
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--cpu-check', action='store_true')
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    args = parser.parse_args()
    if not args.cpu_check and not args.allow_experimental_gpu:
        parser.error('actual Vulkan execution requires --allow-experimental-gpu')
    old_handlers = {}

    def interrupted(_signum, _frame):
        raise KeyboardInterrupt

    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            old_handlers[sig] = signal.signal(sig, interrupted)
        result = run_host(args.binary, args.browser, args.output, args.fixtures,
                          args.loader_directory, args.timeout, args.cpu_check)
    except (OSError, ValueError) as exc:
        parser.exit(1, f'{exc}\n')
    finally:
        for sig, handler in old_handlers.items():
            signal.signal(sig, handler)
    print(json.dumps({'success': result['success'], 'adapter_count': result['adapter_count'],
                      'results': str(args.output / 'host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
