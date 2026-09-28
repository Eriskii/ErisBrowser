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


class StringConcatCorpusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = runner.ROOT / 'tests/upstream/test262-string-concat'
        cls.manifest, cls.files, cls.cases, cls.fixtures, cls.digest = runner.load_corpus(
            cls.directory, 'string-concat')

    def test_complete_inventory_keeps_unsupported_constructor_modes_and_exact_helpers(self):
        self.assertEqual(self.manifest['test_files'], 22)
        self.assertEqual(len(self.cases), 44)
        self.assertEqual(self.fixtures, [])
        self.assertEqual(self.digest, 'e904181959ebb9592f63f75802b7a6f84a40e108379257e6cb8bb29354d5aa60')
        self.assertEqual(runner.STRING_CONCAT_FEATURES, runner.SUPPORTED_FEATURES)
        excluded = [c for c in self.cases if runner.unsupported_reason(c, runner.STRING_CONCAT_FEATURES)]
        self.assertEqual(len(excluded), 2)
        self.assertEqual({c['mode'] for c in excluded}, {'sloppy', 'strict'})
        self.assertTrue(all(c['file'].endswith('/not-a-constructor.js') for c in excluded))
        self.assertTrue(all('Reflect.construct' in runner.unsupported_reason(c, runner.STRING_CONCAT_FEATURES) for c in excluded))
        symbol_dir = runner.ROOT / 'tests/upstream/test262-symbols'
        for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js']:
            self.assertEqual(self.files['harness/' + name], (symbol_dir / 'harness' / name).read_bytes())
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(self.directory, 'string-json')

    def test_preflight_rejects_disabled_assertions_and_wrong_exception_identity(self):
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(self.files, Path('/fake'), 1)
            checks = runner.harness_preflight(self.files, Path('/fake'), 1, 'string-concat')
        self.assertEqual(checks[:32], core)
        self.assertEqual(len(checks), 56)
        self.assertEqual(sum(c['verified'] for c in checks[32:]), 12)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in checks[32:] if not c['verified']))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(self.files, Path('/fake'), 1, 'string-concat')
        self.assertFalse(any(c['verified'] for c in checks[32:]))

    def test_import_refuses_incomplete_inventory_and_modified_blobs(self):
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        prefix = 'test/built-ins/String/prototype/concat/'
        listing = [dict(type='file', name=Path(p).name,
                        sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                   for p, data in self.files.items() if p.startswith(prefix)]
        def fetch(url):
            if url.startswith(raw):
                return self.files[url[len(raw):]]
            return json.dumps(listing).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'string-concat')
            self.assertEqual((output / 'manifest.json').read_bytes(), (self.directory / 'manifest.json').read_bytes())
            original = copy.deepcopy(listing)
            listing.pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'string-concat')
            listing = original
            listing[0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'string-concat')
