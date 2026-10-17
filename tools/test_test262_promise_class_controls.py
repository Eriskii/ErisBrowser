"""Independent class/Promise gate oracles; adapters and main dispatch are mocked."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

import test262_conformance as runner

ORACLE = Path(__file__).with_name('promise-class-controls-oracle.json')
ORACLE_SHA = 'c1d75f68d8b49e8eebe4dee1627fce658c6a632ef462d15d381cb31d8263bb2e'
ORDINARY = 'promise-with-resolvers'
APIS = ('promise_class_preflight_variants', 'promise_class_preflight_cases',
        'promise_class_preflight_reference', 'promise_class_preflight_checks',
        'promise_class_preflight_healthy', 'promise_class_harness_preflight')
OLD_GATES = {'promise-methods-async': 'promise_preflight_healthy',
             'promise-all': 'promise_all_preflight_healthy',
             'promise-race': 'promise_race_preflight_healthy',
             'promise-all-settled': 'promise_all_settled_preflight_healthy'}
FIELDS = ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def fingerprint(case):
    identity = dict(source_sha256=digest(case['source']), mode=case['mode'],
                    metadata=case['metadata'],
                    harness=[dict(name=n, sha256=digest(b)) for n, b in case['harness']])
    if 'completion_policy' in case:
        identity['completion_policy'] = case['completion_policy']
    return digest(json.dumps(identity, sort_keys=True, ensure_ascii=True).encode())


def set_path(value, path, replacement):
    for key in path[:-1]:
        value = value[key]
    value[path[-1]] = replacement


class PromiseClassControlsTests(unittest.TestCase):
    def setUp(self):
        data = ORACLE.read_bytes()
        self.assertEqual(digest(data), ORACLE_SHA)
        self.oracle = json.loads(data)
        self.profiles = self.oracle['profiles']
        self.file_cache = {}
        guard = patch.object(runner, 'bounded_process',
                             side_effect=AssertionError('real adapter execution forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def require_class(self):
        for name in APIS:
            self.assertTrue(callable(getattr(runner, name, None)),
                            'selected class gate API is absent: ' + name)

    def files(self, profile):
        if profile not in self.file_cache:
            corpus = runner.ROOT / 'tests/upstream' / ('test262-' + profile)
            files = {}
            for held in self.oracle['helpers'][profile]:
                data = (corpus / held['path']).read_bytes()
                self.assertEqual((len(data), digest(data)), (held['bytes'], held['sha256']))
                files[held['path']] = data
            self.file_cache[profile] = files
        return dict(self.file_cache[profile])

    def refs(self, indices):
        return [copy.deepcopy(self.oracle['references'][i]) for i in indices]

    def legacy(self, profile):
        rows = []
        for i in self.oracle['legacy_prefixes'][profile]:
            item = copy.deepcopy(self.oracle['legacy_rows'][i])
            item['result']['observation'] = copy.deepcopy(
                self.oracle['legacy_observations'][item['result']['observation']])
            item['prerequisites'] = self.refs(item['prerequisites'])
            rows.append(item)
        return rows

    def new_rows(self, profile):
        return self.oracle['new_rows'][profile]

    def case(self, held):
        files = self.files(held['profile'])
        value = dict(id=held['id'], file=held['file'], mode=held['mode'],
                     source=self.oracle['literals'][held['literal']]['source'].encode(),
                     metadata=copy.deepcopy(held['metadata']),
                     harness=[(h['name'], files['harness/' + h['name']]) for h in held['harness']],
                     case_sha256=held['case_sha256'])
        if 'completion_policy' in held:
            value['completion_policy'] = held['completion_policy']
        return value

    def result(self, held):
        observation = copy.deepcopy(held['required_observation'])
        if held['message_rule']['kind'] == 'bounded-diagnostic':
            observation['message'] = 'intentional paired assertion diagnostic'
        return dict(id=held['id'], file=held['file'], mode=held['mode'],
                    case_sha256=held['case_sha256'], source_sha256=held['source_sha256'],
                    expected_negative=None, status=held['expected_status'], observation=observation)

    def healthy_fixture(self, profile):
        rows = self.legacy(profile)
        for held in self.new_rows(profile):
            rows.append(dict(name=held['name'], expected=held['expected'], verified=True,
                             result=self.result(held), prerequisites=self.refs(held['prerequisites'])))
        return rows

    def refuse(self, rows, files, profile):
        self.assertFalse(runner.promise_class_preflight_healthy(rows, files, profile))

    def inventory(self, profile):
        corpus = runner.ROOT / 'tests/upstream' / ('test262-' + profile)
        loaded = runner.load_corpus(corpus, profile)
        observed = [[case['id'], case['case_sha256'], digest(case['source'])]
                    for case in loaded[2]]
        self.assertEqual(observed, self.oracle['inventory_identities'][profile])
        return corpus, loaded

    def main_call(self, profile, loaded, corpus, binary, output, baseline):
        args = ['test262_conformance.py', '--profile', profile, '--corpus', str(corpus),
                '--binary', str(binary), '--output', str(output), '--record-baseline',
                str(baseline), '--jobs', '1']
        with patch.object(runner.sys, 'argv', args), \
                patch.object(runner, 'load_corpus', return_value=loaded), \
                contextlib.redirect_stdout(io.StringIO()), \
                contextlib.redirect_stderr(io.StringIO()):
            return runner.main()

    def test_frozen_oracle_retains_eighteen_sources_and_single_assertion_pairs(self):
        self.assertEqual(self.oracle['counts'], dict(old=588, new=84, combined=672, inventory=1280))
        self.assertEqual(sum(len(x) for x in self.oracle['legacy_prefixes'].values()), 588)
        self.assertEqual(sum(len(x) for x in self.oracle['new_rows'].values()), 84)
        self.assertEqual(sum(len(x) for x in self.oracle['inventory_identities'].values()), 1280)
        sources = self.oracle['literals']
        self.assertEqual(len(sources), 18)
        for positive, mismatch in zip(sources[::2], sources[1::2]):
            self.assertEqual(positive['variant'], 'positive')
            self.assertEqual(mismatch['variant'], 'mismatch')
            self.assertEqual(positive['family'], mismatch['family'])
            for row in (positive, mismatch):
                self.assertEqual(digest(row['source'].encode()), row['sha256'])
            a, b = positive['source'].splitlines(), mismatch['source'].splitlines()
            self.assertEqual(len(a), len(b))
            changes = [(x, y) for x, y in zip(a, b) if x != y]
            self.assertEqual(len(changes), 1)
            self.assertTrue(all('assert.sameValue(' in line for line in changes[0]))

    def test_canonical_cases_preserve_588_prefixes_and_all_84_exact_new_definitions(self):
        self.require_class()
        old_count = new_count = 0
        for profile in self.profiles:
            with self.subTest(profile=profile):
                definitions = runner.promise_class_preflight_cases(self.files(profile), profile)
                old, new = self.legacy(profile), self.new_rows(profile)
                self.assertEqual(len(definitions), len(old) + 12)
                for (row, case), item in zip(definitions, old):
                    result = item['result']
                    self.assertEqual((row['name'], row['expected']), (item['name'], item['expected']))
                    self.assertEqual({k: case[k] for k in ('id', 'file', 'mode', 'case_sha256')},
                                     {k: result[k] for k in ('id', 'file', 'mode', 'case_sha256')})
                    self.assertEqual(digest(case['source']), result['source_sha256'])
                    self.assertEqual(fingerprint(case), result['case_sha256'])
                    self.assertEqual(case['metadata']['negative'], result['expected_negative'])
                    self.assertEqual(list(row['prerequisites']),
                                     [(r['name'], r['mode']) for r in item['prerequisites']])
                    old_count += 1
                for (row, case), held in zip(definitions[len(old):], new):
                    self.assertEqual(case, self.case(held))
                    self.assertEqual(fingerprint(case), held['case_sha256'])
                    self.assertEqual((row['name'], row['expected']), (held['name'], held['expected']))
                    refs = self.refs(held['prerequisites'])
                    self.assertEqual(list(row['prerequisites']), [(r['name'], r['mode']) for r in refs])
                    expected_ref = {k: held[k] for k in
                                    ('name', 'mode', 'id', 'case_sha256', 'source_sha256', 'expected')}
                    self.assertEqual(runner.promise_class_preflight_reference(row, case), expected_ref)
                    self.assertEqual(case['metadata']['features'], ['class'])
                    new_count += 1
        self.assertEqual((old_count, new_count), (588, 84))

    def test_wrapper_preserves_old_records_and_routes_only_twelve_new_cases_per_profile(self):
        self.require_class()
        for profile in self.profiles:
            old, good, files = self.legacy(profile), self.healthy_fixture(profile), self.files(profile)
            index = {(r['id'], r['mode']): r for r in self.new_rows(profile)}
            captured = []
            def observe(case, binary, timeout, features, selected):
                self.assertEqual((binary, timeout, selected), (Path('/inert'), 3.0, profile))
                self.assertEqual(features, runner.PROFILE_FEATURES[profile])
                self.assertIn('class', features)
                captured.append(copy.deepcopy(case))
                return self.result(index[(case['id'], case['mode'])])
            with patch.object(runner, 'harness_preflight', return_value=copy.deepcopy(old)) as original, \
                    patch.object(runner, 'run_case', side_effect=observe):
                result = runner.promise_class_harness_preflight(files, Path('/inert'), 3.0, profile)
            original.assert_called_once_with(files, Path('/inert'), 3.0, profile)
            self.assertEqual(captured, [self.case(row) for row in self.new_rows(profile)])
            self.assertEqual(result, good)
            self.assertEqual(result[:len(old)], old)
            checks = runner.promise_class_preflight_checks(result, files, profile)
            self.assertEqual(checks, {(x['name'], x['result']['mode']): True for x in good})
            self.assertTrue(runner.promise_class_preflight_healthy(result, files, profile))
            # The wrapper may not repair a legacy producer's failed verified flag.
            old[0]['verified'] = False
            with patch.object(runner, 'harness_preflight', return_value=copy.deepcopy(old)), \
                    patch.object(runner, 'run_case', side_effect=observe):
                result = runner.promise_class_harness_preflight(files, Path('/inert'), 3.0, profile)
            self.assertEqual(result[:len(old)], old)
            self.refuse(result, files, profile)

    def test_missing_extra_reordered_and_malformed_identity_rows_are_unhealthy(self):
        self.require_class()
        for profile in self.profiles:
            good, files = self.healthy_fixture(profile), self.files(profile)
            for bad in (None, {}, 'rows', [None] * len(good), good[:-1],
                        good + [copy.deepcopy(good[-1])], list(reversed(good)),
                        good[:-1] + [copy.deepcopy(good[0])]):
                self.refuse(bad, files, profile)
            for position in (0, len(good) - 12, len(good) - 1):
                mutations = [(['name'], []), (['name'], 'forged'), (['expected'], {}),
                             (['verified'], 1), (['verified'], False), (['result'], []),
                             (['result', 'id'], 'other'), (['result', 'file'], 'other.js'),
                             (['result', 'mode'], []), (['result', 'mode'], 'module'),
                             (['result', 'source_sha256'], '0' * 64),
                             (['result', 'case_sha256'], '1' * 64),
                             (['result', 'expected_negative'], {'type': 'TypeError'}),
                             (['result', 'status'], 'unsupported'), (['prerequisites'], {})]
                for path, value in mutations:
                    with self.subTest(profile=profile, position=position, path=path):
                        bad = copy.deepcopy(good)
                        set_path(bad[position], path, value)
                        self.refuse(bad, files, profile)
                for location in ((), ('result',)):
                    bad = copy.deepcopy(good)
                    target = bad[position]
                    for key in location:
                        target = target[key]
                    target['unbound'] = True
                    self.refuse(bad, files, profile)
            for position in (0, len(good) - 13, *range(len(good) - 12, len(good))):
                bad = copy.deepcopy(good)
                bad[position]['verified'] = False
                self.refuse(bad, files, profile)

    def test_new_raw_fields_host_counters_and_utf8_types_cannot_be_forged(self):
        self.require_class()
        for profile in ('promise-methods-async', 'promise-any', ORDINARY):
            good, files = self.healthy_fixture(profile), self.files(profile)
            for position in (len(good) - 12, len(good) - 11, len(good) - 2, len(good) - 1):
                mutations = [('status', []), ('status', 'complete' if position % 2 else 'exception'),
                             ('phase', 'parse'), ('error_type', 'WrongDiagnostic'),
                             ('error_identity', 'AggregateError'), ('harness', 'assert.js'),
                             ('message', '\ud800'), ('message', 'é' * 8193)]
                for field, value in mutations:
                    # A repeated value is not a mutation; every selected row is still checked.
                    if good[position]['result']['observation'].get(field) == value:
                        continue
                    bad = copy.deepcopy(good)
                    bad[position]['result']['observation'][field] = value
                    self.refuse(bad, files, profile)
                for field in FIELDS:
                    bad = copy.deepcopy(good)
                    del bad[position]['result']['observation'][field]
                    self.refuse(bad, files, profile)
                bad = copy.deepcopy(good)
                bad[position]['result']['observation']['extra'] = ''
                self.refuse(bad, files, profile)
                if profile == ORDINARY:
                    continue
                evidence = good[position]['result']['observation']['completion']
                for field, value in [('policy', True), ('policy', 1 - evidence['policy']),
                                     ('done_calls', True), ('success_prints', True),
                                     ('failure_prints', True), ('flags', -1), ('flags', 512),
                                     ('quiescent', 1), ('audited', 1), ('quiescent', False),
                                     ('audited', False), ('unhandled', True), ('unhandled', 1)]:
                    bad = copy.deepcopy(good)
                    bad[position]['result']['observation']['completion'][field] = value
                    self.refuse(bad, files, profile)
                for change in ('missing', 'extra', 'not-dict'):
                    bad = copy.deepcopy(good)
                    observation = bad[position]['result']['observation']
                    if change == 'missing':
                        del observation['completion']
                    elif change == 'extra':
                        observation['completion']['extra'] = 0
                    else:
                        observation['completion'] = []
                    self.refuse(bad, files, profile)

    def test_sync_diagnostic_bound_is_utf8_bytes_and_async_message_is_exact(self):
        self.require_class()
        for profile in self.profiles:
            good, files = self.healthy_fixture(profile), self.files(profile)
            sync = len(good) - 11
            for text in ('', 'x' * 16384, 'é' * 8192):
                changed = copy.deepcopy(good)
                changed[sync]['result']['observation']['message'] = text
                self.assertTrue(runner.promise_class_preflight_healthy(changed, files, profile))
            for text in ('x' * 16385, 'é' * 8193, '\udfff', None, b'message'):
                changed = copy.deepcopy(good)
                changed[sync]['result']['observation']['message'] = text
                self.refuse(changed, files, profile)
            if profile != ORDINARY:
                for text in ('', 'queued-exception', 'completion-error '):
                    changed = copy.deepcopy(good)
                    changed[-1]['result']['observation']['message'] = text
                    self.refuse(changed, files, profile)

    def test_exact_prerequisites_and_failed_positive_evidence_control_dependents(self):
        self.require_class()
        for profile in self.profiles:
            good, files = self.healthy_fixture(profile), self.files(profile)
            for action in ('drop', 'duplicate', 'reverse', 'mode', 'id', 'name', 'expected',
                           'source_sha256', 'case_sha256', 'extra', 'missing', 'unhashable'):
                bad = copy.deepcopy(good)
                refs = bad[-1]['prerequisites']
                if action == 'drop':
                    refs.pop()
                elif action == 'duplicate':
                    refs.append(copy.deepcopy(refs[-1]))
                elif action == 'reverse':
                    refs.reverse()
                elif action == 'extra':
                    refs[-1]['verified'] = True
                elif action == 'missing':
                    del refs[-1]['id']
                elif action == 'unhashable':
                    refs[-1]['name'] = []
                else:
                    refs[-1][action] = 'sloppy' if action == 'mode' else 'forged'
                self.refuse(bad, files, profile)
            bad = copy.deepcopy(good)
            first = len(good) - 12
            # This is a structurally valid synchronous exception, not a malformed dictionary.
            bad[first]['result']['observation'] = copy.deepcopy(good[first + 1]['result']['observation'])
            bad[first]['result']['status'] = 'failed'
            checks = runner.promise_class_preflight_checks(bad, files, profile)
            self.assertIsInstance(checks, dict)
            for item in good[first:first + 6]:
                self.assertFalse(checks[(item['name'], 'sloppy')])
            self.assertTrue(checks[(good[first + 6]['name'], 'strict')])
            self.refuse(bad, files, profile)

    def test_canonical_self_and_two_node_cycles_reject_without_recursion(self):
        self.require_class()
        for profile in self.profiles:
            files, good = self.files(profile), self.healthy_fixture(profile)
            original = runner.promise_class_preflight_cases(files, profile)
            for two_nodes in (False, True):
                definitions, outcomes = copy.deepcopy(original), copy.deepcopy(good)
                targets = [-2, -1] if two_nodes else [-1]
                for i in targets:
                    target = -1 if i == -2 else -2 if two_nodes else -1
                    row, case = definitions[target]
                    definitions[i][0]['prerequisites'] = [(row['name'], case['mode'])]
                    outcomes[i]['prerequisites'] = [runner.promise_class_preflight_reference(row, case)]
                with patch.object(runner, 'promise_class_preflight_cases', return_value=definitions):
                    self.assertIsNone(runner.promise_class_preflight_checks(outcomes, files, profile))
                    self.refuse(outcomes, files, profile)

    def test_all_pinned_helpers_reject_missing_corrupt_or_wrong_type_before_producer(self):
        self.require_class()
        for profile in self.profiles:
            files, good = self.files(profile), self.healthy_fixture(profile)
            for name, original in files.items():
                for kind in ('missing', 'corrupt', 'str', 'bytearray', 'none'):
                    bad_files = dict(files)
                    if kind == 'missing':
                        del bad_files[name]
                    else:
                        bad_files[name] = {'corrupt': original + b'\n', 'str': original.decode(),
                                           'bytearray': bytearray(original), 'none': None}[kind]
                    with self.subTest(profile=profile, name=name, kind=kind):
                        self.assertIsNone(runner.promise_class_preflight_checks(good, bad_files, profile))
                        self.refuse(good, bad_files, profile)
                        with patch.object(runner, 'harness_preflight', side_effect=AssertionError('old producer reached')), \
                                patch.object(runner, 'run_case', side_effect=AssertionError('adapter reached')):
                            self.assertEqual(runner.promise_class_harness_preflight(
                                bad_files, Path('/inert'), 3.0, profile), [])
            # Self-consistent caller fingerprints still cannot authorize different helper bytes.
            changed_files = dict(files)
            changed_files['harness/assert.js'] += b'\n'
            forged = copy.deepcopy(good)
            definitions = runner.promise_class_preflight_cases(files, profile)
            hashes = {}
            for (row, case), item in zip(definitions, forged):
                case = copy.deepcopy(case)
                case['harness'] = [(name, changed_files['harness/' + name]) for name, _ in case['harness']]
                item['result']['case_sha256'] = fingerprint(case)
                hashes[(row['name'], case['mode'])] = item['result']['case_sha256']
            for item in forged:
                for ref in item['prerequisites']:
                    ref['case_sha256'] = hashes[(ref['name'], ref['mode'])]
            self.refuse(forged, changed_files, profile)

    def test_oldest_four_verified_flags_do_not_hide_forged_legacy_raw_results(self):
        self.require_class()
        for profile, old_gate in OLD_GATES.items():
            old, good, files = self.legacy(profile), self.healthy_fixture(profile), self.files(profile)
            for position in (0, 6, len(old) - 1):
                bad = copy.deepcopy(good)
                bad[position]['result']['observation']['status'] = 'invented'
                self.assertTrue(getattr(runner, old_gate)(bad[:len(old)]))
                self.refuse(bad, files, profile)
            with patch.object(runner, old_gate, return_value=False):
                self.refuse(good, files, profile)

    def test_ordinary_rows_refuse_completion_and_keep_selected_erjr3_rejection(self):
        self.require_class()
        good, files = self.healthy_fixture(ORDINARY), self.files(ORDINARY)
        for position in (0, len(good) - 12, len(good) - 1):
            for location in ('result', 'observation'):
                bad = copy.deepcopy(good)
                target = bad[position]['result']
                if location == 'observation':
                    target = target['observation']
                target['completion'] = dict(policy=0, done_calls=0, success_prints=0,
                                            failure_prints=0, flags=0, quiescent=True,
                                            audited=True, unhandled=0)
                self.refuse(bad, files, ORDINARY)
        case = self.case(self.new_rows(ORDINARY)[0])
        case['completion_policy'] = 0
        result = runner.run_case(case, Path('/inert'), 3.0,
                                 runner.PROFILE_FEATURES[ORDINARY], ORDINARY)
        self.assertEqual(result['status'], 'adapter-error')
        self.assertIn('ordinary', result['reason'])
        wire = bytearray(b'ERJR3')
        for text in ('complete', 'runtime', '', '', '', ''):
            encoded = text.encode()
            wire.extend(struct.pack('<I', len(encoded)))
            wire.extend(encoded)
        wire.extend(struct.pack('<BBBBHBBI', 0, 0, 0, 0, 0, 1, 1, 0))
        with self.assertRaises(ValueError):
            runner.decode_response(bytes(wire), profile=ORDINARY)

    def test_failed_class_gate_blocks_full_seven_inventories_and_never_records_baseline(self):
        self.require_class()
        total = 0
        for profile in self.profiles:
            corpus, loaded = self.inventory(profile)
            total += len(loaded[2])
            bad = self.healthy_fixture(profile)
            # Keep every claimed verified flag True: only raw canonical checking catches this.
            bad[-1]['result']['observation']['harness'] = 'forged-harness.js'
            for exists in (False, True):
                with tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    binary, output, baseline = root / 'inert', root / 'report.json', root / 'baseline.json'
                    binary.write_bytes(b'inert test fixture, never executed')
                    saved = b'{"sentinel":"preserve existing baseline bytes"}\n'
                    if exists:
                        baseline.write_bytes(saved)
                    with patch.object(runner, 'promise_class_harness_preflight', return_value=copy.deepcopy(bad)), \
                            patch.object(runner, 'run_case', side_effect=AssertionError('inventory adapter reached')):
                        code = self.main_call(profile, loaded, corpus, binary, output, baseline)
                    self.assertEqual(code, 1)
                    report = json.loads(output.read_bytes())
                    self.assertFalse(report['harness_preflight_passed'])
                    self.assertEqual(report['counts'], {'blocked': len(loaded[2])})
                    self.assertEqual([[x['id'], x['case_sha256'], x['source_sha256']] for x in report['cases']],
                                     self.oracle['inventory_identities'][profile])
                    self.assertTrue(all(x['status'] == 'blocked' for x in report['cases']))
                    self.assertEqual(baseline.exists(), exists)
                    if exists:
                        self.assertEqual(baseline.read_bytes(), saved)
        self.assertEqual(total, 1280)

    def test_healthy_mock_gate_dispatches_full_inventory_with_unchanged_protocol(self):
        self.require_class()
        total = 0
        for profile in self.profiles:
            corpus, loaded = self.inventory(profile)
            good, observed = self.healthy_fixture(profile), []
            def dispatch(case, binary, timeout, features, selected):
                self.assertEqual(selected, profile)
                self.assertEqual(features, runner.PROFILE_FEATURES[profile])
                self.assertIn('class', features)
                self.assertEqual(runner.encode_request(case)[:5], b'ERJS1' if profile == ORDINARY else b'ERJS2')
                observed.append([case['id'], case['case_sha256'], digest(case['source'])])
                # Synthetic pass is a dispatch fixture, never an upstream outcome prediction.
                observation = dict(status='complete', phase='runtime', error_type='',
                                   error_identity='', message='', harness='')
                if profile != ORDINARY:
                    policy = case['completion_policy']
                    observation['completion'] = dict(policy=policy, done_calls=policy,
                        success_prints=policy, failure_prints=0, flags=0,
                        quiescent=True, audited=True, unhandled=0)
                return dict(id=case['id'], file=case['file'], mode=case['mode'],
                            case_sha256=case['case_sha256'], source_sha256=digest(case['source']),
                            expected_negative=case['metadata']['negative'], status='passed', observation=observation)
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary, output, baseline = root / 'inert', root / 'report.json', root / 'baseline.json'
                binary.write_bytes(b'inert test fixture, never executed')
                with patch.object(runner, 'promise_class_harness_preflight', return_value=copy.deepcopy(good)), \
                        patch.object(runner, 'run_case', side_effect=dispatch):
                    code = self.main_call(profile, loaded, corpus, binary, output, baseline)
                self.assertEqual(code, 0)
                self.assertEqual(observed, self.oracle['inventory_identities'][profile])
                report, recorded = json.loads(output.read_bytes()), json.loads(baseline.read_bytes())
                self.assertTrue(report['harness_preflight_passed'])
                self.assertEqual(report['counts'], {'passed': len(loaded[2])})
                self.assertEqual(set(recorded), {'revision', 'corpus_manifest_sha256',
                                 'runner_policy_sha256', 'binary_sha256', 'cases'})
                self.assertEqual(recorded['cases'], {case['id']: {key: case[key]
                    for key in ('status', 'case_sha256', 'source_sha256')} for case in report['cases']})
                total += len(observed)
        self.assertEqual(total, 1280)


if __name__ == '__main__':
    unittest.main()
