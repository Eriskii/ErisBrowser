import contextlib
import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner
from test_test262_conformance import response


class FunctionConstructorCorpusTests(unittest.TestCase):
    def corpus(self):
        return runner.load_corpus(runner.ROOT / 'tests/upstream/test262-function-constructor', 'function-constructor')

    def test_complete_four_directories_have_fixed_bytes_modes_and_exclusions(self):
        manifest, files, cases, fixtures, digest = self.corpus()
        self.assertEqual(importer.FUNCTION_CONSTRUCTOR_DIRECTORIES,
                         {'Function': 179, 'Function/length': 13,
                          'Function/internals/Construct': 6, 'Function/internals/Call': 2})
        self.assertEqual(manifest['test_files'], 200)
        self.assertEqual(len(cases), 291)
        self.assertEqual(len(files), 207)
        self.assertEqual(fixtures, [])
        self.assertEqual(digest, '9630ad1dc81216d3717a0df435e53b15cf2f2334210984a1965c7d40277aa2a7')
        self.assertEqual(runner.FUNCTION_CONSTRUCTOR_FEATURES,
                         runner.CONSTRUCTION_FEATURES | runner.REST_PARAMETER_FEATURES)
        self.assertEqual(sum(bool(runner.unsupported_reason(c, runner.FUNCTION_CONSTRUCTOR_FEATURES)) for c in cases), 47)
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js']:
            self.assertEqual(files['harness/' + name],
                             (runner.ROOT / 'tests/upstream/test262-reflect-construction/harness' / name).read_bytes())

    def test_noop_assertions_and_wrong_errors_cannot_verify_dynamic_controls(self):
        _, files, _, _, _ = self.corpus()
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'function-constructor')
        dynamic = [c for c in checks if c['name'].startswith('dynamic-')]
        self.assertEqual(len(checks), 64)
        self.assertEqual(len(dynamic), 32)
        self.assertEqual(sum(c['verified'] for c in dynamic), 16)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in dynamic if not c['verified']))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'function-constructor')
        self.assertFalse(any(c['verified'] for c in checks if c['name'].startswith('dynamic-')))

    def test_import_checks_direct_inventory_and_git_blob_identity(self):
        _, files, _, _, _ = self.corpus()
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
        listings = {}
        for name in importer.FUNCTION_CONSTRUCTOR_DIRECTORIES:
            prefix = 'test/built-ins/' + name
            listings[prefix] = [dict(type='file', name=Path(p).name,
                sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                for p, data in files.items() if str(Path(p).parent) == prefix]
        def fetch(url):
            return files[url[len(raw):]] if url.startswith(raw) else json.dumps(listings[url[len(api):].split('?')[0]]).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'function-constructor')
            self.assertEqual((output / 'manifest.json').read_bytes(),
                             (runner.ROOT / 'tests/upstream/test262-function-constructor/manifest.json').read_bytes())
            original = copy.deepcopy(listings)
            listings['test/built-ins/Function'].pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'function-constructor')
            listings = original
            listings['test/built-ins/Function'][0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'function-constructor')
