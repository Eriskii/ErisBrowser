"""Independent frozen Promise.try control identities and gate refusal cases."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner

PROFILE = 'promise-try'
ORACLE = Path(__file__).with_name('promise-try-controls-oracle.json')
ORACLE_SHA = '336f6ff5bb74013d09dd7c37174713a50df8ed282fd45c52622070a3dc5ff7d0'
CORPUS = runner.ROOT / 'tests/upstream/test262-promise-try'


def digest(data):
    return hashlib.sha256(data).hexdigest()


class PromiseTryControlsTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.rows = self.oracle['records']
        self.index = {(row['name'], row['mode']): row for row in self.rows}
        for owner, name in ((importer, 'fetch'), (runner, 'bounded_process')):
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def require_try(self):
        self.assertTrue(hasattr(runner, 'promise_try_preflight_healthy'),
                        'selected Promise.try gate is absent')

    def files(self):
        files = {}
        for name, expected in self.oracle['helpers'].items():
            data = (CORPUS / 'harness' / name).read_bytes()
            self.assertEqual((len(data), digest(data)), (expected['bytes'], expected['sha256']))
            files['harness/' + name] = data
        return files

    def case(self, row):
        files = self.files()
        return dict(id=row['id'], file=row['file'], mode=row['mode'],
                    metadata=copy.deepcopy(row['metadata']), source=row['source'].encode(),
                    harness=[(h['name'], files['harness/' + h['name']]) for h in row['harness']],
                    completion_policy=row['completion_policy'], case_sha256=row['case_sha256'])

    def result(self, row):
        return dict(id=row['id'], file=row['file'], mode=row['mode'],
                    source_sha256=row['source_sha256'], case_sha256=row['case_sha256'],
                    expected_negative=None,
                    status=('passed' if row['expected'] == 'passed' else
                            'resource' if row['expected'] == 'resource' else 'failed'),
                    observation=copy.deepcopy(row['mock_observation']))

    def reference(self, identity):
        row = self.index[tuple(identity)]
        return dict(name=row['name'], mode=row['mode'], verified=True,
                    case_sha256=row['case_sha256'], source_sha256=row['source_sha256'])

    def healthy(self):
        return [dict(name=row['name'], expected=row['expected'], verified=True,
                     result=self.result(row),
                     prerequisites=[self.reference(key) for key in row['prerequisites']])
                for row in self.rows]

    def capture(self, change=None):
        captured = []
        def observe(case, binary, timeout, features, profile):
            self.assertEqual((binary, timeout, features, profile),
                             (Path('/inert'), 3.0, runner.PROMISE_TRY_FEATURES, PROFILE))
            row = self.index[(case['id'].removeprefix('harness-preflight:'), case['mode'])]
            captured.append(copy.deepcopy(case))
            result = self.result(row)
            return change(row, result) if change else result
        with patch.object(runner, 'run_case', side_effect=observe):
            outcomes = runner.harness_preflight(self.files(), Path('/inert'), 3.0, PROFILE)
        return captured, outcomes

    def test_common_74_literals_fingerprints_and_same_mode_prerequisites_remain_exact(self):
        legacy = json.loads(Path(__file__).with_name('promise-methods-runner-oracle.json').read_bytes())
        self.assertEqual(self.oracle['common_variants'], legacy['controls'])
        self.assertEqual(runner.promise_preflight_variants(), self.oracle['common_variants'])
        captured = []
        def observe(case, *args):
            row = self.index[(case['id'].removeprefix('harness-preflight:'), case['mode'])]
            captured.append(copy.deepcopy(case))
            return self.result(row)
        with patch.object(runner, 'run_case', side_effect=observe):
            outcomes = runner.promise_harness_preflight(self.files(), Path('/inert'), 3.0)
        self.assertEqual(captured, [self.case(row) for row in self.rows[:74]])
        self.assertEqual(outcomes, self.healthy()[:74])

    def test_five_literal_pairs_expand_to_exact_94_case_and_helper_identities(self):
        self.require_try()
        self.assertEqual(runner.promise_try_preflight_variants(), self.oracle['variants'])
        definitions = runner.promise_try_preflight_cases(self.files())
        self.assertEqual(len(definitions), 94)
        for (definition, case), held in zip(definitions, self.rows):
            self.assertEqual(case, self.case(held), held['id'])
            self.assertEqual((definition['name'], definition['expected']),
                             (held['name'], held['expected']))
            self.assertEqual([list(key) for key in definition['prerequisites']], held['prerequisites'])
            self.assertEqual(runner.promise_try_preflight_reference(definition, case),
                             self.reference((held['name'], held['mode'])))
            identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                            metadata=case['metadata'], completion_policy=case['completion_policy'],
                            harness=[dict(name=n, sha256=digest(b)) for n, b in case['harness']])
            self.assertEqual(digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode()),
                             held['case_sha256'])
        self.assertEqual([sum(row['mode'] == mode for row in self.rows) for mode in ('sloppy', 'strict')], [47, 47])

    def test_selected_composer_routes_all_rows_and_builds_verified_canonical_outcomes(self):
        self.require_try()
        captured, outcomes = self.capture()
        self.assertEqual(captured, [self.case(row) for row in self.rows])
        self.assertEqual(outcomes, self.healthy())
        self.assertTrue(runner.promise_try_preflight_healthy(outcomes, self.files()))
        self.assertTrue(runner.promise_try_preflight_healthy(list(reversed(outcomes)), self.files()))

    def test_missing_duplicate_extra_or_forged_identity_cannot_form_a_healthy_gate(self):
        self.require_try()
        good, files = self.healthy(), self.files()
        for altered in (good[:-1], good + [copy.deepcopy(good[-1])], good[:-1] + [copy.deepcopy(good[0])]):
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files))
        for position in range(94):
            altered = copy.deepcopy(good)
            altered[position]['verified'] = False
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files), position)
        for position in (0, 37, 74, 83, 84, 93):
            for field, value in (('id', 'unbound'), ('file', 'other.js'), ('mode', 'module'),
                                 ('case_sha256', '0' * 64), ('source_sha256', '1' * 64),
                                 ('expected_negative', {'type': 'TypeError'}), ('status', 'unsupported')):
                with self.subTest(position=position, field=field):
                    altered = copy.deepcopy(good)
                    altered[position]['result'][field] = value
                    self.assertFalse(runner.promise_try_preflight_healthy(altered, files))
        for field, value in (('name', 'forged'), ('expected', 'passed'), ('verified', 1)):
            altered = copy.deepcopy(good)
            altered[-1][field] = value
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files))
        altered = copy.deepcopy(good)
        altered[-1]['result']['trusted'] = True
        self.assertFalse(runner.promise_try_preflight_healthy(altered, files))

    def test_raw_schema_completion_and_harness_forgery_refuse_without_exceptions(self):
        self.require_try()
        good, files = self.healthy(), self.files()
        for malformed in (None, {}, [None] * 94, 'rows', 94):
            self.assertFalse(runner.promise_try_preflight_healthy(malformed, files))
        malformed_paths = [(['name'], []), (['result'], []), (['result', 'mode'], []),
                           (['result', 'case_sha256'], {}), (['result', 'observation'], []),
                           (['result', 'observation', 'completion'], []),
                           (['result', 'observation', 'message'], '\ud800'),
                           (['prerequisites'], {}), (['prerequisites'], [[]])]
        for path, value in malformed_paths:
            altered = copy.deepcopy(good)
            target = altered[-1]
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] = value
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files), path)
        for position in (0, 74, 75, 82, 83, 93):
            policy = self.rows[position]['completion_policy']
            for field, value in (('policy', True), ('policy', 1 - policy), ('done_calls', True),
                                 ('flags', -1), ('flags', 512), ('audited', 1),
                                 ('quiescent', False), ('unhandled', 1)):
                altered = copy.deepcopy(good)
                altered[position]['result']['observation']['completion'][field] = value
                self.assertFalse(runner.promise_try_preflight_healthy(altered, files), (position, field))
            for field, value in (('error_type', []), ('error_identity', 'AggregateError'),
                                 ('message', 'x' * 16385), ('harness', 'assert.js'), ('phase', 'parse')):
                altered = copy.deepcopy(good)
                altered[position]['result']['observation'][field] = value
                self.assertFalse(runner.promise_try_preflight_healthy(altered, files), (position, field))
        altered = copy.deepcopy(good)
        del altered[-1]['result']['observation']['completion']
        self.assertFalse(runner.promise_try_preflight_healthy(altered, files))
        altered = copy.deepcopy(good)
        altered[-1]['result']['observation']['extra'] = 'unbound'
        self.assertFalse(runner.promise_try_preflight_healthy(altered, files))

    def test_same_mode_raw_failure_and_forged_prerequisite_edges_poison_dependents(self):
        self.require_try()
        targets = [('promise-methods-property-helper-positive', 'strict'),
                   ('promise-try-api', 'strict'), ('promise-try-async-helper', 'strict')]
        for identity in targets:
            def change(row, result):
                if (row['name'], row['mode']) == identity:
                    result['observation']['error_type'] = 'WrongName'
                return result
            _, outcomes = self.capture(change)
            observed = {(item['name'], item['result']['mode']): item for item in outcomes}
            self.assertFalse(observed[identity]['verified'])
            dependents = [r for r in self.rows if list(identity) in r['prerequisites']]
            self.assertTrue(dependents)
            for row in dependents:
                self.assertFalse(observed[(row['name'], row['mode'])]['verified'], row['id'])
            self.assertTrue(observed[('promise-try-api', 'sloppy')]['verified'])
            self.assertFalse(runner.promise_try_preflight_healthy(outcomes, self.files()))
        good, files = self.healthy(), self.files()
        for action in ('drop', 'duplicate', 'reorder', 'self-cycle', 'mode', 'name', 'case_sha256',
                       'source_sha256', 'verified', 'boolean', 'extra', 'unhashable'):
            altered = copy.deepcopy(good)
            refs = altered[-1]['prerequisites']
            if action == 'drop':
                refs.pop()
            elif action == 'duplicate':
                refs.append(copy.deepcopy(refs[-1]))
            elif action == 'reorder':
                refs.reverse()
            elif action == 'self-cycle':
                refs[-1] = self.reference(('promise-try-async-helper-mismatch', 'strict'))
            elif action == 'mode':
                refs[-1]['mode'] = 'sloppy'
            elif action == 'verified':
                refs[-1]['verified'] = False
            elif action == 'boolean':
                refs[-1]['verified'] = 1
            elif action == 'extra':
                refs[-1]['extra'] = True
            elif action == 'unhashable':
                refs[-1]['name'] = []
            else:
                refs[-1][action] = 'forged'
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files), action)

    def test_each_missing_or_changed_helper_invalidates_held_fingerprints(self):
        self.require_try()
        good = self.healthy()
        for name in self.oracle['helpers']:
            files = self.files()
            files['harness/' + name] += b'\n'
            self.assertFalse(runner.promise_try_preflight_healthy(good, files), name)
            del files['harness/' + name]
            self.assertFalse(runner.promise_try_preflight_healthy(good, files), name)

    def test_gate_reclassifies_every_record_through_the_unchanged_promise_classifier(self):
        self.require_try()
        good, files = self.healthy(), self.files()
        with patch.object(runner, 'classify', side_effect=AssertionError('ordinary classifier forbidden')), \
                patch.object(runner, 'classify_promise', wraps=runner.classify_promise) as classified:
            self.assertTrue(runner.promise_try_preflight_healthy(good, files))
        self.assertEqual(classified.call_count, 94)
        self.assertTrue(all(not call.kwargs for call in classified.call_args_list))
        with patch.object(runner, 'classify_promise', return_value='passed'):
            self.assertFalse(runner.promise_try_preflight_healthy(good, files))

    def test_new_semantic_raw_fields_are_exact_but_sync_diagnostics_remain_bounded(self):
        self.require_try()
        good, files = self.healthy(), self.files()
        sync = next(i for i, r in enumerate(self.rows) if r['name'] == 'promise-try-api-mismatch')
        altered = copy.deepcopy(good)
        altered[sync]['result']['observation']['message'] = 'alternative bounded diagnostic'
        self.assertTrue(runner.promise_try_preflight_healthy(altered, files))
        for position in range(74, 94):
            counterpart = position + 1 if position % 2 == 0 else position - 1
            altered = copy.deepcopy(good)
            altered[position]['result']['observation'] = copy.deepcopy(good[counterpart]['result']['observation'])
            self.assertFalse(runner.promise_try_preflight_healthy(altered, files), position)
        altered = copy.deepcopy(good)
        altered[-1]['result']['observation']['message'] = 'queued-exception'
        self.assertFalse(runner.promise_try_preflight_healthy(altered, files))
        altered = copy.deepcopy(good)
        altered[-1]['result']['observation']['harness'] = 'asyncHelpers.js'
        self.assertFalse(runner.promise_try_preflight_healthy(altered, files))

    def test_unhealthy_gate_blocks_all_30_modes_and_preserves_both_baseline_destinations(self):
        self.require_try()
        outcomes = self.healthy()
        outcomes[-1]['result']['observation']['harness'] = 'forged-helper-attribution'
        cases = [self.case(row) for row in self.oracle['original_cases_for_blocking']]
        self.assertEqual(len(cases), 30)
        self.assertEqual(sum('class' in c['metadata']['features'] for c in cases), 6)
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            binary = base / 'adapter'
            binary.write_bytes(b'inert')
            for exists in (False, True):
                baseline = base / ('existing.json' if exists else 'absent.json')
                sentinel = b'unchanged baseline bytes\n'
                if exists:
                    baseline.write_bytes(sentinel)
                report = base / ('existing-report.json' if exists else 'absent-report.json')
                manifest = dict(repository='fixture', revision='fixture', test_files=15)
                with patch.object(runner, 'load_corpus', return_value=(manifest, self.files(), cases, [], '0' * 64)), \
                        patch.object(runner, 'harness_preflight', return_value=outcomes), \
                        patch.object(runner, 'run_case', side_effect=AssertionError('blocked case dispatched')) as called, \
                        patch('sys.argv', ['test262_conformance.py', '--profile', PROFILE, '--binary', str(binary),
                                           '--output', str(report), '--record-baseline', str(baseline)]), \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.main(), 1)
                    called.assert_not_called()
                if exists:
                    self.assertEqual(baseline.read_bytes(), sentinel)
                else:
                    self.assertFalse(baseline.exists())
                document = json.loads(report.read_bytes())
                self.assertEqual(document['counts'], {'blocked': 30})
                self.assertIs(document['harness_preflight_passed'], False)
                self.assertEqual([c['id'] for c in document['cases']], [c['id'] for c in cases])
                self.assertEqual(document['runner_policy']['response_protocol'], 'ERJR3')


if __name__ == '__main__':
    unittest.main()
