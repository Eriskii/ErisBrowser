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


class StringSearchCorpusTests(unittest.TestCase):
    def corpus(self):
        return runner.load_corpus(runner.ROOT / 'tests/upstream/test262-string-search', 'string-search')

    def test_complete_five_directories_have_fixed_bytes_modes_and_exclusions(self):
        manifest, files, cases, fixtures, digest = self.corpus()
        self.assertEqual(importer.STRING_SEARCH_DIRECTORIES,
                         {'String/prototype/includes': 27, 'String/prototype/indexOf': 47,
                          'String/prototype/startsWith': 21, 'String/prototype/endsWith': 27,
                          'String/prototype/split': 120})
        self.assertEqual(manifest['test_files'], 242)
        self.assertEqual(len(cases), 484)
        self.assertEqual(len(files), 249)
        self.assertEqual(fixtures, [])
        self.assertEqual(digest, '51f7812dbdccfac8179261013188f15a8ba1b965bf0bfb9ebb61e8aff742338b')
        self.assertEqual(runner.STRING_SEARCH_FEATURES,
                         runner.SYMBOL_FEATURES | runner.FUNCTION_FEATURES |
                         {'String.prototype.includes', 'String.prototype.endsWith'})
        self.assertEqual(sum(bool(runner.unsupported_reason(c, runner.STRING_SEARCH_FEATURES)) for c in cases), 4)
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        # This upstream file omits its BigInt prerequisite. Keep both modes in
        # execution so the parser failure remains visible instead of filtering it.
        unlabelled = [c for c in cases if c['file'].endswith('cstm-split-on-bigint-primitive.js')]
        self.assertEqual(len(unlabelled), 2)
        self.assertTrue(all(runner.unsupported_reason(c, runner.STRING_SEARCH_FEATURES) is None for c in unlabelled))
        for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js']:
            self.assertEqual(files['harness/' + name],
                             (runner.ROOT / 'tests/upstream/test262-reflect-construction/harness' / name).read_bytes())

    def test_noop_assertions_and_wrong_errors_cannot_verify_conversion_controls(self):
        _, files, _, _, _ = self.corpus()
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'string-search')
        dynamic = [c for c in checks if c['name'].startswith('string-')]
        self.assertEqual(len(checks), 72)
        self.assertEqual(len(dynamic), 40)
        self.assertEqual(sum(c['verified'] for c in dynamic), 20)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in dynamic if not c['verified']))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'string-search')
        self.assertFalse(any(c['verified'] for c in checks if c['name'].startswith('string-')))

    def test_import_checks_direct_inventory_and_git_blob_identity(self):
        _, files, _, _, _ = self.corpus()
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
        listings = {}
        for name in importer.STRING_SEARCH_DIRECTORIES:
            prefix = 'test/built-ins/' + name
            listings[prefix] = [dict(type='file', name=Path(p).name,
                sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                for p, data in files.items() if str(Path(p).parent) == prefix]
        def fetch(url):
            return files[url[len(raw):]] if url.startswith(raw) else json.dumps(listings[url[len(api):].split('?')[0]]).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'string-search')
            self.assertEqual((output / 'manifest.json').read_bytes(),
                             (runner.ROOT / 'tests/upstream/test262-string-search/manifest.json').read_bytes())
            original = copy.deepcopy(listings)
            listings['test/built-ins/String/prototype/includes'].pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'string-search')
            listings = original
            listings['test/built-ins/String/prototype/includes'][0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'string-search')
