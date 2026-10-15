"""Independent Any protocol contracts; adapter execution is always mocked."""
import copy
from pathlib import Path
import struct
import unittest
from unittest.mock import patch

import test262_conformance as runner


PROFILE = 'promise-any'
ORDINARY = 'aggregate-error'
OLD_HOST = ('promise-methods-async', 'promise-all', 'promise-race', 'promise-all-settled')
SEVEN = frozenset({'Error', 'TypeError', 'RangeError', 'SyntaxError',
                   'ReferenceError', 'EvalError', 'URIError'})
FEATURES = {'AggregateError', 'Promise.any', 'Reflect.construct', 'Symbol',
            'Symbol.iterator', 'Symbol.species', 'arrow-function',
            'computed-property-names'}


def evidence(policy=0, *, flags=0, calls=0, success=0, failure=0,
             quiescent=True, audited=True, unhandled=0):
    return dict(policy=policy, done_calls=calls, success_prints=success,
                failure_prints=failure, flags=flags, quiescent=quiescent,
                audited=audited, unhandled=unhandled)


def observation(*, identity='', diagnostic='', policy=0, abrupt=False):
    return dict(status='exception' if abrupt else 'complete', phase='runtime',
                error_type=diagnostic, error_identity=identity,
                message='marker' if abrupt else '', harness='',
                completion=evidence(policy, flags=256 if abrupt else 0,
                                    calls=policy if not abrupt else 0,
                                    success=policy if not abrupt else 0))


def frame(value, magic=b'ERJR3'):
    wire = bytearray(magic)
    for key in ('status', 'phase', 'error_type', 'error_identity', 'message', 'harness'):
        text = value[key].encode('utf-8')
        wire.extend(struct.pack('<I', len(text)))
        wire.extend(text)
    if magic == b'ERJR3':
        e = value['completion']
        wire.extend(struct.pack('<BBBBHBBI', e['policy'], e['done_calls'],
                                e['success_prints'], e['failure_prints'], e['flags'],
                                e['quiescent'], e['audited'], e['unhandled']))
    return bytes(wire)


def case(policy=0, *, ordinary=False):
    result = dict(id='identity-fixture', file='fixture.js', mode='sloppy',
                  case_sha256='0' * 64, source=b'1', harness=[],
                  metadata=dict(flags=['async'] if policy else [], includes=[],
                                features=[], locale=[], negative=None))
    if not ordinary:
        result['completion_policy'] = policy
    return result


class PromiseAnyProtocolTests(unittest.TestCase):
    def setUp(self):
        guard = patch.object(runner, 'bounded_process',
                             side_effect=AssertionError('real adapter execution forbidden'))
        guard.start()
        self.addCleanup(guard.stop)

    def test_any_capability_is_local_and_old_sets_are_not_mutated(self):
        self.assertEqual(runner.INTRINSIC_ERRORS, set(SEVEN))
        self.assertEqual(runner.profile_intrinsic_errors(PROFILE), SEVEN | {'AggregateError'})
        self.assertEqual(runner.profile_intrinsic_errors(ORDINARY), SEVEN | {'AggregateError'})
        for profile in (None, 'unknown-profile', *OLD_HOST):
            with self.subTest(profile=profile):
                self.assertEqual(runner.profile_intrinsic_errors(profile), SEVEN)
        self.assertIsInstance(runner.profile_intrinsic_errors(PROFILE), frozenset)
        self.assertEqual(runner.INTRINSIC_ERRORS, set(SEVEN))
        self.assertEqual(runner.PROFILE_FEATURES[PROFILE], FEATURES)
        self.assertEqual(runner.PROMISE_HOST_PROFILES - {PROFILE, 'promise-try'}, set(OLD_HOST))

    def test_any_genuine_renamed_and_fake_wire_identities_stay_distinct(self):
        for diagnostic, identity in [('AggregateError', 'AggregateError'),
                                     ('RenamedAggregateError', 'AggregateError'),
                                     ('AggregateError', '')]:
            value = observation(identity=identity, diagnostic=diagnostic, abrupt=True)
            wire = frame(value)
            with self.subTest(diagnostic=diagnostic, identity=identity):
                self.assertEqual(runner.decode_response(wire, profile=PROFILE), value)
                self.assertEqual(runner.decode_promise_response(wire, profile=PROFILE), value)
                self.assertIsNone(runner.validate_promise_observation(value, profile=PROFILE))
                self.assertEqual(runner.classify(case(), value, profile=PROFILE), 'failed')
                self.assertEqual(runner.classify_promise(case(), value, profile=PROFILE), 'failed')
        unknown = observation(identity='UnknownIntrinsic', diagnostic='AggregateError', abrupt=True)
        with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
            runner.decode_response(frame(unknown), profile=PROFILE)
        self.assertEqual(runner.classify(case(), unknown, profile=PROFILE), 'failed')

    def test_old_host_rejection_and_ordinary_aggregate_transport_remain_exact(self):
        genuine = observation(identity='AggregateError', diagnostic='RenamedAggregateError', abrupt=True)
        for profile in (None, 'unknown-profile', *OLD_HOST):
            with self.subTest(profile=profile):
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.decode_response(frame(genuine), profile=profile)
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.validate_promise_observation(genuine, profile=profile)
        for profile in (None, ORDINARY, *OLD_HOST):
            with self.subTest(direct_profile=profile):
                with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
                    runner.decode_promise_response(frame(genuine), profile=profile)
        ordinary = {k: v for k, v in genuine.items() if k != 'completion'}
        self.assertEqual(runner.decode_response(frame(genuine, b'ERJR2'), profile=ORDINARY), ordinary)
        with self.assertRaisesRegex(ValueError, 'ordinary ERJR2'):
            runner.decode_response(frame(genuine), profile=ORDINARY)
        with self.assertRaisesRegex(ValueError, 'unknown intrinsic error identity'):
            runner.decode_response(frame(genuine, b'ERJR2'))

    def test_any_requires_exact_erjr3_framing_and_bounded_fields(self):
        value = observation()
        valid = frame(value)
        invalid = [valid[:cut] for cut in range(len(valid))]
        invalid += [valid + b'\0', b'ERJR4' + valid[5:], frame(value, b'ERJR2'),
                    b'ERJR3' + struct.pack('<I', 16385) + b'x' * 16385,
                    b'ERJR3' + struct.pack('<I', 1) + b'\xff']
        for wire in invalid:
            with self.subTest(length=len(wire)):
                with self.assertRaises(ValueError):
                    runner.decode_response(wire, profile=PROFILE)
                with self.assertRaises(ValueError):
                    runner.decode_promise_response(wire, profile=PROFILE)
        for index in (7, 8):
            bad = bytearray(valid)
            # The last trailer has policy/counters/H flags, then two booleans.
            bad[-12 + index - 1] = 2
            with self.assertRaises(ValueError):
                runner.decode_response(bytes(bad), profile=PROFILE)

    def test_any_validator_and_classifier_refuse_malformed_completion_evidence(self):
        valid = observation()
        invalid = []
        for key, value in [('policy', True), ('policy', 2), ('done_calls', True),
                           ('done_calls', 3), ('success_prints', 3), ('failure_prints', 3),
                           ('flags', True), ('flags', 512), ('unhandled', -1),
                           ('unhandled', 0x100000000), ('quiescent', 1), ('audited', 1)]:
            changed = copy.deepcopy(valid)
            changed['completion'][key] = value
            invalid.append(changed)
        for change in ({'quiescent': False}, {'audited': False, 'unhandled': 1},
                       {'flags': 8}, {'success_prints': 1}, {'done_calls': 1},
                       {'flags': 4}, {'unhandled': 1}):
            changed = copy.deepcopy(valid)
            changed['completion'].update(change)
            invalid.append(changed)
        for key in ('completion', 'error_identity'):
            changed = copy.deepcopy(valid)
            del changed[key]
            invalid.append(changed)
        changed = copy.deepcopy(valid)
        changed['completion']['extra'] = 0
        invalid.append(changed)
        invalid += [dict(valid, profile=PROFILE), dict(valid, message='not empty'),
                    dict(valid, error_identity='AggregateError'),
                    dict(valid, message='x' * 16385), dict(valid, phase='parse')]
        abrupt = observation(identity='AggregateError', diagnostic='AggregateError', abrupt=True)
        for change in ({'error_type': ''}, {'harness': 'assert.js'},
                       {'completion': evidence()}, {'phase': 'parse'}):
            invalid.append(dict(abrupt, **change))
        for changed in invalid:
            with self.subTest(value=changed):
                with self.assertRaises((TypeError, ValueError)):
                    runner.validate_promise_observation(changed, profile=PROFILE)
                self.assertEqual(runner.classify(case(), changed, profile=PROFILE), 'failed')

    def test_any_classification_keeps_completion_policy_and_host_vetoes(self):
        for policy in (0, 1):
            value = observation(policy=policy)
            self.assertEqual(runner.classify(case(policy), value, profile=PROFILE), 'passed')
            self.assertEqual(runner.classify(case(1 - policy), value, profile=PROFILE), 'failed')
            old = {k: v for k, v in value.items() if k != 'completion'}
            self.assertEqual(runner.classify(case(policy), old, profile=PROFILE), 'failed')
            # Selected context alone must prevent ordinary-classifier fallback.
            self.assertEqual(runner.classify(case(ordinary=True), old, profile=PROFILE), 'failed')
            self.assertEqual(runner.classify(case(ordinary=True), value, profile=PROFILE), 'failed')
        negative = case(ordinary=True)
        negative['metadata']['negative'] = {'phase': 'runtime', 'type': 'AggregateError'}
        genuine = observation(identity='AggregateError', diagnostic='AggregateError', abrupt=True)
        ordinary_genuine = {k: v for k, v in genuine.items() if k != 'completion'}
        self.assertEqual(runner.classify(negative, ordinary_genuine), 'passed')
        self.assertEqual(runner.classify(negative, ordinary_genuine, profile=PROFILE), 'failed')
        self.assertEqual(runner.classify(negative, genuine, profile=PROFILE), 'failed')
        for reason, completion in [
                ('missing-completion', evidence(1)),
                ('completion-error', evidence(1, calls=1, failure=1, flags=1)),
                ('queued-exception', evidence(1, calls=1, success=1, flags=4)),
                ('unhandled-rejection', evidence(1, calls=1, success=1, unhandled=1)),
                ('fatal-observation', evidence(1, flags=8, quiescent=False, audited=False))]:
            value = dict(status='host-failure', phase='host', error_type='',
                         error_identity='', message=reason, harness='', completion=completion)
            self.assertEqual(runner.decode_response(frame(value), profile=PROFILE), value)
            self.assertEqual(runner.classify(case(1), value, profile=PROFILE), 'failed')
        resource = dict(status='resource', phase='runtime', error_type='ResourceLimit',
                        error_identity='', message='marker', harness='',
                        completion=evidence(flags=264, quiescent=False, audited=False))
        self.assertEqual(runner.classify(case(), resource, profile=PROFILE), 'resource')

    def test_selected_context_propagates_without_changing_legacy_delegate_arities(self):
        complete = observation()
        wire = frame(complete)
        requested = case()
        result = dict(status='passed', observation=complete)
        for profile in (None, *OLD_HOST, PROFILE):
            keyword = {'profile': profile} if profile == PROFILE else {}
            with self.subTest(profile=profile):
                with patch.object(runner, 'decode_promise_response', wraps=runner.decode_promise_response) as called:
                    runner.decode_response(wire, profile=profile)
                    called.assert_called_once_with(wire, **keyword)
                with patch.object(runner, 'classify_promise', wraps=runner.classify_promise) as called:
                    runner.classify(requested, complete, profile=profile)
                    called.assert_called_once_with(requested, complete, **keyword)
                for operation in (
                        lambda: runner.decode_promise_response(wire, profile=profile),
                        lambda: runner.classify_promise(requested, complete, profile=profile),
                        lambda: runner.promise_preflight_verified(requested, result, 'passed', profile=profile)):
                    with patch.object(runner, 'validate_promise_observation', wraps=runner.validate_promise_observation) as called:
                        operation()
                        called.assert_called_once_with(complete, **keyword)
        genuine = observation(identity='AggregateError', diagnostic='AggregateError', abrupt=True)
        self.assertFalse(runner.promise_preflight_verified(case(), dict(status='failed', observation=genuine),
                                                         'synchronous-exception', profile=PROFILE))

    def test_run_case_routes_only_selected_context_and_never_executes_an_adapter(self):
        binary = Path('/unused/identity-adapter')
        for profile in (None, *OLD_HOST, ORDINARY, PROFILE):
            ordinary = profile == ORDINARY
            requested = case(ordinary=ordinary)
            value = observation()
            wire = frame(value, b'ERJR2' if ordinary else b'ERJR3')
            decoded = {k: v for k, v in value.items() if k != 'completion'} if ordinary else value
            with self.subTest(profile=profile), \
                    patch.object(runner, 'bounded_process', return_value=(0, wire, b'')) as process, \
                    patch.object(runner, 'decode_response', wraps=runner.decode_response) as decode, \
                    patch.object(runner, 'classify', wraps=runner.classify) as classify:
                result = runner.run_case(requested, binary, 3, FEATURES, profile)
                self.assertEqual(result['status'], 'passed')
                decode.assert_called_once_with(wire, **({'profile': profile} if profile in (ORDINARY, PROFILE) else {}))
                classify.assert_called_once_with(requested, decoded, **({'profile': profile} if profile == PROFILE else {}))
                self.assertEqual(process.call_args.args[0], [str(binary)])
                self.assertEqual(process.call_args.args[1][:5], b'ERJS1' if ordinary else b'ERJS2')
        genuine = observation(identity='AggregateError', diagnostic='AggregateError', abrupt=True)
        for profile, wire, expected in [(PROFILE, frame(genuine), 'failed'),
                                        (OLD_HOST[0], frame(genuine), 'adapter-error'),
                                        (PROFILE, frame(genuine, b'ERJR2'), 'adapter-error')]:
            with patch.object(runner, 'bounded_process', return_value=(0, wire, b'')):
                result = runner.run_case(case(), binary, 3, FEATURES, profile)
                self.assertEqual(result['status'], expected)

    def test_any_selection_keeps_negative_raw_module_and_feature_refusals(self):
        invalid = []
        for negative in ({'phase': 'runtime', 'type': 'AggregateError'},
                         {'phase': 'parse', 'type': 'SyntaxError'}):
            value = case()
            value['metadata']['negative'] = negative
            invalid.append((value, 'ordinary sloppy/strict'))
        for mode in ('raw', 'module'):
            value = case()
            value['mode'] = mode
            invalid.append((value, 'ordinary sloppy/strict'))
        for feature in ('class', 'destructuring-binding'):
            value = case()
            value['metadata']['features'] = [feature]
            invalid.append((value, feature))
        for value, reason in invalid:
            with self.subTest(reason=reason):
                result = runner.run_case(value, Path('/unused'), 3, FEATURES, PROFILE)
                self.assertEqual(result['status'], 'unsupported')
                self.assertIn(reason, result['reason'])


if __name__ == '__main__':
    unittest.main()
