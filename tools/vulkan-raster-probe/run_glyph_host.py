#!/usr/bin/env python3
"""Run the bounded offscreen glyph checker with frozen independent/reference inputs."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal

import glyph_protocol
from run_browser_host import check_success, digest, run_supervised
from run_host import child_environment

ROOT = Path(__file__).resolve().parent
MAX_BINARY = 128 * 1024 * 1024
BASELINE_MANIFEST_SHA256 = 'da746d8cab49544a4294139b0a651ed018fa4913bd3210a4e19f9464cfa31d5b'
EMPTY_SHA256 = hashlib.sha256(b'').hexdigest()


def loader_binding(directory: Path | None, environment: dict) -> dict:
    """Bind explicit loader-directory files, not all transitive system libraries."""
    rows = []
    if directory is not None:
        directory = directory.resolve()
        entries = []
        with os.scandir(directory) as scan:
            for entry in scan:
                if len(entries) == 256:
                    raise ValueError('loader-directory entry bound exceeded')
                entries.append(Path(entry.path))
        entries.sort()
        for path in entries:
            if path.is_file():
                rows.append({'name': path.name, 'resolved': str(path.resolve()),
                             'bytes': path.stat().st_size, 'sha256': digest(path, MAX_BINARY)})
    return {'directory': str(directory) if directory else None, 'files': rows,
            'environment': {name: environment.get(name) for name in
                            ('LD_LIBRARY_PATH', 'VK_DRIVER_FILES', 'VK_ICD_FILENAMES', 'VK_LAYER_PATH')},
            'scope': 'explicit loader directory and selected environment; not a complete transitive system-library inventory'}


def _empty_errors(record: dict, output: Path, name: str) -> None:
    check_success(record)
    if record.get('stderr_sha256') != EMPTY_SHA256:
        raise ValueError('checker stderr must be empty')
    if (output / f'{name}.supervisor.stderr.log').stat().st_size:
        raise ValueError('supervisor stderr must be empty')
    if record.get('capture_gate_required') or record.get('capture_gate_granted'):
        raise ValueError('worker capture gate is not part of the direct glyph checker')


def run_host(binary: Path, output: Path, fixtures: Path, baselines: Path,
             loader: Path | None = None, timeout: float = 60.0, cpu_check: bool = False) -> dict:
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError('timeout must be finite and in 0.05..120 seconds')
    if output.exists():
        raise ValueError('output directory must be new; retained evidence is not overwritten')
    binary, fixtures, baselines = binary.resolve(), fixtures.resolve(), baselines.resolve()
    binary_sha = digest(binary, MAX_BINARY)
    fixture_binding = glyph_protocol.load_fixtures(fixtures)['bindings']
    baseline_binding = glyph_protocol.check_baselines(baselines, BASELINE_MANIFEST_SHA256, fixtures)
    env = child_environment(loader)
    for key in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    loader_record = loader_binding(loader, env)
    source_names = ('run_glyph_host.py', 'glyph_protocol.py', 'run_browser_host.py',
                    'run_host.py', 'bridge_supervisor.py')
    source_bindings = {name: digest(ROOT / name, 1024 * 1024) for name in source_names}
    output.mkdir(parents=True)
    summary = {'schema': 1, 'success': False, 'mode': 'cpu-check' if cpu_check else 'gpu',
               'runs': [], 'adapter_count': 0, 'binary': str(binary), 'binary_sha256': binary_sha,
               'fixture_directory': str(fixtures), 'fixture_bindings': fixture_binding,
               'baseline_directory': str(baselines), 'baselines': baseline_binding,
               'loader': loader_record, 'host_source_bindings': source_bindings,
               'timeout_seconds': timeout, 'worker_cases': 0,
               'scope': 'offscreen direct lists; independent literal masks and parent-CPU-derived font references; no worker text capture, native-window or performance claim'}

    def persist():
        temporary = output / 'host-results.json.tmp'
        temporary.write_text(json.dumps(summary, indent=2) + '\n')
        temporary.replace(output / 'host-results.json')

    def unchanged():
        if digest(binary, MAX_BINARY) != binary_sha:
            raise ValueError('checker binary changed')
        if glyph_protocol.load_fixtures(fixtures)['bindings'] != fixture_binding:
            raise ValueError('frozen fixture binding changed')
        if glyph_protocol.check_baselines(baselines, BASELINE_MANIFEST_SHA256, fixtures) != baseline_binding:
            raise ValueError('parent reference binding changed')
        if loader_binding(loader, env) != loader_record:
            raise ValueError('explicit loader binding changed')
        if any(digest(ROOT / name, 1024 * 1024) != sha for name, sha in source_bindings.items()):
            raise ValueError('host/supervisor source changed')

    def execute(name: str, arguments: list[str]):
        unchanged()
        record, stdout = run_supervised([str(binary), *arguments], output, name, env, timeout)
        summary['runs'].append(record)
        persist()
        _empty_errors(record, output, name)
        unchanged()
        return record, stdout

    persist()
    try:
        if cpu_check:
            record, stdout = execute('cpu-check', ['--cpu-check', '--baselines', str(baselines)])
            record['validation'] = glyph_protocol.validate_cpu(stdout, fixtures)
        else:
            record, stdout = execute('enumeration', ['--list'])
            adapters = glyph_protocol.validate_listing(stdout)
            record['validation'] = {'adapters': adapters}
            summary['adapter_count'] = len(adapters)
            persist()
            for index in range(len(adapters)):
                record, stdout = execute(f'adapter-{index}', ['--adapter', str(index), '--baselines', str(baselines)])
                record['validation'] = glyph_protocol.validate_run(stdout, adapters, index, fixtures)
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
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fixtures', type=Path, default=ROOT / 'glyph-fixtures')
    parser.add_argument('--baselines', type=Path, required=True)
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
        result = run_host(args.binary, args.output, args.fixtures, args.baselines,
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
