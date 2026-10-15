"""Independent ordinary transport boundary; real adapter calls are forbidden."""
from pathlib import Path
import struct
import unittest
from unittest.mock import patch

import test262_conformance as runner

PROFILE = 'promise-with-resolvers'
FEATURES = {'promise-with-resolvers'}
HOST = ('promise-methods-async', 'promise-all', 'promise-race',
        'promise-all-settled', 'promise-any')
SEVEN = frozenset({'Error', 'EvalError', 'RangeError', 'ReferenceError',
                   'SyntaxError', 'TypeError', 'URIError'})
FIELDS = ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness')


def observation(**changes):
    value = dict(status='complete', phase='runtime', error_type='',
                 error_identity='', message='', harness='')
    value.update(changes)
    return value


def frame(value, magic=b'ERJR2'):
    wire = bytearray(magic)
    for key in FIELDS:
        text = value[key].encode('utf-8')
        wire.extend(struct.pack('<I', len(text)))
        wire.extend(text)
    if magic == b'ERJR3':
        wire.extend(struct.pack('<BBBBHBBI', 0, 0, 0, 0, 0, 1, 1, 0))
    return bytes(wire)


def case(mode='sloppy'):
    return dict(id='ordinary-fixture:' + mode, file='fixture.js', mode=mode,
                case_sha256='0' * 64, source=b'1', harness=[],
                metadata=dict(flags=[], includes=[], features=[], locale=[], negative=None))


class WithResolversProtocolTests(unittest.TestCase):
    def setUp(self):
        guard = patch.object(runner, 'bounded_process',
                             side_effect=AssertionError('real adapter execution forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def test_feature_identity_and_host_sets_remain_separate(self):
        self.assertEqual(runner.PROMISE_WITH_RESOLVERS_PROFILE, PROFILE)
        self.assertEqual(runner.PROMISE_WITH_RESOLVERS_FEATURES, FEATURES)
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES)
        self.assertEqual(runner.PROMISE_HOST_PROFILES - {'promise-try'}, set(HOST))
        self.assertEqual(runner.INTRINSIC_ERRORS, set(SEVEN))
        self.assertEqual(runner.profile_intrinsic_errors(PROFILE), SEVEN)
        for old in ('aggregate-error', 'promise-any'):
            self.assertEqual(runner.profile_intrinsic_errors(old), SEVEN | {'AggregateError'})
        for old in (None, *HOST[:-1]):
            self.assertEqual(runner.profile_intrinsic_errors(old), SEVEN)

    def test_erjr2_preserves_genuine_renamed_and_forged_identity_distinctions(self):
        requested = case()
        requested['metadata']['negative'] = {'phase': 'runtime', 'type': 'TypeError'}
        for diagnostic, identity, expected in [('TypeError', 'TypeError', 'passed'),
                                                ('Renamed', 'TypeError', 'passed'),
                                                ('TypeError', '', 'failed')]:
            value = observation(status='exception', error_type=diagnostic,
                                error_identity=identity, message='marker')
            with self.subTest(diagnostic=diagnostic, identity=identity):
                self.assertEqual(runner.decode_response(frame(value), profile=PROFILE), value)
                self.assertEqual(runner.classify(requested, value), expected)
                self.assertEqual(runner.classify(case(), value), 'failed')
        for identity in sorted(SEVEN):
            value = observation(status='exception', error_type='Renamed', error_identity=identity)
            self.assertEqual(runner.decode_response(frame(value), profile=PROFILE), value)
        value = observation(status='exception', error_type='AggregateError', error_identity='AggregateError')
        with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
            runner.decode_response(frame(value), profile=PROFILE)
        self.assertEqual(runner.decode_response(frame(value), profile='aggregate-error'), value)

    def test_selected_erjr2_framing_refuses_all_truncations_and_invalid_fields(self):
        valid = frame(observation())
        invalid = [valid[:cut] for cut in range(len(valid))]
        invalid += [valid + b'\0', b'ERJR4' + valid[5:],
                    b'ERJR2' + struct.pack('<I', 16385) + b'x' * 16385,
                    b'ERJR2' + struct.pack('<I', 1) + b'\xff']
        for change in ({'status': 'host-failure'}, {'status': 'unknown'},
                       {'phase': 'host'}, {'error_type': 'TypeError'},
                       {'error_identity': 'TypeError'}, {'harness': 'assert.js'},
                       {'status': 'exception'}, {'status': 'harness-error'}):
            invalid.append(frame(observation(**change)))
        for wire in invalid:
            with self.subTest(wire=wire[:15], size=len(wire)):
                with self.assertRaises(ValueError):
                    runner.decode_response(wire, profile=PROFILE)
        self.assertEqual(runner.decode_response(valid, profile=PROFILE), observation())

    def test_selected_erjr3_refusal_happens_before_host_decoder_delegation(self):
        wire = frame(observation(), b'ERJR3')
        for value in (wire, wire[:-1], wire + b'\0'):
            with self.subTest(size=len(value)), patch.object(
                    runner, 'decode_promise_response', side_effect=AssertionError('host decoder forbidden')) as decoder:
                with self.assertRaisesRegex(ValueError, 'ordinary ERJR2'):
                    runner.decode_response(value, profile=PROFILE)
                decoder.assert_not_called()
        for old in (None, *HOST):
            with self.subTest(old=old), patch.object(
                    runner, 'decode_promise_response', wraps=runner.decode_promise_response) as decoder:
                runner.decode_response(wire, profile=old)
                decoder.assert_called_once_with(wire, **({'profile': old} if old == 'promise-any' else {}))
        with self.assertRaisesRegex(ValueError, 'ordinary ERJR2'):
            runner.decode_response(wire, profile='aggregate-error')

    def test_run_case_context_and_request_version_preserve_all_legacy_arities(self):
        for profile in (None, 'aggregate-error', *HOST, PROFILE):
            requested = case('strict')
            host = profile in HOST
            if host:
                requested['completion_policy'] = 0
            wire = frame(observation(), b'ERJR3' if host else b'ERJR2')
            decoded = runner.decode_response(wire, profile=profile)
            with self.subTest(profile=profile), \
                    patch.object(runner, 'bounded_process', return_value=(0, wire, b'')) as process, \
                    patch.object(runner, 'decode_response', wraps=runner.decode_response) as decode, \
                    patch.object(runner, 'classify', wraps=runner.classify) as classify:
                result = runner.run_case(requested, Path('/never-executed'), 3, FEATURES, profile)
                self.assertEqual(result['status'], 'passed')
                decode.assert_called_once_with(wire, **({'profile': profile} if profile in
                                                      {'aggregate-error', 'promise-any', PROFILE} else {}))
                classify.assert_called_once_with(requested, decoded, **({'profile': profile}
                                                                         if profile == 'promise-any' else {}))
                self.assertEqual(process.call_args.args[0], ['/never-executed'])
                self.assertEqual(process.call_args.args[1][:6], (b'ERJS2' if host else b'ERJS1') + b'\x01')
                self.assertEqual(result['source_sha256'], runner.digest(b'1'))
        with patch.object(runner, 'bounded_process', return_value=(0, frame(observation(), b'ERJR3'), b'')):
            result = runner.run_case(case(), Path('/never-executed'), 3, FEATURES, PROFILE)
            self.assertEqual(result['status'], 'adapter-error')
            self.assertIn('ordinary ERJR2', result['reason'])

    def test_forged_completion_policy_is_refused_before_metadata_encoding_or_dispatch(self):
        for policy in (0, 1, None, False, True, '0', {}, []):
            for features in ([], ['class']):
                requested = case()
                requested['completion_policy'] = policy
                requested['metadata']['features'] = features
                with self.subTest(policy=policy, features=features), \
                        patch.object(runner, 'unsupported_reason', side_effect=AssertionError('metadata must follow guard')) as unsupported, \
                        patch.object(runner, 'encode_request', side_effect=AssertionError('encoding forbidden')) as encode, \
                        patch.object(runner, 'bounded_process', side_effect=AssertionError('dispatch forbidden')) as process:
                    result = runner.run_case(requested, Path('/never-executed'), 3, FEATURES, PROFILE)
                    expected = {key: requested[key] for key in ('id', 'file', 'mode', 'case_sha256')}
                    expected.update(source_sha256=runner.digest(b'1'), expected_negative=None,
                                    status='adapter-error', reason='Promise.withResolvers requires an ordinary request')
                    self.assertEqual(result, expected)
                    unsupported.assert_not_called()
                    encode.assert_not_called()
                    process.assert_not_called()

    def test_existing_feature_async_and_module_refusals_happen_without_dispatch(self):
        requested_cases = []
        for features in (['class'], ['async-functions'], ['unknown-feature']):
            requested = case()
            requested['metadata']['features'] = features
            requested_cases.append(requested)
        requested = case()
        requested['metadata']['flags'] = ['async']
        requested_cases.append(requested)
        requested_cases.append(case('module'))
        for requested in requested_cases:
            with self.subTest(metadata=requested['metadata'], mode=requested['mode']):
                result = runner.run_case(requested, Path('/never-executed'), 3, FEATURES, PROFILE)
                self.assertEqual(result['status'], 'unsupported')
                self.assertTrue(result['reason'])

    def test_ordinary_error_and_failure_categories_are_not_promoted_by_the_profile(self):
        for value, expected in [(observation(), 'passed'),
                                (observation(status='exception', error_type='Error'), 'failed'),
                                (observation(status='resource', error_type='ResourceLimit'), 'resource'),
                                (observation(status='unsupported'), 'unsupported'),
                                (observation(status='harness-error', phase='harness', harness='assert.js'), 'harness-error')]:
            with self.subTest(expected=expected), patch.object(
                    runner, 'bounded_process', return_value=(0, frame(value), b'')):
                result = runner.run_case(case(), Path('/never-executed'), 3, FEATURES, PROFILE)
                self.assertEqual(result['status'], expected)
                self.assertEqual(result['observation'], value)


if __name__ == '__main__':
    unittest.main()
