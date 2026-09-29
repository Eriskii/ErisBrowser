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


class RegExpSplitCorpusTests(unittest.TestCase):
    def corpus(self):
        return runner.load_corpus(runner.ROOT / 'tests/upstream/test262-regexp-split', 'regexp-split')

    def test_complete_two_directories_have_fixed_bytes_modes_and_exclusions(self):
        manifest, files, cases, fixtures, digest = self.corpus()
        self.assertEqual(importer.REGEXP_SPLIT_DIRECTORIES,
                         {'RegExp/prototype/Symbol.split': 44, 'RegExp/Symbol.species': 4})
        self.assertEqual(manifest['test_files'], 48)
        self.assertEqual(len(cases), 96)
        self.assertEqual(len(files), 55)
        self.assertEqual(fixtures, [])
        self.assertEqual(digest, '7e380ee0ee12fed435b33bebb97ce784c024e21faee72809e8cda2d97f1e5028')
        self.assertEqual(runner.REGEXP_SPLIT_FEATURES,
                         runner.CONSTRUCTION_FEATURES)
        self.assertEqual(sum(bool(runner.unsupported_reason(c, runner.REGEXP_SPLIT_FEATURES)) for c in cases), 2)
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        # Unicode regexp syntax is undeclared in these upstream tests. Keep
        # their four variants in execution and retain explicit unsupported results.
        unicode_cases = [c for c in cases if '/u-lastindex-' in c['file']]
        self.assertEqual(len(unicode_cases), 4)
        self.assertTrue(all(runner.unsupported_reason(c, runner.REGEXP_SPLIT_FEATURES) is None for c in unicode_cases))
        for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js']:
            self.assertEqual(files['harness/' + name],
                             (runner.ROOT / 'tests/upstream/test262-reflect-construction/harness' / name).read_bytes())

    def test_noop_assertions_and_wrong_errors_cannot_verify_conversion_controls(self):
        _, files, _, _, _ = self.corpus()
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'regexp-split')
        dynamic = [c for c in checks if c['name'].startswith('regexp-split-')]
        self.assertEqual(len(checks), 72)
        self.assertEqual(len(dynamic), 40)
        self.assertEqual(sum(c['verified'] for c in dynamic), 20)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in dynamic if not c['verified']))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'regexp-split')
        self.assertFalse(any(c['verified'] for c in checks if c['name'].startswith('regexp-split-')))

    def test_import_checks_direct_inventory_and_git_blob_identity(self):
        _, files, _, _, _ = self.corpus()
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
        listings = {}
        for name in importer.REGEXP_SPLIT_DIRECTORIES:
            prefix = 'test/built-ins/' + name
            listings[prefix] = [dict(type='file', name=Path(p).name,
                sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                for p, data in files.items() if str(Path(p).parent) == prefix]
        def fetch(url):
            return files[url[len(raw):]] if url.startswith(raw) else json.dumps(listings[url[len(api):].split('?')[0]]).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'regexp-split')
            self.assertEqual((output / 'manifest.json').read_bytes(),
                             (runner.ROOT / 'tests/upstream/test262-regexp-split/manifest.json').read_bytes())
            original = copy.deepcopy(listings)
            listings['test/built-ins/RegExp/prototype/Symbol.split'].pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'regexp-split')
            listings = original
            listings['test/built-ins/RegExp/prototype/Symbol.split'][0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'regexp-split')
