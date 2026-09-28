import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import subprocess
import sys

import html_conformance as runner


class CorpusParsingTests(unittest.TestCase):
    def test_input_line_endings_and_multiline_tree_data_are_preserved(self):
        source = '#data\na\r\nb\n#errors\none\n#new-errors\ntwo\n#document\n| "a\nb"\n'
        count, cases = runner.parse_cases(source, 'sample.dat')
        self.assertEqual(count, 1)
        self.assertEqual(len(cases), 2)
        for case in cases:
            self.assertEqual(case['source'], 'a\r\nb')
            self.assertEqual(case['expected'], '| "a\nb"\n')
            self.assertEqual(case['expected_parse_errors'], 2)

    def test_case_boundaries_flags_and_fragment_context_are_explicit(self):
        text = '#data\nA\n#errors\n#script-on\n#document\n| "A"\n\n#data\nB\n#errors\n#document-fragment\nsvg foreignObject\n#script-off\n#document\n| "B"\n'
        count, cases = runner.parse_cases(text, 'sample.dat')
        self.assertEqual(count, 2)
        self.assertEqual([case['scripting'] for case in cases], ['enabled', 'disabled'])
        self.assertIsNone(cases[0]['fragment'])
        self.assertEqual(cases[1]['fragment'], 'svg foreignObject')
        self.assertEqual(cases[0]['expected'], '| "A"\n')
        self.assertEqual(cases[1]['expected'], '| "B"\n')

    def test_input_is_not_interpreted_as_metadata_before_errors(self):
        _, cases = runner.parse_cases('#data\n#document\n#script-off\n#errors\n#document\n| "#document\n#script-off"\n', 'sample.dat')
        self.assertEqual(cases[0]['source'], '#document\n#script-off')
        self.assertEqual(len(cases), 2)

    def test_multiline_node_text_can_contain_a_data_marker(self):
        text = '#data\n<p>first\n#data\nlast\n#errors\n#document\n| "first\n#data\nlast"\n\n#data\nnext\n#errors\n#document\n| "next"\n'
        count, cases = runner.parse_cases(text, 'sample.dat')
        self.assertEqual(count, 2)
        self.assertEqual(cases[0]['expected'], '| "first\n#data\nlast"\n')

    def test_incomplete_corpus_is_rejected(self):
        with self.assertRaisesRegex(ValueError, 'missing #errors'):
            runner.parse_cases('#data\ntext', 'sample.dat')
        with self.assertRaisesRegex(ValueError, 'missing #document'):
            runner.parse_cases('#data\ntext\n#errors\none', 'sample.dat')

    def test_pinned_corpus_bytes_are_checked_before_any_execution(self):
        with tempfile.TemporaryDirectory(prefix='eris-corpus-') as temporary:
            root = Path(temporary)
            data = b'#data\ntext\n#errors\n#document\n| "text"\n'
            (root / 'sample.dat').write_bytes(data)
            manifest = dict(revision='example', files=[dict(path='sample.dat', bytes=len(data), sha256=hashlib.sha256(data).hexdigest())])
            (root / 'manifest.json').write_text(json.dumps(manifest))
            _, count, cases = runner.load_corpus(root)
            self.assertEqual((count, len(cases)), (1, 2))
            (root / 'sample.dat').write_bytes(data.replace(b'text', b'evil'))
            with self.assertRaisesRegex(ValueError, 'integrity mismatch'):
                runner.load_corpus(root)

    def test_baseline_binds_expected_tree_and_context_as_well_as_source(self):
        _, cases = runner.parse_cases('#data\nx\n#errors\n#document\n| "x"\n', 'sample.dat')
        case = cases[0]
        old = dict(revision='test', corpus_manifest_sha256='manifest', cases={case['id']: dict(status='matched', case_sha256=runner.case_fingerprint(case))})
        for field, replacement in [('expected', '| "wrong"\n'), ('fragment', 'table'), ('scripting', 'enabled')]:
            changed = dict(case, **{field: replacement})
            current = dict(old, cases={case['id']: dict(status='matched', case_sha256=runner.case_fingerprint(changed))})
            with self.assertRaisesRegex(ValueError, 'expectation or context differs'):
                runner.check_baseline(old, current)
        with self.assertRaisesRegex(ValueError, 'manifest fingerprint differs'):
            runner.check_baseline(old, dict(old, corpus_manifest_sha256='changed'))

    def test_regression_check_cannot_rewrite_its_baseline(self):
        with tempfile.TemporaryDirectory(prefix='eris-baseline-') as temporary:
            baseline=Path(temporary) / 'baseline.json'
            baseline.write_text('original')
            result = subprocess.run([sys.executable, str(Path(runner.__file__)), '--baseline', str(baseline), '--record-baseline', str(baseline)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(baseline.read_text(), 'original')

    def test_adapter_output_and_runtime_are_bounded_during_capture(self):
        for stream in ('stdout', 'stderr'):
            command=[sys.executable, '-c', f'import sys; sys.{stream}.write("x"*100000)']
            with self.assertRaisesRegex(ValueError, 'capture limit'):
                runner.bounded_process(command, b'', 2, stdout_limit=1024, stderr_limit=1024)
        with self.assertRaises(subprocess.TimeoutExpired):
            runner.bounded_process([sys.executable, '-c', 'import time; time.sleep(10)'], b'', 0.05)
        status, output, errors = runner.bounded_process([sys.executable, '-c', 'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())'], b'a\r\nb\0c', 2)
        self.assertEqual((status, output, errors), (0, b'a\r\nb\0c', b''))

    def test_failed_adapter_cannot_replace_a_recorded_baseline(self):
        with tempfile.TemporaryDirectory(prefix='eris-failed-adapter-') as temporary:
            root = Path(temporary)
            data = b'#data\nx\n#errors\n#document\n| "x"\n'
            (root / 'sample.dat').write_bytes(data)
            manifest = dict(repository='test', revision='test', files=[dict(path='sample.dat', bytes=len(data), sha256=hashlib.sha256(data).hexdigest())])
            (root / 'manifest.json').write_text(json.dumps(manifest))
            adapter = root / 'adapter'
            adapter.write_text('#!/usr/bin/env python3\nimport sys\nsys.exit(7)\n')
            adapter.chmod(0o700)
            baseline = root / 'baseline.json'
            baseline.write_text('original')
            result = subprocess.run([sys.executable, str(Path(runner.__file__)), '--binary', str(adapter), '--corpus', str(root), '--record-baseline', str(baseline), '--output', str(root / 'report.json')], capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertEqual(baseline.read_text(), 'original')
            self.assertEqual(json.loads((root / 'report.json').read_text())['counts'], {'error': 2})


if __name__ == '__main__':
    unittest.main()
