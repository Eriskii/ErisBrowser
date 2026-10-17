"""Frozen Try ERJR3 protocol boundary; all adapter calls are mocked."""
import ast
import copy
import hashlib
import inspect
from pathlib import Path
import struct
import unittest
from unittest.mock import patch

import test262_conformance as runner

PROFILE = 'promise-try'
FEATURES = {'Reflect.construct', 'promise-try'}
OLD_HOST = ('promise-methods-async', 'promise-all', 'promise-race',
            'promise-all-settled', 'promise-any')
ORDINARY = ('aggregate-error', 'promise-with-resolvers')
SEVEN = frozenset({'Error', 'TypeError', 'RangeError', 'SyntaxError',
                   'ReferenceError', 'EvalError', 'URIError'})
FIELDS = ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness')
# Replaced from published15db source by the independent authoring script.
PRESERVED_FUNCTIONS = {'profile_intrinsic_errors': '81eb6f16cd521465d034cedf94908fef12cde66217f221aaae42229ecfa9dc85', 'unsupported_reason': 'b016abc5568cf339cade09d4ec3fea84952875310318ddf2f30f73aa26c6da4f', 'encode_request': '44a11389bc161abdbe5c099873cc0f1b98d6639e0fab9bec4292750fe659cbd5', 'classify': 'cf84ebb1ad3f0228bd8f847f147e573792b02d03d3550e17b850a76845257cd1', 'promise_completion_passes': '9beabc2c6352b19c8ce25cfb7259fcf04d42d98a7bd350fc0a29e483a3998ace', 'validate_promise_observation': 'c2df5a178c4ef539e2ca9ab0e4a4e0b37cf07cb5db0596725aaffe857b029bee', 'decode_promise_response': 'ab99e6645199b17ca0683e6ad97575bd7561a448dfa04c212a7d5f33b15b1168', 'classify_promise': 'c9558f3b137a2b1e96a11ae4e7e67e813a288d8290455efd8a04d15ce16ce68d', 'promise_host_reason': '1b549ac065bd1b163851bd899e7e3eb3b67cab01b67e8465175d5b9469baa648', 'promise_preflight_verified': '9971d67120e463dc9bad2597f45422190c52a36510a8210c6b54251b9d6189f1'}


def evidence(policy=0, *, flags=0, calls=0, success=0, failure=0,
             quiescent=True, audited=True, unhandled=0):
    return dict(policy=policy, done_calls=calls, success_prints=success,
                failure_prints=failure, flags=flags, quiescent=quiescent,
                audited=audited, unhandled=unhandled)


def observation(policy=0, *, identity='', diagnostic='', abrupt=False):
    return dict(status='exception' if abrupt else 'complete', phase='runtime',
                error_type=diagnostic, error_identity=identity,
                message='marker' if abrupt else '', harness='',
                completion=evidence(policy, flags=256 if abrupt else 0,
                                    calls=policy if not abrupt else 0,
                                    success=policy if not abrupt else 0))


def frame(value, magic=b'ERJR3'):
    wire = bytearray(magic)
    for key in FIELDS:
        text = value[key].encode('utf-8')
        wire.extend(struct.pack('<I', len(text)))
        wire.extend(text)
    if magic == b'ERJR3':
        e = value['completion']
        wire.extend(struct.pack('<BBBBHBBI', e['policy'], e['done_calls'],
                                e['success_prints'], e['failure_prints'], e['flags'],
                                e['quiescent'], e['audited'], e['unhandled']))
    return bytes(wire)


def case(policy=0, mode='sloppy', *, ordinary=False):
    value = dict(id='try-protocol-fixture:' + mode, file='fixture.js', mode=mode,
                 case_sha256='0' * 64, source=b'1', harness=[],
                 metadata=dict(flags=['async'] if policy else [], includes=[],
                               features=[], locale=[], negative=None))
    if not ordinary:
        value['completion_policy'] = policy
    return value


class PromiseTryProtocolTests(unittest.TestCase):
    def setUp(self):
        guard = patch.object(runner, 'bounded_process',
                             side_effect=AssertionError('real adapter execution forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def test_selected_features_host_family_and_seven_identity_boundary(self):
        self.assertEqual(runner.PROMISE_TRY_PROFILE, PROFILE)
        self.assertEqual(runner.PROMISE_TRY_FEATURES, FEATURES | {'class'})
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES | {'class'})
        self.assertEqual(runner.PROMISE_HOST_PROFILES, set(OLD_HOST) | {PROFILE})
        self.assertEqual(runner.INTRINSIC_ERRORS, set(SEVEN))
        for selected in (None, 'unknown-profile', PROFILE, *OLD_HOST[:-1], ORDINARY[1]):
            self.assertEqual(runner.profile_intrinsic_errors(selected), SEVEN)
        for selected in (ORDINARY[0], OLD_HOST[-1]):
            self.assertEqual(runner.profile_intrinsic_errors(selected), SEVEN | {'AggregateError'})
        for identity in sorted(SEVEN):
            for diagnostic in (identity, 'RenamedIntrinsic'):
                value = observation(identity=identity, diagnostic=diagnostic, abrupt=True)
                self.assertEqual(runner.decode_response(frame(value), profile=PROFILE), value)
                self.assertEqual(runner.classify_promise(case(), value), 'failed')
        fake = observation(identity='', diagnostic='AggregateError', abrupt=True)
        self.assertEqual(runner.decode_response(frame(fake), profile=PROFILE), fake)
        for identity in ('AggregateError', 'UnknownIntrinsic'):
            value = observation(identity=identity, diagnostic='RenamedIntrinsic', abrupt=True)
            with self.subTest(identity=identity), self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                runner.decode_response(frame(value), profile=PROFILE)
        true_aggregate = observation(identity='AggregateError', diagnostic='Renamed', abrupt=True)
        self.assertEqual(runner.decode_response(frame(true_aggregate), profile='promise-any'), true_aggregate)
        with self.assertRaises(ValueError):
            runner.decode_promise_response(frame(true_aggregate))

    def test_selected_erjr3_rejects_downgrades_all_truncations_and_trailing_bytes(self):
        value = observation()
        valid = frame(value)
        invalid = [valid[:cut] for cut in range(len(valid))]
        invalid += [valid + b'\0', b'ERJR4' + valid[5:], frame(value, b'ERJR2'),
                    b'ERJR3' + struct.pack('<I', 16385) + b'x' * 16385,
                    b'ERJR3' + struct.pack('<I', 1) + b'\xff']
        for offset in (6, 7):
            changed = bytearray(valid)
            changed[-12 + offset] = 2
            invalid.append(bytes(changed))
        for wire in invalid:
            with self.subTest(magic=wire[:5], length=len(wire)), self.assertRaises(ValueError):
                runner.decode_response(wire, profile=PROFILE)
        with patch.object(runner, 'decode_promise_response', side_effect=AssertionError('delegate must not run')) as called:
            with self.assertRaisesRegex(ValueError, 'ERJR3'):
                runner.decode_response(frame(value, b'ERJR2'), profile=PROFILE)
            called.assert_not_called()
        self.assertEqual(runner.decode_response(valid, profile=PROFILE), value)
        ordinary = {k: v for k, v in value.items() if k != 'completion'}
        for profile in (None, *ORDINARY):
            self.assertEqual(runner.decode_response(frame(value, b'ERJR2'), profile=profile), ordinary)

    def test_request_modes_and_completion_policy_match_exact_erjs2_bytes(self):
        for mode, mode_byte in (('sloppy', 0), ('strict', 1)):
            for policy in (0, 1):
                requested = case(policy, mode)
                requested['harness'] = [('first.js', b'alpha'), ('second.js', b'beta')]
                expected = bytearray(b'ERJS2' + bytes([mode_byte, 0, policy]) + struct.pack('<I', 2))
                for payload in (b'first.js', b'alpha', b'second.js', b'beta', b'1'):
                    expected += struct.pack('<I', len(payload)) + payload
                self.assertEqual(runner.encode_request(requested), bytes(expected))
        for policy in (None, False, True, '0', {}, [], 2, -1):
            requested = case()
            requested['completion_policy'] = policy
            with self.subTest(policy=policy), self.assertRaises(ValueError):
                runner.encode_request(requested)
        self.assertEqual(runner.encode_request(case(ordinary=True))[:5], b'ERJS1')

    def test_selected_policy_metadata_refusals_precede_encoding_and_dispatch(self):
        cases = []
        for policy in (None, False, True, '0', {}, [], 2, -1, 1):
            requested = case()
            requested['completion_policy'] = policy
            cases.append((requested, 'completion policy'))
        cases.append((case(ordinary=True), 'completion policy'))
        requested = case(1)
        requested['completion_policy'] = 0
        cases.append((requested, 'completion policy'))
        for mode in ('raw', 'module'):
            cases.append((case(mode=mode), 'ordinary sloppy/strict'))
        for negative in ({'phase': 'runtime', 'type': 'TypeError'},
                         {'phase': 'parse', 'type': 'SyntaxError'}):
            requested = case()
            requested['metadata']['negative'] = negative
            cases.append((requested, 'ordinary sloppy/strict'))
        for feature in ('class', 'async-functions', 'unknown-feature'):
            requested = case()
            requested['metadata']['features'] = [feature]
            cases.append((requested, feature))
        requested = case()
        requested['source'] = b'$262.createRealm();'
        cases.append((requested, 'realm host hooks'))
        for requested, reason in cases:
            with self.subTest(reason=reason, requested=requested), patch.object(
                    runner, 'encode_request', side_effect=AssertionError('encoding forbidden')) as encode:
                result = runner.run_case(requested, Path('/never'), 3, FEATURES, PROFILE)
                self.assertEqual(result['status'], 'unsupported')
                self.assertIn(reason, result['reason'])
                encode.assert_not_called()
        self.assertIsNone(runner.unsupported_reason(case(1), FEATURES, PROFILE))
        self.assertIsNone(runner.unsupported_reason(case(0), FEATURES, PROFILE))

    def test_selected_routing_uses_direct_shared_classifier_and_preserves_old_arities(self):
        for profile in (None, *OLD_HOST, *ORDINARY, PROFILE):
            host = profile in (*OLD_HOST, PROFILE)
            requested = case(ordinary=not host)
            value = observation()
            wire = frame(value, b'ERJR3' if host else b'ERJR2')
            decoded = value if host else {k: v for k, v in value.items() if k != 'completion'}
            with self.subTest(profile=profile), patch.object(
                    runner, 'bounded_process', return_value=(0, wire, b'')) as process, patch.object(
                    runner, 'decode_response', wraps=runner.decode_response) as decode, patch.object(
                    runner, 'classify', wraps=runner.classify) as generic, patch.object(
                    runner, 'classify_promise', wraps=runner.classify_promise) as promise:
                result = runner.run_case(requested, Path('/never'), 3, FEATURES, profile)
                self.assertEqual(result['status'], 'passed')
                decode.assert_called_once_with(wire, **({'profile': profile} if profile in
                    (*ORDINARY, 'promise-any', PROFILE) else {}))
                if profile == PROFILE:
                    generic.assert_not_called()
                    promise.assert_called_once_with(requested, decoded)
                else:
                    generic.assert_called_once_with(requested, decoded, **({'profile': profile}
                        if profile == 'promise-any' else {}))
                    if host:
                        promise.assert_called_once_with(requested, decoded, **({'profile': profile}
                            if profile == 'promise-any' else {}))
                    else:
                        promise.assert_not_called()
                self.assertEqual(process.call_args.args[0], ['/never'])
                self.assertEqual(process.call_args.args[1][:5], b'ERJS2' if host else b'ERJS1')
        wire = frame(observation())
        for profile in (None, *OLD_HOST, PROFILE):
            with self.subTest(delegate=profile), patch.object(
                    runner, 'decode_promise_response', wraps=runner.decode_promise_response) as called:
                runner.decode_response(wire, profile=profile)
                called.assert_called_once_with(wire, **({'profile': profile} if profile == 'promise-any' else {}))

    def test_selected_raw_dictionary_downgrade_and_forged_completion_never_pass(self):
        valid = observation()
        malformed = []
        for key in ('completion', 'error_identity'):
            changed = copy.deepcopy(valid)
            del changed[key]
            malformed.append(changed)
        for value in (None, [], 'complete', {'policy': 0}):
            malformed.append(dict(valid, completion=value))
        malformed.extend([dict(valid, extra='forged'), dict(valid, harness='assert.js'),
                          dict(valid, message='forged'), dict(valid, phase='parse')])
        for key, value in [('policy', True), ('policy', 2), ('done_calls', True),
                           ('done_calls', 3), ('flags', True), ('flags', 512),
                           ('quiescent', 1), ('audited', 1), ('unhandled', -1),
                           ('unhandled', 0x100000000), ('extra', 0)]:
            changed = copy.deepcopy(valid)
            changed['completion'][key] = value
            malformed.append(changed)
        malformed.append(observation(1))
        for changed in malformed:
            with self.subTest(changed=changed), patch.object(
                    runner, 'bounded_process', return_value=(0, frame(valid), b'')), patch.object(
                    runner, 'decode_response', return_value=changed), patch.object(
                    runner, 'classify', side_effect=AssertionError('ordinary fallback forbidden')):
                self.assertEqual(runner.run_case(case(), Path('/never'), 3, FEATURES, PROFILE)['status'], 'failed')
        ordinary = {k: v for k, v in valid.items() if k != 'completion'}
        # Preserve the generic ordinary API; the selected run_case is what isolates Try.
        self.assertEqual(runner.classify(case(ordinary=True), ordinary), 'passed')
        self.assertEqual(runner.classify_promise(case(ordinary=True), ordinary), 'failed')

    def test_host_vetoes_resource_unsupported_and_harness_failures_remain_distinct(self):
        values = []
        for reason, completion in [
                ('missing-completion', evidence(1)),
                ('completion-error', evidence(1, calls=1, failure=1, flags=1)),
                ('queued-exception', evidence(1, calls=1, success=1, flags=4)),
                ('unhandled-rejection', evidence(1, calls=1, success=1, unhandled=1)),
                ('fatal-observation', evidence(1, flags=8, quiescent=False, audited=False))]:
            values.append((dict(status='host-failure', phase='host', error_type='', error_identity='',
                message=reason, harness='', completion=completion), 'failed'))
        for status, diagnostic in [('resource', 'ResourceLimit'), ('unsupported', 'Unsupported')]:
            values.append((dict(status=status, phase='runtime', error_type=diagnostic, error_identity='',
                message='marker', harness='', completion=evidence(1, flags=264, quiescent=False, audited=False)), status))
        values.append((dict(status='harness-error', phase='harness', error_type='Error', error_identity='Error',
            message='marker', harness='assert.js', completion=evidence(1)), 'harness-error'))
        for value, expected in values:
            with self.subTest(expected=expected, value=value), patch.object(
                    runner, 'bounded_process', return_value=(0, frame(value), b'')):
                result = runner.run_case(case(1), Path('/never'), 3, FEATURES, PROFILE)
                self.assertEqual(result['status'], expected)
                self.assertEqual(result['observation'], value)
        for policy in (0, 1):
            value = observation(policy)
            with patch.object(runner, 'bounded_process', return_value=(0, frame(value), b'')):
                self.assertEqual(runner.run_case(case(policy), Path('/never'), 3, FEATURES, PROFILE)['status'], 'passed')

    def test_adapter_exit_timeout_and_invalid_wire_are_not_semantic_passes(self):
        import subprocess
        requested = case()
        for code, wire, expected in [(0, frame(observation(), b'ERJR2'), 'adapter-error'),
                                     (7, frame(observation()), 'adapter-error'),
                                     (0, b'ERJR3broken', 'adapter-error')]:
            with self.subTest(code=code, wire=wire), patch.object(
                    runner, 'bounded_process', return_value=(code, wire, b'diagnostic')):
                self.assertEqual(runner.run_case(requested, Path('/never'), 3, FEATURES, PROFILE)['status'], expected)
        with patch.object(runner, 'bounded_process', side_effect=subprocess.TimeoutExpired('/never', 3)):
            self.assertEqual(runner.run_case(requested, Path('/never'), 3, FEATURES, PROFILE)['status'], 'timeout')
        with patch.object(runner, 'bounded_process', side_effect=OSError('fixture transport failure')):
            self.assertEqual(runner.run_case(requested, Path('/never'), 3, FEATURES, PROFILE)['status'], 'adapter-error')

    def test_shared_function_bodies_and_default_keyword_arities_remain_exact(self):
        for name, expected in PRESERVED_FUNCTIONS.items():
            source = inspect.getsource(getattr(runner, name))
            self.assertEqual(hashlib.sha256(source.encode()).hexdigest(), expected, name)
            ast.parse(source)
        for name in ('decode_response', 'decode_promise_response', 'validate_promise_observation',
                     'classify', 'classify_promise', 'profile_intrinsic_errors'):
            signature = inspect.signature(getattr(runner, name))
            self.assertIsNone(signature.parameters['profile'].default)
            if name != 'profile_intrinsic_errors':
                self.assertIs(signature.parameters['profile'].kind, inspect.Parameter.KEYWORD_ONLY)
        value = observation()
        for invoke in (lambda: runner.decode_promise_response(frame(value)),
                       lambda: runner.classify_promise(case(), value),
                       lambda: runner.promise_preflight_verified(case(), dict(status='passed', observation=value), 'passed')):
            with patch.object(runner, 'validate_promise_observation', wraps=runner.validate_promise_observation) as called:
                invoke()
                called.assert_called_once_with(value)


if __name__ == '__main__':
    unittest.main()
