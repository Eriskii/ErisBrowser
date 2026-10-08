"""TypedArray corpus authentication and rejection of false control health.

Root runs these after the offline import. All adapters and network calls are
mocked; no test may fall back to a real JavaScript engine or network request.
"""
import contextlib
import copy
import io
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner

PROFILE = 'typedarray-foundation'
CORPUS = runner.ROOT / 'tests/upstream/test262-typedarray-foundation'
CONTROLS = Path(__file__).with_name('typedarray-foundation-controls.json')
ERROR = dict(status='exception', phase='runtime', error_type='Error', error_identity='Error')
PAIRS = ['literal-units', 'ten-number-conversions', 'float16-direct-tie',
         'iterable-before-conversion', 'resized-write-live-bounds',
         'canonical-distinct-receiver', 'metadata-and-index-flags',
         'iterator-tracking-and-sticky']


def proof_reader(altered_route=None, mutate=None):
    def read(route):
        data = (CORPUS / importer.tree_proof_path(route)).read_bytes()
        if route == altered_route:
            value = json.loads(data)
            mutate(value)
            return json.dumps(value).encode()
        return data
    return read


def capture_controls(fake=None):
    _, files, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
    captured = []

    def execute(case, *args):
        captured.append(case)
        result = fake(case) if fake else (
            {'status': 'failed', 'observation': ERROR.copy()}
            if case['id'].endswith('-wrong') else {'status': 'passed'})
        return dict(mode=case['mode'], case_sha256=case['case_sha256'],
                    source_sha256=runner.digest(case['source']), **result)

    with patch.object(runner, 'run_case', side_effect=execute):
        outcomes = runner.harness_preflight(files, Path('/capture-only-no-adapter'), 3, PROFILE)
    return captured, outcomes


class TypedArrayFoundationTests(unittest.TestCase):
    def setUp(self):
        adapter = patch.object(runner, 'bounded_process',
                               side_effect=AssertionError('adapter execution forbidden'))
        network = patch.object(importer, 'fetch',
                               side_effect=AssertionError('network access forbidden'))
        adapter.start()
        network.start()
        self.addCleanup(adapter.stop)
        self.addCleanup(network.stop)

    def test_complete_selection_retains_bigint_and_explicit_policy_exclusions(self):
        manifest, files, cases, fixtures, _ = runner.load_corpus(CORPUS, PROFILE)
        self.assertEqual((manifest['test_files'], len(cases), len(files), fixtures),
                         (632, 1238, 644, []))
        self.assertEqual([sum(c['mode'] == m for c in cases) for m in ('sloppy', 'strict')],
                         [620, 618])
        self.assertEqual(sum(len(v) for k, v in files.items() if k.startswith('test/')), 773126)
        self.assertEqual(len(manifest['directories']), 67)
        self.assertEqual({p[8:] for p in files if p.startswith('harness/')},
                         importer.TYPEDARRAY_HELPERS)
        proof = manifest['inventory_proof']
        self.assertEqual((proof['complete_directories'], proof['complete_sources'],
                          proof['selected_sources'], len(proof['files'])), (160, 2191, 632, 7))
        self.assertEqual(proof['selection_sha256'],
                         'cdc70bbb65c93629ee519af8344bf46552e41b1892677827e385535a49a637e5')
        bigint = {c['file'] for c in cases if 'BigInt' in c['metadata']['features']}
        self.assertEqual(len(bigint), 159)
        reasons = [runner.unsupported_reason(c, runner.PROFILE_FEATURES[PROFILE]) for c in cases]
        self.assertEqual((reasons.count(None), sum(r is not None for r in reasons)), (766, 472))
        self.assertTrue(all(reasons[i] is not None for i, c in enumerate(cases)
                            if c['file'] in bigint))
        self.assertEqual(runner.TYPEDARRAY_FEATURES,
                         runner.DATA_VIEW_FEATURES | {'TypedArray', 'Float16Array'})
        self.assertNotIn('TypedArray', runner.SUPPORTED_FEATURES)
        self.assertEqual(len(set(runner.PROFILES) - {'typedarray-search', 'typedarray-views'}), 47)

    def test_recursive_authentication_rejects_changes_outside_selected_directories(self):
        root = importer.TYPEDARRAY_ROOTS['TypedArrayConstructors']
        route = f'trees/{root}?recursive=1'

        def change_unselected(doc):
            # Dedicated BigInt constructor is outside the 67 imported dirs, but
            # remains authenticated by the full original recursive root.
            row = next(e for e in doc['tree'] if e['type'] == 'blob'
                       and e['path'].startswith('BigInt64Array/'))
            row['sha'] = '0' * 40

        mutations = {
            'unselected-blob': change_unselected,
            'truncated': lambda d: d.update(truncated=True),
            'omitted': lambda d: d['tree'].pop(),
            'duplicate': lambda d: d['tree'].append(copy.deepcopy(d['tree'][0])),
            'parent-path': lambda d: d['tree'][0].update(path='../outside.js'),
            'deep-path': lambda d: d['tree'][0].update(path='/'.join(['a'] * 9)),
            'entry-limit': lambda d: d.update(tree=[d['tree'][0]] * 4097),
            'boolean-size': lambda d: next(e for e in d['tree'] if e['type'] == 'blob').update(size=True),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, mutate))
        for altered in ({}, {'TypedArray': 9}, dict(importer.TYPEDARRAY_DIRECTORIES, TypedArray=8)):
            with self.subTest(selection=altered), patch.dict(importer.PROFILES, {PROFILE: altered}), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_root_ancestor_helper_proofs_and_reader_limits_remain_enforced(self):
        manifest, _, _, _, _ = runner.load_corpus(CORPUS, PROFILE)
        for entry in manifest['inventory_proof']['files']:
            route = entry['url'].split('/git/', 1)[1]
            with self.subTest(route=route), self.assertRaises(ValueError):
                importer.profile_tree_inventory(PROFILE, proof_reader(route, lambda d: d.update(sha='0' * 40)))
        with patch.object(importer, 'MAX_FILE', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())
        with patch.object(importer, 'MAX_TOTAL', 100), self.assertRaises(ValueError):
            importer.profile_tree_inventory(PROFILE, proof_reader())

    def test_offline_import_preserves_all_bytes_and_rejects_corrupt_blobs(self):
        manifest, files, cases, _, _ = runner.load_corpus(CORPUS, PROFILE)
        raw = f'https://raw.githubusercontent.com/{importer.REPOSITORY}/{importer.REVISION}/'
        api = f'https://api.github.com/repos/{importer.REPOSITORY}/git/'

        def fetch(url):
            if url.startswith(api):
                return proof_reader()(url[len(api):])
            if url.startswith(raw):
                return files[url[len(raw):]]
            raise AssertionError('unexpected import URL')

        with tempfile.TemporaryDirectory() as temporary, patch.object(importer, 'fetch', side_effect=fetch), contextlib.redirect_stdout(io.StringIO()):
            importer.import_corpus(Path(temporary), PROFILE)
            got, retained, actual, _, _ = runner.load_corpus(Path(temporary), PROFILE)
            self.assertEqual((got, retained), (manifest, files))
            self.assertEqual([(c['id'], c['case_sha256']) for c in actual],
                             [(c['id'], c['case_sha256']) for c in cases])
        for suffix in ('test/built-ins/TypedArray/length.js', 'harness/testTypedArray.js', 'LICENSE'):
            def corrupt(url):
                data = fetch(url)
                return data + b'\n' if url.endswith(suffix) else data
            with self.subTest(path=suffix), tempfile.TemporaryDirectory() as temporary, patch.object(
                    importer, 'fetch', side_effect=corrupt), self.assertRaisesRegex(ValueError, 'pinned Git blob'):
                importer.import_corpus(Path(temporary), PROFILE)

    def test_rehashed_manifest_cannot_bless_source_helper_or_extra_script(self):
        for name in ('test/built-ins/TypedArray/length.js', 'harness/testTypedArray.js', 'LICENSE', None):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                dest = Path(temporary) / 'corpus'
                shutil.copytree(CORPUS, dest)
                if name is None:
                    (dest / 'harness/unlisted.js').write_text('// extra\n')
                else:
                    target = dest / name
                    data = target.read_bytes() + b'\n'
                    target.write_bytes(data)
                    manifest = json.loads((dest / 'manifest.json').read_bytes())
                    next(e for e in manifest['files'] if e['path'] == name).update(
                        bytes=len(data), sha256=runner.digest(data))
                    (dest / 'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):
                    runner.load_corpus(dest, PROFILE)

    def test_exact_control_sources_modes_and_same_mode_partner_bindings(self):
        controls = json.loads(CONTROLS.read_bytes())['controls']
        captured, outcomes = capture_controls()
        self.assertEqual((len(captured), len(outcomes)), (64, 64))
        expected = [('typedarray-foundation-' + pair + '-' + variant, mode)
                    for pair in PAIRS for variant in ('positive', 'wrong')
                    for mode in ('sloppy', 'strict')]
        self.assertEqual([(o['name'], c['mode']) for c, o in zip(captured[32:], outcomes[32:])], expected)
        for c, o, source in zip(captured[32:], outcomes[32:], [v for v in controls for _ in v['modes']]):
            self.assertEqual(c['source'], source['source'].encode())
            self.assertEqual(runner.digest(c['source']), source['source_sha256'])
            self.assertTrue(o['verified'])
            if not source['positive']:
                self.assertEqual(o['prerequisite']['mode'], c['mode'])
                self.assertTrue(o['prerequisite']['verified'])
                partner = next(p for p in captured if p['id'] == c['id'].removesuffix('-wrong') + '-positive'
                               and p['mode'] == c['mode'])
                self.assertEqual(o['prerequisite']['case_sha256'], partner['case_sha256'])
                self.assertEqual(o['prerequisite']['source_sha256'], runner.digest(partner['source']))

    def test_incidental_errors_and_failed_positive_cannot_verify_wrong_controls(self):
        bad = [{}, dict(ERROR, error_type='TypeError'), dict(ERROR, error_identity='Other'),
               dict(ERROR, phase='parse'), dict(ERROR, status='unsupported')]
        for observation in bad:
            def fake(case):
                return ({'status': 'failed', 'observation': observation}
                        if case['id'].endswith('-wrong') else {'status': 'passed'})
            with self.subTest(observation=observation):
                _, outcomes = capture_controls(fake)
                self.assertTrue(all(not o['verified'] for o in outcomes[32:] if o['expected'] == 'failed'))
        for status in ('resource', 'timeout', 'adapter-error', 'unsupported', 'harness-error'):
            _, outcomes = capture_controls(lambda c: {'status': status})
            self.assertFalse(any(o['verified'] for o in outcomes[32:]))

        def one_mode(case):
            if case['id'].endswith('-wrong'):
                return {'status': 'failed', 'observation': ERROR.copy()}
            return {'status': 'passed' if case['mode'] == 'sloppy' else 'resource'}
        _, outcomes = capture_controls(one_mode)
        self.assertTrue(all(o['verified'] == (o['result']['mode'] == 'sloppy') for o in outcomes[32:]))

    def test_control_file_is_hash_bound_and_bounded(self):
        original = CONTROLS.read_bytes()
        for altered in (original + b'\n', b' ' * 65537):
            with self.subTest(size=len(altered)), patch.object(Path, 'open', return_value=io.BytesIO(altered)), self.assertRaises(ValueError):
                runner.typedarray_foundation_preflight_variants()


if __name__ == '__main__':
    unittest.main()
