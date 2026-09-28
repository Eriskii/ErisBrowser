#!/usr/bin/env python3
"""Compare actual parser trees to pinned WPT data, retaining every unsupported case."""
import argparse
import concurrent.futures
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
SECTIONS = {'#errors', '#new-errors', '#document-fragment', '#script-off', '#script-on', '#document'}


def paths_alias(first, second):
    return (first.resolve() == second.resolve()
            or first.exists() and second.exists() and first.samefile(second))


def parse_cases(text, filename):
    # splitlines()/universal-newline decoding would corrupt the CR input cases.
    lines = text.split('\n')
    cursor = 0
    number = 0
    cases = []
    while cursor < len(lines):
        if not lines[cursor]:
            cursor += 1
            continue
        if lines[cursor] != '#data':
            raise ValueError(f'{filename}:{cursor + 1}: expected #data')
        number += 1
        cursor += 1
        start = cursor
        while cursor < len(lines) and lines[cursor] != '#errors':
            cursor += 1
        if cursor == len(lines):
            raise ValueError(f'{filename}:{number}: missing #errors')
        source = '\n'.join(lines[start:cursor])
        sections = {}
        while cursor < len(lines) and lines[cursor] != '#document':
            header = lines[cursor]
            if header not in SECTIONS:
                raise ValueError(f'{filename}:{cursor + 1}: unknown section {header}')
            cursor += 1
            start = cursor
            while cursor < len(lines) and lines[cursor] not in SECTIONS:
                cursor += 1
            sections[header] = lines[start:cursor]
        if cursor == len(lines):
            raise ValueError(f'{filename}:{number}: missing #document')
        cursor += 1
        start = cursor
        while cursor < len(lines) and not (lines[cursor] == '#data' and cursor > start and not lines[cursor - 1]):
            cursor += 1
        expected = lines[start:cursor]
        if expected and not expected[-1]:
            expected.pop()  # exactly one separator/final LF, never trim node data
        expected = '\n'.join(expected) + ('\n' if expected else '')
        modes = ['disabled'] if '#script-off' in sections else ['enabled'] if '#script-on' in sections else ['disabled', 'enabled']
        for mode in modes:
            cases.append(dict(id=f'{filename}:{number}:{mode}', file=filename, case=number,
                              scripting=mode, source=source, expected=expected,
                              fragment='\n'.join(sections['#document-fragment']) if '#document-fragment' in sections else None,
                              expected_parse_errors=len(sections.get('#errors', [])) + len(sections.get('#new-errors', []))))
    return number, cases


def load_corpus(directory):
    manifest = json.loads((directory / 'manifest.json').read_text())
    cases = []
    source_count = 0
    for entry in manifest['files']:
        path = Path(entry['path'])
        if path.is_absolute() or '..' in path.parts:
            raise ValueError('unsafe path in corpus manifest')
        data = (directory / path).read_bytes()
        if len(data) != entry['bytes'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise ValueError(f'upstream corpus integrity mismatch: {path}')
        if path.suffix == '.dat':
            count, parsed = parse_cases(data.decode('utf-8'), str(path))
            source_count += count
            cases.extend(parsed)
    if not cases or len({case['id'] for case in cases}) != len(cases):
        raise ValueError('empty corpus or duplicate case identifiers')
    return manifest, source_count, cases


def case_fingerprint(case):
    fields = {key: case[key] for key in ('source', 'expected', 'fragment', 'scripting', 'expected_parse_errors')}
    return hashlib.sha256(json.dumps(fields, sort_keys=True, ensure_ascii=True).encode()).hexdigest()


def check_baseline(previous, current):
    if previous['revision'] != current['revision']:
        raise ValueError('baseline uses another upstream revision; explicitly record a new baseline')
    if previous.get('corpus_manifest_sha256') != current['corpus_manifest_sha256']:
        raise ValueError('baseline/corpus manifest fingerprint differs')
    if previous['cases'].keys() != current['cases'].keys():
        raise ValueError('baseline/corpus case inventory differs')
    regressions, improvements = [], []
    for name, old in previous['cases'].items():
        new = current['cases'][name]
        if old.get('case_sha256') != new['case_sha256']:
            raise ValueError(f'baseline input, expectation or context differs for {name}')
        if old['status'] == 'matched' and new['status'] != 'matched':
            regressions.append(name)
        if old['status'] != 'matched' and new['status'] == 'matched':
            improvements.append(name)
    return regressions, improvements


def bounded_process(command, payload, timeout, stdout_limit=32 * 1024 * 1024, stderr_limit=64 * 1024):
    """Multiplex POSIX pipes so input, noisy output and timeouts all stay bounded."""
    output, errors = bytearray(), bytearray()
    with subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
        with selectors.DefaultSelector() as selector:
            try:
                for stream, name in ((process.stdin, 'input'), (process.stdout, 'output'), (process.stderr, 'errors')):
                    os.set_blocking(stream.fileno(), False)
                    selector.register(stream, selectors.EVENT_WRITE if name == 'input' else selectors.EVENT_READ, name)
                position = 0
                deadline = time.monotonic() + timeout
                while selector.get_map():
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise subprocess.TimeoutExpired(command, timeout)
                    for key, _ in selector.select(min(remaining, 0.1)):
                        stream = key.fileobj
                        if key.data == 'input':
                            try:
                                position += os.write(stream.fileno(), payload[position:position + 65536]) if position < len(payload) else 0
                            except BrokenPipeError:
                                position = len(payload)
                            except BlockingIOError:
                                continue
                            if position == len(payload):
                                selector.unregister(stream)
                                stream.close()
                            continue
                        try:
                            chunk = os.read(stream.fileno(), 65536)
                        except BlockingIOError:
                            continue
                        if not chunk:
                            selector.unregister(stream)
                            stream.close()
                            continue
                        target, limit = (output, stdout_limit) if key.data == 'output' else (errors, stderr_limit)
                        if len(target) + len(chunk) > limit:
                            raise ValueError(f'adapter {key.data} exceeds capture limit')
                        target.extend(chunk)
                status = process.wait(timeout=max(0.001, deadline - time.monotonic()))
                return status, bytes(output), bytes(errors)
            finally:
                if process.poll() is None:
                    process.kill()
                process.wait()


def run_case(case, binary, timeout, legacy):
    result = {key: case[key] for key in ('id', 'file', 'case', 'scripting', 'expected_parse_errors')}
    result['source_sha256'] = hashlib.sha256(case['source'].encode()).hexdigest()
    result['case_sha256'] = case_fingerprint(case)
    if case['fragment'] is not None:
        return dict(result, status='unsupported', reason='fragment context parsing', context=case['fragment'])
    if Path(case['file']).name.startswith('scripted_'):
        return dict(result, status='unsupported', reason='synchronous parser script execution')
    if legacy and case['scripting'] == 'enabled':
        return dict(result, status='unsupported', reason='baseline parser has no scripting-flag API')
    command = [str(binary), '--scripting', case['scripting']]
    try:
        status, output, errors = bounded_process(command, case['source'].encode(), timeout)
        if status != 0:
            return dict(result, status='error', reason=f'exit {status}', stderr=errors.decode('utf-8', 'replace')[:4096])
        actual = output.decode('utf-8')
        if actual == case['expected']:
            return dict(result, status='matched')
        return dict(result, status='mismatched', expected=case['expected'], actual=actual)
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        return dict(result, status='error', reason=str(error))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/eris-dom')
    parser.add_argument('--corpus', type=Path, default=ROOT / 'tests/upstream/wpt-html')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/html-tree-report.json')
    parser.add_argument('--baseline', type=Path, help='fail on regressions from previously matching cases; still report all mismatches')
    parser.add_argument('--record-baseline', type=Path, help='explicitly record current per-case outcomes for later regression checks')
    parser.add_argument('--legacy-no-scripting-flag', action='store_true', help='measure the initial parser with unavailable flag mode explicitly unsupported')
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--timeout', type=float, default=3.0)
    args = parser.parse_args()
    if args.baseline and args.record_baseline:
        parser.error('--baseline and --record-baseline are mutually exclusive; a regression check never rewrites its baseline')
    if any(path is not None and paths_alias(path, args.output)
           for path in (args.baseline, args.record_baseline)):
        parser.error('report output must be separate from the baseline path')
    if not 1 <= args.jobs <= 16 or not 0 < args.timeout <= 60:
        parser.error('jobs must be 1..16 and timeout must be (0,60] seconds')
    started = time.monotonic()
    try:
        manifest, source_count, cases = load_corpus(args.corpus)
        binary = args.binary.resolve()
        binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            results = list(pool.map(lambda case: run_case(case, binary, args.timeout, args.legacy_no_scripting_flag), cases))
        counts = dict(Counter(case['status'] for case in results))
        if hashlib.sha256(binary.read_bytes()).hexdigest() != binary_hash:
            raise ValueError('adapter binary changed during the run; repeat with a stable build')
        corpus_hash = hashlib.sha256((args.corpus / 'manifest.json').read_bytes()).hexdigest()
        baseline = dict(revision=manifest['revision'], adapter_scope='DOM tree only', binary_sha256=binary_hash,
                        corpus_manifest_sha256=corpus_hash,
                        cases={case['id']: dict(status=case['status'], source_sha256=case['source_sha256'], case_sha256=case['case_sha256']) for case in results})
        regressions = []
        improvements = []
        if args.baseline:
            previous = json.loads(args.baseline.read_text())
            regressions, improvements = check_baseline(previous, baseline)
        report = dict(suite='pinned-wpt-html-tree-comparisons', full_wpt_conformance=False,
                      repository=manifest['repository'], revision=manifest['revision'], corpus_manifest_sha256=corpus_hash, source_cases=source_count,
                      mode_cases=len(results), counts=counts, seconds=round(time.monotonic() - started, 3),
                      measured=['exact DOM tree serialization'],
                      not_measured=['parse-error counts', 'document compatibility/quirks mode', 'encoding detection', 'fragment parsing', 'synchronous script execution', 'document.write input modes', 'full WPT testharness behavior'],
                      binary=str(binary), binary_sha256=binary_hash,
                      regressions=regressions, improvements=improvements, cases=results)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2, ensure_ascii=True) + '\n')
        if args.record_baseline and not counts.get('error'):
            args.record_baseline.parent.mkdir(parents=True, exist_ok=True)
            args.record_baseline.write_text(json.dumps(baseline, indent=2) + '\n')
        print(f"{source_count} upstream inputs; {len(results)} scripting-mode cases; {counts}")
        if args.baseline:
            print(f'{len(regressions)} regressions; {len(improvements)} newly matching trees')
        print(f'Report: {args.output}')
        if counts.get('error') or regressions:
            return 1
        if args.baseline or args.record_baseline:
            return 0
        return 0 if counts.get('matched', 0) == len(results) else 1
    except (OSError, KeyError, ValueError) as error:
        print(f'conformance runner: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
