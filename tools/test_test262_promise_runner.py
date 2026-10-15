"""Frozen runner policy/guard controls; adapters and network stay mocked."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner

PROFILE = 'promise-methods-async'
CORPUS = runner.ROOT / 'tests/upstream/test262-promise-methods-async'
ORACLE = Path(__file__).with_name('promise-methods-runner-oracle.json')
ORACLE_SHA = 'feeeb6ffb8e7b8f80385426e7c7922b9ce9adcc9212d7016b9dde80b10ffb02b'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def fixture_case():
    return dict(id='fixture:sloppy', file='fixture.js', mode='sloppy', source=b'1',
                harness=[], metadata=dict(flags=[], includes=[], features=[],
                negative=None, locale=[], timeout=None), case_sha256='0' * 64)


def completion(policy, calls=0, success=0, failure=0, flags=0,
               quiescent=True, audited=True, unhandled=0):
    return dict(policy=policy, done_calls=calls, success_prints=success,
                failure_prints=failure, flags=flags, quiescent=quiescent,
                audited=audited, unhandled=unhandled)


def selected_observation(held):
    """Literal mock outcomes test verifier policy, never claim a JS outcome."""
    expected, policy = held['expected'], held['completion_policy']
    counts = (1, 1, 0) if policy else (0, 0, 0)
    evidence = completion(policy, *counts)
    observation = dict(status='complete', phase='runtime', error_type='',
                       error_identity='', message='', harness='', completion=evidence)
    status = 'passed'
    if expected == 'synchronous-exception':
        observation.update(status='exception', error_type='Test262Error', message='marker')
        evidence['flags'] = 256
        status = 'failed'
    elif expected == 'resource':
        observation.update(status='resource', error_type='ResourceLimit', message='marker')
        evidence.update(flags=264, quiescent=False, audited=False)
        status = 'resource'
    elif expected != 'passed':
        status = 'failed'
        observation.update(status='host-failure', phase='host', message=expected)
        if expected == 'completion-error':
            evidence.update(success_prints=0, failure_prints=1, flags=1)
            if held['name'] == 'truthy-error-getter-abrupt':
                evidence.update(failure_prints=0, flags=65)
        elif expected == 'missing-completion':
            evidence.update(done_calls=0, success_prints=0)
        elif expected == 'duplicate-completion':
            evidence.update(done_calls=2, success_prints=2)
        elif expected == 'queued-exception':
            evidence['flags'] = 4
        elif expected == 'unhandled-rejection':
            evidence['unhandled'] = 1
        elif expected == 'unauthenticated-completion':
            evidence.update(done_calls=0, success_prints=0, flags=16)
        elif expected == 'completion-protocol-error':
            evidence['success_prints'] = 0
        else:
            raise AssertionError(expected)
    return dict(status=status, observation=observation)


class PromiseRunnerTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.controls = {row['name']: row for row in self.oracle['controls']}
        for owner, name in [(runner, 'bounded_process'), (importer, 'fetch')]:
            guard = patch.object(owner, name, side_effect=AssertionError('adapter/network forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def files(self):
        return {path: (CORPUS / path).read_bytes() for path in self.oracle['auxiliary_files']}

    def capture(self, change=None):
        captured = []
        def observe(case, *args):
            captured.append(copy.deepcopy(case))
            name = case['id'].removeprefix('harness-preflight:promise-methods-')
            held = self.controls.get(name)
            result = selected_observation(held) if held else dict(status='passed')
            if change:
                result = change(case, held, result)
            return dict(id=case['id'], mode=case['mode'],
                        case_sha256=case['case_sha256'],
                        source_sha256=digest(case['source']), **result)
        with patch.object(runner, 'run_case', side_effect=observe):
            outcomes = runner.harness_preflight(self.files(), Path('/inert'), 3.0, PROFILE)
        return captured, outcomes

    def test_complete_corpus_modes_helpers_and_policy_identity(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), len(files), fixtures), (232, 458, 240, []))
        self.assertEqual([sum(c['mode'] == m for c in cases) for m in ['sloppy', 'strict']], [229, 229])
        self.assertEqual([sum('async' in c['metadata']['flags'] for c in cases),
                          sum('async' not in c['metadata']['flags'] for c in cases)], [222, 236])
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], set(self.oracle['features']))
        self.assertEqual((len(manifest['inventory_proof']['files']),
                          sum(x['bytes'] for x in manifest['inventory_proof']['files'])), (13, 86344))
        for path, row in self.oracle['auxiliary_files'].items():
            self.assertEqual((len(files[path]), digest(files[path])), (row['bytes'], row['sha256']))
        for case in cases:
            policy = int('async' in case['metadata']['flags'])
            self.assertIs(type(case['completion_policy']), int)
            self.assertEqual(case['completion_policy'], policy)
            names = ['assert.js', 'sta.js'] + (['doneprintHandle.js'] if policy else []) + case['metadata']['includes']
            self.assertEqual([name for name, _ in case['harness']], names)
            self.assertEqual(len(set(names)), len(names))
            for name, data in case['harness']:
                self.assertEqual(digest(data), self.oracle['auxiliary_files']['harness/' + name]['sha256'])
            self.assertEqual(runner.encode_request(case)[:8], b'ERJS2' + bytes([int(case['mode'] == 'strict'), 0, policy]))
            if {'class', 'cross-realm'} & set(case['metadata']['features']):
                self.assertIsNotNone(runner.unsupported_reason(case, runner.PROFILE_FEATURES[PROFILE], PROFILE))

    def test_policy_participates_only_in_new_case_fingerprint(self):
        value = fixture_case()
        old = dict(source_sha256=digest(value['source']), mode=value['mode'],
                   metadata=value['metadata'], harness=[])
        self.assertEqual(runner.case_fingerprint(value), digest(json.dumps(old, sort_keys=True, ensure_ascii=True).encode()))
        value['completion_policy'] = 0
        expected = dict(old, completion_policy=0)
        self.assertEqual(runner.case_fingerprint(value), digest(json.dumps(expected, sort_keys=True, ensure_ascii=True).encode()))
        value['completion_policy'] = 1
        self.assertNotEqual(runner.case_fingerprint(value), digest(json.dumps(expected, sort_keys=True, ensure_ascii=True).encode()))

    def test_rehashed_helper_and_legal_tampering_still_fails_git_binding(self):
        for name in self.oracle['auxiliary_files']:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                data = (dest / name).read_bytes() + b'\n'
                (dest / name).write_bytes(data)
                manifest = json.loads((dest / 'manifest.json').read_bytes())
                next(x for x in manifest['files'] if x['path'] == name).update(bytes=len(data), sha256=digest(data))
                (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)

    def test_all_37_literal_guards_have_74_exact_mode_and_helper_bindings(self):
        captured, outcomes = self.capture()
        self.assertEqual((len(captured), len(outcomes)), (74, 74))
        expected = [(mode, held) for mode in ['sloppy', 'strict'] for held in self.oracle['controls']]
        for case, outcome, (mode, held) in zip(captured, outcomes, expected):
            name = 'promise-methods-' + held['name']
            self.assertEqual((case['id'], case['mode'], case['source']),
                             ('harness-preflight:' + name, mode, held['source'].encode()))
            self.assertEqual(case['completion_policy'], held['completion_policy'])
            self.assertEqual(case['metadata']['flags'], ['async'] if held['completion_policy'] else [])
            self.assertEqual(case['metadata']['includes'], held['includes'])
            names = ['assert.js', 'sta.js'] + (['doneprintHandle.js'] if held['completion_policy'] else []) + held['includes']
            self.assertEqual(case['harness'], [(n, self.files()['harness/' + n]) for n in names])
            self.assertEqual((outcome['name'], outcome['expected'], outcome['verified']), (name, held['expected'], True))
        original = [x for x in self.oracle['original_guard_records'] if x['kind'] == 'literal-js']
        self.assertEqual([(x['name'], x['source']) for x in self.oracle['controls'] if x['origin'] == 'original-literal'],
                         [(x['id'], x['source']) for x in original])
        self.assertEqual((len(original), len(self.oracle['original_guard_records'])), (31, 37))

    def test_incidental_errors_or_missing_evidence_cannot_verify_guards(self):
        for result in [dict(status='failed'), dict(status='passed'), dict(status='resource'),
                       dict(status='timeout'), dict(status='unsupported'), dict(status='adapter-error'),
                       dict(status='harness-error'),
                       dict(status='failed', observation=dict(status='exception', phase='runtime', error_type='TypeError', error_identity='TypeError'))]:
            with self.subTest(result=result):
                _, outcomes = self.capture(lambda case, held, expected: copy.deepcopy(result))
                self.assertEqual(len(outcomes), 74)
                self.assertFalse(any(x['verified'] for x in outcomes))

    def test_poisoned_prerequisites_and_host_reasons_are_not_generic_failures(self):
        def break_positive(case, held, result):
            if held and held['name'] == 'promise-api-positive' and case['mode'] == 'strict':
                return selected_observation(dict(held, expected='synchronous-exception'))
            return result
        _, outcomes = self.capture(break_positive)
        original = [x for x in outcomes if x['name'].removeprefix('promise-methods-') in
                    {r['name'] for r in self.oracle['controls'] if r['origin'] == 'original-literal'}]
        self.assertTrue(all(x['verified'] == (x['result']['mode'] == 'sloppy') for x in original))
        def generic(case, held, result):
            if result.get('observation', {}).get('status') == 'host-failure':
                result['observation']['message'] = 'wrong-host-reason'
            return result
        _, outcomes = self.capture(generic)
        hosts = [x for x in outcomes if x['result'].get('observation', {}).get('status') == 'host-failure']
        self.assertTrue(hosts)
        self.assertFalse(any(x['verified'] for x in hosts))
        def disable_poison(case, held, result):
            if held and held['name'] == 'promise-helper-poisoned':
                return selected_observation(dict(held, expected='passed'))
            return result
        _, outcomes = self.capture(disable_poison)
        originals = [x for x in outcomes if x['name'].removeprefix('promise-methods-') in
                     {r['name'] for r in self.oracle['controls'] if r['origin'] == 'original-literal'}]
        self.assertFalse(any(x['verified'] for x in originals))

    def complete_preflight(self):
        return [dict(name='promise-methods-' + held['name'], expected=held['expected'],
                     verified=True, result=dict(mode=mode))
                for mode in ['sloppy', 'strict'] for held in self.oracle['controls']]

    def report(self, profile, preflight, cases=None, observed=None):
        case = fixture_case()
        cases = cases if cases is not None else [case]
        manifest = dict(revision=importer.REVISION, repository='https://github.com/' + importer.REPOSITORY,
                        test_files=len({c['file'] for c in cases}))
        calls = []
        def execute(c, *args):
            calls.append(c['id'])
            result = observed(c) if observed else dict(status='passed')
            return dict(id=c['id'], mode=c['mode'], case_sha256=c['case_sha256'],
                        source_sha256=digest(c['source']), **result)
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            binary, output, baseline = [path / n for n in ['inert', 'report.json', 'baseline.json']]
            binary.write_bytes(b'never executed')
            argv = ['runner', '--profile', profile, '--binary', str(binary), '--output', str(output),
                    '--record-baseline', str(baseline), '--jobs', '1']
            with patch.object(runner.sys, 'argv', argv), patch.object(runner, 'load_corpus',
                    return_value=(manifest, {}, cases, [], '0' * 64)), patch.object(runner, 'harness_preflight',
                    return_value=preflight), patch.object(runner, 'run_case', side_effect=execute), contextlib.redirect_stdout(io.StringIO()):
                code = runner.main()
            report = json.loads(output.read_bytes())
            return code, report, baseline.exists(), calls

    def test_all_53_old_serialized_policies_and_hashes_remain_exact(self):
        names = set(runner.PROFILES) - {PROFILE, 'promise-all'}
        self.assertEqual(names, set(self.oracle['old_policies']))
        self.assertEqual(len(names), 53)
        for name in sorted(names):
            with self.subTest(profile=name):
                code, report, baseline, calls = self.report(name, [dict(verified=True)])
                held = self.oracle['old_policies'][name]
                self.assertEqual((code, baseline, calls), (0, True, ['fixture:sloppy']))
                self.assertEqual((report['runner_policy'], report['runner_policy_sha256']), (held['policy'], held['sha256']))
                self.assertEqual(runner.encode_request(fixture_case())[:5], b'ERJS1')

    def test_failed_preflight_blocks_all_modes_and_refuses_baseline(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        healthy = self.complete_preflight()
        duplicate = copy.deepcopy(healthy)
        duplicate[-1] = copy.deepcopy(duplicate[0])
        wrong_name = copy.deepcopy(healthy)
        wrong_name[0]['name'] = 'promise-methods-forged'
        wrong_mode = copy.deepcopy(healthy)
        wrong_mode[0]['result']['mode'] = 'raw'
        unverified = copy.deepcopy(healthy)
        unverified[0]['verified'] = False
        for label, preflight in [('empty', []), ('truncated', healthy[:-1]),
                                 ('duplicate', duplicate), ('extra', healthy + healthy[:1]),
                                 ('wrong-name', wrong_name), ('wrong-mode', wrong_mode),
                                 ('one-unverified', unverified)]:
            with self.subTest(label=label):
                code, report, baseline, calls = self.report(PROFILE, preflight, cases)
                self.assertEqual((code, baseline, calls), (1, False, []))
                self.assertEqual((report['mode_cases'], report['counts']), (458, {'blocked': 458}))
                self.assertEqual({x['id'] for x in report['cases']}, {x['id'] for x in cases})
                self.assertTrue(all(x['reason'] == 'Promise host preflight failed' for x in report['cases']))
                self.assertFalse(report['harness_preflight_passed'])
                self.assertEqual(report['runner_policy'], self.oracle['new_policy'])
                self.assertEqual(report['runner_policy_sha256'], self.oracle['new_policy_sha256'])

    def test_healthy_report_preserves_host_policy_reason_and_unsupported_denominator(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        def observed(case):
            reason = runner.unsupported_reason(case, set(self.oracle['features']), PROFILE)
            if reason:
                return dict(status='unsupported', reason=reason)
            held = dict(name='unhandled', completion_policy=case['completion_policy'], expected='unhandled-rejection')
            return selected_observation(held)
        code, report, baseline, calls = self.report(PROFILE, self.complete_preflight(), cases, observed)
        self.assertEqual((code, baseline, len(calls)), (0, True, 458))
        self.assertEqual(sum(report['counts'].values()), 458)
        self.assertEqual(report['counts'].get('passed', 0), 0)
        self.assertGreater(report['counts']['unsupported'], 0)
        self.assertTrue(all(x['observation']['message'] == 'unhandled-rejection'
                            and x['observation']['status'] == 'host-failure'
                            for x in report['cases'] if x['status'] == 'failed'))
        self.assertEqual(report['runner_policy'], self.oracle['new_policy'])
        self.assertEqual(report['runner_policy_sha256'], self.oracle['new_policy_sha256'])


if __name__ == '__main__':
    unittest.main()
