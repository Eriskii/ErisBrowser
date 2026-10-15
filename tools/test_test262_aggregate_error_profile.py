"""Frozen AggregateError profile contracts; all adapter/network activity is mocked."""
import contextlib
import copy
import hashlib
import io
import json
from pathlib import Path
import shutil
import struct
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner


PROFILE = 'aggregate-error'
CORPUS = importer.ROOT / 'tests/upstream/test262-aggregate-error'
ORACLE = Path(__file__).with_name('aggregate-error-profile-oracle.json')
ORACLE_SHA = 'c7778f7fbeaa685ca0329cd614a4f935151b6cb5e7fd4e7889bc66f92574d5e2'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def frame(observation, version=b'ERJR2'):
    output = bytearray(version)
    for name in ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness'):
        data = observation.get(name, '').encode()
        output.extend(struct.pack('<I', len(data)))
        output.extend(data)
    if version == b'ERJR3':
        # Internally valid checked-sync completion or abrupt evidence. The new
        # profile must reject the wire version, not rely on malformed evidence.
        flags = 256 if observation['status'] == 'exception' else 0
        output.extend(struct.pack('<BBBBHBBI', 0, 0, 0, 0, flags, 1, 1, 0))
    return bytes(output)


class AggregateErrorProfileTests(unittest.TestCase):
    def setUp(self):
        raw = ORACLE.read_bytes()
        self.assertEqual(digest(raw), ORACLE_SHA)
        self.oracle = json.loads(raw)
        self.controls = self.oracle['controls']
        self.by_id = {row['id']: row for row in self.controls}
        # References are derived only from the frozen oracle, independently of
        # the runner's gate/factory. This avoids repeating each full common row.
        for row in self.controls:
            row['prerequisites'] = [{key: self.by_id[name][key] for key in
                ('id', 'name', 'mode', 'expected', 'source_sha256', 'case_sha256')}
                for name in row['prerequisite_ids']]
        for owner, name in [(importer, 'fetch'), (runner, 'bounded_process')]:
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def files(self):
        return {name: (CORPUS / name).read_bytes() for name in self.oracle['auxiliary_files']}

    def case(self, row):
        files = self.files()
        return dict(id=row['id'], file=row['file'], mode=row['mode'],
                    metadata=copy.deepcopy(row['metadata']), source=row['source'].encode(),
                    harness=[(item['name'], files['harness/' + item['name']]) for item in row['harness']],
                    case_sha256=row['case_sha256'])

    def observed(self, case, row):
        return dict(id=case['id'], file=case['file'], mode=case['mode'],
                    case_sha256=case['case_sha256'], source_sha256=digest(case['source']),
                    expected_negative=copy.deepcopy(case['metadata']['negative']),
                    status=row['expected'], observation=dict(row['required_observation'], message='synthetic control evidence'))

    def capture(self, mutate=None, files=None, direct=False):
        files = self.files() if files is None else files
        captured = []
        def observe(case, binary, timeout, features=None, profile=None):
            captured.append(copy.deepcopy(case))
            row = self.by_id[case['id']]
            result = self.observed(case, row)
            return mutate(case, row, result) if mutate else result
        with patch.object(runner, 'run_case', side_effect=observe):
            if direct:
                outcomes = runner.aggregate_error_harness_preflight(files, Path('/inert'), 3.0)
            else:
                outcomes = runner.harness_preflight(files, Path('/inert'), 3.0, PROFILE)
        return captured, outcomes

    def projection(self, case):
        return dict(id=case['id'], file=case['file'], mode=case['mode'], metadata=case['metadata'],
                    source=case['source'].decode(), source_sha256=digest(case['source']),
                    case_sha256=case['case_sha256'],
                    harness=[dict(name=name, sha256=digest(data)) for name, data in case['harness']])

    def report(self, preflight, cases, outcome=None, record=True, baseline=None):
        # Only the runner's dispatch/report/baseline logic executes. Neither the
        # temporary adapter nor any JS source is run.
        calls = []
        manifest = self.oracle['expected_manifest']
        def result(case, *args):
            calls.append(case['id'])
            if outcome:
                return outcome(case)
            reason = runner.unsupported_reason(case, set(self.oracle['features']), PROFILE)
            base = dict(id=case['id'], file=case['file'], mode=case['mode'], case_sha256=case['case_sha256'],
                        source_sha256=digest(case['source']), expected_negative=case['metadata']['negative'])
            if reason:
                return dict(base, status='unsupported', reason=reason)
            return dict(base, status='failed', observation=dict(status='exception', phase='runtime',
                        error_type='Error', error_identity='Error', message='explicit synthetic nonpass', harness=''))
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            binary, output, saved = (base / name for name in ('inert', 'report.json', 'baseline.json'))
            binary.write_bytes(b'never execute this file')
            args = ['test262_conformance.py', '--profile', PROFILE, '--binary', str(binary),
                    '--output', str(output)]
            if baseline is not None:
                saved.write_text(json.dumps(baseline))
                args += ['--baseline', str(saved)]
            elif record:
                args += ['--record-baseline', str(saved)]
            with patch.object(runner, 'load_corpus', return_value=(manifest, self.files(), cases, [], self.oracle['manifest_sha256'])), \
                    patch.object(runner, 'harness_preflight', return_value=preflight), \
                    patch.object(runner, 'run_case', side_effect=result), patch('sys.argv', args), \
                    contextlib.redirect_stdout(io.StringIO()):
                code = runner.main()
            return code, json.loads(output.read_bytes()), json.loads(saved.read_bytes()) if saved.exists() else None, calls

    def test_complete_raw_corpus_and_metadata_are_frozen_without_profile_lookup(self):
        raw = (CORPUS / 'manifest.json').read_bytes()
        self.assertEqual(digest(raw), self.oracle['manifest_sha256'])
        self.assertEqual(json.loads(raw), self.oracle['expected_manifest'])
        manifest = self.oracle['expected_manifest']
        self.assertEqual((manifest['test_files'], len(manifest['files']), len(manifest['inventory_proof']['files'])), (25, 33, 7))
        for row in manifest['files'] + manifest['inventory_proof']['files']:
            data = (CORPUS / row['path']).read_bytes()
            self.assertEqual((len(data), digest(data)), (row['bytes'], row['sha256']), row['path'])
        helpers = {'assert.js', 'sta.js'}
        for path, expected in self.oracle['metadata'].items():
            metadata = importer.parse_metadata((CORPUS / path).read_text())
            self.assertEqual(metadata, expected)
            self.assertEqual(metadata['flags'], [])
            self.assertIsNone(metadata['negative'])
            self.assertNotIn('timeout', metadata)
            self.assertEqual(runner.modes(metadata), ['sloppy', 'strict'])
            helpers.update(metadata['includes'])
        self.assertEqual(helpers, set(self.oracle['helpers']))

    def test_loader_keeps_50_ordinary_identities_and_metadata_scope(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(files), len(cases), fixtures, manifest_hash),
                         (25, 33, 50, [], self.oracle['manifest_sha256']))
        actual = [dict(id=c['id'], file=c['file'], mode=c['mode'], case_sha256=c['case_sha256'],
                       source_sha256=digest(c['source'])) for c in cases]
        self.assertEqual(actual, self.oracle['case_identities'])
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], set(self.oracle['features']))
        for case in cases:
            self.assertNotIn('completion_policy', case)
            self.assertEqual(runner.encode_request(case)[:7], b'ERJS1' + bytes([case['mode'] == 'strict', 0]))
        excluded = [c['id'] for c in cases if runner.unsupported_reason(c, runner.PROFILE_FEATURES[PROFILE], PROFILE)]
        self.assertEqual(excluded, self.oracle['metadata_exclusions'])
        for case in cases:
            if case['file'] in self.oracle['untagged_proxy_sources']:
                self.assertIsNone(runner.unsupported_reason(case, runner.PROFILE_FEATURES[PROFILE], PROFILE))
        source = 'test/built-ins/AggregateError/cause-property.js'
        helper = 'harness/assert.js'
        proof = 'inventory-proof/tree-29aa43922ae5a34467878bf9f498ffc4a89bdf67.json'
        for path in (source, helper, proof):
            for operation in ('corrupt', 'delete', 'unbound'):
                with self.subTest(path=path, operation=operation), tempfile.TemporaryDirectory() as temporary:
                    target = Path(temporary) / 'corpus'
                    shutil.copytree(CORPUS, target)
                    modified = copy.deepcopy(manifest)
                    entries = modified['inventory_proof']['files'] if path == proof else modified['files']
                    entry = next(r for r in entries if r['path'] == path)
                    if operation == 'delete':
                        (target / path).unlink()
                    else:
                        raw = (target / path).read_bytes() + b'\n'
                        if operation == 'unbound' and path == proof:
                            document = json.loads(raw)
                            document['tree'][0]['sha'] = '0' * 40
                            raw = json.dumps(document).encode()
                        (target / path).write_bytes(raw)
                        if operation == 'unbound':
                            entry.update(bytes=len(raw), sha256=digest(raw))
                    (target / 'manifest.json').write_text(json.dumps(modified))
                    with self.assertRaises((ValueError, OSError)):
                        runner.load_corpus(target, PROFILE)
        for extra in ('test/built-ins/AggregateError/unlisted.js', 'harness/unlisted.js',
                      'inventory-proof/tree-' + '0' * 40 + '.json'):
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as temporary:
                target = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, target)
                (target / extra).write_bytes(b'{}')
                modified = copy.deepcopy(manifest)
                if extra.endswith('.json'):
                    modified['inventory_proof']['files'].append(dict(path=extra, bytes=2, sha256=digest(b'{}'),
                        url='https://api.github.com/repos/tc39/test262/git/trees/' + '0' * 40))
                    (target / 'manifest.json').write_text(json.dumps(modified))
                with self.assertRaises(ValueError):
                    runner.load_corpus(target, PROFILE)
        # Altering a declared count together with disk/manifest inventory must
        # not conceal the root tree's authenticated prototype descendant.
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / 'corpus'
            shutil.copytree(CORPUS, target)
            shutil.rmtree(target / 'test/built-ins/AggregateError/prototype')
            modified = copy.deepcopy(manifest)
            del modified['directories']['AggregateError/prototype']
            modified['test_files'] = 19
            modified['files'] = [r for r in modified['files'] if not r['path'].startswith('test/built-ins/AggregateError/prototype/')]
            (target / 'manifest.json').write_text(json.dumps(modified))
            with patch.dict(importer.PROFILES, {PROFILE: {'AggregateError': 19}}), self.assertRaises(ValueError):
                runner.load_corpus(target, PROFILE)

    def test_eighth_identity_and_negative_metadata_are_profile_local(self):
        original = set(runner.INTRINSIC_ERRORS)
        selected = runner.profile_intrinsic_errors(PROFILE)
        self.assertEqual(set(selected), original | {'AggregateError'})
        self.assertEqual(set(runner.profile_intrinsic_errors()), original)
        self.assertEqual(set(runner.profile_intrinsic_errors('unknown-profile')), original)
        self.assertEqual(runner.INTRINSIC_ERRORS, original)
        row = next(r for r in self.controls if r['name'] == 'aggregate-error-genuine-identity' and r['mode'] == 'sloppy')
        case = self.case(row)
        self.assertIsNone(runner.unsupported_reason(case, set(self.oracle['features']), PROFILE))
        self.assertEqual(runner.unsupported_reason(case), 'negative error constructor identity is not implemented: AggregateError')

    def test_selected_decoder_accepts_eighth_identity_and_rejects_bad_wire(self):
        row = next(r for r in self.controls if r['name'] == 'aggregate-error-genuine-identity')
        observation = dict(row['required_observation'], message='framing marker')
        wire = frame(observation)
        self.assertEqual(runner.decode_response(wire, profile=PROFILE), observation)
        with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
            runner.decode_response(wire)
        bad_identity = frame(dict(observation, error_identity='UnknownIntrinsic'))
        bad_phase = frame(dict(observation, phase='host'))
        bad_status = frame(dict(observation, status='host-failure'))
        bad_complete = frame(dict(observation, status='complete'))
        invalid_utf8 = b'ERJR2' + struct.pack('<I', 1) + b'\xff' + wire[5 + 4 + len('exception'):]
        malformed = [b'', b'ERJR1', wire[:-1], wire + b'x', b'ERJR2' + struct.pack('<I', 16385),
                     bad_identity, bad_phase, bad_status, bad_complete, invalid_utf8,
                     frame(observation, b'ERJR3'),
                     frame(dict(status='complete', phase='runtime', error_type='', error_identity='', message='', harness=''), b'ERJR3')]
        for bad in malformed:
            with self.subTest(wire=bad[:24]), self.assertRaises(ValueError):
                runner.decode_response(bad, profile=PROFILE)

    def test_run_case_uses_selected_identity_and_ordinary_transport(self):
        names = {'aggregate-error-genuine-identity', 'aggregate-error-genuine-identity-mismatch',
                 'aggregate-error-fake-own-name-constructor', 'aggregate-error-fake-prototype'}
        for row in [r for r in self.controls if r['name'] in names]:
            case = self.case(row)
            with self.subTest(id=row['id']), patch.object(runner, 'bounded_process', return_value=(0, frame(row['required_observation']), b'')) as process:
                result = runner.run_case(case, Path('/inert'), 3.0, set(self.oracle['features']), PROFILE)
            self.assertEqual(result['status'], row['expected'])
            self.assertEqual(result['observation'], dict(row['required_observation'], message=''))
            self.assertEqual(process.call_count, 1)
            self.assertTrue(process.call_args.args[1].startswith(b'ERJS1'))

    def test_exact_52_preflight_cases_and_required_observations(self):
        captured, outcomes = self.capture()
        self.assertEqual(len(captured), 52)
        keys = ('id', 'file', 'mode', 'metadata', 'source', 'source_sha256', 'case_sha256', 'harness')
        self.assertEqual([self.projection(c) for c in captured], [{k: r[k] for k in keys} for r in self.controls])
        for case in captured:
            self.assertNotIn('completion_policy', case)
        self.assertEqual([(r['name'], r['expected'], r['verified']) for r in outcomes],
                         [(r['name'], r['expected'], True) for r in self.controls])
        self.assertEqual([r['prerequisites'] for r in outcomes], [r['prerequisites'] for r in self.controls])
        self.assertTrue(runner.aggregate_error_preflight_healthy(outcomes, self.files()))
        direct_cases, direct_outcomes = self.capture(direct=True)
        self.assertEqual(direct_cases, captured)
        self.assertEqual(direct_outcomes, outcomes)

    def test_gate_rejects_roster_and_canonical_identity_corruption(self):
        _, healthy = self.capture()
        self.assertTrue(runner.aggregate_error_preflight_healthy(healthy, self.files()))
        mutations = [[], healthy[:32], healthy[:-1], healthy + healthy[:1]]
        duplicate = copy.deepcopy(healthy)
        duplicate[-1] = copy.deepcopy(duplicate[0])
        mutations.append(duplicate)
        edits = [('name', 'wrong'), ('expected', 'passed'), ('verified', False), ('verified', 1)]
        for key, value in edits:
            bad = copy.deepcopy(healthy)
            bad[-1][key] = value
            mutations.append(bad)
        for key, value in [('id', 'forged'), ('file', 'forged.js'), ('mode', 'raw'),
                           ('case_sha256', '0' * 64), ('source_sha256', '0' * 64),
                           ('expected_negative', {'phase': 'runtime', 'type': 'TypeError'})]:
            bad = copy.deepcopy(healthy)
            bad[-1]['result'][key] = value
            mutations.append(bad)
        for field, value in [('id', 'forged'), ('name', 'forged'), ('case_sha256', '0' * 64), ('mode', 'sloppy'),
                             ('source_sha256', '0' * 64), ('expected', 'failed')]:
            bad = copy.deepcopy(healthy)
            bad[-1]['prerequisites'][-1][field] = value
            mutations.append(bad)
        bad = copy.deepcopy(healthy)
        bad[-1]['prerequisites'].pop()
        mutations.append(bad)
        for bad in mutations:
            self.assertFalse(runner.aggregate_error_preflight_healthy(bad, self.files()))

    def test_gate_requires_raw_observations_not_only_status_or_verified(self):
        _, healthy = self.capture()
        self.assertTrue(runner.aggregate_error_preflight_healthy(healthy, self.files()))
        for row_index in (1, 33, 38, 40, 41):
            for key, value in [('status', 'resource'), ('phase', 'parse'), ('error_type', 'WrongName'),
                               ('error_identity', 'UnknownIntrinsic'), ('harness', 'assert.js')]:
                bad = copy.deepcopy(healthy)
                bad[row_index]['result']['observation'][key] = value
                self.assertFalse(runner.aggregate_error_preflight_healthy(bad, self.files()))
        for alteration in ('missing', 'empty', 'completion', 'status', 'oversized-message', 'untyped-message'):
            bad = copy.deepcopy(healthy)
            if alteration == 'missing':
                del bad[-1]['result']['observation']
            elif alteration == 'empty':
                bad[-1]['result']['observation'] = {}
            elif alteration == 'completion':
                bad[-1]['result']['observation']['completion'] = {'policy': 0}
            elif alteration == 'oversized-message':
                bad[-1]['result']['observation']['message'] = 'x' * 16385
            elif alteration == 'untyped-message':
                bad[-1]['result']['observation']['message'] = None
            else:
                bad[-1]['result']['status'] = 'passed'
            self.assertFalse(runner.aggregate_error_preflight_healthy(bad, self.files()))

    def test_failed_positives_cannot_authenticate_dependent_negatives(self):
        _, healthy = self.capture()
        self.assertTrue(runner.aggregate_error_preflight_healthy(healthy, self.files()))
        targets = [r for r in self.controls if (r['name'].startswith('aggregate-error-') and r['expected'] == 'passed')
                   or r['name'] in ('success', 'strict-success')]
        for target in targets:
            def corrupt(case, row, result):
                if row['id'] == target['id']:
                    result['observation']['error_type'] = 'WrongName'
                return result
            _, outcomes = self.capture(corrupt)
            indexed = {r['result']['id']: r for r in outcomes}
            self.assertFalse(indexed[target['id']]['verified'])
            dependents = [r for r in self.controls if target['id'] in r['prerequisite_ids']]
            self.assertTrue(dependents)
            for dependent in dependents:
                self.assertFalse(indexed[dependent['id']]['verified'], dependent['id'])
            self.assertFalse(runner.aggregate_error_preflight_healthy(outcomes, self.files()))

    def test_changed_helper_bytes_cannot_reuse_healthy_control_identities(self):
        _, healthy = self.capture()
        self.assertTrue(runner.aggregate_error_preflight_healthy(healthy, self.files()))
        for helper in self.oracle['helpers']:
            files = self.files()
            files['harness/' + helper] += b'\n'
            self.assertFalse(runner.aggregate_error_preflight_healthy(healthy, files), helper)
        files = self.files()
        del files['harness/assert.js']
        self.assertFalse(runner.aggregate_error_preflight_healthy(healthy, files))

    def test_unhealthy_gate_blocks_all_50_and_prevents_baseline_recording(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, healthy = self.capture()
        for bad in ([], healthy[:32], healthy[:-1], healthy + healthy[:1]):
            code, report, recorded, calls = self.report(bad, cases)
            self.assertEqual((code, recorded, calls), (1, None, []))
            self.assertEqual((report['mode_cases'], report['counts']), (50, {'blocked': 50}))
            self.assertEqual([r['id'] for r in report['cases']], [c['id'] for c in cases])
            self.assertFalse(report['harness_preflight_passed'])
            self.assertEqual(report['runner_policy'], self.oracle['policy'])
            self.assertEqual(report['runner_policy_sha256'], self.oracle['policy_sha256'])

    def test_synthetic_nonpass_reporting_recording_replay_and_infrastructure_refusal(self):
        _, _, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        _, healthy = self.capture()
        # Mocked Error outcomes for eligible sources: not an upstream forecast.
        code, report, recorded, calls = self.report(healthy, cases)
        self.assertEqual((code, len(calls)), (0, 50))
        self.assertEqual(report['counts'], {'failed': 48, 'unsupported': 2})
        self.assertEqual(len(recorded['cases']), 50)
        self.assertFalse(report['full_test262_conformance'])
        self.assertEqual(report['suite'], 'pinned-test262-aggregate-error-selection')
        self.assertEqual((report['runner_policy'], report['runner_policy_sha256']),
                         (self.oracle['policy'], self.oracle['policy_sha256']))
        code, _, absent, _ = self.report(healthy, cases, record=False)
        self.assertEqual((code, absent), (1, None))
        code, replay, _, _ = self.report(healthy, cases, baseline=recorded)
        self.assertEqual((code, replay['regressions'], replay['improvements']), (0, [], []))
        self.assertEqual(replay['cases'], report['cases'])
        for status in ('adapter-error', 'timeout', 'resource'):
            def outcome(case):
                return dict(id=case['id'], file=case['file'], mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=digest(case['source']), expected_negative=case['metadata']['negative'],
                            status=status, reason='explicit synthetic infrastructure refusal')
            code, report, recorded, calls = self.report(healthy, cases, outcome)
            self.assertEqual((code, recorded, len(calls), report['counts']), (1, None, 50, {status: 50}))

    def test_wrong_wire_missing_api_and_fatal_evidence_never_verify_controls(self):
        complete = dict(status='complete', phase='runtime', error_type='', error_identity='', message='', harness='')
        missing = dict(status='exception', phase='runtime', error_type='TypeError', error_identity='TypeError', message='missing API', harness='')
        candidates = [(frame(complete, b'ERJR3'), 'adapter-error'), (frame(missing), None)]
        for status, kind in [('resource', 'ResourceLimit'), ('unsupported', 'UnsupportedFeature')]:
            candidates.append((frame(dict(status=status, phase='runtime', error_type=kind, error_identity='', message='injected', harness='')), status))
        for wire, expected_status in candidates:
            with patch.object(runner, 'bounded_process', return_value=(0, wire, b'')) as process:
                outcomes = runner.harness_preflight(self.files(), Path('/inert'), 3.0, PROFILE)
            self.assertEqual((len(outcomes), process.call_count), (52, 52))
            self.assertTrue(all(not row['verified'] for row in outcomes))
            if expected_status:
                self.assertEqual({row['result']['status'] for row in outcomes}, {expected_status})
            self.assertTrue(all(call.args[1].startswith(b'ERJS1') for call in process.call_args_list))
            self.assertFalse(runner.aggregate_error_preflight_healthy(outcomes, self.files()))


if __name__ == '__main__':
    unittest.main()
