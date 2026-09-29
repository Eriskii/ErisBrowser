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


class StringLastIndexOfCorpusTests(unittest.TestCase):
    def corpus(self):
        return runner.load_corpus(runner.ROOT / 'tests/upstream/test262-string-last-index-of', 'string-last-index-of')

    def test_complete_directory_has_fixed_bytes_modes_and_no_exclusions(self):
        manifest, files, cases, fixtures, digest = self.corpus()
        self.assertEqual(importer.STRING_LAST_INDEX_OF_DIRECTORIES,
                         {'String/prototype/lastIndexOf': 25})
        self.assertEqual(manifest['test_files'], 25)
        self.assertEqual(len(cases), 50)
        self.assertEqual(len(files), 32)
        self.assertEqual(fixtures, [])
        self.assertEqual(digest, '2ed8995ab41234ccc82eaf827a5c406f7fca62d2f530fc36fbc0b8f1e73224bb')
        self.assertEqual(runner.STRING_LAST_INDEX_OF_FEATURES,
                         runner.CONSTRUCTION_FEATURES | runner.REGEXP_FEATURES)
        self.assertEqual(sum(bool(runner.unsupported_reason(c, runner.STRING_LAST_INDEX_OF_FEATURES)) for c in cases), 0)
        self.assertFalse(any(c['metadata']['negative'] for c in cases))
        excluded = [c for c in cases if runner.unsupported_reason(c, runner.STRING_LAST_INDEX_OF_FEATURES)]
        self.assertEqual({f for c in excluded for f in c['metadata']['features']}
                         - runner.STRING_LAST_INDEX_OF_FEATURES,
                         set())
        for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js', 'isConstructor.js']:
            self.assertEqual(files['harness/' + name],
                             (runner.ROOT / 'tests/upstream/test262-reflect-construction/harness' / name).read_bytes())

    def test_noop_assertions_and_wrong_errors_cannot_verify_conversion_controls(self):
        _, files, _, _, _ = self.corpus()
        with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'string-last-index-of')
        dynamic = [c for c in checks if c['name'].startswith('string-last-index-of-')]
        self.assertEqual(len(checks), 80)
        self.assertEqual(len(dynamic), 48)
        self.assertEqual(sum(c['verified'] for c in dynamic), 24)
        self.assertTrue(all(c['name'].endswith('-mismatch') for c in dynamic if not c['verified']))
        with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
            checks = runner.harness_preflight(files, Path('/fake'), 1, 'string-last-index-of')
        self.assertFalse(any(c['verified'] for c in checks if c['name'].startswith('string-last-index-of-')))

    def test_import_checks_direct_inventory_and_git_blob_identity(self):
        _, files, _, _, _ = self.corpus()
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
        listings = {}
        for name in importer.STRING_LAST_INDEX_OF_DIRECTORIES:
            prefix = 'test/built-ins/' + name
            listings[prefix] = [dict(type='file', name=Path(p).name,
                sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                for p, data in files.items() if str(Path(p).parent) == prefix]
        def fetch(url):
            return files[url[len(raw):]] if url.startswith(raw) else json.dumps(listings[url[len(api):].split('?')[0]]).encode()
        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            output = Path(temporary)
            importer.import_corpus(output, 'string-last-index-of')
            self.assertEqual((output / 'manifest.json').read_bytes(),
                             (runner.ROOT / 'tests/upstream/test262-string-last-index-of/manifest.json').read_bytes())
            original = copy.deepcopy(listings)
            listings['test/built-ins/String/prototype/lastIndexOf'].pop()
            with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                importer.import_corpus(output, 'string-last-index-of')
            listings = original
            listings['test/built-ins/String/prototype/lastIndexOf'][0]['sha'] = '0' * 40
            with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(output, 'string-last-index-of')
