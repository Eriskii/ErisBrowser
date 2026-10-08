#!/usr/bin/env python3
"""Run the frozen local TypedArray view/string cases with the existing bounded JS adapter.

--verify-only checks source inputs and the complete expected matrix without
launching an adapter. Execution requires an explicit binary SHA and fresh output.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / 'tests/conformance/typedarray-views-local'
PINS = {
    "cases.js": "bcc0bc2523e5cb6151bc963b59927b4e4a638daa3118d0c8f2d0a9d26838fb58",
    "controls.json": "2839a99c14e8c58d8356ed2249ca4bf0e2f2ea9568e05c636d716a9470eb3d2e",
    "README.md": "337fd1b707828f6bd685f919da70913585c34540423f3789de27781b9255db36",
    "design.md": "f92fb6689cf1dae3435617e1b1030e0a294c086b70ccf5b7bb6f768ca4be6d23",
    "inventory.json": "9a70f98d92fb73549bd73769dc703c3d946c67ec2556aaa68ee6330c0728e9b6",
    "spec-notes.json": "f40199719ace25192ef331a3aa33875656e25152b04ce12fbb4af549202ded53",
    "context.json": "ac6f61c045cc638cd1819e5f90ad4e978611e5cd0436d775ad63ef213caa8b0d",
    "matrix.json": "f0e2f9bedbb0a11daf3f4752ff23d76a9bdd7df7784d95183d4b62d52e8338a8",
    "freeze.py": "60ed7d21914581eac41858a8123f74c0480b2b706f5688ed0976db90f77ad363"
}
COMPLETE = dict(status='complete', phase='runtime', error_type='', error_identity='')
WRONG = dict(status='exception', phase='runtime', error_type='Error', error_identity='Error')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def binding(path):
    data = path.read_bytes()
    return dict(path=str(path.resolve()), bytes=len(data), sha256=digest(data))


def require(condition, message):
    if not condition:
        raise ValueError(message)


def matches(result, expected):
    observation = result.get('observation')
    # All requests have negative=None. The shared adapter classifier therefore
    # calls successful completion "passed" and the deliberate Error "failed".
    status = 'passed' if expected['status'] == 'complete' else 'failed'
    return (result.get('status') == status
            and isinstance(observation, dict)
            and all(observation.get(key) == value for key, value in expected.items()))


def load_matrix(runner):
    manifest = json.loads((CORPUS / 'manifest.json').read_bytes())
    require(manifest['collection'] == 'typedArrayViewCases', 'collection changed')
    require((manifest['case_count'], manifest['case_modes'], manifest['control_pairs'],
             manifest['control_modes']) == (23, 46, 6, 24), 'population changed')
    require(len(manifest['files']) == len(PINS), 'input manifest population changed')
    seen = set()
    for row in manifest['files']:
        name = row['path']
        require(name in PINS and name not in seen, 'unknown or repeated fixture path')
        seen.add(name)
        raw = (CORPUS / name).read_bytes()
        require(len(raw) == row['bytes'] and digest(raw) == row['sha256'] == PINS[name],
                'frozen fixture changed: ' + name)
    inventory = json.loads((CORPUS / 'inventory.json').read_bytes())
    controls = json.loads((CORPUS / 'controls.json').read_bytes())
    fixture = (CORPUS / 'cases.js').read_bytes()
    require(inventory['collection'] == manifest['collection'], 'inventory collection changed')
    require(manifest['case_invocation_arguments'] == [], 'zero-argument invocation changed')
    require(manifest['ordinary_observation'] == dict(COMPLETE, return_value=True),
            'case expectation changed')
    require(inventory['named_bodies'] == 23 and inventory['mode_cases'] == 46
            and inventory['modes'] == ['sloppy', 'strict'], 'inventory population changed')
    names = inventory['cases']
    require(len(names) == len(set(names)) == 23, 'case names are incomplete or repeated')
    require(all(re.fullmatch(r'[a-z][a-z0-9_]*', name) for name in names), 'unsafe case name')
    rows = controls['controls']
    require(controls['pairs'] == 6 and controls['control_modes'] == 24 and len(rows) == 12,
            'control population changed')
    by_name = {row['name']: row for row in rows}
    require(len(by_name) == 12, 'repeated control name')
    require(len({row['pair'] for row in rows}) == 6, 'control pair population changed')
    for row in rows:
        positive = by_name[row['pair'] + '-positive']
        require(row['positive'] is True or row['positive'] is False, 'invalid control polarity')
        require(positive['positive'] is True and positive['pair'] == row['pair'],
                'control partner mismatch')
        require(row['name'] == row['pair'] + ('-positive' if row['positive'] else '-wrong'),
                'control partner name mismatch')
        require(row['modes'] == ['sloppy', 'strict'], 'control modes changed')
        require(digest(row['source'].encode()) == row['source_sha256'], 'control source changed')
        require(row['expected_observation'] == (COMPLETE if row['positive'] else WRONG),
                'control expectation changed')
        require(row['literal_observation'] == positive['asserted_observation'],
                'paired literal changed')
        require((row['asserted_observation'] == row['literal_observation']) == row['positive'],
                'wrong control must assert a different concrete value')
        tail = ('if(actual!==' + json.dumps(row['asserted_observation'])
                + ')throw new Error(' + json.dumps(row['pair'] + ' expectation') + ');')
        positive_tail = ('if(actual!==' + json.dumps(positive['asserted_observation'])
                         + ')throw new Error(' + json.dumps(row['pair'] + ' expectation') + ');')
        require(row['source'].endswith(tail) and positive['source'].endswith(positive_tail),
                'control assertion shape changed')
        require(row['source'][:-len(tail)] == positive['source'][:-len(positive_tail)],
                'positive/wrong prerequisite prefixes differ')

    matrix = []
    for kind, items in [('case', [dict(name=name) for name in names]), ('control', rows)]:
        for row in items:
            for mode in ['sloppy', 'strict']:
                if kind == 'case':
                    body = (fixture + b'\nif (typedArrayViewCases.' + row['name'].encode()
                            + b'() !== true) '
                            + b'throw new Error("case did not return true");\n')
                    expected = COMPLETE
                else:
                    body = row['source'].encode()
                    expected = row['expected_observation']
                case = dict(id=row['name'] + '#' + mode, mode=mode, source=body,
                            file='cases.js' if kind == 'case' else 'controls.json',
                            metadata=dict(flags=[], features=[], includes=[], locale=[], negative=None),
                            harness=[])
                case['case_sha256'] = runner.case_fingerprint(case)
                identity = dict(id=case['id'], mode=mode, kind=kind,
                                source_sha256=digest(body), case_sha256=case['case_sha256'],
                                expected_observation=expected)
                if kind == 'control':
                    identity.update(pair=row['pair'], positive=row['positive'],
                                    partner=row['pair'] + '-positive#' + mode)
                matrix.append((case, identity))
    require(len(matrix) == len({case['id'] for case, _ in matrix}) == 70,
            'full matrix population changed')
    frozen = json.loads((CORPUS / 'matrix.json').read_bytes())
    require(frozen == [identity for _, identity in matrix], 'frozen expected matrix changed')
    return matrix


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify-only', action='store_true')
    parser.add_argument('--engine', type=Path)
    parser.add_argument('--engine-sha256')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.verify_only:
        require(args.engine is None and args.engine_sha256 is None and args.output is None,
                '--verify-only does not accept execution arguments')
    else:
        require(args.engine is not None and args.output is not None
                and re.fullmatch(r'[0-9a-f]{64}', args.engine_sha256 or ''),
                'execution requires --engine, its exact --engine-sha256, and a fresh --output')
    input_paths = [Path(__file__).resolve()] + [ROOT / 'tools' / name for name in
                   ['test262_conformance.py', 'html_conformance.py', 'import_test262.py']]
    input_paths += [CORPUS / name for name in [*PINS, 'manifest.json', 'RUNNER.md']]
    # Bind actual helper bytes before import and retain the same bindings after
    # all requests. The upstream profile may evolve independently before this run.
    before = [binding(path) for path in input_paths]
    import test262_conformance as runner
    matrix = load_matrix(runner)
    require([binding(path) for path in input_paths] == before, 'inputs changed during preparation')
    if args.verify_only:
        print(json.dumps(dict(status='source inputs verified; no adapter launched',
                              cases=46, controls=24, rows=70, inputs=before), indent=2))
        return 0

    binary = args.engine.resolve(strict=True)
    binary_before = binding(binary)
    require(binary_before['sha256'] == args.engine_sha256, 'binary differs from supplied digest')
    # Exclusive creation precedes every process launch. Existing result/source/
    # binary paths therefore cannot be overwritten by an observation run.
    with args.output.open('x', encoding='utf-8') as output:
        records = []
        for case, identity in matrix:
            try:
                result = runner.run_case(case, binary, 3)
                require(isinstance(result, dict), 'run_case returned a non-object')
                require(all(result.get(key) == case[key] for key in
                            ['id', 'file', 'mode', 'case_sha256']), 'adapter row identity changed')
                require(result.get('source_sha256') == identity['source_sha256'],
                        'adapter row source binding changed')
            except Exception as error:
                result = dict(status='runner-error', reason=f'{type(error).__name__}: {error}')
            result.update(identity)
            result['matches_expectation'] = matches(result, identity['expected_observation'])
            records.append(result)

        cases = [row for row in records if row['kind'] == 'case']
        controls = [row for row in records if row['kind'] == 'control']
        by_id = {row['id']: row for row in controls}
        for row in controls:
            row['positive_partner_matches'] = by_id[row['partner']]['matches_expectation']
            row['verified'] = row['matches_expectation'] and row['positive_partner_matches']
        observed_after, failures = [], []
        for expected in before + [binary_before]:
            try:
                actual = binding(Path(expected['path']))
                observed_after.append(actual)
                if actual != expected:
                    failures.append(dict(path=expected['path'], reason='binding changed'))
            except OSError as error:
                actual = dict(path=expected['path'], error=str(error))
                observed_after.append(actual)
                failures.append(actual)
        summary = dict(case_matches=sum(row['matches_expectation'] for row in cases),
                       case_total=len(cases), control_matches=sum(row['matches_expectation'] for row in controls),
                       controls_verified=sum(row['verified'] for row in controls), control_total=len(controls),
                       runner_errors=sum(row['status'] == 'runner-error' for row in records))
        report = dict(schema=1, scope='local TypedArray view/string methods, separate from upstream profile',
                      binary=str(binary), binary_sha256=binary_before['sha256'],
                      binary_bytes=binary_before['bytes'], inputs=before,
                      observed_after=observed_after, binding_failures=failures,
                      matrix_sha256=next(row['sha256'] for row in before
                                         if row['path'] == str((CORPUS / 'matrix.json').resolve())),
                      summary=summary, cases=cases, controls=controls)
        json.dump(report, output, indent=2)
        output.write('\n')
    print(json.dumps(dict(summary, binding_failures=failures), indent=2))
    if failures:
        return 2
    if summary['runner_errors']:
        return 3
    return 0 if all(row['matches_expectation'] for row in cases) and all(
        row['verified'] for row in controls) else 1


if __name__ == '__main__':
    raise SystemExit(main())
