import copy
import contextlib
import io
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner


def sample(source=b'/*---\ndescription: sample\n---*/\nassert(true);', mode='sloppy'):
    metadata = importer.parse_metadata(source.decode())
    case = dict(id='sample:' + mode, file='sample.js', mode=mode,
                source=source, metadata=metadata, harness=[])
    case['case_sha256'] = runner.case_fingerprint(case)
    return case


def response(status, phase='runtime', error_type='', message='', harness='', error_identity=None):
    if error_identity is None:
        error_identity = error_type if error_type in runner.INTRINSIC_ERRORS else ''
    result = bytearray(b'ERJR2')
    for value in (status, phase, error_type, error_identity, message, harness):
        value = value.encode()
        result.extend(struct.pack('<I', len(value)))
        result.extend(value)
    return bytes(result)


class MetadataTests(unittest.TestCase):
    def test_mode_variants_include_every_default_and_respect_flags(self):
        expected = [('', ['sloppy', 'strict']), ('noStrict', ['sloppy']),
                    ('onlyStrict', ['strict']), ('module', ['module']), ('raw', ['raw'])]
        for flag, variants in expected:
            metadata = importer.parse_metadata(f'/*---\nflags: [{flag}]\n---*/')
            self.assertEqual(runner.modes(metadata), variants)
        with self.assertRaisesRegex(ValueError, 'conflicting'):
            importer.parse_metadata('/*---\nflags: [noStrict, onlyStrict]\n---*/')

    def test_list_formats_negative_type_and_order_are_preserved(self):
        metadata = importer.parse_metadata('''/*---
description: >
  Ordinary description with [brackets] and negative: runtime text.
includes:
  - second.js
  - first.js
features: ["Symbol", 'arrow-function',]
negative:
  phase: runtime
  type: TypeError
---*/''')
        self.assertEqual(metadata['includes'], ['second.js', 'first.js'])
        self.assertEqual(metadata['features'], ['Symbol', 'arrow-function'])
        self.assertEqual(metadata['negative'], dict(phase='runtime', type='TypeError'))
        self.assertEqual(runner.harness_names(metadata), ['assert.js', 'sta.js', 'second.js', 'first.js'])
        metadata['flags'] = ['raw']
        self.assertEqual(runner.harness_names(metadata), [])

    def test_ambiguous_or_unsupported_metadata_never_silently_defaults(self):
        for metadata in ('flags: [raw]\nflags: [noStrict]', 'negative:\n  phase: parse',
                         'unknownKey: true', 'includes: [../escape.js]',
                         'features: !!python/object:whatever', 'flags: [raw, module]'):
            with self.subTest(metadata=metadata), self.assertRaises(ValueError):
                importer.parse_metadata('/*---\n' + metadata + '\n---*/')

    def test_strict_async_module_host_and_unimplemented_features_are_visible(self):
        for metadata in ('flags: [onlyStrict]', 'flags: [async]', 'flags: [module]',
                         'features: [Reflect.construct]', 'features: [cross-realm]',
                         'features: [Symbol]', 'flags: [FutureFlag]'):
            case = sample(('/*---\n' + metadata + '\n---*/\n').encode())
            case['mode'] = runner.modes(case['metadata'])[0]
            with patch.object(runner, 'bounded_process', side_effect=AssertionError('must not execute')):
                self.assertEqual(runner.run_case(case, Path('/missing'), 1)['status'], 'unsupported')
        case = sample(b'/*---\n---*/\n$262.createRealm();')
        self.assertIn('host hooks', runner.unsupported_reason(case))

    def test_for_in_order_feature_is_executed_without_relaxing_strict_policy(self):
        case = sample(b'/*---\nfeatures: [for-in-order]\n---*/\nfor(var x in {}) {}')
        self.assertIsNone(runner.unsupported_reason(case))
        case['mode'] = 'strict'
        self.assertIn('strict', runner.unsupported_reason(case))

    def test_request_framing_preserves_cr_unicode_and_harness_bytes(self):
        source = b'/*---\r\n---*/\r\n"\r\n";' + '\U0001f980'.encode()
        case = sample(source)
        case['harness'] = [('one.js', b'// exact\r\n'), ('two.js', b'// second\n')]
        encoded = runner.encode_request(case)
        self.assertEqual(encoded[:7], b'ERJS1\0\0')
        self.assertEqual(struct.unpack_from('<I', encoded, 7)[0], 2)
        self.assertTrue(encoded.endswith(struct.pack('<I', len(source)) + source))
        self.assertIn(b'// exact\r\n', encoded)


class ExecutionTests(unittest.TestCase):
    def test_negative_result_requires_both_phase_and_error_constructor(self):
        case = sample(b'/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\n')
        for status, phase, error_type, expected in [
            ('exception', 'runtime', 'TypeError', 'passed'),
            ('exception', 'parse', 'TypeError', 'failed'),
            ('exception', 'runtime', 'SyntaxError', 'failed'),
            ('complete', 'runtime', '', 'failed'),
            ('harness-error', 'harness', 'TypeError', 'harness-error'),
            ('unsupported', 'parse', 'UnsupportedFeature', 'unsupported'),
            ('resource', 'runtime', 'ResourceLimit', 'resource'),
        ]:
            with self.subTest(status=status, phase=phase, error_type=error_type):
                observation = dict(status=status, phase=phase, error_type=error_type, error_identity=error_type)
                self.assertEqual(runner.classify(case, observation), expected)
        case['metadata']['negative']['phase'] = 'parse'
        self.assertEqual(runner.encode_request(case)[6], 1)
        self.assertEqual(runner.classify(case, dict(status='exception', phase='parse', error_type='TypeError', error_identity='TypeError')), 'passed')

    def test_negative_builtins_use_identity_instead_of_mutable_constructor_name(self):
        case = sample(b'/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\n')
        forged = runner.decode_response(response('exception', error_type='TypeError', error_identity=''))
        self.assertEqual(runner.classify(case, forged), 'failed')
        renamed = runner.decode_response(response('exception', error_type='Renamed', error_identity='TypeError'))
        self.assertEqual(runner.classify(case, renamed), 'passed')
        case['metadata']['negative']['type'] = 'CustomError'
        self.assertIn('constructor identity', runner.unsupported_reason(case))

    def test_parse_error_cannot_pass_a_positive_test(self):
        case = sample()
        self.assertEqual(runner.classify(case, dict(status='exception', phase='parse', error_type='SyntaxError')), 'failed')
        self.assertEqual(runner.classify(case, dict(status='complete', phase='parse', error_type='')), 'failed')

    def test_harness_failure_crash_timeout_and_resource_have_separate_outcomes(self):
        case = sample()
        observations = [
            ((0, response('harness-error', 'harness', 'SyntaxError', 'bad harness', 'include.js'), b''), 'harness-error'),
            ((0, response('resource', 'runtime', 'ResourceLimit'), b''), 'resource'),
            ((-9, b'', b'killed'), 'adapter-error'),
            ((0, b'not a frame', b''), 'adapter-error'),
        ]
        for result, status in observations:
            with patch.object(runner, 'bounded_process', return_value=result):
                self.assertEqual(runner.run_case(case, Path('/fake'), 1)['status'], status)
        with patch.object(runner, 'bounded_process', side_effect=subprocess.TimeoutExpired('adapter', 1)):
            self.assertEqual(runner.run_case(case, Path('/fake'), 1)['status'], 'timeout')

    def test_response_protocol_rejects_truncation_trailing_data_and_contradictions(self):
        self.assertEqual(runner.decode_response(response('complete'))['status'], 'complete')
        for data in (response('complete')[:-1], response('complete') + b'junk',
                     response('complete', 'harness'), response('exception', 'runtime'),
                     response('complete', error_identity='TypeError'),
                     response('exception', error_type='Fake', error_identity='Fake'),
                     response('harness-error', 'runtime', 'Error', '', 'a.js'),
                     b'ERJR2\xff\xff\xff\xff'):
            with self.assertRaises(ValueError):
                runner.decode_response(data)

    def test_fresh_process_is_used_per_executed_variant(self):
        case = sample()
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')) as run:
            self.assertEqual(runner.run_case(case, Path('/fake'), 1)['status'], 'passed')
            self.assertEqual(runner.run_case(case, Path('/fake'), 1)['status'], 'passed')
            self.assertEqual(run.call_count, 2)
            self.assertEqual(run.call_args.args[0], ['/fake'])

    def test_preflight_detects_disabled_assertions_and_wrong_error_identity(self):
        files = {'harness/assert.js': b'unchanged assert', 'harness/sta.js': b'unchanged sta',
                 'harness/propertyHelper.js': b'unchanged property helper'}
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            results = runner.harness_preflight(files, Path('/fake'), 1)
        self.assertFalse(all(result['verified'] for result in results))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            results = runner.harness_preflight(files, Path('/fake'), 1)
        self.assertFalse(any(result['verified'] for result in results))


class IntegrityTests(unittest.TestCase):
    def test_full_pinned_inventory_and_every_mode_are_present(self):
        manifest, _, cases, fixtures, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262')
        self.assertEqual(manifest['test_files'], 326)
        self.assertEqual(len(cases), 652)
        self.assertEqual(fixtures, [])
        self.assertEqual(len({case['file'] for case in cases}), 326)
        self.assertEqual({case['mode'] for case in cases}, {'sloppy', 'strict'})

    def test_hash_binds_source_harness_order_metadata_and_mode(self):
        case = sample()
        case['harness'] = [('a.js', b'one'), ('b.js', b'two')]
        original = runner.case_fingerprint(case)
        for change in ('source', 'harness', 'metadata', 'mode'):
            altered = copy.deepcopy(case)
            if change == 'source': altered['source'] += b'\r'
            if change == 'harness': altered['harness'].reverse()
            if change == 'metadata': altered['metadata']['negative'] = dict(phase='parse', type='SyntaxError')
            if change == 'mode': altered['mode'] = 'strict'
            self.assertNotEqual(runner.case_fingerprint(altered), original)

    def test_manifest_corruption_or_dropped_inventory_cannot_hide_a_case(self):
        source_root = runner.ROOT / 'tests/upstream/test262'
        original = json.loads((source_root / 'manifest.json').read_bytes())
        for mutation in ('inventory', 'hash', 'duplicate', 'path'):
            manifest = copy.deepcopy(original)
            if mutation == 'inventory': manifest['directories']['JSON/parse'].pop()
            if mutation == 'hash': manifest['files'][0]['sha256'] = '0' * 64
            if mutation == 'duplicate': manifest['files'].append(manifest['files'][0])
            if mutation == 'path': manifest['files'][0]['path'] = '../escape'
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                (directory / 'manifest.json').write_text(json.dumps(manifest))
                for entry in original['files']:
                    target = directory / entry['path']
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes((source_root / entry['path']).read_bytes())
                with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                    runner.load_corpus(directory)

    def test_regression_gate_binds_inventory_policy_and_case_hashes(self):
        previous = dict(revision=importer.REVISION, corpus_manifest_sha256='corpus', runner_policy_sha256='policy',
                        cases={'a:sloppy': dict(status='passed', case_sha256='a'),
                               'b:sloppy': dict(status='failed', case_sha256='b')})
        current = copy.deepcopy(previous)
        current['cases']['a:sloppy']['status'] = 'unsupported'
        current['cases']['b:sloppy']['status'] = 'passed'
        self.assertEqual(runner.check_baseline(previous, current), (['a:sloppy'], ['b:sloppy']))
        for field in ('revision', 'corpus_manifest_sha256', 'runner_policy_sha256'):
            altered = copy.deepcopy(current)
            altered[field] = 'changed'
            with self.assertRaises(ValueError): runner.check_baseline(previous, altered)
        current['cases']['a:sloppy']['case_sha256'] = 'changed'
        with self.assertRaises(ValueError): runner.check_baseline(previous, current)
        current['cases'].pop('a:sloppy')
        with self.assertRaises(ValueError): runner.check_baseline(previous, current)

    def test_cli_cannot_overwrite_baseline_while_checking(self):
        result = subprocess.run([sys.executable, str(runner.ROOT / 'tools/test262_conformance.py'),
                                 '--baseline', 'same.json', '--record-baseline', 'same.json'],
                                capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertIn(b'mutually exclusive', result.stderr)

    def test_report_cannot_overwrite_the_baseline_path_or_a_symlink_alias(self):
        with tempfile.TemporaryDirectory() as temporary:
            baseline = Path(temporary) / 'baseline.json'
            baseline.write_bytes(b'preserved baseline')
            alias = Path(temporary) / 'report.json'
            alias.symlink_to(baseline)
            hardlink = Path(temporary) / 'hardlink.json'
            hardlink.hardlink_to(baseline)
            for output in (baseline, alias, hardlink):
                result = subprocess.run([sys.executable, str(runner.ROOT / 'tools/test262_conformance.py'),
                                         '--baseline', str(baseline), '--output', str(output)],
                                        capture_output=True, timeout=5)
                self.assertEqual(result.returncode, 2)
                self.assertIn(b'separate from the baseline', result.stderr)
                self.assertEqual(baseline.read_bytes(), b'preserved baseline')

    def test_unhealthy_runs_never_replace_a_recorded_baseline(self):
        case = sample()
        for status, preflight in [('passed', False), ('resource', True),
                                  ('timeout', True), ('adapter-error', True)]:
            with self.subTest(status=status, preflight=preflight), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary = root / 'adapter'
                binary.write_bytes(b'stable test adapter identity')
                baseline = root / 'baseline.json'
                baseline.write_bytes(b'previous reviewed baseline')
                output = root / 'report.json'
                arguments = ['test262_conformance.py', '--binary', str(binary),
                             '--record-baseline', str(baseline), '--output', str(output)]
                manifest = dict(revision=importer.REVISION, repository='tc39/test262', test_files=1)
                outcome = {key: case[key] for key in ('id', 'file', 'mode', 'case_sha256')}
                outcome.update(status=status, source_sha256=runner.digest(case['source']))
                with (patch.object(sys, 'argv', arguments),
                      patch.object(runner, 'load_corpus', return_value=(manifest, {}, [case], [], 'manifest')),
                      patch.object(runner, 'harness_preflight', return_value=[dict(verified=preflight)]),
                      patch.object(runner, 'run_case', return_value=outcome),
                      contextlib.redirect_stdout(io.StringIO())):
                    self.assertEqual(runner.main(), 1)
                self.assertEqual(baseline.read_bytes(), b'previous reviewed baseline')
                self.assertTrue(output.exists())


if __name__ == '__main__':
    unittest.main()
