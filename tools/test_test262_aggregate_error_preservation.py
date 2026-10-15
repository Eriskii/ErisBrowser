"""Parent-compatible old-profile contracts; no network or adapter execution."""
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

import import_test262 as importer
import test262_conformance as runner


NEW_PROFILE = 'aggregate-error'
ORACLE = Path(__file__).with_name('aggregate-error-preservation-oracle.json')
ORACLE_SHA = 'fa09b3d3df060993c02926eafbf862ae88bb7399d39659ea911a41f8b767394e'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def response(identity, version=b'ERJR2'):
    # Valid ordinary exception; ERJR3 adds valid checked-sync abrupt evidence.
    frame = bytearray(version)
    for value in ('exception', 'runtime', 'RenamedError', identity, 'marker', ''):
        data = value.encode()
        frame.extend(struct.pack('<I', len(data)))
        frame.extend(data)
    if version == b'ERJR3':
        frame.extend(struct.pack('<BBBBHBBI', 0, 0, 0, 0, 256, 1, 1, 0))
    return bytes(frame)


class AggregateErrorPreservationTests(unittest.TestCase):
    def setUp(self):
        data = ORACLE.read_bytes()
        self.assertEqual(digest(data), ORACLE_SHA)
        self.oracle = json.loads(data)
        self.old = self.oracle['old_contract']
        for owner, name in [(importer, 'fetch'), (runner, 'bounded_process')]:
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_all_57_old_tables_features_and_actual_report_policies_remain_exact(self):
        self.assertEqual({k: v for k, v in importer.PROFILES.items() if k not in {NEW_PROFILE, 'promise-any', 'promise-with-resolvers'}}, self.old['profiles'])
        self.assertEqual({k: v for k, v in importer.PROFILE_ROOTS.items() if k not in {NEW_PROFILE, 'promise-any', 'promise-with-resolvers'}}, self.old['roots'])
        self.assertEqual(sorted(importer.TREE_PROFILES - {NEW_PROFILE, 'promise-any', 'promise-with-resolvers'}), self.old['tree_profiles'])
        self.assertEqual({k: sorted(v) for k, v in runner.PROFILE_FEATURES.items() if k not in {NEW_PROFILE, 'promise-any', 'promise-with-resolvers'}}, self.old['features'])
        self.assertEqual(len(self.old['profiles']), 57)
        self.assertEqual(sorted(runner.INTRINSIC_ERRORS), self.old['intrinsic_errors'])
        # Exercise main's real policy construction. An explicitly empty mocked
        # case inventory/preflight avoids engines, corpus work and result claims.
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            binary = base / 'inert-adapter'
            binary.write_bytes(b'never executed')
            output = base / 'report.json'
            manifest = dict(repository='fixture', revision='fixture', test_files=0)
            for profile, expected in self.old['policies_at_timeout3'].items():
                with self.subTest(profile=profile), contextlib.ExitStack() as stack:
                    stack.enter_context(patch.object(runner, 'load_corpus', return_value=(manifest, {}, [], [], '0' * 64)))
                    stack.enter_context(patch.object(runner, 'harness_preflight', return_value=[]))
                    for health in ('promise_preflight_healthy', 'promise_all_preflight_healthy',
                                   'promise_race_preflight_healthy', 'promise_all_settled_preflight_healthy'):
                        stack.enter_context(patch.object(runner, health, return_value=True))
                    call = stack.enter_context(patch.object(runner, 'run_case', side_effect=AssertionError('no cases')))
                    stack.enter_context(patch('sys.argv', ['test262_conformance.py', '--profile', profile,
                                                        '--binary', str(binary), '--output', str(output)]))
                    stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                    self.assertEqual(runner.main(), 0)
                    call.assert_not_called()
                report = json.loads(output.read_bytes())
                self.assertEqual((report['runner_policy'], report['runner_policy_sha256']),
                                 (expected['policy'], expected['sha256']), profile)
                self.assertEqual((report['mode_cases'], report['counts']), (0, {}))

    def test_default_seven_identities_and_existing_erjr3_rejection_are_preserved(self):
        self.assertEqual(runner.INTRINSIC_ERRORS, set(self.old['intrinsic_errors']))
        self.assertEqual(runner.PROMISE_HOST_PROFILES - {'promise-any'},
                         {'promise-methods-async', 'promise-all', 'promise-race', 'promise-all-settled'})
        ordinary = runner.decode_response(response('Error'))
        self.assertEqual((ordinary['error_type'], ordinary['error_identity']), ('RenamedError', 'Error'))
        host = runner.decode_response(response('Error', b'ERJR3'))
        self.assertEqual(host, runner.decode_promise_response(response('Error', b'ERJR3')))
        self.assertIsNone(runner.validate_promise_observation(host))
        for identity in ('AggregateError', 'UnknownIntrinsic'):
            with self.subTest(identity=identity):
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.decode_response(response(identity))
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.decode_response(response(identity, b'ERJR3'))
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.decode_promise_response(response(identity, b'ERJR3'))
                forged = copy.deepcopy(host)
                forged['error_identity'] = identity
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.validate_promise_observation(forged)
        case = dict(mode='sloppy', source=b'throw 1;', harness=[],
                    metadata=dict(flags=[], includes=[], features=[], locale=[],
                                  negative={'phase': 'runtime', 'type': 'AggregateError'}))
        reason = 'negative error constructor identity is not implemented: AggregateError'
        self.assertEqual(runner.unsupported_reason(case), reason)
        for profile in self.old['profiles']:
            with self.subTest(profile=profile):
                selected = copy.deepcopy(case)
                expected = reason
                if profile in runner.PROMISE_HOST_PROFILES:
                    selected['completion_policy'] = 0
                    expected = 'Promise profile supports only ordinary sloppy/strict script modes'
                self.assertEqual(runner.unsupported_reason(selected, runner.PROFILE_FEATURES[profile], profile), expected)

    def test_four_existing_promise_corpora_fingerprints_and_guard_literals_are_exact(self):
        held = {}
        for name, expected in self.oracle['existing_oracles'].items():
            raw = Path(__file__).with_name(name).read_bytes()
            self.assertEqual((len(raw), digest(raw)), (expected['bytes'], expected['sha256']), name)
            held[name] = json.loads(raw)
        all_oracle = held['promise-all-profile-oracle.json']
        sources = [('promise-methods-async', all_oracle['old_profile'], 'manifest', 458),
                   ('promise-all', all_oracle, 'expected_manifest', 196),
                   ('promise-race', held['promise-race-profile-oracle.json'], 'expected_manifest', 188),
                   ('promise-all-settled', held['promise-all-settled-profile-oracle.json'], 'expected_manifest', 208)]
        total = 0
        for profile, expected, manifest_key, count in sources:
            with self.subTest(profile=profile):
                corpus = importer.ROOT / 'tests/upstream' / ('test262-' + profile)
                manifest, _, cases, fixtures, _ = runner.load_corpus(corpus, profile)
                self.assertEqual(manifest, expected[manifest_key])
                self.assertEqual(fixtures, [])
                actual = [dict(id=c['id'], case_sha256=c['case_sha256'], policy=c['completion_policy'],
                               harness=[name for name, _ in c['harness']],
                               class_excluded='class' in c['metadata']['features']) for c in cases]
                self.assertEqual(actual, expected['case_identities'])
                self.assertEqual(len(cases), count)
                for case in cases:
                    self.assertNotIn('timeout', case['metadata'])
                    self.assertEqual(case['case_sha256'], runner.case_fingerprint(case))
                total += count
        self.assertEqual(total, 1050)
        for actual, expected in [
                (runner.promise_preflight_variants(), held['promise-methods-runner-oracle.json']['controls']),
                (runner.promise_all_preflight_variants(), all_oracle['controls']),
                (runner.promise_race_preflight_variants(), held['promise-race-profile-oracle.json']['controls']),
                (runner.promise_all_settled_preflight_variants(), held['promise-all-settled-profile-oracle.json']['controls'])]:
            self.assertEqual(actual, expected)

    def test_common_32_ordinary_control_bytes_metadata_and_fingerprints_are_exact(self):
        corpus = importer.ROOT / 'tests/upstream/test262'
        files = {}
        for name, expected in self.oracle['common_helpers'].items():
            data = (corpus / name).read_bytes()
            self.assertEqual((len(data), digest(data)), (expected['bytes'], expected['sha256']))
            files[name] = data
        by_id = {row['id']: row for row in self.oracle['common_controls']}
        captured = []
        def observe(case, binary, timeout):
            row = by_id[case['id']]
            captured.append(dict(id=case['id'], mode=case['mode'], metadata=case['metadata'],
                                 source=case['source'].decode(), case_sha256=case['case_sha256'],
                                 harness=[dict(name=name, sha256=digest(data)) for name, data in case['harness']]))
            return dict(id=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                        source_sha256=digest(case['source']), status=row['expected'],
                        observation=dict(status='exception' if row['expected'] == 'failed' else 'complete',
                                         phase='runtime', error_type='Test262Error' if row['expected'] == 'failed' else '',
                                         error_identity='', message='', harness=''))
        with patch.object(runner, 'run_case', side_effect=observe) as call:
            results = runner.harness_preflight(files, Path('/inert'), 3.0)
        self.assertEqual(call.call_count, 32)
        self.assertEqual(captured, [{k: v for k, v in row.items() if k != 'expected'}
                                    for row in self.oracle['common_controls']])
        self.assertEqual([(r['name'], r['expected'], r['verified']) for r in results],
                         [(r['id'].removeprefix('harness-preflight:'), r['expected'], True)
                          for r in self.oracle['common_controls']])


if __name__ == '__main__':
    unittest.main()
