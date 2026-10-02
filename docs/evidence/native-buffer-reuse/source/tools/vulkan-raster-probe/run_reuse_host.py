#!/usr/bin/env python3
"""Run the fixed native reuse checker through the existing bounded supervisor."""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from run_surface_host import run_host

PASS = re.compile(r'PASS reuse-([a-z-]+) format=(Bgra8Unorm|Rgba8Unorm) width=(\d+) height=(\d+) reused=(true|false) planned_bytes=(\d+) compared_bytes=(\d+) reference=literal exact=true')
SEQUENCE = (
    ('inputs-a', 4, 3, False), ('inputs-b', 4, 3, True), ('inputs-c', 4, 3, True),
    ('inputs-a', 4, 3, True), ('inputs-b', 4, 3, True), ('inputs-c', 4, 3, True),
    ('rectangle-a', 4, 3, False), ('rectangle-b', 4, 3, True),
    ('clear-a', 4, 3, False), ('hidden-clear', 4, 3, True),
    ('resize-width', 5, 3, False), ('resize-height', 5, 4, False),
)


def expected_sequence():
    result = []
    for fmt, other in [('Bgra8Unorm', 'Rgba8Unorm'), ('Rgba8Unorm', 'Bgra8Unorm')]:
        result.extend((name, fmt, width, height, reused)
                      for name, width, height, reused in SEQUENCE)
        result.append(('resize-height', other, 5, 4, False))
    result.extend([('resize-height', 'Bgra8Unorm', 5, 4, False),
                   ('resize-height', 'Bgra8Unorm', 5, 4, True)])
    return result


def validate_run(stdout: bytes | str, adapters: list[str], selected: int) -> dict:
    if isinstance(stdout, bytes):
        stdout = stdout.decode('utf-8', errors='strict')
    lines = stdout.splitlines()
    if lines[:len(adapters)] != adapters:
        raise ValueError('adapter inventory changed')
    rows = lines[len(adapters):]
    expected = expected_sequence()
    if len(rows) != 6 + len(expected) + 1:
        raise ValueError('reuse comparison record count')
    for cut in range(1, 7):
        if rows[cut - 1] != (f'PASS cancellation checkpoint={cut} flush_submissions=1 '
                             'retired=true reusable=false scopes=3'):
            raise ValueError('reuse cancellation retirement record')
    total, reused_count = 0, 0
    for line, wanted in zip(rows[6:-1], expected, strict=True):
        match = PASS.fullmatch(line)
        if not match:
            raise ValueError('malformed reuse comparison')
        name, fmt, width, height, reused, planned, compared = match.groups()
        width, height, planned, compared = map(int, [width, height, planned, compared])
        if (name, fmt, width, height, reused == 'true') != wanted:
            raise ValueError('reuse sequence or cache decision changed')
        padded = 256 * height
        if compared != width * height * 4 or not compared + padded + 272 <= planned <= 16 * 1024 * 1024:
            raise ValueError('reuse byte accounting')
        if planned + padded > 24 * 1024 * 1024:
            raise ValueError('reuse active allocation cap')
        total += compared
        reused_count += reused == 'true'
    if total != 1560 or reused_count != 15:
        raise ValueError('reuse fixture totals')
    if rows[-1] != (f'COMPLETE adapter={selected} frames=28 allocations=13 reuses=15 '
                    'evictions=13 formats=2 compared_bytes=1560 offscreen=true '
                    'acquired_surface=false exact=true'):
        raise ValueError('reuse completion mismatch')
    return {'frames': 28, 'allocations': 13, 'reuses': 15, 'evictions': 13,
            'cancellation_cutpoints': 6, 'gpu_compared_bytes': total,
            'reference': 'literal', 'acquired_surface': False}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--loader-directory', type=Path)
    parser.add_argument('--timeout', type=float, default=30)
    args = parser.parse_args()
    result = run_host(args.binary, args.output, args.loader_directory, args.timeout,
                      validator=validate_run, extra_helpers=('run_reuse_host.py',))
    print(json.dumps({'success': result['success'], 'adapter_count': result['adapter_count'],
                      'results': str(args.output / 'host-results.json')}))
    return 0 if result['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
