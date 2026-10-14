#!/usr/bin/env python3
"""Check ten frozen rounded Color core scenes on installed Vulkan adapters."""
from __future__ import annotations

import argparse
import json
import re
import signal
from pathlib import Path

from run_surface_host import run_host as run_surface_host

# width,height,masks,coverage,rows,box_work,cpu_work,lowered,known total invocations
CASES = {
    'core-radius1': (512, 513, 4, 2046, 1024, 791114, 791114, 5, 811008),
    'core-alpha128': (512, 513, 4, 2046, 1024, 791114, 791114, 5, 811008),
    'core-opacity-half': (512, 513, 4, 2046, 1024, 791114, 1316426, 7, None),
    'core-alpha128-opacity-half': (512, 513, 4, 2046, 1024, 791114, 1316426, 7, None),
    'core-suppressed': (512, 513, 4, 2046, 1024, 791114, 1316426, 7, None),
    'core-full-viewport': (1280, 1024, 6, 4604, 2048, 3938888, 3938888, 7, 3969024),
    'core-fractional-clip': (514, 514, 2, 1025, 513, 791073, 791073, 3, None),
    'core-seven-tiles': (1280, 205, 7, 226063, 613, 1032096, 1032096, 8, None),
    'old-cap-new-admission': (1280, 205, 6, 2966, 410, 790588, 790588, 7, None),
    'core-page-padding-box': (512, 513, 4, 2046, 1024, 791114, 1053770, 6, 1077248),
}
FORMATS = ('Bgra8Unorm', 'Rgba8Unorm')
COMPARED_BYTES = 29_405_216
PASS = re.compile(
    r'PASS ([a-z0-9-]+) format=(Bgra8Unorm|Rgba8Unorm) '
    r'width=(\d+) height=(\d+) draws=(\d+) total_invocations=(\d+) '
    r'conversion_invocations=(\d+) planned_bytes=(\d+) compared_bytes=(\d+) '
    r'cores=(\d+) masks=(\d+) coverage=(\d+) rows=(\d+) box_work=(\d+) '
    r'cpu_work=(\d+) lowered=(\d+) exact=true reference=literal canvas=true '
    r'packed_native=true rounded_color_core=true')


def validate_run(stdout: bytes | str, adapters: list[str], selected: int) -> dict:
    if isinstance(stdout, bytes):
        stdout = stdout.decode('utf-8', errors='strict')
    if not 1 <= len(adapters) <= 16 or not 0 <= selected < len(adapters):
        raise ValueError('selected adapter outside frozen inventory')
    lines = stdout.splitlines()
    if lines[:len(adapters)] != adapters:
        raise ValueError('adapter inventory changed')
    rows = lines[len(adapters):]
    if len(rows) != 21:
        raise ValueError('rounded-core comparison record count')
    observed, total = set(), 0
    for line in rows[:-1]:
        match = PASS.fullmatch(line)
        if not match:
            raise ValueError('malformed rounded-core comparison record')
        name, fmt, *fields = match.groups()
        if name not in CASES or (name, fmt) in observed:
            raise ValueError('unknown or duplicate rounded-core case')
        (width, height, draws, invocations, conversion, planned, compared,
         cores, masks, coverage, row_entries, box_work, cpu_work, lowered) = map(int, fields)
        expected = CASES[name]
        if (width, height, masks, coverage, row_entries, box_work, cpu_work, lowered) != expected[:8]:
            raise ValueError('rounded-core literal geometry or counters changed')
        if cores != 1 or compared != width * height * 4:
            raise ValueError('rounded-core count or comparison bytes changed')
        if expected[8] is not None and invocations != expected[8]:
            raise ValueError('rounded-core frozen invocation total changed')
        expected_conversion = ((width + 7) // 8) * ((height + 7) // 8) * 64
        # The plan total already includes this reserved conversion work.
        if conversion != expected_conversion or not conversion <= invocations <= 4_000_000:
            raise ValueError('rounded-core work outside native bounds')
        if not 1 <= draws <= 257:
            raise ValueError('rounded-core draw count outside native bounds')
        padded = ((width * 4 + 255) // 256) * 256 * height
        minimum = width * height * 4 + padded + 16 + draws * 256
        if not minimum <= planned <= 16 * 1024 * 1024 or planned + padded > 24 * 1024 * 1024:
            raise ValueError('rounded-core storage outside native bounds')
        observed.add((name, fmt))
        total += compared
    if observed != {(name, fmt) for name in CASES for fmt in FORMATS} or total != COMPARED_BYTES:
        raise ValueError('missing rounded-core comparison or literal byte mismatch')
    expected_end = (
        f'COMPLETE adapter={selected} fixtures=10 frames=20 compared_bytes=29405216 '
        'exact=true rounded_color_core=true custom_rectangle_glyph_shaders=true '
        'offscreen=true acquired_surface=false')
    if rows[-1] != expected_end:
        raise ValueError('rounded-core completion mismatch')
    return {'fixtures': 10, 'frames': 20, 'formats': 2,
            'gpu_compared_bytes': total, 'independent_literal_bytes': total,
            'real_page_fixtures': 1, 'acquired_surface': False}


def run_host(binary: Path, output: Path, loader: Path | None, timeout: float = 60) -> dict:
    return run_surface_host(binary, output, loader, timeout,
                            validator=validate_run, extra_helpers=('run_box_core_host.py',))


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
