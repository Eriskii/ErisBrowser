#!/usr/bin/env python3
"""Supervise every Vulkan adapter for fixed literal opacity and scratch-reuse checks."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import signal

from run_browser_host import MAX_OUTPUT, check_success, digest, run_supervised
from run_glyph_host import loader_binding
from run_host import child_environment, inventory

ROOT = Path(__file__).resolve().parent
MAX_BINARY = 128 * 1024 * 1024
LITERAL_SHA256 = '3e879cf28127f6bc1400b3c26c481bb5613fc1a4e75b024ea195faaaa6f0d764'
TRANSPARENT_SHA256 = '7e7f9a2cde6df607daf0b1f0bfd8a31ed8219c04fdddd27e52e503e77ad7ef03'
FULL_SHA256 = '32339cbfd3204e9f9b40e227b1d7166b66815e2fd61d31f974a1c9211d6565c2'
EMPTY_SHA256 = hashlib.sha256(b'').hexdigest()
FORMATS = ('Bgra8Unorm', 'Rgba8Unorm')
# Name, scratch bytes, complete active-pixel bytes. Literal frame geometry and
# conservative descendant unions fix these independently of candidate output.
LITERALS = (
    ('halfway-red-65-over-black', 8, 12),
    ('opaque-backed-overlap-and-after-pop', 96, 64),
    ('nested-opaque-blue-over-red', 32, 16),
    ('quarter-group-with-translucent-image-on-white', 16, 12),
    ('coverage-times-alpha-truncates-before-group-pop', 16, 12),
    ('fixed-escape-stays-inside-opaque-backing', 80, 48),
    ('unit-scope-preserves-current-direct-rgb8-target', 0, 4),
    ('fractional-caller-clip-integer-origin-containment', 8, 12),
    ('multiple-rect-group-without-first-covering-backing', 24, 16),
    ('translucent-image-only-group', 8, 8),
    ('zero-scope-suppresses-pixels-and-restores-root', 0, 8),
    ('non-grid-point-one-white-over-black', 8, 4),
    ('transparent-black-alpha55-three-quarters-root-rounding', 8, 4),
    ('inner-rounding-survives-outer-pop', 16, 4),
    ('transparent-parent-retains-partial-alpha', 16, 4),
    ('opaque-islands-with-transparent-hole', 24, 12),
    ('transparent-glyph-coverage-without-backing', 16, 12),
    ('zero-alpha-image-preserves-root', 8, 8),
    ('decimal-point-one-is-not-nearest-grid', 8, 4),
    ('decimal-point-seven-needs-source-product-rounding', 8, 4),
    ('smallest-subnormal', 8, 4),
    ('largest-subnormal', 8, 4),
    ('smallest-normal', 8, 4),
    ('tiny-identity-threshold', 8, 4),
    ('next-above-identity-threshold', 8, 4),
    ('largest-value-below-unit-retains-layer-semantics', 8, 4),
    ('raw-nested-point-one-over-point-seven', 16, 4),
    ('raw-inner-point-one-under-grid-half', 16, 4),
    ('grid-inner-half-under-raw-point-one', 16, 4),
)
# All reuse targets are 4x2. Six ordered draws mean 1536 parameter bytes;
# root 32 + inputs 20 + conversion 528 + scratch 32/48 => 2148/2164 bytes.
REUSE_SEQUENCE = (('a', False), ('b', True), ('c', True), ('a', True), ('resized', False))
FIRST_REUSE_DRAWS = 6  # root clear, group clear, backing, image, glyph, composite
CANCEL_CHECKPOINTS = FIRST_REUSE_DRAWS + 2  # initial and conversion guards
REUSE = re.compile(
    r'PASS reuse-(opacity-reuse|opacity-transparent-reuse|opacity-full-reuse)-(a|b|c|resized) format=(Bgra8Unorm|Rgba8Unorm) '
    r'width=4 height=2 reused=(true|false) planned_bytes=(2148|2164) '
    r'compared_bytes=32 reference=literal exact=true')


def _lines(stdout: bytes) -> list[str]:
    if not isinstance(stdout, bytes) or not 0 < len(stdout) <= MAX_OUTPUT:
        raise ValueError('stdout size bound')
    if not stdout.endswith(b'\n') or b'\r' in stdout or b'\0' in stdout:
        raise ValueError('stdout must contain complete LF records')
    lines = stdout[:-1].decode('utf-8', errors='strict').split('\n')
    # At most16 adapters are listed by each of4 sessions, plus58 literal,
    # 3*(8 cancellation+14 reuse) and1 completion records:189 lines.
    if len(lines) > 192 or any(not line or len(line) > 4096 for line in lines):
        raise ValueError('stdout record bound')
    return lines


def validate_listing(stdout: bytes) -> list[str]:
    lines = _lines(stdout)
    adapters = inventory(lines)
    if lines != adapters:
        raise ValueError('unexpected adapter listing output')
    return adapters


def expected_reuse_sequence() -> list[tuple[str, str, bool]]:
    rows = []
    for fmt, other in ((FORMATS[0], FORMATS[1]), (FORMATS[1], FORMATS[0])):
        rows.extend((name, fmt, reused) for name, reused in REUSE_SEQUENCE)
        rows.append(('resized', other, False))
    rows.extend((('resized', FORMATS[0], False), ('resized', FORMATS[0], True)))
    return rows


def validate_run(stdout: bytes, adapters: list[str], selected: int) -> dict:
    if inventory(adapters) != adapters or type(selected) is not int or not 0 <= selected < len(adapters):
        raise ValueError('invalid selected adapter inventory')
    lines = _lines(stdout)
    count = len(adapters)
    if lines[:count] != adapters:
        raise ValueError('literal adapter inventory changed')
    rows = lines[count:]
    expected_literals = [
        f'PASS opacity-{name} format={fmt} scratch_bytes={scratch} compared_bytes={size} exact=true reference=literal'
        for name, scratch, size in LITERALS for fmt in FORMATS
    ]
    if rows[:58] != expected_literals:
        raise ValueError('literal case/format/route/byte sequence changed')
    rows = rows[58:]
    expected_reuse = expected_reuse_sequence()
    per_reuse = count + CANCEL_CHECKPOINTS + len(expected_reuse)
    if len(rows) != 3 * per_reuse + 1:
        raise ValueError('opacity record count')
    for prefix in ('opacity-reuse', 'opacity-transparent-reuse', 'opacity-full-reuse'):
        # Each corpus creates a fresh Vulkan instance/device. Retain and check
        # all three enumerations and every cut, including both additive corpora.
        if rows[:count] != adapters:
            raise ValueError('reuse adapter inventory changed')
        rows = rows[count:]
        for cut in range(1, CANCEL_CHECKPOINTS + 1):
            if rows[cut - 1] != (f'PASS cancellation checkpoint={cut} flush_submissions=1 '
                                 'retired=true reusable=false scopes=3'):
                raise ValueError('cancellation retirement/scopes mismatch')
        start = CANCEL_CHECKPOINTS
        end = start + len(expected_reuse)
        for line, wanted in zip(rows[start:end], expected_reuse, strict=True):
            match = REUSE.fullmatch(line)
            if not match:
                raise ValueError('malformed reuse evidence')
            actual_prefix, name, fmt, reused, planned = match.groups()
            if actual_prefix != prefix or (name, fmt, reused == 'true') != wanted:
                raise ValueError('reuse sequence/cache decision changed')
            if int(planned) != (2164 if name == 'resized' else 2148):
                raise ValueError('reuse exact scratch accounting changed')
        rows = rows[end:]
    completion = (f'COMPLETE adapter={selected} literal_frames=58 literal_bytes=608 refusals=0 '
                  'reuse_frames=14 reuse_bytes=448 allocations=7 reuses=7 evictions=7 '
                  'transparent_reuse_frames=14 transparent_reuse_bytes=448 transparent_allocations=7 '
                  'transparent_reuses=7 transparent_evictions=7 '
                  'full_reuse_frames=14 full_reuse_bytes=448 full_allocations=7 '
                  'full_reuses=7 full_evictions=7 '
                  'offscreen=true acquired_surface=false exact=true')
    if rows[-1] != completion:
        raise ValueError('opacity completion mismatch')
    return {'literal_frames': 58, 'literal_bytes': 608, 'literal_cases': 29, 'formats': 2,
            'reuse_frames': 14, 'reuse_bytes': 448, 'allocations': 7, 'reuses': 7,
            'evictions': 7, 'cancellation_checkpoints': CANCEL_CHECKPOINTS,
            'transparent_reuse_frames': 14, 'transparent_reuse_bytes': 448,
            'transparent_allocations': 7, 'transparent_reuses': 7, 'transparent_evictions': 7,
            'transparent_cancellation_checkpoints': CANCEL_CHECKPOINTS,
            'full_reuse_frames': 14, 'full_reuse_bytes': 448,
            'full_allocations': 7, 'full_reuses': 7, 'full_evictions': 7,
            'full_cancellation_checkpoints': CANCEL_CHECKPOINTS,
            'planner_refusals': 0, 'reference': 'literal', 'acquired_surface': False,
            'scope': 'offscreen Native GPU pixels and explicit lease events; no native-window or speed claim'}


def run_host(binary: Path, output: Path, loader: Path | None, timeout: float = 60) -> dict:
    if not math.isfinite(timeout) or not 0.05 <= timeout <= 60:
        raise ValueError('timeout must be finite and in 0.05..60 seconds')
    if output.exists():
        raise ValueError('output directory must be new; raw evidence is never overwritten')
    binary = binary.resolve()
    binary_sha = digest(binary, MAX_BINARY)
    env = child_environment(loader)
    for key in ('PYTHONOPTIMIZE', 'PYTHONPATH', 'PYTHONHOME'):
        env.pop(key, None)
    loader_record = loader_binding(loader, env)
    names = ('run_opacity_host.py', 'run_browser_host.py', 'run_glyph_host.py',
             'run_host.py', 'bridge_supervisor.py', 'browser_protocol.py', 'glyph_protocol.py',
             'src/bin/opacity-check.rs', 'src/opacity_fixtures.rs', 'src/reuse_gpu.rs',
             'src/surface_gpu.rs', 'opacity-fixtures/literal-fixtures.json',
             'opacity-fixtures/transparent-fixtures.json', 'opacity-fixtures/full-fixtures.json')
    bindings = {name: digest(ROOT / name, 1024 * 1024) for name in names}
    if bindings['opacity-fixtures/literal-fixtures.json'] != LITERAL_SHA256:
        raise ValueError('independent opacity fixture changed')
    if bindings['opacity-fixtures/transparent-fixtures.json'] != TRANSPARENT_SHA256:
        raise ValueError('independent transparent opacity fixture changed')
    if bindings['opacity-fixtures/full-fixtures.json'] != FULL_SHA256:
        raise ValueError('independent full opacity fixture changed')
    output.mkdir(parents=True)
    result = {'schema': 1, 'success': False, 'binary': str(binary), 'binary_sha256': binary_sha,
              'loader': loader_record, 'source_bindings': bindings, 'runs': [],
              'adapter_count': 0, 'timeout_seconds': timeout,
              'scope': 'offscreen Native opacity only; source bindings do not independently attest the executable build'}

    def persist():
        temporary = output / 'host-results.json.tmp'
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(output / 'host-results.json')

    def unchanged():
        if digest(binary, MAX_BINARY) != binary_sha or loader_binding(loader, env) != loader_record:
            raise ValueError('checker or explicit loader binding changed')
        if any(digest(ROOT / name, 1024 * 1024) != sha for name, sha in bindings.items()):
            raise ValueError('bound helper/checker/fixture source changed')

    def execute(name: str, arguments: list[str]):
        unchanged()
        record, stdout = run_supervised([str(binary), *arguments], output, name, env, timeout)
        result['runs'].append(record)
        persist()  # Retain the failed exit and raw logs before checking success.
        check_success(record)
        if record.get('stderr_sha256') != EMPTY_SHA256:
            raise ValueError('checker stderr must be empty')
        if (output / f'{name}.supervisor.stderr.log').stat().st_size:
            raise ValueError('supervisor stderr must be empty')
        if any(record.get(key) for key in ('capture_gate_required', 'capture_gate_granted',
                                           'worker_text_gate_required', 'worker_text_gate_granted')):
            raise ValueError('direct opacity checker has no worker capture gate')
        unchanged()
        return record, stdout

    persist()
    try:
        record, stdout = execute('enumeration', ['--list'])
        adapters = validate_listing(stdout)
        record['validation'] = {'adapters': adapters}
        result['adapter_count'] = len(adapters)
        persist()
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
