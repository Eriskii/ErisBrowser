"""Literal parent policies and source boundaries for the Any-only profile."""
import ast
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import import_test262 as importer
import test262_conformance as runner


ORACLE = Path(__file__).with_name('promise-any-preservation-oracle.json')
ORACLE_SHA = '01ac062ca6999de4b36cac2d3a36fe58a5210ea17ca52db032bb823d41c30406'
NEW_PROFILE = 'promise-any'


class PromiseAnyPreservationTests(unittest.TestCase):
    def setUp(self):
        data = ORACLE.read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(), ORACLE_SHA)
        self.oracle = json.loads(data)
        for owner, name in ((importer, 'fetch'), (runner, 'bounded_process')):
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_all_58_old_tables_features_and_actual_main_policies_remain_exact(self):
        old = self.oracle
        self.assertEqual({k: v for k, v in importer.PROFILES.items() if k not in {NEW_PROFILE, 'promise-with-resolvers', 'promise-try'}}, old['profiles'])
        self.assertEqual({k: v for k, v in importer.PROFILE_ROOTS.items() if k not in {NEW_PROFILE, 'promise-with-resolvers', 'promise-try'}}, old['roots'])
        self.assertEqual(sorted(importer.TREE_PROFILES - {NEW_PROFILE, 'promise-with-resolvers', 'promise-try'}), old['tree_profiles'])
        self.assertEqual({k: sorted(v) for k, v in runner.PROFILE_FEATURES.items() if k not in {NEW_PROFILE, 'promise-with-resolvers', 'promise-try'}}, old['features'])
        self.assertEqual(sorted(runner.PROMISE_HOST_PROFILES - {NEW_PROFILE, 'promise-try'}), old['promise_host_profiles'])
        self.assertEqual(sorted(runner.INTRINSIC_ERRORS), old['intrinsic_errors'])
        self.assertEqual(len(old['profiles']), 58)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / 'inert-adapter'
            binary.write_bytes(b'never executed')
            output = root / 'report.json'
            for profile, expected in old['policies_at_timeout3'].items():
                with self.subTest(profile=profile), contextlib.ExitStack() as stack:
                    stack.enter_context(patch.object(runner, 'load_corpus', return_value=(
                        dict(repository='fixture', revision='fixture', test_files=0), {}, [], [], '0' * 64)))
                    stack.enter_context(patch.object(runner, 'harness_preflight', return_value=[]))
                    for health in ('aggregate_error_preflight_healthy', 'promise_preflight_healthy',
                                   'promise_all_preflight_healthy', 'promise_race_preflight_healthy',
                                   'promise_all_settled_preflight_healthy'):
                        stack.enter_context(patch.object(runner, health, return_value=True))
                    call = stack.enter_context(patch.object(runner, 'run_case', side_effect=AssertionError('no cases')))
                    stack.enter_context(patch('sys.argv', ['test262_conformance.py', '--profile', profile,
                                                          '--binary', str(binary), '--output', str(output)]))
                    stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
                    self.assertEqual(runner.main(), 0)
                    call.assert_not_called()
                report = json.loads(output.read_bytes())
                self.assertEqual((report['runner_policy'], report['runner_policy_sha256']),
                                 (expected['policy'], expected['sha256']), profile)
                self.assertEqual((report['mode_cases'], report['counts']), (0, {}))

    def test_existing_gates_literals_fingerprints_and_request_encoding_are_unchanged(self):
        source = Path(runner.__file__).read_text()
        expected = self.oracle['unchanged_runner_functions']
        actual = {node.name: hashlib.sha256(ast.get_source_segment(source, node).encode()).hexdigest()
                  for node in ast.parse(source).body
                  if isinstance(node, ast.FunctionDef) and node.name in expected}
        self.assertEqual(actual, expected)
        self.assertEqual(len(actual), 22)


if __name__ == '__main__':
    unittest.main()
