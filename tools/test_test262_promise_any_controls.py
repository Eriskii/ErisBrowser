"""Frozen Any literals/canonical gate controls; no adapter or network executes."""
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

PROFILE = 'promise-any'
ORACLE = Path(__file__).with_name('promise-any-controls-oracle.json')
ORACLE_SHA = 'f08ed8e2dd1c65e2d63b44286a8ab24d4bcf01308655942a9da463fb0c5ba136'
CORPUS = runner.ROOT / 'tests/upstream/test262-promise-any'


def digest(data):
    return hashlib.sha256(data).hexdigest()


class PromiseAnyControlsTests(unittest.TestCase):
    def setUp(self):
        self.assertEqual(digest(ORACLE.read_bytes()), ORACLE_SHA)
        self.oracle = json.loads(ORACLE.read_bytes())
        self.rows = self.oracle['records']
        self.index = {(row['name'], row['mode']): row for row in self.rows}
        for owner, name in ((importer, 'fetch'), (runner, 'bounded_process')):
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def files(self):
        files = {}
        for name, binding in self.oracle['helpers'].items():
            data = (CORPUS / 'harness' / name).read_bytes()
            self.assertEqual((len(data), digest(data)), (binding['bytes'], binding['sha256']))
            files['harness/' + name] = data
        return files

    def case(self, row):
        files = self.files()
        return dict(id=row['id'], file=row['file'], mode=row['mode'],
                    source=row['source'].encode(), metadata=copy.deepcopy(row['metadata']),
                    harness=[(h['name'], files['harness/' + h['name']]) for h in row['harness']],
                    completion_policy=row['completion_policy'], case_sha256=row['case_sha256'])

    def result(self, row):
        return dict(id=row['id'], file=row['file'], mode=row['mode'],
                    case_sha256=row['case_sha256'], source_sha256=row['source_sha256'],
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
                             (Path('/inert'), 3.0, runner.PROMISE_ANY_FEATURES, PROFILE))
            row = self.index[(case['id'].removeprefix('harness-preflight:'), case['mode'])]
            captured.append(copy.deepcopy(case))
            result = self.result(row)
            return change(row, result) if change else result
        with patch.object(runner, 'run_case', side_effect=observe):
            outcomes = runner.harness_preflight(self.files(), Path('/inert'), 3.0, PROFILE)
        return captured, outcomes

    def test_14_literal_variants_and_102_independently_derived_case_identities(self):
        self.assertEqual(runner.promise_any_preflight_variants(), self.oracle['variants'])
        self.assertEqual(runner.promise_preflight_variants(), self.oracle['common_variants'])
        definitions = runner.promise_any_preflight_cases(self.files())
        self.assertEqual(len(definitions), 102)
        for (definition, case), held in zip(definitions, self.rows):
            self.assertEqual(case, self.case(held), held['id'])
            self.assertEqual((definition['name'], definition['expected']), (held['name'], held['expected']))
            self.assertEqual([list(key) for key in definition['prerequisites']], held['prerequisites'])
            identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                            metadata=case['metadata'], completion_policy=case['completion_policy'],
                            harness=[dict(name=n, sha256=digest(b)) for n, b in case['harness']])
            self.assertEqual(digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode()),
                             held['case_sha256'])
        self.assertEqual([sum(row['mode'] == mode for row in self.rows) for mode in ('sloppy', 'strict')], [51, 51])

    def test_composer_routes_every_row_under_any_and_reconstructs_canonical_records(self):
        captured, outcomes = self.capture()
        self.assertEqual(captured, [self.case(row) for row in self.rows])
        self.assertEqual(outcomes, self.healthy())
        self.assertTrue(runner.promise_any_preflight_healthy(outcomes, self.files()))
        self.assertTrue(runner.promise_any_preflight_healthy(list(reversed(outcomes)), self.files()))

    def test_common_74_sources_fingerprints_and_prerequisite_records_remain_exact(self):
        legacy = json.loads(Path(__file__).with_name('promise-methods-runner-oracle.json').read_bytes())
        self.assertEqual(self.oracle['common_variants'], legacy['controls'])
        captured = []
        def observe(case, *args):
            row = self.index[(case['id'].removeprefix('harness-preflight:'), case['mode'])]
            captured.append(copy.deepcopy(case))
            return self.result(row)
        with patch.object(runner, 'run_case', side_effect=observe):
            original = runner.promise_harness_preflight(self.files(), Path('/inert'), 3.0)
        self.assertEqual(captured, [self.case(row) for row in self.rows[:74]])
        self.assertEqual(original, self.healthy()[:74])

    def test_missing_duplicate_extra_and_forged_canonical_result_fields_refuse(self):
        good = self.healthy()
        for altered in (good[:-1], good + [copy.deepcopy(good[-1])], good[:-1] + [copy.deepcopy(good[0])]):
            self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))
        for index in (0, 37, 74, 88, 101):
            for key, value in (('id', 'forged'), ('file', 'other.js'), ('mode', 'other'),
                               ('case_sha256', '0' * 64), ('source_sha256', '1' * 64),
                               ('expected_negative', {'type': 'AggregateError'}), ('status', 'unsupported')):
                with self.subTest(index=index, key=key):
                    altered = copy.deepcopy(good)
                    altered[index]['result'][key] = value
                    self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))
            for key, value in (('name', 'forged'), ('expected', 'wrong'), ('verified', False)):
                altered = copy.deepcopy(good)
                altered[index][key] = value
                self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))
        altered = copy.deepcopy(good)
        altered[-1]['result']['trusted'] = True
        self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))

    def test_malformed_unhashable_nested_fields_return_false_without_exceptions(self):
        good = self.healthy()
        for malformed in (None, {}, [None] * 102, 'rows', 102):
            self.assertFalse(runner.promise_any_preflight_healthy(malformed, self.files()))
        changes = [(['name'], []), (['expected'], {}), (['verified'], 1),
                   (['result'], []), (['result', 'mode'], []),
                   (['result', 'case_sha256'], {}), (['result', 'observation'], []),
                   (['result', 'observation', 'completion'], []),
                   (['result', 'observation', 'error_identity'], []),
                   (['result', 'observation', 'message'], '\ud800'),
                   (['prerequisites'], {}), (['prerequisites'], [[]]),
                   (['prerequisites', 0, 'name'], []), (['prerequisites', 0, 'mode'], {}),
                   (['prerequisites', 0, 'verified'], 1)]
        for path, value in changes:
            with self.subTest(path=path):
                altered = copy.deepcopy(good)
                target = altered[-1]
                for key in path[:-1]:
                    target = target[key]
                target[path[-1]] = value
                self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))

    def test_verified_true_does_not_replace_raw_schema_evidence_or_classification(self):
        good = self.healthy()
        for identity in (('promise-methods-promise-api-positive', 'sloppy'),
                         ('promise-any-errors-order-mismatch', 'strict'),
                         ('promise-any-genuine-identity', 'sloppy')):
            position = next(i for i, r in enumerate(self.rows) if (r['name'], r['mode']) == identity)
            for field, value in (('policy', True), ('policy', 1 - self.rows[position]['completion_policy']),
                                 ('done_calls', True), ('flags', -1), ('flags', 512),
                                 ('audited', 1), ('quiescent', False), ('unhandled', 1)):
                altered = copy.deepcopy(good)
                altered[position]['result']['observation']['completion'][field] = value
                self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()), (identity, field))
            for field, value in (('error_type', []), ('message', 'x' * 16385),
                                 ('harness', 'assert.js'), ('phase', 'parse'), ('error_identity', 'UnknownIntrinsic')):
                altered = copy.deepcopy(good)
                altered[position]['result']['observation'][field] = value
                self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()), (identity, field))
        altered = copy.deepcopy(good)
        del altered[-1]['result']['observation']['completion']['audited']
        self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))
        altered = copy.deepcopy(good)
        altered[-1]['result']['observation']['extra'] = 'unbound'
        self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))

    def test_genuine_renamed_and_fake_identity_observations_cannot_substitute(self):
        good = self.healthy()
        for mode in ('sloppy', 'strict'):
            positions = [next(i for i, r in enumerate(self.rows) if r['name'] == 'promise-any-' + n and r['mode'] == mode)
                         for n in ('genuine-identity', 'renamed-identity', 'fake-own-constructor', 'fake-prototype')]
            for target, source in ((positions[0], positions[1]), (positions[0], positions[2]),
                                   (positions[1], positions[0]), (positions[2], positions[0]),
                                   (positions[3], positions[0])):
                altered = copy.deepcopy(good)
                altered[target]['result']['observation'] = copy.deepcopy(good[source]['result']['observation'])
                self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))
            altered = copy.deepcopy(good)
            altered[positions[3]]['result'] = copy.deepcopy(good[positions[2]]['result'])
            self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()))

    def test_failed_same_mode_positives_poison_all_canonical_dependents(self):
        selected = [r for r in self.rows if r['mode'] == 'strict' and
                    (r['name'] in ('promise-methods-promise-helper-positive',
                                   'promise-methods-property-helper-positive') or
                     (r['name'].startswith('promise-any-') and r['expected'] == 'passed'))]
        for target in selected:
            def change(row, result):
                if (row['name'], row['mode']) == (target['name'], 'strict'):
                    result['observation']['error_type'] = 'WrongName'
                return result
            _, outcomes = self.capture(change)
            observed = {(r['name'], r['result']['mode']): r for r in outcomes}
            self.assertFalse(observed[(target['name'], 'strict')]['verified'])
            dependents = [r for r in self.rows if [target['name'], 'strict'] in r['prerequisites']]
            self.assertTrue(dependents)
            for row in dependents:
                self.assertFalse(observed[(row['name'], row['mode'])]['verified'], row['id'])
            self.assertTrue(observed[('promise-any-api', 'sloppy')]['verified'])
            self.assertFalse(runner.promise_any_preflight_healthy(outcomes, self.files()))

    def test_prerequisite_missing_extra_order_wrong_mode_hash_or_boolean_refuses(self):
        good = self.healthy()
        for action in ('drop', 'duplicate', 'reorder', 'mode', 'name', 'case_sha256', 'source_sha256', 'verified', 'extra'):
            altered = copy.deepcopy(good)
            refs = altered[-1]['prerequisites']
            if action == 'drop':
                refs.pop()
            elif action == 'duplicate':
                refs.append(copy.deepcopy(refs[-1]))
            elif action == 'reorder':
                refs.reverse()
            elif action == 'mode':
                refs[-1]['mode'] = 'sloppy'
            elif action == 'verified':
                refs[-1]['verified'] = False
            elif action == 'extra':
                refs[-1]['unbound'] = True
            else:
                refs[-1][action] = 'forged'
            self.assertFalse(runner.promise_any_preflight_healthy(altered, self.files()), action)

    def test_changed_or_absent_helper_bytes_cannot_reuse_healthy_fingerprints(self):
        good = self.healthy()
        for helper in self.oracle['helpers']:
            files = self.files()
            files['harness/' + helper] += b'\n'
            self.assertFalse(runner.promise_any_preflight_healthy(good, files), helper)
            del files['harness/' + helper]
            self.assertFalse(runner.promise_any_preflight_healthy(good, files), helper)

    def test_gate_reclassifies_every_raw_record_under_selected_any_context(self):
        good = self.healthy()
        original = runner.classify
        with patch.object(runner, 'classify', wraps=original) as checked:
            self.assertTrue(runner.promise_any_preflight_healthy(good, self.files()))
        self.assertEqual(checked.call_count, 102)
        self.assertTrue(all(call.kwargs == {'profile': PROFILE} for call in checked.call_args_list))
        with patch.object(runner, 'classify', return_value='passed'):
            self.assertFalse(runner.promise_any_preflight_healthy(good, self.files()))

    def test_main_failed_canonical_gate_blocks_dispatch_and_baseline_creation(self):
        outcomes = self.healthy()
        outcomes[-1]['result']['observation']['error_identity'] = 'AggregateError'
        case = self.case(self.rows[0])
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            binary = base / 'adapter'
            binary.write_bytes(b'inert')
            report, baseline = base / 'report.json', base / 'baseline.json'
            manifest = dict(repository='fixture', revision='fixture', test_files=1)
            with patch.object(runner, 'load_corpus', return_value=(manifest, self.files(), [case], [], '0' * 64)), \
                    patch.object(runner, 'harness_preflight', return_value=outcomes), \
                    patch.object(runner, 'run_case', side_effect=AssertionError('blocked case dispatched')) as called, \
                    patch('sys.argv', ['test262_conformance.py', '--profile', PROFILE, '--binary', str(binary),
                                       '--output', str(report), '--record-baseline', str(baseline)]), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(runner.main(), 1)
                called.assert_not_called()
            self.assertFalse(baseline.exists())
            document = json.loads(report.read_bytes())
            self.assertEqual(document['counts'], {'blocked': 1})
            self.assertIs(document['harness_preflight_passed'], False)
            self.assertEqual(document['runner_policy']['response_protocol'], 'ERJR3')


if __name__ == '__main__':
    unittest.main()
