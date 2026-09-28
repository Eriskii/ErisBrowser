import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest


RUNNER = Path(__file__).resolve().with_name('ci_test_runner.py')


class CiTestRunnerTests(unittest.TestCase):
    def run_program(self, source, *arguments, **kwargs):
        return subprocess.run(
            [sys.executable, str(RUNNER), sys.executable, '-c', source, *arguments],
            capture_output=True, timeout=5, **kwargs,
        )

    @unittest.skipUnless(os.name == 'posix', 'POSIX descriptor inheritance')
    def test_inherited_pipe_is_closed_before_the_test_program_starts(self):
        read_fd, write_fd = os.pipe()
        source = '''
import errno, os, sys
for descriptor in map(int, sys.argv[1:]):
    try:
        os.fstat(descriptor)
    except OSError as error:
        assert error.errno == errno.EBADF
    else:
        sys.exit(17)
'''
        arguments = [str(read_fd), str(write_fd)]
        try:
            # Confirm the fixture actually exposes an inheritable pipe before
            # testing that the runner closes it. Python's defaults would hide
            # the defect if pass_fds were accidentally omitted here.
            control = subprocess.run(
                [sys.executable, '-c', source, *arguments],
                pass_fds=(read_fd, write_fd), capture_output=True, timeout=5,
            )
            self.assertEqual(control.returncode, 17)
            result = self.run_program(source, *arguments, pass_fds=(read_fd, write_fd))
            self.assertEqual(result.returncode, 0, result.stderr)
            os.fstat(read_fd)
            os.fstat(write_fd)
        finally:
            os.close(read_fd)
            os.close(write_fd)

    def test_stdio_arguments_cwd_and_failure_status_are_forwarded(self):
        source = '''
import json, os, sys
sys.stdout.buffer.write(sys.stdin.buffer.read())
sys.stdout.buffer.write(json.dumps([os.getcwd(), sys.argv[1:]]).encode())
sys.stderr.buffer.write(b"stderr\\x00\\r\\n")
sys.exit(23)
'''
        arguments = ['space inside', 'line\nbreak', 'λ', '$(exit 0)']
        with tempfile.TemporaryDirectory(prefix='eris-ci-runner-') as directory:
            payload = b'stdin\x00\r\n'
            result = self.run_program(source, *arguments, cwd=directory, input=payload)
            self.assertEqual(result.returncode, 23)
            self.assertEqual(result.stderr, b'stderr\x00\r\n')
            self.assertTrue(result.stdout.startswith(payload))
            self.assertEqual(json.loads(result.stdout[len(payload):]), [directory, arguments])

    def test_jobserver_environment_is_removed_and_other_values_survive(self):
        env = os.environ.copy()
        for key in ('CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS'):
            env[key] = '--jobserver-auth=100,101'
        env['ERIS_CI_RUNNER_TEST'] = 'preserved'
        source = '''
import json, os
keys = ('CARGO_MAKEFLAGS', 'MAKEFLAGS', 'MFLAGS', 'ERIS_CI_RUNNER_TEST')
print(json.dumps({key: os.environ.get(key) for key in keys}))
'''
        result = self.run_program(source, env=env)
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = dict(CARGO_MAKEFLAGS=None, MAKEFLAGS=None, MFLAGS=None,
                        ERIS_CI_RUNNER_TEST='preserved')
        self.assertEqual(json.loads(result.stdout), expected)

    @unittest.skipUnless(os.name == 'posix', 'POSIX signal termination')
    def test_signal_termination_is_a_nonzero_shell_status(self):
        result = self.run_program('import os, signal; os.kill(os.getpid(), signal.SIGTERM)')
        self.assertEqual(result.returncode, 128 + signal.SIGTERM)

    def test_missing_arguments_report_usage_and_fail(self):
        result = subprocess.run(
            [sys.executable, str(RUNNER)], capture_output=True, timeout=5,
        )
        self.assertEqual(result.returncode, 2)
        self.assertEqual(result.stdout, b'')
        self.assertIn(b'usage:', result.stderr)

    def test_program_start_failure_cannot_be_reported_as_success(self):
        with tempfile.TemporaryDirectory(prefix='eris-ci-runner-') as directory:
            result = subprocess.run(
                [sys.executable, str(RUNNER), str(Path(directory) / 'missing')],
                capture_output=True, timeout=5,
            )
        self.assertEqual(result.returncode, 127)
        self.assertIn(b'cannot start test program', result.stderr)


if __name__ == '__main__':
    unittest.main()
