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


class ConstructionCorpusTests(unittest.TestCase):
    def test_expanded_policy_admits_exactly_the_reviewed_constructor_inventory(self):
        additions = {'Reflect', 'Reflect.apply', 'Reflect.construct', 'new.target'}
        self.assertEqual(runner.CONSTRUCTOR_FEATURES, additions)
        expected = {'string-concat': 2, 'symbols': 14, 'string-json': 22,
                    'regexp': 10, 'functions': 4, 'is-prototype-of': 4,
                    'array-sort': 2, 'array-reduce': 4, 'number-statics': 10,
                    'numeric-conversion': 4, 'numeric-parsing': 4, 'uri': 8}
        selected = {}
        for profile, features in runner.PROFILE_FEATURES.items():
            self.assertTrue(additions <= features, profile)
            if profile in {'for-of', 'core-iterators', 'date', 'array-find', 'object-integrity', 'array-predicates', 'array-descriptors', 'array-last-index-of', 'string-last-index-of', 'regexp-match-search', 'regexp-constructor', 'regexp-split', 'string-search', 'function-constructor', 'reflect-construction', 'new-target'}:
                continue  # Later selections are outside the historical policy delta.
            prior = features - additions
            if profile == 'symbols':
                prior.add('Reflect')
            _, _, cases, _, _ = runner.load_corpus(
                runner.ROOT / 'tests/upstream' / runner.corpus_name(profile), profile)
            newly_executed = [c for c in cases if runner.unsupported_reason(c, prior)
                              and not runner.unsupported_reason(c, features)]
            self.assertEqual(len(newly_executed), expected.get(profile, 0), profile)
            self.assertTrue(all(set(c['metadata']['features']) & additions for c in newly_executed))
            selected[profile] = len(newly_executed)
        self.assertEqual(sum(selected.values()), 88)

    def test_complete_selections_preserve_metadata_and_negative_source(self):
        for profile, sources, modes in [('reflect-construction', 19, 38), ('new-target', 14, 28)]:
            directory = runner.ROOT / 'tests/upstream' / ('test262-' + profile)
            manifest, files, cases, fixtures, digest = runner.load_corpus(directory, profile)
            self.assertEqual(manifest['test_files'], sources)
            self.assertEqual(len(cases), modes)
            self.assertEqual(fixtures, [])
            self.assertEqual(digest, {'reflect-construction': 'a0efcbf267933681e09b4f9a75d4e43f75c0911ae1a93770571bc54f67a665b6', 'new-target': '73a1e0d8478b0dfbb7d98d0ca00d19a634033f708cca6e6c93ddcfbf16d826f7'}[profile])
            self.assertEqual(sum(len(v) for v in manifest['directories'].values()), sources)
            excluded = [c for c in cases if runner.unsupported_reason(c, runner.CONSTRUCTION_FEATURES)]
            self.assertEqual(len(excluded), 0 if profile == 'reflect-construction' else 6)
            self.assertEqual({c['mode'] for c in excluded}, {'strict', 'sloppy'} if excluded else set())
            self.assertIn('Reflect.construct', runner.CONSTRUCTION_FEATURES)
            self.assertIn('new.target', runner.CONSTRUCTION_FEATURES)
            for name in ['assert.js', 'sta.js', 'propertyHelper.js', 'compareArray.js']:
                self.assertEqual(files['harness/' + name], (runner.ROOT / 'tests/upstream/test262-symbols/harness' / name).read_bytes())
            if profile == 'new-target':
                negatives = [c for c in cases if c['metadata']['negative']]
                self.assertEqual(len(negatives), 4)
                self.assertEqual({c['metadata']['negative']['phase'] for c in negatives}, {'parse'})

    def test_disabled_assertions_or_wrong_exception_identity_fail_preflight(self):
        for profile in ['reflect-construction', 'new-target']:
            _, files, _, _, _ = runner.load_corpus(runner.ROOT / 'tests/upstream' / ('test262-' + profile), profile)
            with patch.object(runner, 'bounded_process', return_value=(0, response('complete'), b'')):
                checks = runner.harness_preflight(files, Path('/fake'), 1, profile)
            self.assertEqual(len(checks), 96)
            self.assertEqual(sum(c['verified'] for c in checks[32:]), 32)
            self.assertTrue(all(c['name'].endswith('-mismatch') for c in checks[32:] if not c['verified']))
            with patch.object(runner, 'bounded_process', return_value=(0, response('exception', 'runtime', 'TypeError'), b'')):
                checks = runner.harness_preflight(files, Path('/fake'), 1, profile)
            self.assertFalse(any(c['verified'] for c in checks[32:]))

    def test_import_requires_complete_inventory_and_unchanged_git_blobs(self):
        for profile in ['reflect-construction', 'new-target']:
            directory = runner.ROOT / 'tests/upstream' / ('test262-' + profile)
            _, files, _, _, _ = runner.load_corpus(directory, profile)
            raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
            api = f'https://api.github.com/repos/{importer.REPOSITORY}/contents/'
            listings = {}
            for name in importer.PROFILES[profile]:
                prefix = importer.PROFILE_ROOTS[profile] + '/' + name
                listings[prefix] = [dict(type='file', name=Path(p).name,
                    sha=importer.hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest())
                    for p, data in files.items() if p.startswith(prefix + '/')]
            def fetch(url):
                return files[url[len(raw):]] if url.startswith(raw) else json.dumps(listings[url[len(api):].split('?')[0]]).encode()
            with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
                output = Path(temporary)
                importer.import_corpus(output, profile)
                self.assertEqual((output / 'manifest.json').read_bytes(), (directory / 'manifest.json').read_bytes())
                original = copy.deepcopy(listings)
                first = next(iter(listings))
                listings[first].pop()
                with self.assertRaisesRegex(ValueError, 'inventory mismatch'):
                    importer.import_corpus(output, profile)
                listings = original
                listings[first][0]['sha'] = '0' * 40
                with self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                    importer.import_corpus(output, profile)
