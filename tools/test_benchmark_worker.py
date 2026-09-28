import copy
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

from benchmark_worker import PHASES, run_fixture, summarize, validate_run


class WorkerBenchmarkTests(unittest.TestCase):
    def run_record(self):
        return dict(schema=1, viewport=[1180, 880], scripts=True, iterations=2,
                    nodes=10, commands=20, **{key: 1.25 for key in PHASES},
                    warm_samples=[dict(render_exchange_ms=1, clear_paint_ms=2, total_ms=3.1),
                                  dict(render_exchange_ms=1.1, clear_paint_ms=2.2, total_ms=3.4)])

    def test_invalid_or_incomplete_measurements_cannot_be_aggregated(self):
        valid = self.run_record()
        validate_run(valid, 2)
        invalid = []
        for key, value in [('schema', 2), ('schema', True), ('viewport', [320, 240]),
                           ('viewport', [1180.0, 880.0]), ('scripts', False),
                           ('iterations', 3), ('iterations', 2.0), ('nodes', True), ('commands', -1),
                           ('startup_ms', float('nan')), ('load_ms', float('inf')),
                           ('cold_render_exchange_ms', -1), ('teardown_ms', '1.0'),
                           ('warm_samples', [])]:
            invalid.append(dict(valid, **{key: value}))
        for key, value in [('render_exchange_ms', None), ('clear_paint_ms', float('nan')),
                           ('total_ms', 2.9)]:
            run = copy.deepcopy(valid)
            run['warm_samples'][0][key] = value
            invalid.append(run)
        for run in invalid:
            with self.subTest(run=run), self.assertRaises(ValueError):
                validate_run(run, 2)

    def test_quantile_convention_and_sample_count_are_explicit(self):
        self.assertEqual(summarize([4, 1, 3, 2]), dict(samples=4, median_ms=3, p95_ms=4))
        self.assertEqual(summarize(list(range(1, 101))), dict(samples=100, median_ms=51, p95_ms=95))
        self.assertEqual(summarize([1.25]), dict(samples=1, median_ms=1.25, p95_ms=1.25))

    @unittest.skipUnless(sys.platform == 'linux', 'Linux process group cleanup')
    def test_timeout_terminates_descendants_and_cannot_return_a_success_record(self):
        with tempfile.TemporaryDirectory(prefix='eris-benchmark-timeout-') as directory:
            pid_file = Path(directory) / 'pid'
            source = "import pathlib,subprocess,sys,time; child=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)']);pathlib.Path(sys.argv[1]).write_text(str(child.pid));time.sleep(60)"
            with self.assertRaises(subprocess.TimeoutExpired):
                run_fixture([sys.executable, '-c', source, str(pid_file)], timeout=.5)
            pid = int(pid_file.read_text())
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                try:
                    # A reparented zombie has exited and cannot execute work;
                    # its final reaping belongs to the host's init process.
                    state = (Path('/proc') / str(pid) / 'stat').read_text().rsplit(')', 1)[1].split()[0]
                except (FileNotFoundError, ProcessLookupError):
                    # procfs can report ESRCH if the process disappears after
                    # stat is opened but before its contents are read.
                    break
                if state == 'Z':
                    break
                time.sleep(.01)
            else:
                os.kill(pid, 9)
                self.fail('benchmark descendant survived process-group timeout')


if __name__ == '__main__':
    unittest.main()
