"""Independent transport/policy expectations; no upstream or JS execution.

This module imports only existing runner APIs, so unchanged parent can baseline
it. New valid ERJS2/ERJR3/admission assertions intentionally fail there; old and
malformed-frame controls may already pass. Private Runtime quotas are separate.
"""
import struct
import unittest

import test262_conformance as runner


FEATURES = {'Promise', 'Promise.prototype.finally', 'Symbol.species',
            'Reflect.construct', 'arrow-function', 'Reflect',
            'Symbol.toStringTag', 'Symbol'}
PROFILE = 'promise-methods-async'


def case(*, asynchronous=False, policy=None, source=b'1', mode='sloppy'):
    value = {'id': 'fixture', 'file': 'fixture.js', 'mode': mode,
             'case_sha256': '0' * 64, 'source': source, 'harness': [],
             'metadata': {'flags': ['async'] if asynchronous else [],
                          'features': [], 'includes': [], 'negative': None,
                          'locale': [], 'timeout': None}}
    if policy is not None:
        value['completion_policy'] = policy
    return value


def evidence(policy=1, calls=1, success=1, failure=0, flags=0,
             quiescent=1, audited=1, unhandled=0):
    return {'policy': policy, 'done_calls': calls, 'success_prints': success,
            'failure_prints': failure, 'flags': flags,
            'quiescent': bool(quiescent), 'audited': bool(audited),
            'unhandled': unhandled}


def response(*, version=b'ERJR3', status='complete', phase='runtime',
             error_type='', identity='', message='', harness='', tail=None):
    output = bytearray(version)
    for value in (status, phase, error_type, identity, message, harness):
        data = value.encode('utf-8')
        output.extend(struct.pack('<I', len(data)))
        output.extend(data)
    if version == b'ERJR3':
        if tail is None:
            tail = (1, 1, 1, 0, 0, 1, 1, 0)
        output.extend(struct.pack('<BBBBHBBI', *tail))
    return bytes(output)


class Test262AsyncTransport(unittest.TestCase):
    def test_legacy_framing_and_outcomes_remain_exact_without_opt_in(self):
        self.assertEqual(runner.encode_request(case()),
                         b'ERJS1\x00\x00\x00\x00\x00\x00\x01\x00\x00\x001')
        parsed = runner.decode_response(response(version=b'ERJR2'))
        self.assertEqual(parsed, {'status': 'complete', 'phase': 'runtime',
                                  'error_type': '', 'error_identity': '',
                                  'message': '', 'harness': ''})
        self.assertEqual(runner.classify(case(), parsed), 'passed')
        negative = case(source=b'var =')
        negative['metadata']['negative'] = {'phase': 'parse', 'type': 'SyntaxError'}
        self.assertEqual(runner.encode_request(negative)[:7], b'ERJS1\x00\x01')
        observed = runner.decode_response(response(version=b'ERJR2', status='exception',
                                                   phase='parse', error_type='SyntaxError',
                                                   identity='SyntaxError', message='marker'))
        self.assertEqual(runner.classify(negative, observed), 'passed')
        observed['error_identity'] = ''
        self.assertEqual(runner.classify(negative, observed), 'failed')

    def test_old_profile_host_and_async_exclusions_do_not_change(self):
        self.assertEqual(runner.unsupported_reason(case(asynchronous=True)),
                         'asynchronous completion and jobs are not implemented')
        self.assertEqual(runner.unsupported_reason(case(source=b'$DONE();')),
                         'Test262 host hooks are not implemented (conservative source check)')
        self.assertEqual(runner.unsupported_reason(case(source=b"print('marker');")),
                         'Test262 host hooks are not implemented (conservative source check)')

    def test_opt_in_erjs2_has_exact_policy_byte_and_retains_payload(self):
        for policy, asynchronous, mode, header in [
            (0, False, 'sloppy', b'ERJS2\x00\x00\x00'),
            (1, True, 'strict', b'ERJS2\x01\x00\x01'),
        ]:
            value = case(asynchronous=asynchronous, policy=policy, mode=mode)
            self.assertEqual(runner.encode_request(value),
                             header + b'\x00\x00\x00\x00\x01\x00\x00\x001')
        value = case(asynchronous=True, policy=1, source=b'$DONE();')
        value['harness'] = [('x.js', b'2')]
        self.assertEqual(runner.encode_request(value),
                         b'ERJS2\x00\x00\x01\x01\x00\x00\x00'
                         b'\x04\x00\x00\x00x.js\x01\x00\x00\x002'
                         b'\x08\x00\x00\x00$DONE();')
        # This is a framing-only fixture. Exact helper/order authentication is
        # the adapter's separate pre-execution responsibility.

    def test_invalid_explicit_policy_and_new_mode_combinations_are_refused(self):
        values = [case(policy=-1), case(policy=2), case(policy=True),
                  case(asynchronous=True, policy=True), case(policy=False),
                  case(policy='1'), case(policy=1),
                  case(asynchronous=True, policy=0),
                  case(policy=0, mode='raw'), case(policy=0, mode='module')]
        parse_only = case(policy=0)
        parse_only['metadata']['negative'] = {'phase': 'parse', 'type': 'SyntaxError'}
        values.append(parse_only)
        for value in values:
            with self.subTest(value=value), self.assertRaises(ValueError):
                runner.encode_request(value)

    def test_only_new_profile_admits_done_and_print_harness_routes(self):
        value = case(asynchronous=True, policy=1, source=b'Promise.resolve().then($DONE,$DONE);')
        value['metadata']['features'] = ['Promise', 'Promise.prototype.finally']
        value['harness'] = [('doneprintHandle.js', b'function $DONE(){print("marker");}')]
        self.assertIsNone(runner.unsupported_reason(value, FEATURES, PROFILE))
        self.assertIsNotNone(runner.unsupported_reason(value, FEATURES))
        synchronous = case(policy=0, source=b'Promise.resolve(7).then(function(){});')
        self.assertIsNone(runner.unsupported_reason(synchronous, FEATURES, PROFILE))

    def test_new_profile_keeps_unimplemented_features_and_cross_realm_nonpass(self):
        for feature in ('class', 'cross-realm'):
            value = case(policy=0)
            value['metadata']['features'] = [feature]
            reason = runner.unsupported_reason(value, FEATURES, PROFILE)
            self.assertIsNotNone(reason)
            self.assertIn(feature, reason)
        value = case(policy=0, source=b'$262.createRealm();')
        self.assertIsNotNone(runner.unsupported_reason(value, FEATURES, PROFILE))

    def test_complete_erjr3_requires_matching_private_evidence(self):
        for policy, asynchronous, tail, expected in [
            (1, True, (1, 1, 1, 0, 0, 1, 1, 0), evidence()),
            (0, False, (0, 0, 0, 0, 0, 1, 1, 0), evidence(0, 0, 0)),
        ]:
            observed = runner.decode_response(response(tail=tail))
            self.assertEqual(observed, {'status': 'complete', 'phase': 'runtime',
                                        'error_type': '', 'error_identity': '',
                                        'message': '', 'harness': '',
                                        'completion': expected})
            self.assertEqual(runner.classify(case(asynchronous=asynchronous, policy=policy),
                                             observed), 'passed')

    def test_host_vetoes_remain_failed_with_distinct_evidence(self):
        for reason, tail in [
            ('missing-completion', (1, 0, 0, 0, 0, 1, 1, 0)),
            ('duplicate-completion', (1, 2, 2, 0, 0, 1, 1, 0)),
            ('completion-error', (1, 1, 0, 1, 1, 1, 1, 0)),
            ('queued-exception', (1, 1, 1, 0, 4, 1, 1, 0)),
            ('unhandled-rejection', (1, 1, 1, 0, 0, 1, 1, 1)),
            ('unauthenticated-completion', (1, 0, 0, 0, 16, 1, 1, 0)),
            ('completion-protocol-error', (1, 1, 0, 0, 0, 1, 1, 0)),
            ('fatal-observation', (1, 1, 1, 0, 8, 0, 0, 0)),
        ]:
            with self.subTest(reason=reason):
                observed = runner.decode_response(response(status='host-failure', phase='host',
                                                           message=reason, tail=tail))
                self.assertEqual(observed['message'], reason)
                self.assertEqual(runner.classify(case(asynchronous=True, policy=1), observed),
                                 'failed')

    def test_new_framing_rejects_missing_extra_and_out_of_domain_evidence(self):
        valid = response()
        malformed = [valid[:-1], valid + b'\x00', valid[:-12], b'ERJR4' + valid[5:],
                     response(tail=(2, 1, 1, 0, 0, 1, 1, 0)),
                     response(tail=(1, 3, 1, 0, 0, 1, 1, 0)),
                     response(tail=(1, 1, 3, 0, 0, 1, 1, 0)),
                     response(tail=(1, 1, 0, 3, 1, 1, 1, 0)),
                     response(tail=(1, 1, 1, 0, 512, 1, 1, 0)),
                     response(tail=(1, 1, 1, 0, 0, 2, 1, 0)),
                     response(tail=(1, 1, 1, 0, 0, 1, 2, 0))]
        for value in malformed:
            with self.subTest(value=value[-12:]), self.assertRaises(ValueError):
                runner.decode_response(value)

    def test_new_complete_and_audit_contradictions_cannot_be_defaulted_to_pass(self):
        invalid = [
            response(tail=(1, 0, 0, 0, 0, 1, 1, 0)),
            response(tail=(1, 2, 2, 0, 0, 1, 1, 0)),
            response(tail=(1, 1, 1, 0, 4, 1, 1, 0)),
            response(tail=(1, 1, 1, 0, 0, 1, 1, 1)),
            response(tail=(1, 1, 1, 0, 0, 0, 1, 0)),
            response(tail=(1, 1, 1, 0, 0, 1, 0, 1)),
            response(tail=(1, 1, 1, 0, 8, 1, 1, 0)),
            response(tail=(1, 0, 1, 0, 0, 1, 1, 0)),
            response(tail=(0, 1, 1, 0, 0, 1, 1, 0)),
            response(message='not empty'), response(phase='parse'),
            response(status='host-failure', phase='host', message='missing-completion'),
        ]
        for value in invalid:
            with self.subTest(value=value), self.assertRaises(ValueError):
                runner.decode_response(value)

    def test_classify_requires_protocol_and_policy_correspondence(self):
        old = {'status': 'complete', 'phase': 'runtime', 'error_type': '',
               'error_identity': '', 'message': '', 'harness': ''}
        new = dict(old, completion=evidence())
        requested = case(asynchronous=True, policy=1)
        self.assertEqual(runner.classify(requested, old), 'failed')
        self.assertEqual(runner.classify(case(policy=0), new), 'failed')
        self.assertEqual(runner.classify(case(), new), 'failed')
        self.assertEqual(runner.classify(requested, new), 'passed')
        new['completion'] = evidence(flags=4)
        self.assertEqual(runner.classify(requested, new), 'failed')

    def test_actual_errors_keep_phase_identity_and_never_use_sentinel_messages(self):
        requested = case(asynchronous=True, policy=1)
        observed = runner.decode_response(response(status='exception', phase='runtime',
                                                   error_type='TypeError', identity='TypeError',
                                                   message='Test262:AsyncTestComplete',
                                                   tail=(1, 1, 1, 0, 256, 1, 1, 0)))
        self.assertEqual(runner.classify(requested, observed), 'failed')
        resource = runner.decode_response(response(status='resource', phase='runtime',
                                                   error_type='ResourceLimit', message='marker',
                                                   tail=(1, 1, 1, 0, 264, 0, 0, 0)))
        self.assertEqual(runner.classify(requested, resource), 'resource')
        parsed = runner.decode_response(response(status='exception', phase='parse',
                                                 error_type='SyntaxError', identity='SyntaxError',
                                                 message='marker', tail=(1, 0, 0, 0, 0, 0, 0, 0)))
        self.assertEqual(runner.classify(requested, parsed), 'failed')


if __name__ == '__main__':
    unittest.main()
