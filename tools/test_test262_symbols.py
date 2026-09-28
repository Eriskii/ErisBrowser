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
from test_test262_conformance import response, sample


class SymbolCorpusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = runner.ROOT / 'tests/upstream/test262-symbols'
        cls.manifest, cls.files, cls.cases, cls.fixtures, _ = runner.load_corpus(cls.directory, 'symbols')

    def test_complete_symbol_tree_and_reflection_inventory_preserves_all_modes(self):
        self.assertEqual(len(self.manifest['directories']), 25)
        self.assertEqual(self.manifest['test_files'], 123)
        self.assertEqual(len(self.cases), 242)
        self.assertEqual(self.fixtures, [])
        self.assertEqual(sum(len(v) for k, v in self.manifest['directories'].items() if k.startswith('Symbol')), 98)
        self.assertEqual(len(self.manifest['directories']['Object/getOwnPropertySymbols']), 12)
        self.assertEqual(len(self.manifest['directories']['Reflect/ownKeys']), 13)
        self.assertEqual({p for p in self.files if p.startswith('harness/')}, {
            'harness/assert.js', 'harness/sta.js', 'harness/compareArray.js',
            'harness/propertyHelper.js', 'harness/isConstructor.js',
        })
        self.assertEqual(len({case['id'] for case in self.cases}), 242)
        self.assertEqual(sum(runner.unsupported_reason(c, runner.SYMBOL_FEATURES) is not None for c in self.cases), 66)
        with self.assertRaisesRegex(ValueError, 'inventory'):
            runner.load_corpus(self.directory, 'string-json')

    def test_symbol_policy_keeps_exotic_hosts_and_unimplemented_protocols_explicit(self):
        for feature in ('Proxy', 'cross-realm', 'Reflect.construct', 'explicit-resource-management', 'class', 'generators'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIn(feature, runner.unsupported_reason(case, runner.SYMBOL_FEATURES))
        for feature in ('Symbol', 'Symbol.toPrimitive', 'Symbol.toStringTag', 'Reflect'):
            case = sample(('/*---\nfeatures: [' + feature + ']\n---*/\n').encode())
            self.assertIsNone(runner.unsupported_reason(case, runner.SYMBOL_FEATURES))
            self.assertIsNotNone(runner.unsupported_reason(case, runner.SUPPORTED_FEATURES))

    def test_preflight_rejects_disabled_assertions_and_wrong_exception_identities(self):
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            core = runner.harness_preflight(self.files, Path('/fake'), 1)
            checks = runner.harness_preflight(self.files, Path('/fake'), 1, 'symbols')
        self.assertEqual(len(core), 32)
        self.assertEqual(checks[:32], core)
        self.assertEqual(len(checks), 64)
        self.assertEqual(sum(c['verified'] for c in checks[32:]), 16)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in checks[32:] if not c['verified']))
        self.assertEqual({c['result']['mode'] for c in checks[32:]}, {'sloppy', 'strict'})
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(self.files, Path('/fake'), 1, 'symbols')
        self.assertFalse(any(c['verified'] for c in checks[32:]))

    def test_import_checks_pinned_counts_and_blob_hashes_before_publication(self):
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        listings = {}
        for directory in importer.SYMBOL_DIRECTORIES:
            prefix = 'test/built-ins/' + directory + '/'
            listings[directory] = [dict(type='file', name=Path(path).name,
                sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                for path, data in self.files.items() if path.startswith(prefix) and '/' not in path[len(prefix):]]
        def fetch(url):
            if url.startswith(raw):
                return self.files[url[len(raw):]]
            directory = url.split('/contents/test/built-ins/', 1)[1].split('?')[0]
            return json.dumps(listings[directory]).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'symbols')
            self.assertEqual((output / 'manifest.json').read_bytes(), (self.directory / 'manifest.json').read_bytes())
            for path, data in self.files.items():
                self.assertEqual((output / path).read_bytes(), data)
            original = copy.deepcopy(listings)
            listings['Symbol'].pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'symbols')
            listings = original
            listings['Symbol'][0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'symbols')
