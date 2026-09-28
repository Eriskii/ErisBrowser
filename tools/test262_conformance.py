#!/usr/bin/env python3
"""Execute unchanged pinned Test262 scripts with Eris; retain every mode/result."""
import argparse
import concurrent.futures
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys
import time

from html_conformance import bounded_process, paths_alias
from import_test262 import DIRECTORIES, MAX_FILE, MAX_TOTAL, REPOSITORY, REVISION, parse_metadata

ROOT = Path(__file__).resolve().parents[1]
SUPPORTED_FEATURES = {'arrow-function', 'String.fromCodePoint', 'well-formed-json-stringify'}
INTRINSIC_ERRORS = {'Error', 'TypeError', 'RangeError', 'SyntaxError', 'ReferenceError', 'EvalError', 'URIError'}
KNOWN_FLAGS = {'onlyStrict', 'noStrict', 'module', 'raw', 'async', 'generated',
               'CanBlockIsFalse', 'CanBlockIsTrue', 'non-deterministic'}
BAD_RUN_STATUSES = {'adapter-error', 'timeout', 'resource'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def modes(metadata):
    flags = set(metadata['flags'])
    if 'module' in flags:
        return ['module']
    if 'raw' in flags:
        return ['raw']
    if 'onlyStrict' in flags:
        return ['strict']
    if 'noStrict' in flags:
        return ['sloppy']
    return ['sloppy', 'strict']


def harness_names(metadata):
    if 'raw' in metadata['flags']:
        return []
    names = ['assert.js', 'sta.js']
    if 'async' in metadata['flags']:
        names.append('doneprintHandle.js')
    return names + metadata['includes']


def load_corpus(directory):
    manifest_bytes = (directory / 'manifest.json').read_bytes()
    if len(manifest_bytes) > MAX_FILE:
        raise ValueError('manifest exceeds size limit')
    manifest = json.loads(manifest_bytes)
    if (manifest.get('format') != 1 or manifest.get('revision') != REVISION
            or manifest.get('repository') != f'https://github.com/{REPOSITORY}'):
        raise ValueError('unexpected Test262 corpus format, repository or pinned revision')
    inventory = manifest.get('directories', {})
    if inventory.keys() != DIRECTORIES.keys():
        raise ValueError('Test262 directory inventory differs from pinned selection')
    expected_tests = set()
    for name, count in DIRECTORIES.items():
        filenames = inventory[name]
        if (not isinstance(filenames, list) or len(filenames) != count
                or len(set(filenames)) != count
                or any(not re.fullmatch(r'[A-Za-z0-9_.-]+\.js', value) for value in filenames)):
            raise ValueError(f'pinned Test262 directory inventory mismatch: {name}')
        expected_tests.update(f'test/built-ins/{name}/{value}' for value in filenames)
    if manifest.get('test_files') != len(expected_tests):
        raise ValueError('Test262 test file count differs from pinned inventory')
    files = {}
    total = 0
    for entry in manifest['files']:
        path = Path(entry['path'])
        if (path.is_absolute() or '..' in path.parts or str(path) != entry['path']
                or entry['path'] != entry.get('upstream_path') or str(path) in files):
            raise ValueError('unsafe, duplicate or remapped path in Test262 manifest')
        target = directory / path
        if target.is_symlink() or directory.resolve() not in target.resolve().parents:
            raise ValueError('Test262 corpus symlink escapes or aliases pinned bytes')
        if not isinstance(entry['bytes'], int) or not 0 <= entry['bytes'] <= MAX_FILE:
            raise ValueError('invalid Test262 file size')
        with target.open('rb') as handle:
            data = handle.read(MAX_FILE + 1)
        total += len(data)
        if total > MAX_TOTAL or len(data) != entry['bytes'] or digest(data) != entry['sha256']:
            raise ValueError(f'Test262 corpus integrity mismatch: {path}')
        files[str(path)] = data
    actual_tests = {path for path in files if path.startswith('test/')}
    if actual_tests != expected_tests:
        raise ValueError('Test262 manifest test inventory is incomplete or contains extras')
    disk_scripts = {str(path.relative_to(directory)) for path in directory.rglob('*.js')}
    if disk_scripts != {path for path in files if path.endswith('.js')}:
        raise ValueError('unlisted or missing Test262 JavaScript files on disk')
    if not {'LICENSE', 'INTERPRETING.md', 'harness/assert.js', 'harness/sta.js'} <= files.keys():
        raise ValueError('missing Test262 license, interpretation rules or default harness')
    cases = []
    fixtures = []
    for path in sorted(expected_tests):
        if '_FIXTURE' in Path(path).name:
            fixtures.append(path)
            continue
        source = files[path].decode('utf-8')
        metadata = parse_metadata(source)
        names = harness_names(metadata)
        harness = []
        for name in names:
            key = f'harness/{name}'
            if key not in files:
                raise ValueError(f'missing requested Test262 harness file: {key}')
            harness.append((name, files[key]))
        for mode in modes(metadata):
            case = dict(id=f'{path}:{mode}', file=path, mode=mode,
                        source=files[path], metadata=metadata, harness=harness)
            case['case_sha256'] = case_fingerprint(case)
            cases.append(case)
    if not cases or len({case['id'] for case in cases}) != len(cases):
        raise ValueError('empty or duplicate Test262 case inventory')
    return manifest, files, cases, fixtures, digest(manifest_bytes)


def case_fingerprint(case):
    identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                    metadata=case['metadata'],
                    harness=[dict(name=name, sha256=digest(source)) for name, source in case['harness']])
    return digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode())


def unsupported_reason(case):
    metadata = case['metadata']
    flags = set(metadata['flags'])
    if case['mode'] == 'strict':
        return 'strict-mode execution is not implemented'
    if case['mode'] == 'module' or (metadata['negative'] or {}).get('phase') == 'resolution':
        return 'module parsing/resolution is not implemented'
    if metadata['negative'] and metadata['negative']['type'] not in INTRINSIC_ERRORS:
        return 'negative error constructor identity is not implemented: ' + metadata['negative']['type']
    if 'async' in flags:
        return 'asynchronous completion and jobs are not implemented'
    if flags - KNOWN_FLAGS:
        return 'unknown execution flags: ' + ', '.join(sorted(flags - KNOWN_FLAGS))
    if flags & {'CanBlockIsFalse', 'CanBlockIsTrue'}:
        return 'Test262 agent blocking policy is not implemented'
    if metadata['locale']:
        return 'locale-dependent execution is not implemented'
    if metadata.get('timeout'):
        return 'metadata-specific timeout policy is not implemented'
    unavailable = set(metadata['features']) - SUPPORTED_FEATURES
    if unavailable:
        return 'unimplemented declared features: ' + ', '.join(sorted(unavailable))
    sources = [case['source']] + [source for _, source in case['harness']]
    if any(re.search(rb'\$262\b|\$DONE\b|\bprint\s*\(', source) for source in sources):
        return 'Test262 host hooks are not implemented (conservative source check)'
    return None


def encode_request(case):
    output = bytearray(b'ERJS1')
    output.append({'sloppy': 0, 'strict': 1, 'raw': 2, 'module': 3}[case['mode']])
    output.append(int((case['metadata']['negative'] or {}).get('phase') == 'parse'))
    output.extend(struct.pack('<I', len(case['harness'])))
    def string(value):
        output.extend(struct.pack('<I', len(value)))
        output.extend(value)
    for name, source in case['harness']:
        string(name.encode())
        string(source)
    string(case['source'])
    if len(output) > 4 * 1024 * 1024:
        raise ValueError('test request exceeds adapter budget')
    return bytes(output)


def decode_response(output):
    if not output.startswith(b'ERJR2'):
        raise ValueError('invalid JS adapter response magic')
    offset = 5
    fields = {}
    for key in ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness'):
        if len(output) - offset < 4:
            raise ValueError('truncated JS adapter response length')
        length, = struct.unpack_from('<I', output, offset)
        offset += 4
        if length > 16384 or length > len(output) - offset:
            raise ValueError('invalid JS adapter response field length')
        fields[key] = output[offset:offset + length].decode('utf-8')
        offset += length
    if offset != len(output):
        raise ValueError('trailing JS adapter response data')
    status, phase = fields['status'], fields['phase']
    if status not in {'complete', 'exception', 'unsupported', 'resource', 'harness-error'}:
        raise ValueError('unknown JS adapter response status')
    if phase not in {'parse', 'runtime', 'harness', 'mode'}:
        raise ValueError('unknown JS adapter response phase')
    if fields['error_identity'] and fields['error_identity'] not in INTRINSIC_ERRORS:
        raise ValueError('unknown intrinsic error identity')
    if ((status == 'complete' and (phase not in {'parse', 'runtime'} or fields['error_type'] or fields['error_identity'] or fields['harness']))
            or (status == 'exception' and (phase not in {'parse', 'runtime'} or not fields['error_type'] or fields['harness']))
            or (status == 'harness-error' and (phase != 'harness' or not fields['harness']))):
        raise ValueError('inconsistent JS adapter response')
    return fields


def classify(case, observation):
    status = observation['status']
    expected = case['metadata']['negative']
    if status in {'unsupported', 'resource', 'harness-error'}:
        return status
    if expected is None:
        return 'passed' if status == 'complete' and observation['phase'] == 'runtime' else 'failed'
    return ('passed' if status == 'exception' and observation['phase'] == expected['phase']
            and observation['error_identity'] == expected['type'] else 'failed')


def run_case(case, binary, timeout):
    result = {key: case[key] for key in ('id', 'file', 'mode', 'case_sha256')}
    result['source_sha256'] = digest(case['source'])
    result['expected_negative'] = case['metadata']['negative']
    reason = unsupported_reason(case)
    if reason:
        return dict(result, status='unsupported', reason=reason)
    try:
        exit_code, output, stderr = bounded_process([str(binary)], encode_request(case), timeout,
                                                   stdout_limit=96 * 1024, stderr_limit=64 * 1024)
        if exit_code:
            return dict(result, status='adapter-error', reason=f'adapter exit {exit_code}',
                        stderr=stderr.decode('utf-8', 'replace')[:4096])
        observation = decode_response(output)
        return dict(result, status=classify(case, observation), observation=observation)
    except subprocess.TimeoutExpired:
        return dict(result, status='timeout', reason=f'adapter exceeded {timeout:g} seconds')
    except (OSError, ValueError) as error:
        return dict(result, status='adapter-error', reason=str(error))


def harness_preflight(files, binary, timeout):
    """Fail closed if the upstream assertions no longer enforce basic failures."""
    scripts = [
        ('success', 'assert.sameValue(1, 1); assert.sameValue(NaN, NaN);', 'passed'),
        ('assert-false', 'assert(false);', 'failed'),
        ('same-value', 'assert.sameValue(1, 2);', 'failed'),
        ('signed-zero', 'assert.sameValue(0, -0);', 'failed'),
        ('not-same-value', 'assert.notSameValue(1, 1);', 'failed'),
        ('throws-success', 'assert.throws(TypeError, function () { throw new TypeError(); });', 'passed'),
        ('throws-wrong-type', 'assert.throws(TypeError, function () { throw new RangeError(); });', 'failed'),
        ('throws-missing', 'assert.throws(TypeError, function () {});', 'failed'),
        ('array-mismatch', 'assert.compareArray([1], [2]);', 'failed'),
    ]
    outcomes = []
    for name, source, expected in scripts:
        case = dict(id=f'harness-preflight:{name}', file='<preflight>', mode='sloppy',
                    metadata=dict(flags=[], includes=[], features=[], locale=[], negative=None),
                    source=source.encode(), harness=[(name, files[f'harness/{name}']) for name in ('assert.js', 'sta.js')])
        case['case_sha256'] = case_fingerprint(case)
        result = run_case(case, binary, timeout)
        correct = result['status'] == expected
        if expected == 'failed':
            correct = correct and result.get('observation', {}).get('error_type') == 'Test262Error'
        outcomes.append(dict(name=name, expected=expected, verified=correct, result=result))
    return outcomes


def check_baseline(previous, current):
    for key in ('revision', 'corpus_manifest_sha256', 'runner_policy_sha256'):
        if previous.get(key) != current[key]:
            raise ValueError(f'baseline {key} differs; explicitly review a new baseline')
    if previous['cases'].keys() != current['cases'].keys():
        raise ValueError('baseline Test262 mode inventory differs')
    regressions, improvements = [], []
    for name, old in previous['cases'].items():
        new = current['cases'][name]
        if old.get('case_sha256') != new['case_sha256']:
            raise ValueError(f'baseline source/metadata/mode/harness differs: {name}')
        if old['status'] == 'passed' and new['status'] != 'passed':
            regressions.append(name)
        if old['status'] != 'passed' and new['status'] == 'passed':
            improvements.append(name)
    return regressions, improvements


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/eris-js')
    parser.add_argument('--corpus', type=Path, default=ROOT / 'tests/upstream/test262')
    parser.add_argument('--output', type=Path, default=ROOT / 'artifacts/test262-report.json')
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--record-baseline', type=Path)
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--timeout', type=float, default=3.0)
    args = parser.parse_args()
    if args.baseline and args.record_baseline:
        parser.error('baseline checking and recording are mutually exclusive')
    if any(path is not None and paths_alias(path, args.output)
           for path in (args.baseline, args.record_baseline)):
        parser.error('report output must be separate from the baseline path')
    if not 1 <= args.jobs <= 16 or not 0 < args.timeout <= 60:
        parser.error('jobs must be 1..16 and timeout must be (0,60] seconds')
    started = time.monotonic()
    try:
        manifest, files, cases, fixtures, corpus_hash = load_corpus(args.corpus)
        binary = args.binary.resolve()
        binary_hash = digest(binary.read_bytes())
        preflight = harness_preflight(files, binary, args.timeout)
        preflight_ok = all(item['verified'] for item in preflight)
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            results = list(pool.map(lambda case: run_case(case, binary, args.timeout), cases))
        counts = dict(Counter(result['status'] for result in results))
        policy = dict(format=2, supported_features=sorted(SUPPORTED_FEATURES),
                      negative_intrinsic_errors=sorted(INTRINSIC_ERRORS), strict=False,
                      modules=False, async_completion=False, host_hooks=False,
                      timeout_seconds=args.timeout)
        policy_hash = digest(json.dumps(policy, sort_keys=True).encode())
        if digest(binary.read_bytes()) != binary_hash:
            raise ValueError('adapter binary changed during the run; repeat with a stable build')
        current = dict(revision=manifest['revision'], corpus_manifest_sha256=corpus_hash,
                       runner_policy_sha256=policy_hash, binary_sha256=binary_hash,
                       cases={item['id']: {key: item[key] for key in ('status', 'case_sha256', 'source_sha256')}
                              for item in results})
        regressions, improvements = [], []
        if args.baseline:
            regressions, improvements = check_baseline(json.loads(args.baseline.read_bytes()), current)
        report = dict(suite='pinned-test262-string-json-selection', full_test262_conformance=False,
                      repository=manifest['repository'], revision=manifest['revision'],
                      corpus_manifest_sha256=corpus_hash, runner_policy=policy, runner_policy_sha256=policy_hash,
                      source_files=manifest['test_files'], source_tests=len({case['file'] for case in cases}),
                      negative_source_tests=len({case['file'] for case in cases if case['metadata']['negative']}),
                      fixture_files=fixtures, mode_cases=len(cases), counts=counts,
                      seconds=round(time.monotonic() - started, 3), binary=str(binary), binary_sha256=binary_hash,
                      harness_preflight=preflight, harness_preflight_passed=preflight_ok,
                      regressions=regressions, improvements=improvements, cases=results)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2, ensure_ascii=True) + '\n', encoding='utf-8')
        healthy = preflight_ok and not any(counts.get(status) for status in BAD_RUN_STATUSES)
        if args.record_baseline and healthy:
            args.record_baseline.parent.mkdir(parents=True, exist_ok=True)
            args.record_baseline.write_text(json.dumps(current, indent=2) + '\n', encoding='utf-8')
        print(f'{manifest["test_files"]} pinned files; {len(cases)} mode cases; {counts}')
        print(f'Upstream harness preflight: {"passed" if preflight_ok else "FAILED"}')
        if args.baseline:
            print(f'{len(regressions)} regressions; {len(improvements)} newly passing cases')
        print(f'Report: {args.output}')
        if not healthy or regressions:
            return 1
        if args.baseline or args.record_baseline:
            return 0
        return 0 if counts.get('passed', 0) == len(cases) else 1
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f'Test262 runner: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
