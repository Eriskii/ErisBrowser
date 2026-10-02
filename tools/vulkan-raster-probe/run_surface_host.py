#!/usr/bin/env python3
"""Check bounded native-profile raster/conversion offscreen; no acquired surface."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import signal
from pathlib import Path

from glyph_protocol import validate_listing
from run_browser_host import check_success, digest, run_supervised
from run_glyph_host import loader_binding
from run_host import child_environment

ROOT = Path(__file__).resolve().parent
MAX_BINARY = 128 * 1024 * 1024
CASES = {
    'round-corner': (2, 2, 'literal-coverage'),
    'round-alpha-twice': (2, 2, 'literal-coverage'),
    'round-clamped': (2, 2, 'literal-coverage'),
    'round-quarter': (1, 1, 'literal-coverage'),
    'round-fractional': (2, 1, 'literal-coverage'),
    'round-clipped': (2, 1, 'literal-coverage'),
    'asymmetric-colors': (3, 2, 'literal-geometry'),
    'desktop-padded': (1180, 880, 'literal-geometry'),
    'native-maximum': (1280, 1024, 'literal-geometry'),
    'odd-width-last-pixel': (1279, 3, 'literal-geometry'),
    'desktop-address-bar': (1180, 880, 'Canvas-differential'),
}
FORMATS = ('Bgra8Unorm', 'Rgba8Unorm')
PASS = re.compile(r'PASS ([a-z0-9-]+) format=(Bgra8Unorm|Rgba8Unorm) width=(\d+) height=(\d+) draws=(\d+) raster_invocations=(\d+) conversion_invocations=(\d+) planned_bytes=(\d+) compared_bytes=(\d+) reference=([A-Za-z-]+) exact=true')


def validate_run(stdout: bytes | str, adapters: list[str], selected: int) -> dict:
    if isinstance(stdout, bytes):
        stdout = stdout.decode('utf-8', errors='strict')
    lines = stdout.splitlines()
    if lines[:len(adapters)] != adapters:
        raise ValueError('adapter inventory changed')
    rows = lines[len(adapters):]
    if len(rows) != len(CASES) * len(FORMATS) + 1:
        raise ValueError('surface comparison record count')
    observed, total, independent, differential = set(), 0, 0, 0
    for line in rows[:-1]:
        match = PASS.fullmatch(line)
        if not match:
            raise ValueError('malformed surface comparison record')
        name, fmt, *fields, reference = match.groups()
        if name not in CASES or (name, fmt) in observed:
            raise ValueError('unknown or duplicate surface case')
        width, height, draws, raster, conversion, planned, compared = map(int, fields)
        if (width, height, reference) != CASES[name] or compared != width * height * 4:
            raise ValueError('surface fixture dimensions/reference/bytes changed')
        expected_conversion = ((width + 7) // 8) * ((height + 7) // 8) * 64
        if conversion != expected_conversion or not conversion <= raster:
            raise ValueError('surface conversion/clear work mismatch')
        if raster + conversion > 4_000_000 or not 1 <= draws <= 257:
            raise ValueError('surface work outside native bounds')
        padded = ((width * 4 + 255) // 256) * 256 * height
        minimum = width * height * 4 + padded + 16 + draws * 256
        if not minimum <= planned <= 16 * 1024 * 1024 or planned + padded > 24 * 1024 * 1024:
            raise ValueError('surface storage outside native bounds')
        observed.add((name, fmt))
        total += compared
        if reference == 'Canvas-differential':
            differential += compared
        else:
            independent += compared
    if observed != {(name, fmt) for name in CASES for fmt in FORMATS}:
        raise ValueError('missing surface comparison')
    expected_end = (f'COMPLETE adapter={selected} cases={len(CASES)} formats=2 '
                    f'compared_bytes={total} offscreen=true acquired_surface=false exact=true')
    if rows[-1] != expected_end:
        raise ValueError('surface completion mismatch')
    return {'cases': len(CASES), 'formats': len(FORMATS), 'gpu_compared_bytes': total,
            'independent_literal_bytes': independent, 'Canvas_differential_bytes': differential,
            'acquired_surface': False}


def run_host(binary: Path, output: Path, loader: Path | None, timeout: float = 60) -> dict:
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 120:
        raise ValueError('timeout must be finite and in 0.05..120 seconds')
    if output.exists():
        raise ValueError('output directory must be new')
    binary = binary.resolve()
    binary_sha = digest(binary, MAX_BINARY)
    env = child_environment(loader)
    for key in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    loader_record = loader_binding(loader, env)
    names = ('run_surface_host.py', 'run_browser_host.py', 'run_glyph_host.py',
             'glyph_protocol.py', 'run_host.py', 'bridge_supervisor.py')
    bindings = {name: digest(ROOT / name, 1024 * 1024) for name in names}
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'binary': str(binary), 'binary_sha256': binary_sha,
              'loader': loader_record, 'host_source_bindings': bindings, 'runs': [],
              'adapter_count': 0, 'scope': 'offscreen raster and conversion only; no acquired native surface or performance claim'}

    def persist():
        temp = output / 'host-results.json.tmp'
        temp.write_text(json.dumps(result, indent=2) + '\n')
        temp.replace(output / 'host-results.json')

    def unchanged():
        if digest(binary, MAX_BINARY) != binary_sha or loader_binding(loader, env) != loader_record:
            raise ValueError('binary or explicit loader changed')
        if any(digest(ROOT / name, 1024 * 1024) != sha for name, sha in bindings.items()):
            raise ValueError('host or supervisor source changed')

    def execute(name, arguments):
        unchanged()
        record, stdout = run_supervised([str(binary), *arguments], output, name, env, timeout)
        result['runs'].append(record)
        persist()
        check_success(record)
        if record.get('stderr_sha256') != hashlib.sha256(b'').hexdigest():
            raise ValueError('checker stderr must be empty')
        if (output / f'{name}.supervisor.stderr.log').stat().st_size:
            raise ValueError('supervisor stderr must be empty')
        if record.get('capture_gate_required') or record.get('capture_gate_granted'):
            raise ValueError('surface checker has no worker capture gate')
        unchanged()
        return record, stdout

    persist()
    try:
        record, stdout = execute('enumeration', ['--list'])
        adapters = validate_listing(stdout)
        record['validation'] = {'adapters': adapters}
        result['adapter_count'] = len(adapters)
        for index in range(len(adapters)):
            record, stdout = execute(f'adapter-{index}', ['--adapter', str(index)])
            record['validation'] = validate_run(stdout, adapters, index)
            persist()
        unchanged()
        result['success'] = True
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, KeyboardInterrupt) as error:
        result['error'] = f'{type(error).__name__}: {error}'[:4096]
    finally:
        persist()
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=60)
    parser.add_argument('--allow-experimental-gpu', action='store_true')
    args = parser.parse_args()
    if not args.allow_experimental_gpu:
        parser.error('actual Vulkan execution requires --allow-experimental-gpu')
    previous = {}

    def interrupted(_signum, _frame):
        raise KeyboardInterrupt

    try:
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous[sig] = signal.signal(sig, interrupted)
        result = run_host(args.binary, args.output, args.loader_directory, args.timeout)
    except (OSError, ValueError) as error:
        parser.exit(1, f'{error}\n')
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    print(json.dumps({'success': result['success'], 'adapter_count': result['adapter_count'],
                      'results': str(args.output / 'host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
