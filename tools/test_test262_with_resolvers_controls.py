"""Frozen ordinary withResolvers gate controls; mocks never execute an adapter."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import test262_conformance as runner


PROFILE = 'promise-with-resolvers'
ORACLE = Path(__file__).with_name('test262-with-resolvers-controls-oracle.json')
ORACLE_SHA = '5659b69ba7a219f10a1242479fae77ed407fbddb3b9cb25f816d06809195372c'
DEFAULT_FILES = object()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def normalized(value):
    return json.loads(json.dumps(value))


class WithResolversControlTests(unittest.TestCase):
    def setUp(self):
        raw = ORACLE.read_bytes()
        self.assertEqual(digest(raw), ORACLE_SHA)
        self.oracle = json.loads(raw)
        self.files = {}
        for name, held in self.oracle['helpers'].items():
            data = held['source'].encode()
            self.assertEqual((len(data), digest(data)), (held['bytes'], held['sha256']))
            self.files['harness/' + name] = data
        self.good = self.oracle['healthy_outcomes']
        self.keys = [(row['name'], row['mode']) for row in self.oracle['variants']]
        self.index = {key: i for i, key in enumerate(self.keys)}
        guard = patch.object(runner, 'bounded_process', side_effect=AssertionError('adapter forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def assert_bad(self, rows, files=DEFAULT_FILES):
        selected = self.files if files is DEFAULT_FILES else files
        self.assertFalse(runner.with_resolvers_preflight_healthy(rows, selected))

    def test_frozen_44_literals_metadata_helpers_modes_and_references_are_exact(self):
        self.assertEqual(runner.PROMISE_WITH_RESOLVERS_PROFILE, PROFILE)
        self.assertEqual(runner.PROMISE_WITH_RESOLVERS_FEATURES, {'promise-with-resolvers'})
        rows = runner.with_resolvers_preflight_variants()
        self.assertEqual(normalized(rows), self.oracle['variants'])
        self.assertEqual(len(rows), 44)
        self.assertEqual(sum(row['expected'] == 'passed' for row in rows), 18)
        self.assertEqual(sum(row['expected'] == 'failed' for row in rows), 26)
        definitions = runner.with_resolvers_preflight_cases(self.files)
        self.assertEqual(len(definitions), 44)
        for i, ((row, case), expected) in enumerate(zip(definitions, self.oracle['cases'])):
            with self.subTest(identity=self.keys[i]):
                self.assertEqual(normalized(row), self.oracle['variants'][i])
                self.assertEqual(set(case), {'id', 'file', 'mode', 'metadata', 'source', 'harness', 'case_sha256'})
                actual = {key: case[key] for key in ('id', 'file', 'mode', 'metadata', 'case_sha256')}
                actual.update(source=case['source'].decode(), source_sha256=digest(case['source']),
                              harness=[dict(name=name, sha256=digest(data)) for name, data in case['harness']])
                self.assertEqual(actual, expected)
                self.assertEqual(runner.with_resolvers_preflight_reference(row, case),
                                 dict(name=row['name'], mode=case['mode'], id=case['id'],
                                      case_sha256=expected['case_sha256'], source_sha256=expected['source_sha256'],
                                      expected=row['expected']))
        for i in range(32, 44):
            refs = self.good[i]['prerequisites']
            self.assertEqual([(p['name'], p['mode']) for p in refs[:32]], self.keys[:32])
            if '-mismatch' in rows[i]['name']:
                self.assertEqual((refs[-1]['name'], refs[-1]['mode']),
                                 (rows[i]['name'].removesuffix('-mismatch'), rows[i]['mode']))
                self.assertEqual(len(refs), 33)
            else:
                self.assertEqual(len(refs), 32)

    def test_healthy_builder_uses_selected_route_and_independently_checks_raw_rows(self):
        self.assertTrue(runner.with_resolvers_preflight_healthy(self.good, self.files))
        self.assertEqual(runner.with_resolvers_preflight_checks(self.good, self.files),
                         dict.fromkeys(self.keys, True))
        held = {(r['result']['id'], r['result']['mode']): r for r in self.good}
        captured = []

        def run(case, binary, timeout, features, profile):
            self.assertEqual((binary, timeout, features, profile),
                             (Path('/inert'), 3.0, {'promise-with-resolvers'}, PROFILE))
            self.assertNotIn('completion_policy', case)
            captured.append((case['id'], case['mode']))
            return copy.deepcopy(held[(case['id'], case['mode'])]['result'])

        with patch.object(runner, 'run_case', side_effect=run) as call:
            actual = runner.with_resolvers_harness_preflight(self.files, Path('/inert'), 3.0)
        self.assertEqual(call.call_count, 44)
        self.assertEqual(captured, [(c['id'], c['mode']) for c in self.oracle['cases']])
        self.assertEqual(actual, self.good)
        unchecked = copy.deepcopy(self.good)
        for row in unchecked:
            row['verified'] = False
        self.assertEqual(runner.with_resolvers_preflight_checks(unchecked, self.files),
                         dict.fromkeys(self.keys, True))
        self.assert_bad(unchecked)
        with patch.object(runner, 'classify', return_value='unsupported') as classify:
            self.assert_bad(self.good)
            self.assertEqual(classify.call_count, 44)

    def test_roster_and_outer_schema_refuse_missing_duplicate_extra_or_unhashable_rows(self):
        for malformed in (None, {}, (), [], self.good[:-1], self.good + [self.good[0]]):
            with self.subTest(kind=type(malformed).__name__, length=len(malformed) if malformed is not None else None):
                self.assert_bad(malformed)
        duplicate = copy.deepcopy(self.good)
        duplicate[-1] = copy.deepcopy(duplicate[0])
        self.assert_bad(duplicate)
        mutations = [
            ('name', []), ('name', {}), ('name', 'invented-control'),
            ('expected', []), ('expected', True), ('expected', 'passed-but-unverified'),
            ('verified', 1), ('verified', 'true'), ('verified', None), ('verified', False),
            ('result', []), ('result', None), ('prerequisites', None), ('prerequisites', {}),
        ]
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                changed = copy.deepcopy(self.good)
                changed[32][field] = value
                self.assert_bad(changed)
        for field in self.good[32]:
            with self.subTest(missing=field):
                changed = copy.deepcopy(self.good)
                del changed[32][field]
                self.assert_bad(changed)
        changed = copy.deepcopy(self.good)
        changed[32]['extra'] = False
        self.assert_bad(changed)
        changed = copy.deepcopy(self.good)
        changed[32]['result']['mode'] = []
        self.assert_bad(changed)

    def test_canonical_result_metadata_source_and_helper_identity_cannot_be_relabelled(self):
        for index in [0, 1, 9, 31] + list(range(32, 44)):
            mutations = dict(id='harness-preflight:invented', file='different.js', mode='invalid-mode',
                             case_sha256='0' * 64, source_sha256='1' * 64,
                             expected_negative={'phase': 'runtime', 'type': 'Test262Error'}, status='unsupported')
            for field, value in mutations.items():
                with self.subTest(identity=self.keys[index], field=field):
                    changed = copy.deepcopy(self.good)
                    changed[index]['result'][field] = value
                    self.assert_bad(changed)
            changed = copy.deepcopy(self.good)
            changed[index]['expected'] = 'failed' if changed[index]['expected'] == 'passed' else 'passed'
            self.assert_bad(changed)
        for field in self.good[32]['result']:
            with self.subTest(missing=field):
                changed = copy.deepcopy(self.good)
                del changed[32]['result'][field]
                self.assert_bad(changed)
        changed = copy.deepcopy(self.good)
        changed[32]['result']['completion_policy'] = 0
        self.assert_bad(changed)
        for name in self.files:
            with self.subTest(helper=name):
                changed_files = dict(self.files)
                changed_files[name] += b'\n// different helper bytes\n'
                self.assert_bad(self.good, changed_files)
                del changed_files[name]
                self.assert_bad(self.good, changed_files)
                changed_files[name] = []
                self.assert_bad(self.good, changed_files)
        self.assert_bad(self.good, None)
        for field, value in [('source', 'assert(false);'), ('metadata', {'features': ['unknown'], 'flags': [],
                                                                   'includes': [], 'locale': [], 'negative': None})]:
            changed_rows = copy.deepcopy(runner.with_resolvers_preflight_variants())
            changed_rows[32][field] = value
            with patch.object(runner, 'with_resolvers_preflight_variants', return_value=changed_rows):
                self.assert_bad(self.good)

    def test_raw_observation_schema_types_and_utf8_bounds_are_not_verified_flag_shortcuts(self):
        for index in range(44):
            for field, value in [('harness', 'assert.js'), ('phase', 'parse'), ('status', 'host-failure'),
                                 ('error_identity', 'TypeError'), ('error_type', 'TypeError')]:
                with self.subTest(identity=self.keys[index], field=field):
                    changed = copy.deepcopy(self.good)
                    changed[index]['result']['observation'][field] = value
                    self.assert_bad(changed)
        for field in self.good[32]['result']['observation']:
            changed = copy.deepcopy(self.good)
            del changed[32]['result']['observation'][field]
            self.assert_bad(changed)
            for value in (None, False, 0, [], {}, b'text'):
                with self.subTest(field=field, value=value):
                    changed = copy.deepcopy(self.good)
                    changed[32]['result']['observation'][field] = value
                    self.assert_bad(changed)
        for value in (None, [], 'complete'):
            changed = copy.deepcopy(self.good)
            changed[32]['result']['observation'] = value
            self.assert_bad(changed)
        for field in ('extra', 'completion', 'completion_policy'):
            changed = copy.deepcopy(self.good)
            changed[32]['result']['observation'][field] = 0
            self.assert_bad(changed)
        for message, expected in [('x' * 16384, True), ('x' * 16385, False),
                                  ('\u00e9' * 8192, True), ('\u00e9' * 8193, False), ('\ud800', False)]:
            changed = copy.deepcopy(self.good)
            changed[34]['result']['observation']['message'] = message
            self.assertEqual(runner.with_resolvers_preflight_healthy(changed, self.files), expected)

    def test_prerequisite_exact_schema_order_same_mode_and_recursive_proof_are_required(self):
        target = self.index[('with-resolvers-generic-mismatch', 'strict')]
        for field, value in [('name', []), ('name', 'invented'), ('mode', 'sloppy'), ('mode', {}),
                             ('id', 'invented'), ('case_sha256', '0' * 64), ('source_sha256', '1' * 64),
                             ('expected', 'failed'), ('extra', True), ('verified', True)]:
            with self.subTest(field=field, value=value):
                changed = copy.deepcopy(self.good)
                changed[target]['prerequisites'][-1][field] = value
                self.assert_bad(changed)
        for field in self.good[target]['prerequisites'][-1]:
            changed = copy.deepcopy(self.good)
            del changed[target]['prerequisites'][-1][field]
            self.assert_bad(changed)
        for replacement in [[], self.good[target]['prerequisites'][:-1],
                            list(reversed(self.good[target]['prerequisites'])), [None], [['bad']],
                            self.good[target]['prerequisites'] + [self.good[target]['prerequisites'][-1]]]:
            changed = copy.deepcopy(self.good)
            changed[target]['prerequisites'] = copy.deepcopy(replacement)
            self.assert_bad(changed)
        changed = copy.deepcopy(self.good)
        positive = self.index[('with-resolvers-generic', 'strict')]
        changed[positive]['result']['observation'].update(status='exception', error_type='TypeError', error_identity='TypeError')
        checks = runner.with_resolvers_preflight_checks(changed, self.files)
        self.assertIsNotNone(checks)
        self.assertFalse(checks[self.keys[positive]])
        self.assertFalse(checks[self.keys[target]])
        self.assert_bad(changed)
        changed = copy.deepcopy(self.good)
        changed[0]['result']['observation']['harness'] = 'sta.js'
        checks = runner.with_resolvers_preflight_checks(changed, self.files)
        self.assertIsNotNone(checks)
        self.assertTrue(all(checks[key] is False for key in self.keys[32:]))

    def test_cyclic_canonical_dependency_fails_without_recursion_error(self):
        # A defensive canonical-definition mutation, not a user-controlled new
        # control. All other literal/case bytes and metadata remain frozen.
        rows = copy.deepcopy(runner.with_resolvers_preflight_variants())
        target = self.index[('with-resolvers-generic', 'sloppy')]
        rows[target]['prerequisites'].append(('with-resolvers-generic', 'sloppy'))
        changed = copy.deepcopy(self.good)
        case = self.oracle['cases'][target]
        changed[target]['prerequisites'].append(dict(name=rows[target]['name'], mode=case['mode'], id=case['id'],
                                                    case_sha256=case['case_sha256'], source_sha256=case['source_sha256'],
                                                    expected='passed'))
        with patch.object(runner, 'with_resolvers_preflight_variants', return_value=rows):
            self.assert_bad(changed)

    def source_fixtures(self):
        # Synthetic main-routing witnesses. They are not retained upstream
        # originals and do not predict any real source outcome.
        cases = []
        for index in range(6):
            for mode in ('sloppy', 'strict'):
                path = 'synthetic-' + str(index) + '.js'
                identity = path + ':' + mode
                source = b'0;'
                cases.append(dict(id=identity, file=path, mode=mode, source=source, harness=[],
                                  case_sha256=digest(identity.encode()),
                                  metadata=dict(features=['class'] if index == 0 else ['promise-with-resolvers'],
                                                flags=[], includes=[], locale=[], negative=None)))
        return cases

    def test_unhealthy_gate_blocks_all_12_rows_preserves_raw_controls_and_cannot_record_baseline(self):
        corrupt = copy.deepcopy(self.good)
        corrupt[32]['result']['observation']['harness'] = 'assert.js'
        unchecked = copy.deepcopy(self.good)
        unchecked[32]['verified'] = False
        for preflight in (corrupt, unchecked, []):
            with self.subTest(kind='raw' if preflight is corrupt else 'flag' if preflight is unchecked else 'empty'):
                with tempfile.TemporaryDirectory() as temporary:
                    directory = Path(temporary)
                    binary = directory / 'inert'
                    binary.write_bytes(b'never executed')
                    output, baseline = directory / 'report.json', directory / 'baseline.json'
                    cases = self.source_fixtures()
                    manifest = dict(repository='synthetic fixture', revision='fixture', test_files=6)
                    for existing in (False, True):
                        if existing:
                            baseline.write_bytes(b'preserve existing baseline exactly')
                        with contextlib.ExitStack() as stack:
                            stack.enter_context(patch.object(runner, 'load_corpus', return_value=(manifest, self.files, cases, [], '0' * 64)))
                            stack.enter_context(patch.object(runner, 'harness_preflight', return_value=preflight))
                            call = stack.enter_context(patch.object(runner, 'run_case', side_effect=AssertionError('source dispatch after bad gate')))
                            stack.enter_context(patch('sys.argv', ['runner', '--profile', PROFILE, '--binary', str(binary),
                                                                 '--output', str(output), '--record-baseline', str(baseline)]))
                            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                            self.assertEqual(runner.main(), 1)
                            call.assert_not_called()
                        report = json.loads(output.read_bytes())
                        self.assertFalse(report['harness_preflight_passed'])
                        self.assertEqual(report['harness_preflight'], preflight)
                        self.assertEqual((report['mode_cases'], report['counts']), (12, {'blocked': 12}))
                        self.assertEqual([r['id'] for r in report['cases']], [c['id'] for c in cases])
                        for actual, case in zip(report['cases'], cases):
                            self.assertEqual(actual['status'], 'blocked')
                            self.assertEqual((actual['case_sha256'], actual['source_sha256'], actual['expected_negative']),
                                             (case['case_sha256'], digest(case['source']), None))
                            self.assertIn('preflight', actual['reason'])
                        if existing:
                            self.assertEqual(baseline.read_bytes(), b'preserve existing baseline exactly')
                        else:
                            self.assertFalse(baseline.exists())

    def test_healthy_gate_allows_mocked_dispatch_and_baseline_without_host_policy(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            binary = directory / 'inert'
            binary.write_bytes(b'never executed')
            output, baseline = directory / 'report.json', directory / 'baseline.json'
            cases = self.source_fixtures()
            manifest = dict(repository='synthetic fixture', revision='fixture', test_files=6)

            def run(case, binary_arg, timeout, features, profile):
                self.assertEqual((binary_arg, timeout, features, profile), (binary, 3.0, {'promise-with-resolvers'}, PROFILE))
                # A mock result, including deliberately successful synthetic
                # class-tagged rows: this test exercises main's gate, not policy.
                return dict(id=case['id'], file=case['file'], mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=digest(case['source']), expected_negative=None, status='passed',
                            observation=dict(status='complete', phase='runtime', error_type='', error_identity='', message='', harness=''))

            with contextlib.ExitStack() as stack:
                stack.enter_context(patch.object(runner, 'load_corpus', return_value=(manifest, self.files, cases, [], '0' * 64)))
                stack.enter_context(patch.object(runner, 'harness_preflight', return_value=self.good))
                call = stack.enter_context(patch.object(runner, 'run_case', side_effect=run))
                stack.enter_context(patch('sys.argv', ['runner', '--profile', PROFILE, '--binary', str(binary),
                                                     '--output', str(output), '--record-baseline', str(baseline)]))
                stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                self.assertEqual(runner.main(), 0)
                self.assertEqual(call.call_count, 12)
            report, saved = json.loads(output.read_bytes()), json.loads(baseline.read_bytes())
            self.assertTrue(report['harness_preflight_passed'])
            self.assertEqual((report['mode_cases'], report['counts']), (12, {'passed': 12}))
            self.assertEqual(set(saved['cases']), {case['id'] for case in cases})
            self.assertEqual(report['runner_policy']['format'], 2)
            self.assertFalse(report['runner_policy']['host_hooks'])
            self.assertFalse(report['runner_policy']['async_completion'])
            self.assertEqual(report['runner_policy']['negative_intrinsic_errors'],
                             ['Error', 'EvalError', 'RangeError', 'ReferenceError', 'SyntaxError', 'TypeError', 'URIError'])


if __name__ == '__main__':
    unittest.main()
