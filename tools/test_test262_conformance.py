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
    def test_common_mapping_indentation_preserves_nested_execution_metadata(self):
        plain = ('description: >\n  A folded description.\n'
                 'flags: [onlyStrict]\nfeatures: [default-parameters]\n'
                 'negative:\n  phase: parse\n  type: SyntaxError\n')
        expected = importer.parse_metadata('/*---\n' + plain + '---*/')
        indented = ''.join(' ' + line for line in plain.splitlines(keepends=True))
        self.assertEqual(importer.parse_metadata('/*---\n' + indented + '---*/'), expected)
        # An inconsistent root mapping or nested execution key remains invalid.
        for bad in (' flags: [onlyStrict]\nfeatures: [default-parameters]',
                    ' flags: [onlyStrict]\n features: [default-parameters]\n   negative: parse'):
            with self.assertRaises(ValueError):
                importer.parse_metadata('/*---\n' + bad + '\n---*/')

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

    def test_async_module_host_and_unimplemented_features_are_visible(self):
        for metadata in ('flags: [async]', 'flags: [module]',
                         'features: [Reflect.construct]', 'features: [cross-realm]',
                         'features: [Symbol]', 'flags: [FutureFlag]'):
            case = sample(('/*---\n' + metadata + '\n---*/\n').encode())
            case['mode'] = runner.modes(case['metadata'])[0]
            with patch.object(runner, 'bounded_process', side_effect=AssertionError('must not execute')):
                self.assertEqual(runner.run_case(case, Path('/missing'), 1)['status'], 'unsupported')
        case = sample(b'/*---\n---*/\n$262.createRealm();')
        self.assertIn('host hooks', runner.unsupported_reason(case))

    def test_strict_and_for_in_order_modes_are_executed_with_explicit_adapter_mode(self):
        case = sample(b'/*---\nfeatures: [for-in-order]\n---*/\nfor(var x in {}) {}')
        self.assertIsNone(runner.unsupported_reason(case))
        case['mode'] = 'strict'
        self.assertIsNone(runner.unsupported_reason(case))
        self.assertEqual(runner.encode_request(case)[5], 1)
        case = sample(b'/*---\nflags: [onlyStrict]\n---*/\nvar value=1;', 'strict')
        self.assertIsNone(runner.unsupported_reason(case))

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
    def test_numeric_parsing_retains_complete_global_directories_and_helpers(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-parsing', 'numeric-parsing')
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'parseInt': 55, 'parseFloat': 54})
        self.assertEqual(manifest_hash, 'f8959cc2e43d94b5b0c671105f87d3fc831fa6359065566791707a6f9b400c6a')
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (109, 218, []))
        self.assertTrue(all(not c['metadata']['flags'] and c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 109)
        self.assertEqual(sum(len(v) for p, v in files.items() if p.startswith('test/')), 110341)
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js',
            'harness/isConstructor.js', 'harness/decimalToHexString.js'})
        _, old, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        for path, data in files.items():
            if not path.startswith('test/') and path in old:
                self.assertEqual(data, old[path])

    def test_numeric_parsing_core_policy_preserves_excluded_prerequisites(self):
        self.assertEqual(runner.NUMERIC_PARSING_FEATURES, runner.SUPPORTED_FEATURES)
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-parsing', 'numeric-parsing')
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.NUMERIC_PARSING_FEATURES)]
        self.assertEqual((len(excluded), len({c['file'] for c in excluded})), (38, 19))
        self.assertTrue(all(set(c['metadata']['features']) & {'numeric-separator-literal', 'u180e', 'Reflect.construct', 'arrow-function'} for c in excluded))

    def test_numeric_parsing_pairs_guard_conversion_and_preserve_identical_setup(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-parsing', 'numeric-parsing')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, 'numeric-parsing')
        added = captured[32:]
        self.assertEqual(len(added), 48)
        self.assertEqual({c['mode'] for c in added}, {'strict', 'sloppy'})
        for case in added:
            method = 'parseFloat' if 'numeric-parsing-parseFloat-' in case['id'] else 'parseInt'
            guard = (f"var m={method};assert.sameValue(typeof m,'function');"
                     "assert.sameValue(m({toString:function(){return '12';}},10),12);"
                     f"assert.sameValue(Number.{method},m);")
            self.assertTrue(case['source'].startswith(guard.encode()))
            if b'assert.throws' in case['source']:
                self.assertGreater(case['source'].index(b'assert.throws'), len(guard) - 1)
            if 'property-metadata' in case['id']:
                self.assertEqual(case['source'].count(b'{restore:true}'), 3)
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'] +
                             (['propertyHelper.js'] if 'property-' in case['id'] else []))
        for offset in range(0, len(added), 2):
            good, bad = added[offset:offset + 2]
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])

    def test_numeric_parsing_preflights_require_actual_assertion_failures(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-parsing', 'numeric-parsing')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            old = runner.harness_preflight(files, Path('/fake'), 1)
            current = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-parsing')
        self.assertEqual(current[:32], old)
        self.assertEqual(len(current), 80)
        self.assertEqual(sum(p['verified'] for p in current[32:]), 24)
        for error_type in ('TypeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(
                    0, response('exception', 'runtime', error_type), b'')):
                wrong = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-parsing')
            self.assertFalse(any(p['verified'] for p in wrong[32:]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'Test262Error'), b'')):
            wrong = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-parsing')
        self.assertTrue(all(p['verified'] == (p['expected'] == 'failed') for p in wrong[32:]))
        self.assertFalse(all(p['verified'] for p in wrong))

    def test_numeric_parsing_import_checks_every_directory_and_source_blob(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-parsing', 'numeric-parsing')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/built-ins/'
        listings = {name: [] for name in importer.NUMERIC_PARSING_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                directory = str(Path(path).parent).removeprefix('test/built-ins/')
                listings[directory].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'numeric-parsing')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'numeric-parsing')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (218, []))
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'numeric-parsing')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
            item = listings[name].pop()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(Path(temporary), 'numeric-parsing')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name].append(item)

    def test_numeric_parsing_preserves_all_twelve_prior_profile_contracts(self):
        profiles = ('string-json', 'regexp', 'template-literal', 'functions', 'rest-parameters',
                    'is-prototype-of', 'global-values', 'array-sort', 'identifiers', 'array-reduce', 'number-statics', 'numeric-conversion')
        retained = {}
        for name in profiles:
            _, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(name), name)
            captured = []
            def capture(case, *args):
                captured.append(dict(name=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                                     source_sha256=runner.digest(case['source'])))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=capture):
                runner.harness_preflight(files, Path('/fake'), 3, name)
            retained[name] = dict(manifest_sha256=manifest_hash,
                                 cases={c['id']: c['case_sha256'] for c in cases},
                                 preflights=captured, fixtures=fixtures,
                                 features=sorted(runner.PROFILE_FEATURES[name]))
        self.assertEqual(sum(len(v['cases']) for v in retained.values()), 4527)
        self.assertEqual(sum(len(v['preflights']) for v in retained.values()), 840)
        self.assertEqual(runner.digest(json.dumps(retained, sort_keys=True, separators=(',', ':')).encode()),
                         'ad0c739d73c9593801c2f8524fc70ef679a58372f7da1b69a26c39f31b7fdd97')

    def test_numeric_conversion_retains_complete_global_directories_and_helpers(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-conversion', 'numeric-conversion')
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'isFinite': 15, 'isNaN': 15})
        self.assertEqual(manifest_hash, 'de7cf1098618ad0fdb7786d4ab4f6c648bfb6fe2543981fe21ca6e0cba50928f')
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (30, 60, []))
        self.assertTrue(all(not c['metadata']['flags'] and c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 30)
        self.assertEqual(sum(len(v) for p, v in files.items() if p.startswith('test/')), 23276)
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js',
            'harness/isConstructor.js', 'harness/nans.js'})
        _, old, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        for path, data in files.items():
            if not path.startswith('test/') and path in old:
                self.assertEqual(data, old[path])

    def test_numeric_conversion_core_policy_preserves_excluded_prerequisites(self):
        self.assertEqual(runner.NUMERIC_CONVERSION_FEATURES, runner.SUPPORTED_FEATURES)
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-conversion', 'numeric-conversion')
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.NUMERIC_CONVERSION_FEATURES)]
        self.assertEqual((len(excluded), len({c['file'] for c in excluded})), (32, 16))
        self.assertTrue(all(set(c['metadata']['features']) & {'Symbol.toPrimitive', 'Symbol', 'Reflect.construct', 'arrow-function'} for c in excluded))

    def test_numeric_conversion_pairs_guard_conversion_and_preserve_identical_setup(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-conversion', 'numeric-conversion')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, 'numeric-conversion')
        added = captured[32:]
        self.assertEqual(len(added), 48)
        self.assertEqual({c['mode'] for c in added}, {'strict', 'sloppy'})
        for case in added:
            guard = ("var N=Number;assert.sameValue(typeof N,'function');"
                     "assert.sameValue(N({valueOf:function(){return 5;}}),5);")
            for method in ('isFinite', 'isNaN'):
                if 'numeric-conversion-' + method + '-' in case['id']:
                    expected = 'true' if method == 'isFinite' else 'false'
                    guard = (f"var m={method};assert.sameValue(typeof m,'function');"
                             f"assert.sameValue(m({{valueOf:function(){{return 1;}}}}),{expected});")
            self.assertTrue(case['source'].startswith(guard.encode()))
            if b'assert.throws' in case['source']:
                self.assertGreater(case['source'].index(b'assert.throws'), len(guard) - 1)
            if 'property-metadata' in case['id']:
                self.assertEqual(case['source'].count(b'{restore:true}'), 2)
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'] +
                             (['propertyHelper.js'] if 'property-' in case['id'] else []))
        for offset in range(0, len(added), 2):
            good, bad = added[offset:offset + 2]
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])

    def test_numeric_conversion_preflights_require_actual_assertion_failures(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-conversion', 'numeric-conversion')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            old = runner.harness_preflight(files, Path('/fake'), 1)
            current = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-conversion')
        self.assertEqual(current[:32], old)
        self.assertEqual(len(current), 80)
        self.assertEqual(sum(p['verified'] for p in current[32:]), 24)
        for error_type in ('TypeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(
                    0, response('exception', 'runtime', error_type), b'')):
                wrong = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-conversion')
            self.assertFalse(any(p['verified'] for p in wrong[32:]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'Test262Error'), b'')):
            wrong = runner.harness_preflight(files, Path('/fake'), 1, 'numeric-conversion')
        self.assertTrue(all(p['verified'] == (p['expected'] == 'failed') for p in wrong[32:]))
        self.assertFalse(all(p['verified'] for p in wrong))

    def test_numeric_conversion_import_checks_every_directory_and_source_blob(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-numeric-conversion', 'numeric-conversion')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/built-ins/'
        listings = {name: [] for name in importer.NUMERIC_CONVERSION_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                directory = str(Path(path).parent).removeprefix('test/built-ins/')
                listings[directory].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'numeric-conversion')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'numeric-conversion')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (60, []))
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'numeric-conversion')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
            item = listings[name].pop()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(Path(temporary), 'numeric-conversion')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name].append(item)

    def test_numeric_conversion_preserves_all_eleven_prior_profile_contracts(self):
        profiles = ('string-json', 'regexp', 'template-literal', 'functions', 'rest-parameters',
                    'is-prototype-of', 'global-values', 'array-sort', 'identifiers', 'array-reduce', 'number-statics')
        retained = {}
        for name in profiles:
            _, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(name), name)
            captured = []
            def capture(case, *args):
                captured.append(dict(name=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                                     source_sha256=runner.digest(case['source'])))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=capture):
                runner.harness_preflight(files, Path('/fake'), 3, name)
            retained[name] = dict(manifest_sha256=manifest_hash,
                                 cases={c['id']: c['case_sha256'] for c in cases},
                                 preflights=captured, fixtures=fixtures,
                                 features=sorted(runner.PROFILE_FEATURES[name]))
        self.assertEqual(sum(len(v['cases']) for v in retained.values()), 4467)
        self.assertEqual(sum(len(v['preflights']) for v in retained.values()), 760)
        self.assertEqual(runner.digest(json.dumps(retained, sort_keys=True, separators=(',', ':')).encode()),
                         '33eaed6126587bc54161f7eb8113e53a0ec6eccae1bd7a6186ace5f4edf57033')

    def test_number_statics_retains_complete_root_directories_and_helpers(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()}, {
            'Number': 120, 'Number/MAX_VALUE': 4, 'Number/MIN_VALUE': 4,
            'Number/NEGATIVE_INFINITY': 4, 'Number/POSITIVE_INFINITY': 4,
            'Number/isFinite': 8, 'Number/isInteger': 9, 'Number/isNaN': 7, 'Number/isSafeInteger': 10})
        self.assertEqual(manifest_hash, '3c5c2503458843bfc64eaf546ee08b6e693714f561df42330d9001a2ec8216cd')
        self.assertEqual(runner.digest(json.dumps(manifest['directories'], sort_keys=True,
                                                separators=(',', ':')).encode()),
                         'd0de2ba2fb2f97a56cd9ee9720145d8c7e74b4096860bdc12a90ed9ea6f641aa')
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (170, 340, []))
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 170)
        self.assertTrue(all(not c['metadata']['flags'] and c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(len(v) for p, v in files.items() if p.startswith('test/')), 131195)
        self.assertEqual(sum(map(len, files.values())), 177192)
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js', 'harness/isConstructor.js'})
        _, old, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        for path, data in files.items():
            if not path.startswith('test/'):
                self.assertEqual(data, old[path])

    def test_number_statics_core_policy_keeps_constructor_and_unrelated_prerequisites(self):
        self.assertEqual(runner.NUMBER_STATIC_FEATURES, runner.SUPPORTED_FEATURES)
        _, files, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.NUMBER_STATIC_FEATURES)]
        self.assertEqual((len(excluded), len({c['file'] for c in excluded})), (90, 45))
        for name in ('EPSILON.js', 'MAX_SAFE_INTEGER.js', 'MIN_SAFE_INTEGER.js', 'NaN.js',
                     'return-abrupt-tonumber-value.js', 'S9.1_A1_T1.js', 'S9.3_A5_T1.js'):
            self.assertIn('test/built-ins/Number/' + name, files)
        for method in ('isFinite', 'isNaN', 'isInteger', 'isSafeInteger'):
            symbol_cases = [c for c in cases if c['file'].endswith(method + '/arg-is-not-number.js')]
            self.assertEqual(len(symbol_cases), 2)
            self.assertTrue(all('Symbol' in runner.unsupported_reason(c, runner.NUMBER_STATIC_FEATURES)
                                for c in symbol_cases))
        for feature in ('u180e', 'numeric-separator-literal', 'Reflect.construct', 'Symbol', 'BigInt'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.NUMBER_STATIC_FEATURES))

    def test_number_statics_preflights_require_actual_assertion_failures(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            old = runner.harness_preflight(files, Path('/fake'), 1)
            current = runner.harness_preflight(files, Path('/fake'), 1, 'number-statics')
        self.assertEqual(current[:32], old)
        self.assertEqual(len(current), 104)
        self.assertEqual(sum(p['verified'] for p in current[32:]), 36)
        for error_type in ('TypeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(
                    0, response('exception', 'runtime', error_type), b'')):
                wrong = runner.harness_preflight(files, Path('/fake'), 1, 'number-statics')
            self.assertFalse(any(p['verified'] for p in wrong[32:]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'Test262Error'), b'')):
            wrong = runner.harness_preflight(files, Path('/fake'), 1, 'number-statics')
        self.assertTrue(all(p['verified'] == (p['expected'] == 'failed') for p in wrong[32:]))
        self.assertFalse(all(p['verified'] for p in wrong))

    def test_number_statics_pairs_guard_methods_and_preserve_identical_setup(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, 'number-statics')
        added = captured[32:]
        self.assertEqual(len(added), 72)
        self.assertEqual({c['mode'] for c in added}, {'strict', 'sloppy'})
        for case in added:
            method = next((name for name in ('isFinite', 'isNaN', 'isInteger', 'isSafeInteger')
                           if 'number-statics-' + name + '-' in case['id']), None)
            if method:
                canonical = 'NaN' if method == 'isNaN' else '1'
                guard = (f"var m=Number.{method};assert.sameValue(typeof m,'function');"
                         f"assert.sameValue(m({canonical}),true);").encode()
                self.assertTrue(case['source'].startswith(guard))
                if b'assert.throws' in case['source']:
                    self.assertGreater(case['source'].index(b'assert.throws'), len(guard) - 1)
            if 'property-metadata' in case['id']:
                self.assertEqual(case['source'].count(b'{restore:true}'), 3)
            if 'property-constants' in case['id']:
                self.assertEqual(case['source'].count(b'{restore:true}'), 8)
            self.assertNotIn(b'@WRITES@', case['source'])
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'] +
                             (['propertyHelper.js'] if 'property-' in case['id'] else []))
        for offset in range(0, len(added), 2):
            good, bad = added[offset:offset + 2]
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])

    def test_number_statics_import_checks_every_directory_and_source_blob(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-number-statics', 'number-statics')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/built-ins/'
        listings = {name: [] for name in importer.NUMBER_STATIC_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                directory = str(Path(path).parent).removeprefix('test/built-ins/')
                listings[directory].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'number-statics')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'number-statics')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (340, []))
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'number-statics')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
            item = listings[name].pop()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(Path(temporary), 'number-statics')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name].append(item)

    def test_number_statics_preserves_all_ten_prior_profile_contracts(self):
        profiles = ('string-json', 'regexp', 'template-literal', 'functions', 'rest-parameters',
                    'is-prototype-of', 'global-values', 'array-sort', 'identifiers', 'array-reduce')
        retained = {}
        for name in profiles:
            _, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(name), name)
            captured = []
            def capture(case, *args):
                captured.append(dict(name=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                                     source_sha256=runner.digest(case['source'])))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=capture):
                runner.harness_preflight(files, Path('/fake'), 3, name)
            retained[name] = dict(manifest_sha256=manifest_hash,
                                 cases={c['id']: c['case_sha256'] for c in cases},
                                 preflights=captured, fixtures=fixtures,
                                 features=sorted(runner.PROFILE_FEATURES[name]))
        self.assertEqual(sum(len(v['cases']) for v in retained.values()), 4127)
        self.assertEqual(sum(len(v['preflights']) for v in retained.values()), 656)
        self.assertEqual(runner.digest(json.dumps(retained, sort_keys=True, separators=(',', ':')).encode()),
                         '9a15cbffcdbf5d6de826c9d56453e38787867f2e8effb3386fc1d08033042f8f')

    def test_array_reduce_inventory_retains_both_complete_directories(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'Array/prototype/reduce': 260, 'Array/prototype/reduceRight': 260})
        self.assertEqual(runner.digest(json.dumps(manifest['directories'], sort_keys=True,
                                                separators=(',', ':')).encode()),
                         'b7200a203682b4c2cd82695de080d24e7bfdc98f36113930bd56d719850cc60e')
        self.assertEqual(manifest_hash, '77ee716188d99fdba64adbe151ef4efd915fbaa6345b2b0ec8fe3550efb5ec24')
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (520, 1034, []))
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 520)
        self.assertTrue(all(c['metadata']['negative'] is None for c in cases))
        self.assertEqual(sum(len(v) for p, v in files.items() if p.startswith('test/')), 364611)
        self.assertEqual(sum(map(len, files.values())), 431206)
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js', 'harness/propertyHelper.js',
            'harness/isConstructor.js', 'harness/resizableArrayBufferUtils.js', 'harness/testTypedArray.js'})
        _, original, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        for path, data in files.items():
            if not path.startswith('test/'):
                if path in original:
                    self.assertEqual(data, original[path])
        for method, expected in [('reduce', '45ca3e00e2102c702d9b907015148b179d394bd48f24b242054dba2f9f6e6bdf'),
                                 ('reduceRight', '3c129440d766f2d3bd286c4f1c0ffb156f3373b83bc90adc526950923659f4cb')]:
            records = sorted((p, runner.digest(data)) for p, data in files.items()
                             if p.startswith('test/') and Path(p).parent.name == method)
            self.assertEqual(runner.digest(''.join(p + '\0' + h + '\n' for p, h in records).encode()), expected)
            selected = [c for c in cases if Path(c['file']).parent.name == method]
            self.assertEqual(len(selected), 517)
            self.assertEqual(sum('noStrict' in c['metadata']['flags'] for c in selected), 3)

    def test_array_reduce_core_policy_retains_unrelated_prerequisites(self):
        self.assertEqual(runner.ARRAY_REDUCE_FEATURES, runner.SUPPORTED_FEATURES)
        _, files, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        unsupported = [c for c in cases if runner.unsupported_reason(c, runner.ARRAY_REDUCE_FEATURES)]
        self.assertEqual((len(unsupported), len({c['file'] for c in unsupported})), (20, 10))
        for c in unsupported:
            self.assertTrue(set(c['metadata']['features']) & {'Reflect.construct', 'resizable-arraybuffer'})
        dependencies = [c for c in cases if b'Date' in c['source'] or b'Number.MAX_SAFE_INTEGER' in c['source']]
        self.assertEqual(len(dependencies), 10)
        self.assertTrue(all(runner.unsupported_reason(c, runner.ARRAY_REDUCE_FEATURES) is None for c in dependencies))
        self.assertIn(b'Number.MAX_SAFE_INTEGER', files['test/built-ins/Array/prototype/reduceRight/length-near-integer-limit.js'])
        for feature in ('stable-array-sort', 'rest-parameters', 'globalThis', 'u180e', 'Proxy', 'Reflect.construct', 'Symbol'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.ARRAY_REDUCE_FEATURES))

    def test_array_reduce_preflight_preserves_core_and_rejects_wrong_error_identity(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            previous = runner.harness_preflight(files, Path('/fake'), 1)
            current = runner.harness_preflight(files, Path('/fake'), 1, 'array-reduce')
        self.assertEqual(current[:32], previous)
        self.assertEqual(len(current), 128)
        self.assertEqual(sum(p['verified'] for p in current[32:]), 48)
        for error_type in ('TypeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(
                    0, response('exception', 'runtime', error_type), b'')):
                wrong = runner.harness_preflight(files, Path('/fake'), 1, 'array-reduce')
            self.assertFalse(any(p['verified'] for p in wrong[32:]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'Test262Error'), b'')):
            mismatch = runner.harness_preflight(files, Path('/fake'), 1, 'array-reduce')
        self.assertTrue(all(p['verified'] == (p['expected'] == 'failed') for p in mismatch[32:]))
        self.assertFalse(all(p['verified'] for p in mismatch))

    def test_array_reduce_guarded_pairs_differ_only_in_final_assertion(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            outcomes = runner.harness_preflight(files, Path('/fake'), 1, 'array-reduce')
        added = captured[32:]
        self.assertEqual(len(added), 96)
        self.assertEqual({c['mode'] for c in added}, {'strict', 'sloppy'})
        for c in added:
            method = 'reduceRight' if 'array-reduce-reduceRight-' in c['id'] else 'reduce'
            guard = (f"var m=Array.prototype.{method};assert.sameValue(typeof m,'function');"
                     "assert.sameValue(m.call([1,2],function(a,b){return a+b;},0),3);").encode()
            self.assertTrue(c['source'].startswith(guard))
            if b'assert.throws' in c['source']:
                self.assertGreater(c['source'].index(b'assert.throws'), len(guard) - 1)
            if 'property-metadata' in c['id']:
                self.assertEqual(c['source'].count(b'{restore:true}'), 3)
            self.assertNotIn(b'@THIS@', c['source'])
            self.assertEqual([name for name, _ in c['harness']], ['assert.js', 'sta.js'] +
                             (['propertyHelper.js'] if 'property-' in c['id'] else []))
        for offset in range(0, len(added), 2):
            good, bad = added[offset:offset + 2]
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertNotEqual(good['source'], bad['source'])
            self.assertEqual(outcomes[offset + 32]['expected'], 'passed')
            self.assertEqual(outcomes[offset + 33]['expected'], 'failed')

    def test_array_reduce_import_checks_both_directory_inventories_and_blobs(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-reduce', 'array-reduce')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/built-ins/'
        listings = {name: [] for name in importer.ARRAY_REDUCE_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                directory = str(Path(path).parent).removeprefix('test/built-ins/')
                listings[directory].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'array-reduce')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'array-reduce')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (1034, []))
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'array-reduce')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
            item = listings[name].pop()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(Path(temporary), 'array-reduce')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name].append(item)

    def test_array_reduce_preserves_all_nine_prior_profile_contracts(self):
        profiles = ('string-json', 'regexp', 'template-literal', 'functions', 'rest-parameters',
                    'is-prototype-of', 'global-values', 'array-sort', 'identifiers')
        retained = {}
        for name in profiles:
            _, files, cases, fixtures, manifest_hash = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(name), name)
            captured = []
            def capture(case, *args):
                captured.append(dict(name=case['id'], mode=case['mode'], case_sha256=case['case_sha256'],
                                     source_sha256=runner.digest(case['source'])))
                return dict(status='passed', mode=case['mode'], case_sha256=case['case_sha256'],
                            source_sha256=runner.digest(case['source']))
            with patch.object(runner, 'run_case', side_effect=capture):
                runner.harness_preflight(files, Path('/fake'), 3, name)
            retained[name] = dict(manifest_sha256=manifest_hash,
                                 cases={c['id']: c['case_sha256'] for c in cases},
                                 preflights=captured, fixtures=fixtures,
                                 features=sorted(runner.PROFILE_FEATURES[name]))
        self.assertEqual(sum(len(v['cases']) for v in retained.values()), 3093)
        self.assertEqual(sum(len(v['preflights']) for v in retained.values()), 528)
        self.assertEqual(runner.digest(json.dumps(retained, sort_keys=True, separators=(',', ':')).encode()),
                         '00bc0a2ee53cc28fd1eda61af5f8cc98cf83c910a1d65680987ef914b0a47177')

    def test_identifier_inventory_keeps_both_complete_directories_and_all_modes(self):
        manifest, files, cases, fixtures, manifest_hash = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        self.assertEqual({key: len(value) for key, value in manifest['directories'].items()},
                         {'identifiers': 268, 'white-space': 67})
        self.assertEqual(runner.digest(json.dumps(manifest['directories'], sort_keys=True,
                                                separators=(',', ':')).encode()),
                         '1a9cbb98a8acf6acc818eaf37d9f1e07f1a87c893d1cd790e84b49a30c0161c1')
        self.assertEqual(manifest_hash, 'b93b9e76f5231a3d342d95fe0a8cf47ca79c2a3101f2a78e4d5bb564c4d06ea8')
        self.assertEqual((manifest['test_files'], len(cases), fixtures), (335, 669, []))
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 334)
        self.assertEqual(sum(bool(c['metadata']['negative']) for c in cases), 243)
        self.assertEqual(len({c['file'] for c in cases if c['metadata']['negative']}), 122)
        self.assertEqual({Path(c['file']).name for c in cases if 'onlyStrict' in c['metadata']['flags']},
                         {'val-yield-strict.js'})
        self.assertEqual(sum(len(v) for p, v in files.items() if p.startswith('test/')), 2725828)
        self.assertEqual(sum(map(len, files.values())), 2771524)
        self.assertEqual(max(len(c['source']) for c in cases), 125267)
        self.assertTrue(all(not c['metadata']['includes'] for c in cases))
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js', 'harness/propertyHelper.js'})
        _, original, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262')
        for path, data in files.items():
            if not path.startswith('test/'):
                self.assertEqual(data, original[path])

    def test_identifier_policy_retains_private_classes_numeric_and_eval_boundaries(self):
        self.assertEqual(runner.IDENTIFIER_FEATURES, runner.SUPPORTED_FEATURES | {'u180e'})
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        unsupported = [c for c in cases if runner.unsupported_reason(c, runner.IDENTIFIER_FEATURES)]
        self.assertEqual(len(unsupported), 122)
        self.assertEqual(len({c['file'] for c in unsupported}), 61)
        for case in unsupported:
            self.assertTrue(set(case['metadata']['features']) & {'class', 'class-fields-private',
                                                                 'numeric-separator-literal'})
        eval_cases = [c for c in cases if b'eval(' in c['source']]
        self.assertTrue(eval_cases)
        self.assertTrue(all(runner.unsupported_reason(c, runner.IDENTIFIER_FEATURES) is None
                            for c in eval_cases))
        for feature in ('class', 'class-fields-private', 'numeric-separator-literal',
                        'Proxy', 'Reflect.construct', 'Symbol', 'identifiers'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.IDENTIFIER_FEATURES))
        for name, policy in runner.PROFILE_FEATURES.items():
            if name not in {'identifiers', 'template-literal'}:
                self.assertNotIn('u180e', policy)

    def test_identifier_preflight_keeps_core_and_rejects_disabled_or_wrong_assertions(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            old = runner.harness_preflight(files, Path('/fake'), 1)
            current = runner.harness_preflight(files, Path('/fake'), 1, 'identifiers')
        self.assertEqual(current[:32], old)
        self.assertEqual(len(current), 88)
        self.assertEqual(sum(p['verified'] for p in current[32:64]), 16)
        self.assertFalse(any(p['verified'] for p in current[64:]))
        self.assertFalse(any('prerequisite' in p for p in current[:64]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'TypeError'), b'')):
            wrong = runner.harness_preflight(files, Path('/fake'), 1, 'identifiers')
        self.assertFalse(any(p['verified'] for p in wrong[32:]))

    def test_identifier_syntax_preflights_require_a_real_positive_control_in_same_mode(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        for control_success in (False, True):
            def observe(case, *args):
                if case['metadata']['negative']:
                    observation = runner.decode_response(response('exception', 'parse', 'SyntaxError'))
                elif case['id'].endswith('-mismatch'):
                    observation = runner.decode_response(response('exception', 'runtime', 'Test262Error'))
                elif control_success:
                    observation = runner.decode_response(response('complete'))
                else:
                    observation = runner.decode_response(response('exception', 'parse', 'SyntaxError'))
                return dict(id=case['id'], file=case['file'], mode=case['mode'],
                            case_sha256=case['case_sha256'], source_sha256=runner.digest(case['source']),
                            expected_negative=case['metadata']['negative'],
                            status=runner.classify(case, observation), observation=observation)
            with patch.object(runner, 'run_case', side_effect=observe):
                outcomes = runner.harness_preflight(files, Path('/fake'), 1, 'identifiers')
            positives = {(p['name'], p['result']['mode']): p for p in outcomes[32:64]}
            for result in outcomes[64:]:
                self.assertEqual(result['result']['status'], 'passed')  # Preserve raw negative observation.
                self.assertEqual(result['verified'], control_success)
                prerequisite = result['prerequisite']
                self.assertEqual(prerequisite['mode'], result['result']['mode'])
                control = positives[(prerequisite['name'], prerequisite['mode'])]
                self.assertEqual(prerequisite['verified'], control['verified'])
                self.assertEqual(prerequisite['result'], control['result'])
                self.assertEqual(prerequisite['case_sha256'], control['result']['case_sha256'])
                self.assertEqual(prerequisite['source_sha256'], control['result']['source_sha256'])
                self.assertEqual(prerequisite['expected'], 'passed')

    def test_identifier_preflights_retain_raw_characters_and_intrinsic_parse_identity(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, 'identifiers')
        self.assertEqual(len(captured), 88)
        new = captured[32:]
        for case in new:
            self.assertNotIn(b'eval(', case['source'])
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'])
        for case in captured[64:]:
            self.assertEqual(case['metadata']['negative'], dict(phase='parse', type='SyntaxError'))
            for status, phase, identity in [('exception', 'runtime', 'SyntaxError'),
                                          ('exception', 'parse', ''),
                                          ('complete', 'parse', '')]:
                self.assertNotEqual(runner.classify(case, dict(status=status, phase=phase,
                                                              error_type='SyntaxError', error_identity=identity)),
                                    'passed')
        literal_nel = next(c for c in new if c['id'].endswith('syntax-forbidden-nel'))
        self.assertIn('\u0085'.encode(), literal_nel['source'])
        whitespace = next(c for c in new if c['id'].endswith('identifier-whitespace-literals'))
        self.assertTrue(whitespace['source'].startswith('\ufeff'.encode()))
        for offset in range(32, 64, 2):
            self.assertEqual(captured[offset]['source'].rsplit(b'assert.sameValue(', 1)[0],
                             captured[offset + 1]['source'].rsplit(b'assert.sameValue(', 1)[0])

    def test_identifier_import_checks_both_directory_inventories_and_blobs(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-identifiers', 'identifiers')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/language/'
        listings = {name: [] for name in importer.IDENTIFIER_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                listings[Path(path).parent.name].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'identifiers')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'identifiers')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (669, []))
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'identifiers')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
            item = listings[name].pop()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(Path(temporary), 'identifiers')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name].append(item)

    def test_array_sort_inventory_retains_all_sources_modes_and_harnesses(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        expected = {
            'S15.4.4.11_A1.1_T1.js', 'S15.4.4.11_A1.2_T1.js', 'S15.4.4.11_A1.2_T2.js',
            'S15.4.4.11_A1.3_T1.js', 'S15.4.4.11_A1.4_T1.js', 'S15.4.4.11_A1.4_T2.js',
            'S15.4.4.11_A1.5_T1.js', 'S15.4.4.11_A3_T1.js', 'S15.4.4.11_A3_T2.js',
            'S15.4.4.11_A4_T3.js', 'S15.4.4.11_A5_T1.js', 'S15.4.4.11_A6_T2.js',
            'S15.4.4.11_A8.js', 'bug_596_1.js', 'bug_596_2.js', 'call-with-primitive.js',
            'comparefn-grow.js', 'comparefn-nonfunction-call-throws.js',
            'comparefn-resizable-buffer.js', 'comparefn-shrink.js', 'length.js', 'name.js',
            'not-a-constructor.js', 'precise-comparefn-throws.js',
            'precise-prototype-accessors.js', 'precise-prototype-element.js', 'prop-desc.js',
            'resizable-buffer-default-comparator.js',
        } | {f'S15.4.4.11_A2.{section}_T{number}.js'
             for section in (1, 2) for number in (1, 2, 3)} | {
            f'precise-{kind}-{mutation}.js' for kind in ('getter', 'setter') for mutation in (
                'appends-elements', 'decreases-length', 'deletes-predecessor',
                'deletes-successor', 'increases-length', 'pops-elements',
                'sets-predecessor', 'sets-successor')
        } | {f'stability-{count}-elements.js' for count in (5, 11, 513, 2048)}
        self.assertEqual(manifest['directories'], {'Array/prototype/sort': sorted(expected)})
        self.assertEqual(manifest['test_files'], 54)
        self.assertEqual(len(cases), 107)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 53)
        self.assertEqual(fixtures, [])
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        self.assertEqual({Path(c['file']).name for c in cases if c['metadata']['flags']},
                         {'S15.4.4.11_A8.js'})
        self.assertEqual(sum(len(data) for path, data in files.items() if path.startswith('test/')),
                         150796)
        self.assertEqual(sum(map(len, files.values())), 200865)
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js',
            'harness/resizableArrayBufferUtils.js'})
        _, original, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262')
        for name in ('LICENSE', 'INTERPRETING.md', 'harness/assert.js',
                     'harness/sta.js', 'harness/propertyHelper.js', 'harness/compareArray.js'):
            self.assertEqual(files[name], original[name])
        for count, size in ((5, 840), (11, 1192), (513, 16507), (2048, 64618)):
            data = files[f'test/built-ins/Array/prototype/sort/stability-{count}-elements.js']
            self.assertEqual(len(data), size)
            self.assertIn(b'.reduce(', data)

    def test_array_sort_policy_is_isolated_and_preserves_exotic_cases(self):
        self.assertEqual(runner.ARRAY_SORT_FEATURES, runner.SUPPORTED_FEATURES | {'stable-array-sort'})
        stable = sample(b'/*---\nfeatures: [stable-array-sort]\n---*/\n')
        self.assertIsNone(runner.unsupported_reason(stable, runner.ARRAY_SORT_FEATURES))
        for name, features in runner.PROFILE_FEATURES.items():
            if name != 'array-sort':
                self.assertIn('stable-array-sort', runner.unsupported_reason(stable, features))
        for feature in ('Symbol', 'BigInt', 'Proxy', 'Reflect.construct',
                        'resizable-arraybuffer', 'Array.prototype.includes'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.ARRAY_SORT_FEATURES))
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        unsupported = [c for c in cases if runner.unsupported_reason(c, runner.ARRAY_SORT_FEATURES)]
        self.assertEqual(len(unsupported), 14)
        self.assertEqual({Path(c['file']).name for c in unsupported}, {
            'call-with-primitive.js', 'comparefn-grow.js', 'comparefn-nonfunction-call-throws.js',
            'comparefn-resizable-buffer.js', 'comparefn-shrink.js', 'not-a-constructor.js',
            'resizable-buffer-default-comparator.js'})

    def test_array_sort_preflight_preserves_core_and_requires_assertion_failures(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            previous = runner.harness_preflight(files, Path('/fake'), 1)
            results = runner.harness_preflight(files, Path('/fake'), 1, 'array-sort')
        self.assertEqual(results[:32], previous)
        self.assertEqual(len(results), 80)
        self.assertEqual(sum(c['verified'] for c in results[32:]), 24)
        self.assertEqual(len({c['name'] for c in results[32:]}), 24)
        for error_type in ('TypeError', 'ReferenceError', 'SyntaxError'):
            with patch.object(runner, 'bounded_process', return_value=(
                    0, response('exception', 'runtime', error_type), b'')):
                wrong_error = runner.harness_preflight(files, Path('/fake'), 1, 'array-sort')
            self.assertFalse(any(c['verified'] for c in wrong_error[32:]))
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'Test262Error'), b'')):
            assertions = runner.harness_preflight(files, Path('/fake'), 1, 'array-sort')
        self.assertTrue(all(c['verified'] == (c['expected'] == 'failed') for c in assertions[32:]))

    def test_array_sort_preflight_guards_missing_method_before_throw_assertions(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            outcomes = runner.harness_preflight(files, Path('/fake'), 1, 'array-sort')
        added = captured[32:]
        for case in added:
            self.assertTrue(case['source'].startswith(
                b"assert.sameValue(typeof Array.prototype.sort,'function');"))
            self.assertIn(b'assert.sameValue(smoke.sort(),smoke);', case['source'])
            if b'assert.throws' in case['source']:
                self.assertLess(case['source'].index(b'smoke.sort()'),
                                case['source'].index(b'assert.throws'))
            self.assertNotIn(b'.reduce(', case['source'])
            if 'property-metadata' in case['id']:
                self.assertEqual(case['source'].count(b'{restore:true}'), 3)
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'] +
                             (['propertyHelper.js'] if 'property-' in case['id'] else []))
        for offset in range(0, len(added), 2):
            good, bad = added[offset:offset + 2]
            # Each mismatch reuses exactly the same guarded setup and changes
            # only the final expectation, never an expected TypeError alone.
            self.assertEqual(good['source'].rsplit(b'assert.sameValue(', 1)[0],
                             bad['source'].rsplit(b'assert.sameValue(', 1)[0])
            self.assertEqual(outcomes[offset + 32]['expected'], 'passed')
            self.assertEqual(outcomes[offset + 33]['expected'], 'failed')

    def test_array_sort_import_checks_complete_inventory_and_pinned_blob(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-array-sort', 'array-sort')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = (f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
               f'test/built-ins/Array/prototype/sort?ref={importer.REVISION}')
        listing = [dict(type='file', name=Path(path).name,
                       sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                   for path, data in files.items() if path.startswith('test/')]
        def fetch(url):
            if url == api:
                return json.dumps(listing).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), 'array-sort')
            _, imported, cases, fixtures, _ = runner.load_corpus(Path(temporary), 'array-sort')
            self.assertEqual(imported, files)
            self.assertEqual((len(cases), fixtures), (107, []))
        saved = listing[0]['sha']
        listing[0]['sha'] = '0' * 40
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), 'array-sort')
            self.assertFalse(any(Path(temporary).iterdir()))
        listing[0]['sha'] = saved
        listing.pop()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(Path(temporary), 'array-sort')
            self.assertFalse(any(Path(temporary).iterdir()))

    def test_global_values_inventory_preserves_all_four_directories_and_modes(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-global-values', 'global-values')
        expected = {
            'global': {
                '10.2.1.1.3-4-16-s.js', '10.2.1.1.3-4-18-s.js',
                '10.2.1.1.3-4-22.js', '10.2.1.1.3-4-27.js',
                'S15.1_A1_T1.js', 'S15.1_A1_T2.js', 'S15.1_A2_T1.js',
                'global-object.js', 'property-descriptor.js',
            } | {f'S10.2.3_A{section}_T{n}.js'
                 for section in ('1.1', '1.2', '1.3', '2.1', '2.3') for n in range(1, 5)},
            'undefined': {'15.1.1.3-0.js', '15.1.1.3-1.js', '15.1.1.3-2.js',
                          '15.1.1.3-3.js', 'S15.1.1.3_A1.js', 'S15.1.1.3_A3_T2.js',
                          'S15.1.1.3_A4.js', 'prop-desc.js'},
            'NaN': {'15.1.1.1-0.js', 'S15.1.1.1_A1.js', 'S15.1.1.1_A2_T2.js',
                    'S15.1.1.1_A3_T2.js', 'S15.1.1.1_A4.js', 'prop-desc.js'},
            'Infinity': {'15.1.1.2-0.js', 'S15.1.1.2_A1.js', 'S15.1.1.2_A2_T2.js',
                         'S15.1.1.2_A3_T2.js', 'S15.1.1.2_A4.js', 'prop-desc.js'},
        }
        self.assertEqual({key: set(value) for key, value in manifest['directories'].items()}, expected)
        self.assertEqual(manifest['test_files'], 49)
        self.assertEqual(len(cases), 88)
        self.assertEqual(sum(c['mode'] == 'sloppy' for c in cases), 46)
        self.assertEqual(sum(c['mode'] == 'strict' for c in cases), 42)
        for directory, counts in {'global': (27, 29), 'undefined': (7, 5),
                                  'NaN': (6, 4), 'Infinity': (6, 4)}.items():
            selected = [c for c in cases if Path(c['file']).parent.name == directory]
            self.assertEqual(tuple(sum(c['mode'] == mode for c in selected)
                                   for mode in ('sloppy', 'strict')), counts)
        self.assertEqual(fixtures, [])
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        self.assertEqual({p for p in files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js'})
        _, original, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream/test262')
        for name in ('LICENSE', 'INTERPRETING.md', 'harness/assert.js',
                     'harness/sta.js', 'harness/propertyHelper.js'):
            self.assertEqual(files[name], original[name])
        self.assertIn(b'Date', files['test/built-ins/global/global-object.js'])
        self.assertIn(b'for (var x in obj)', files['harness/propertyHelper.js'])

    def test_global_values_policy_only_adds_global_this_without_old_profile_changes(self):
        self.assertEqual(runner.GLOBAL_VALUE_FEATURES, runner.SUPPORTED_FEATURES | {'globalThis'})
        global_this = sample(b'/*---\nfeatures: [globalThis]\n---*/\nglobalThis;')
        self.assertIsNone(runner.unsupported_reason(global_this, runner.GLOBAL_VALUE_FEATURES))
        for name, features in runner.PROFILE_FEATURES.items():
            if name != 'global-values':
                self.assertIn('globalThis', runner.unsupported_reason(global_this, features))
        for feature in ('Proxy', 'Reflect.construct', 'Symbol', 'rest-parameters', 'default-parameters'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.GLOBAL_VALUE_FEATURES))
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-global-values', 'global-values')
        self.assertEqual(sum(bool(c['metadata']['features']) for c in cases), 4)
        self.assertTrue(all(runner.unsupported_reason(c, runner.GLOBAL_VALUE_FEATURES) is None
                            for c in cases))

    def test_global_values_preflight_preserves_core_and_requires_real_assertions(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-global-values', 'global-values')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            previous = runner.harness_preflight(files, Path('/fake'), 1)
            results = runner.harness_preflight(files, Path('/fake'), 1, 'global-values')
        self.assertEqual(results[:32], previous)
        self.assertEqual(len(results), 64)
        checks = results[32:]
        self.assertEqual(sum(c['verified'] for c in checks), 16)
        self.assertEqual({c['name'] for c in checks if not c['verified']}, {
            'global-values-' + name + '-mismatch' for name in (
                'identity', 'immutable-descriptors', 'this-descriptor', 'immutable-write',
                'immutable-delete', 'this-replace', 'this-recreate', 'this-lexical')})
        self.assertEqual({c['result']['mode'] for c in checks}, {'sloppy', 'strict'})
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'TypeError'), b'')):
            wrong_error = runner.harness_preflight(files, Path('/fake'), 1, 'global-values')
        self.assertFalse(any(c['verified'] for c in wrong_error[32:]))
        captured = []
        def capture(case, *args):
            captured.append(case)
            return dict(status='passed')
        with patch.object(runner, 'run_case', side_effect=capture):
            runner.harness_preflight(files, Path('/fake'), 1, 'global-values')
        for case in captured[32:]:
            self.assertEqual([name for name, _ in case['harness']], ['assert.js', 'sta.js'])
            for forbidden in (b'hasOwnProperty', b'propertyIsEnumerable', b'verifyProperty', b'for (var x in'):
                self.assertNotIn(forbidden, case['source'])

    def test_global_values_import_checks_all_pinned_blobs_and_directory_counts(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-global-values', 'global-values')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/test/built-ins/'
        listings = {name: [] for name in importer.GLOBAL_VALUE_DIRECTORIES}
        for path, data in files.items():
            if path.startswith('test/'):
                listings[Path(path).parent.name].append(dict(type='file', name=Path(path).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()))
        def fetch(url):
            if url.startswith(api):
                name = url[len(api):].removesuffix('?ref=' + importer.REVISION)
                return json.dumps(listings[name]).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'global-values')
            _, imported, cases, _, _ = runner.load_corpus(output, 'global-values')
            self.assertEqual(imported, files)
            self.assertEqual(len(cases), 88)
        for name in listings:
            saved = listings[name][0]['sha']
            listings[name][0]['sha'] = '0' * 40
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(Path(temporary), 'global-values')
                self.assertFalse(any(Path(temporary).iterdir()))
            listings[name][0]['sha'] = saved
        listings['global'].pop()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(Path(temporary), 'global-values')
            self.assertFalse(any(Path(temporary).iterdir()))

    def test_is_prototype_of_inventory_retains_complete_directory_and_modes(self):
        directory = runner.ROOT / 'tests/upstream/test262-is-prototype-of'
        manifest, files, cases, fixtures, _ = runner.load_corpus(directory, 'is-prototype-of')
        expected = {
            'arg-is-proxy.js', 'builtin.js', 'length.js', 'name.js', 'not-a-constructor.js',
            'null-this-and-object-arg-throws.js', 'null-this-and-primitive-arg-returns-false.js',
            'this-value-is-in-prototype-chain-of-arg.js',
            'undefined-this-and-object-arg-throws.js',
            'undefined-this-and-primitive-arg-returns-false.js',
        }
        self.assertEqual(manifest['test_files'], 10)
        self.assertEqual(set(manifest['directories']['Object/prototype/isPrototypeOf']), expected)
        self.assertEqual({Path(case['file']).name for case in cases}, expected)
        self.assertEqual(len(cases), 20)
        self.assertEqual(sum(case['mode'] == 'sloppy' for case in cases), 10)
        self.assertEqual(sum(case['mode'] == 'strict' for case in cases), 10)
        self.assertFalse(any(case['metadata']['negative'] for case in cases))
        self.assertEqual(fixtures, [])
        self.assertEqual({path for path in files if path.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js', 'harness/proxyTrapsHelper.js',
        })
        self.assertIn(b'Reflect.construct', files['harness/isConstructor.js'])
        self.assertIn(b'new Proxy', files['test/built-ins/Object/prototype/isPrototypeOf/arg-is-proxy.js'])
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(directory, 'string-json')

    def test_is_prototype_of_policy_does_not_admit_proxy_reflect_or_symbol(self):
        self.assertEqual(runner.IS_PROTOTYPE_OF_FEATURES, runner.SUPPORTED_FEATURES)
        for feature in ('Proxy', 'Reflect.construct', 'Symbol', 'rest-parameters'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.IS_PROTOTYPE_OF_FEATURES))
        _, _, cases, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-is-prototype-of', 'is-prototype-of')
        executed = [case for case in cases
                    if runner.unsupported_reason(case, runner.IS_PROTOTYPE_OF_FEATURES) is None]
        self.assertEqual(len(executed), 10)
        self.assertEqual({Path(case['file']).name for case in executed}, {
            'length.js', 'name.js', 'null-this-and-object-arg-throws.js',
            'this-value-is-in-prototype-chain-of-arg.js', 'undefined-this-and-object-arg-throws.js',
        })

    def test_is_prototype_of_preflight_retains_core_and_rejects_disabled_assertions(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-is-prototype-of', 'is-prototype-of')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            previous = runner.harness_preflight(files, Path('/fake'), 1)
            results = runner.harness_preflight(files, Path('/fake'), 1, 'is-prototype-of')
        self.assertEqual(len(previous), 32)
        self.assertEqual(results[:32], previous)
        self.assertEqual(len(results), 64)
        checks = results[32:]
        self.assertEqual(sum(result['verified'] for result in checks), 16)
        self.assertEqual({result['name'] for result in checks if not result['verified']}, {
            'prototype-chain-mismatch', 'prototype-self-mismatch',
            'prototype-conversion-order-mismatch', 'prototype-object-receiver-mismatch',
            'prototype-no-getters-mismatch', 'prototype-boxed-identity-mismatch',
            'prototype-intrinsics-mismatch', 'prototype-property-metadata-mismatch',
        })
        self.assertEqual({result['result']['mode'] for result in checks}, {'sloppy', 'strict'})
        with patch.object(runner, 'bounded_process', return_value=(
                0, response('exception', 'runtime', 'TypeError'), b'')):
            wrong_error = runner.harness_preflight(files, Path('/fake'), 1, 'is-prototype-of')
        self.assertFalse(any(result['verified'] for result in wrong_error[32:]))

    def test_is_prototype_of_import_checks_every_blob_and_rejects_inventory_changes(self):
        _, files, _, _, _ = runner.load_corpus(
            runner.ROOT / 'tests/upstream/test262-is-prototype-of', 'is-prototype-of')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        inventory = [dict(type='file', name=Path(path).name,
                          sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                     for path, data in files.items() if path.startswith('test/')]
        listing_url = (f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
                       f'test/built-ins/Object/prototype/isPrototypeOf?ref={importer.REVISION}')
        def fetch(url):
            if url == listing_url:
                return json.dumps(inventory).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'is-prototype-of')
            _, imported, cases, _, _ = runner.load_corpus(output, 'is-prototype-of')
            self.assertEqual(imported, files)
            self.assertEqual(len(cases), 20)
        inventory[0]['sha'] = '0' * 40
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), 'is-prototype-of')
            self.assertFalse(any(Path(temporary).iterdir()))
        inventory.pop()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(Path(temporary), 'is-prototype-of')
            self.assertFalse(any(Path(temporary).iterdir()))

    def test_rest_inventory_retains_every_source_and_required_mode(self):
        directory = runner.ROOT / 'tests/upstream/test262-rest-parameters'
        manifest, files, cases, fixtures, _ = runner.load_corpus(directory, 'rest-parameters')
        self.assertEqual(manifest['test_files'], 11)
        self.assertEqual(len(cases), 22)
        self.assertEqual(fixtures, [])
        self.assertEqual(sum(case['mode'] == 'sloppy' for case in cases), 11)
        self.assertEqual(sum(case['mode'] == 'strict' for case in cases), 11)
        negative = [case for case in cases if case['metadata']['negative']]
        self.assertEqual(len(negative), 2)
        self.assertEqual({case['file'].rsplit('/', 1)[-1] for case in negative},
                         {'position-invalid.js'})
        self.assertTrue(all(case['metadata']['negative'] == dict(phase='parse', type='SyntaxError')
                            for case in negative))
        # Old metadata lacks feature labels: preserve these files for the adapter
        # to classify, rather than filtering syntax outside the supported slice.
        self.assertTrue(all(not case['metadata']['features'] for case in cases))
        names = {Path(case['file']).name for case in cases}
        self.assertTrue({'array-pattern.js', 'object-pattern.js', 'with-new-target.js'} <= names)
        self.assertIn(b'$DONOTEVALUATE();', files['test/language/rest-parameters/position-invalid.js'])
        self.assertTrue({'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
                         'harness/propertyHelper.js'} <= files.keys())
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(directory, 'functions')

    def test_rest_policy_is_separate_and_only_adds_rest_parameters(self):
        self.assertEqual(runner.REST_PARAMETER_FEATURES,
                         runner.FUNCTION_FEATURES | {'rest-parameters'})
        self.assertNotIn('rest-parameters', runner.FUNCTION_FEATURES)
        rest = sample(b'/*---\nfeatures: [rest-parameters]\n---*/\nfunction f(...args){}')
        self.assertIsNone(runner.unsupported_reason(rest, runner.REST_PARAMETER_FEATURES))
        for profile, policy in runner.PROFILE_FEATURES.items():
            if profile != 'rest-parameters':
                self.assertIn('rest-parameters', runner.unsupported_reason(rest, policy))
        for feature in ('destructuring-binding', 'new.target', 'eval', 'async-functions', 'generators'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.REST_PARAMETER_FEATURES))

    def test_rest_preflight_keeps_previous_checks_and_rejects_disabled_assertions(self):
        directory = runner.ROOT / 'tests/upstream/test262-rest-parameters'
        _, files, _, _, _ = runner.load_corpus(directory, 'rest-parameters')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            previous = runner.harness_preflight(files, Path('/fake'), 1, 'functions')
            results = runner.harness_preflight(files, Path('/fake'), 1, 'rest-parameters')
        self.assertEqual(len(previous), 48)
        self.assertEqual(results[:48], previous)
        self.assertEqual(len(results), 64)
        rest = results[48:]
        self.assertEqual(sum(result['verified'] for result in rest), 8)
        self.assertEqual({result['name'] for result in rest if not result['verified']},
                         {'rest-array-mismatch', 'rest-index-mismatch',
                          'rest-unmapped-mismatch', 'rest-length-mismatch'})
        self.assertEqual({result['result']['mode'] for result in rest}, {'sloppy', 'strict'})

    def test_rest_import_verifies_pinned_blobs_without_filtering_unimplemented_files(self):
        directory = runner.ROOT / 'tests/upstream/test262-rest-parameters'
        _, files, _, _, _ = runner.load_corpus(directory, 'rest-parameters')
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        inventory = [dict(type='file', name=Path(path).name,
                          sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                     for path, data in files.items() if path.startswith('test/')]
        listing_url = (f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
                       f'test/language/rest-parameters?ref={importer.REVISION}')
        def fetch(url):
            if url == listing_url:
                return json.dumps(inventory).encode()
            self.assertTrue(url.startswith(raw))
            return files[url[len(raw):]]
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'rest-parameters')
            _, imported, cases, _, _ = runner.load_corpus(output, 'rest-parameters')
            self.assertEqual(imported, files)
            self.assertEqual(len(cases), 22)
        inventory[0]['sha'] = '0' * 40
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch):
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), 'rest-parameters')
            self.assertFalse(any(Path(temporary).iterdir()))

    def test_function_inventory_keeps_all_directories_modes_and_negative_tests(self):
        directory = runner.ROOT / 'tests/upstream/test262-functions'
        manifest, files, cases, fixtures, _ = runner.load_corpus(directory, 'functions')
        self.assertEqual(manifest['test_files'], 663)
        self.assertEqual(len(cases), 1131)
        self.assertEqual(fixtures, [])
        self.assertEqual(sum(case['mode'] == 'sloppy' for case in cases), 616)
        self.assertEqual(sum(case['mode'] == 'strict' for case in cases), 515)
        negatives = [case for case in cases if case['metadata']['negative']]
        self.assertEqual(len(negatives), 313)
        self.assertEqual(len({case['file'] for case in negatives}), 179)
        self.assertTrue(all(case['metadata']['negative'] == dict(phase='parse', type='SyntaxError')
                            for case in negatives))
        indented = files['test/language/statements/function/13.2-30-s.js']
        self.assertIn(b'/*---\n description: >', indented)
        self.assertEqual(importer.parse_metadata(indented.decode())['flags'], [])
        self.assertTrue({'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js',
                         'harness/compareArray.js'} <= files.keys())
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(directory, 'template-literal')

    def test_function_execution_policy_keeps_unimplemented_features_explicit(self):
        case = sample(b'/*---\nfeatures: [default-parameters]\n---*/\nfunction f(a=1) {}')
        self.assertIsNone(runner.unsupported_reason(case, runner.FUNCTION_FEATURES))
        for policy in (runner.SUPPORTED_FEATURES, runner.REGEXP_FEATURES, runner.TEMPLATE_FEATURES):
            self.assertIsNotNone(runner.unsupported_reason(case, policy))
        for feature in ('rest-parameters', 'async-functions', 'generators', 'new.target', 'Symbol'):
            unavailable = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(unavailable, runner.FUNCTION_FEATURES))

    def test_function_preflight_rejects_disabled_default_parameter_assertions(self):
        directory = runner.ROOT / 'tests/upstream/test262-functions'
        _, files, _, _, _ = runner.load_corpus(directory, 'functions')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            results = runner.harness_preflight(files, Path('/fake'), 1, 'functions')
        self.assertEqual(len(results), 48)
        rejected = {result['name'] for result in results if not result['verified']}
        self.assertTrue({'default-supplied-mismatch', 'default-tdz-wrong-type',
                         'default-arguments-mismatch', 'default-scope-mismatch'} <= rejected)

    def test_language_profile_retains_all_sources_modes_and_parse_negatives(self):
        directory = runner.ROOT / 'tests/upstream/test262-template-literal'
        manifest, files, cases, fixtures, _ = runner.load_corpus(directory, 'template-literal')
        self.assertEqual(manifest['test_files'], 57)
        self.assertEqual(len(cases), 114)
        self.assertEqual(fixtures, [])
        self.assertEqual({case['mode'] for case in cases}, {'sloppy', 'strict'})
        prefix = 'test/language/expressions/template-literal/'
        self.assertTrue(all(case['file'].startswith(prefix) for case in cases))
        negative = [case for case in cases if case['metadata']['negative']]
        self.assertEqual(len(negative), 32)
        self.assertTrue(all(case['metadata']['negative'] == dict(phase='parse', type='SyntaxError')
                            for case in negative))
        self.assertTrue({'harness/assert.js', 'harness/sta.js', 'harness/propertyHelper.js'} <= files.keys())
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(directory, 'regexp')
        # Byte preservation matters for the CR/CRLF template cooking tests.
        line_source = files[prefix + 'tv-line-terminator-sequence.js']
        self.assertIn(b'\r\n', line_source)
        self.assertIn(b'\r', line_source.replace(b'\r\n', b''))

    def test_profile_features_do_not_silently_expand_other_baselines(self):
        case = sample(b'/*---\nfeatures: [u180e]\n---*/\nassert.sameValue(`\xe1\xa0\x8e`, "\\u180e");')
        self.assertIsNone(runner.unsupported_reason(case, runner.TEMPLATE_FEATURES))
        self.assertIsNotNone(runner.unsupported_reason(case, runner.SUPPORTED_FEATURES))
        self.assertIsNotNone(runner.unsupported_reason(case, runner.REGEXP_FEATURES))
        self.assertEqual(importer.corpus_name('string-json'), 'test262')
        self.assertEqual(importer.corpus_name('regexp'), 'test262-regexp')
        self.assertEqual(importer.corpus_name('template-literal'), 'test262-template-literal')

    def test_template_preflight_rejects_disabled_assertions(self):
        directory = runner.ROOT / 'tests/upstream/test262-template-literal'
        _, files, _, _, _ = runner.load_corpus(directory, 'template-literal')
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            results = runner.harness_preflight(files, Path('/fake'), 1, 'template-literal')
        self.assertEqual(len(results), 44)
        rejected = {result['name'] for result in results if not result['verified']}
        self.assertTrue({'template-cooked-mismatch', 'template-order-mismatch',
                         'template-tag-unsupported'} <= rejected)

    def test_regexp_profile_is_complete_and_cannot_replace_original_inventory(self):
        directory = runner.ROOT / 'tests/upstream/test262-regexp'
        manifest, _, cases, fixtures, _ = runner.load_corpus(directory, 'regexp')
        self.assertEqual(manifest['test_files'], 145)
        self.assertEqual(len(cases), 290)
        self.assertEqual(fixtures, [])
        self.assertEqual({case['mode'] for case in cases}, {'sloppy', 'strict'})
        self.assertTrue(all(case['file'].startswith('test/built-ins/RegExp/prototype/') for case in cases))
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(directory)

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
