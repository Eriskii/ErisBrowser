#!/usr/bin/env python3
"""Measure fresh confined worker startup/load and validated warm frame round trips."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import subprocess

from benchmark import CASES, ROOT, environment

PHASES = ('startup_ms', 'load_ms', 'cold_render_exchange_ms',
          'cold_clear_paint_ms', 'teardown_ms')
WARM_PHASES = ('render_exchange_ms', 'clear_paint_ms', 'total_ms')


def run_fixture(command, timeout=180):
    # Workers/brokers inherit this new process group. If a benchmark stalls,
    # terminate the whole group rather than leaving a confined child running.
    with subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          text=True, start_new_session=True) as process:
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt):
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate()
            raise
        return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def summarize(values):
    ordered = sorted(values)
    # Same upper-middle median and nearest-rank p95 as --benchmark.
    return dict(samples=len(ordered), median_ms=ordered[len(ordered) // 2],
                p95_ms=ordered[math.ceil(len(ordered) * .95) - 1])


def validate_run(run, iterations):
    if (not isinstance(run, dict) or type(run.get('schema')) is not int or run['schema'] != 1
            or not isinstance(run.get('viewport'), list)
            or any(type(value) is not int for value in run['viewport'])
            or run['viewport'] != [1180, 880] or run.get('scripts') is not True
            or type(run.get('iterations')) is not int or run['iterations'] != iterations):
        raise ValueError('unexpected worker benchmark configuration')
    for key in ('nodes', 'commands'):
        if type(run.get(key)) is not int or run[key] < 0:
            raise ValueError(f'invalid {key} count')
    samples = run.get('warm_samples')
    if not isinstance(samples, list) or len(samples) != iterations:
        raise ValueError('missing warm benchmark samples')
    for record, keys in [(run, PHASES), *((sample, WARM_PHASES) for sample in samples)]:
        if not isinstance(record, dict):
            raise ValueError('invalid benchmark sample')
        for key in keys:
            number = record.get(key)
            if type(number) not in (int, float) or not math.isfinite(number) or number < 0:
                raise ValueError(f'invalid {key} timing')
    for sample in samples:
        if sample['total_ms'] + .000002 < sample['render_exchange_ms'] + sample['clear_paint_ms']:
            raise ValueError('frame total does not include both measured phases')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/eris-browser')
    parser.add_argument('--iterations', type=int, default=100)
    parser.add_argument('--fresh-runs', type=int, default=5)
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/benchmark-worker.json')
    args = parser.parse_args()
    if not 1 <= args.iterations <= 10000 or not 1 <= args.fresh_runs <= 100:
        parser.error('iterations must be 1..10000 and fresh-runs must be 1..100')
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    metadata = environment(args.binary)
    cases = []
    for name, address in CASES:
        runs = []
        for index in range(args.fresh_runs):
            command = [str(args.binary.resolve()), address, '--benchmark-worker', str(args.iterations),
                       '--width', '1180', '--height', '880', '--output',
                       str(output.parent / f'benchmark-worker-{name}.png')]
            result = run_fixture(command)
            unexpected = [line for line in result.stderr.splitlines()
                          if not line.startswith(('[page] Page process ', '[page] Resource broker '))]
            if result.returncode != 0 or unexpected:
                raise RuntimeError(f'{name} run {index}: {result.stdout}\n{result.stderr}')
            run = json.loads(result.stdout)
            validate_run(run, args.iterations)
            runs.append(dict(**run, diagnostics=result.stderr.splitlines()))
        phases = {key: summarize([run[key] for run in runs]) for key in PHASES}
        warm = {key: summarize([sample[key] for run in runs for sample in run['warm_samples']])
                for key in WARM_PHASES}
        cases.append(dict(name=name, address=address, cold=phases, warm=warm, runs=runs))
        print(f"{name}: warm frame median {warm['total_ms']['median_ms']:.3f} ms; "
              f"p95 {warm['total_ms']['p95_ms']:.3f} ms")
    after = environment(args.binary)
    if any(metadata[key] != after[key] for key in ('input_sha256', 'binary_sha256')):
        raise RuntimeError('source inputs or binary changed during measurement')
    tool_hashes = {name: hashlib.sha256((ROOT / 'tools' / name).read_bytes()).hexdigest()
                   for name in ('benchmark_worker.py', 'benchmark.py')}
    report = dict(**metadata, schema=1, tool_sha256=tool_hashes,
                  viewport=[1180, 880], iterations=args.iterations,
                  fresh_runs=args.fresh_runs, cases=cases,
                  phase_definitions={
                      'startup_ms': 'address authorization + confined renderer spawn/handshake; parent fonts/canvas excluded',
                      'load_ms': 'Load round trip, including renderer fonts, broker startup, resource reads, HTML parse, script setup and any requested image decoding',
                      'cold_render_exchange_ms': 'first Render round trip: style/layout, document/image snapshots, IPC copy/validation',
                      'cold_clear_paint_ms': 'first parent canvas clear + software paint with fresh glyph cache',
                      'render_exchange_ms': 'warm Render round trip including snapshot generation/copy/validation',
                      'clear_paint_ms': 'parent canvas clear + software paint with warmed glyph cache',
                      'total_ms': 'warm render exchange + fragment lookup + clear/paint + snapshot destruction',
                      'teardown_ms': 'worker/broker channel closure, termination and wait',
                  },
                  excluded=['parent fonts/canvas setup', 'PNG encoding', 'desktop event loop and native presentation'],
                  cold_cache_definition='fresh processes and parent glyph cache; OS/filesystem caches are not flushed',
                  aggregation='upper-middle median and nearest-rank p95; warm samples pooled across fresh runs',
                  chromium_comparison=False, controlled_environment=False)
    output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Report: {output}')


if __name__ == '__main__':
    main()
