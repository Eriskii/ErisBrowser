"""Literal parent policies and source boundaries for the ordinary Promise.withResolvers profile."""
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
import test_test262_promise_runner as legacy


ORACLE = Path(__file__).with_name('with-resolvers-preservation-oracle.json')
ORACLE_SHA = '51a2a9bfff4649528852183f1db1ccf0abc6ad66c33411770cfaef4b5a23d9c3'
NEW_PROFILE = 'promise-with-resolvers'


class WithResolversPreservationTests(unittest.TestCase):
    def setUp(self):
        data = ORACLE.read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(), ORACLE_SHA)
        self.oracle = json.loads(data)
        for owner, name in ((importer, 'fetch'), (runner, 'bounded_process')):
            guard = patch.object(owner, name, side_effect=AssertionError('network/adapter forbidden'))
            guard.start()
            self.addCleanup(guard.stop)

    def test_all_59_old_tables_features_and_actual_main_policies_remain_exact(self):
        old = self.oracle
        self.assertEqual({k: v for k, v in importer.PROFILES.items() if k not in {NEW_PROFILE, 'promise-try'}}, old['profiles'])
        self.assertEqual({k: v for k, v in importer.PROFILE_ROOTS.items() if k not in {NEW_PROFILE, 'promise-try'}}, old['roots'])
        self.assertEqual(sorted(importer.TREE_PROFILES - {NEW_PROFILE, 'promise-try'}), old['tree_profiles'])
        self.assertEqual({k: sorted(v) for k, v in runner.PROFILE_FEATURES.items() if k not in {NEW_PROFILE, 'promise-try'}}, {k: legacy.class_admitted_features(k, v) for k, v in old['features'].items()})
        self.assertEqual(sorted(runner.PROMISE_HOST_PROFILES - {'promise-try'}), old['promise_host_profiles'])
        self.assertEqual(sorted(runner.INTRINSIC_ERRORS), old['intrinsic_errors'])
        self.assertEqual(len(old['profiles']), 59)
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
                    # Empty-inventory policy fixture; class raw evidence is tested separately.
                    stack.enter_context(patch.object(runner, 'promise_class_harness_preflight', return_value=[]))
                    stack.enter_context(patch.object(runner, 'promise_class_preflight_healthy', return_value=True))
                    for health in ('promise_any_preflight_healthy', 'aggregate_error_preflight_healthy', 'promise_preflight_healthy',
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
                                 legacy.class_admitted_policy(profile, expected['policy'], expected['sha256']), profile)
                self.assertEqual((report['mode_cases'], report['counts']), (0, {}))

    def test_old_import_authentication_gates_classifiers_and_encoders_remain_exact(self):
        for relative, expected in self.oracle['unchanged_functions'].items():
            with self.subTest(path=relative):
                source = (Path(__file__).resolve().parent.parent / relative).read_text()
                actual = {node.name: hashlib.sha256(ast.get_source_segment(source, node).encode()).hexdigest()
                          for node in ast.parse(source).body
                          if isinstance(node, ast.FunctionDef) and node.name in expected}
                self.assertEqual(actual, expected)
        self.assertIn('classify', self.oracle['unchanged_functions']['tools/test262_conformance.py'])
        self.assertIn('git_tree_inventory', self.oracle['unchanged_functions']['tools/import_test262.py'])


if __name__ == '__main__':
    unittest.main()
