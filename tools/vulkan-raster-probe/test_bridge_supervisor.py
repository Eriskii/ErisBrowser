"""Exercise the isolated supervisor using real disposable process trees, no GPU."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parent


@unittest.skipUnless(sys.platform == 'linux', 'Linux subreaper ownership contract')
class SupervisorTests(unittest.TestCase):
    def run_checker(self, code, *, timeout=2, output_limit=16384, cancel=False,
                    cancel_during_cleanup=False, capture_gate=False, worker_text_gate=False):
        with tempfile.TemporaryDirectory(prefix='eris-bridge-supervisor-') as temp:
            result = Path(temp) / 'receipt.json'
            ready = Path(temp) / 'ready'
            command = [sys.executable, str(ROOT / 'bridge_supervisor.py'),
                       '--result', str(result), '--timeout', str(timeout),
                       '--output-limit', str(output_limit),
                       *(['--capture-gate'] if capture_gate else []),
                       *(['--worker-text-gate'] if worker_text_gate else []), '--', sys.executable,
                       '-c', code, str(ready)]
            if cancel_during_cleanup:
                wrapper = f'''import sys,os,signal
sys.path.insert(0,{str(ROOT)!r})
import bridge_supervisor as b
original=b.cleanup
def cleanup(*args):
    result=original(*args)
    os.kill(os.getpid(),signal.SIGTERM)
    return result
b.cleanup=cleanup
raise SystemExit(b.main())
'''
                command = [sys.executable, '-c', wrapper, *command[2:]]
            proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if cancel:
                deadline = time.monotonic() + 3
                while not ready.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                self.assertTrue(ready.exists(), 'checker must signal readiness before cancellation')
                proc.send_signal(signal.SIGTERM)
            stdout, stderr = proc.communicate(timeout=12)
            self.assertEqual(stdout, b'')
            self.assertEqual(stderr, b'')
            record = json.loads(result.read_text())
            captured = result.with_suffix('.stdout.log').read_bytes()
            errors = result.with_suffix('.stderr.log').read_bytes()
            self.assertTrue(record['cleanup']['complete'], record)
            self.assertLessEqual(record['captured_bytes'], output_limit)
            self.assertEqual(record['captured_bytes'], len(captured) + len(errors))
            self.assertEqual(proc.returncode, 0 if record['status'] == 'exited' and
                             record['returncode'] == 0 else 1)
            return record, captured, errors

    def test_success_and_nonzero_keep_exact_output(self):
        record, out, err = self.run_checker('import sys; print("ready"); print("diagnostic", file=sys.stderr)')
        self.assertEqual((record['status'], record['returncode']), ('exited', 0))
        self.assertEqual((out, err), (b'ready\n', b'diagnostic\n'))
        self.assertEqual(record['cleanup']['reaped'], 1)
        record, _, _ = self.run_checker('raise SystemExit(7)')
        self.assertEqual((record['status'], record['returncode']), ('exited', 7))

    def test_waited_child_does_not_become_an_orphan_failure(self):
        record, _, _ = self.run_checker('import subprocess,sys; subprocess.run([sys.executable,"-c","pass"], check=True)')
        self.assertEqual(record['status'], 'exited')
        self.assertEqual(record['cleanup']['descendants'], 0)

    def test_output_flood_is_bounded_even_after_exit(self):
        record, _, _ = self.run_checker('import os; os.write(1,b"a"*200000)', output_limit=1024)
        self.assertEqual(record['status'], 'output_limit')
        self.assertEqual(record['captured_bytes'], 1024)
        self.assertGreater(record['observed_bytes'], 1024)

    def test_timeout_adopts_and_reaps_worker_in_a_distinct_session(self):
        code = '''import os,time
read,write=os.pipe()
pid=os.fork()
if pid==0:
    os.close(read);os.setsid();os.write(write,b'r');os.close(write);time.sleep(5);os._exit(0)
os.close(write);os.read(read,1);os.close(read)
print('worker-ready',flush=True)
time.sleep(5)
'''
        record, out, _ = self.run_checker(code, timeout=0.3)
        self.assertEqual(record['status'], 'timeout')
        self.assertEqual(out, b'worker-ready\n')
        self.assertEqual(record['cleanup']['descendants'], 1)
        self.assertEqual(record['cleanup']['reaped'], 2)

    def test_normal_exit_with_detached_pipe_holder_is_a_failure(self):
        code = '''import os,time
read,write=os.pipe()
if os.fork()==0:
    os.close(read);os.setsid();os.write(write,b'r');os.close(write);time.sleep(5);os._exit(0)
os.close(write);os.read(read,1);os.close(read)
print('parent-exits',flush=True)
'''
        record, out, _ = self.run_checker(code)
        self.assertEqual(record['status'], 'surviving_descendants')
        self.assertEqual(record['returncode'], 0)
        self.assertEqual(out, b'parent-exits\n')
        self.assertEqual(record['cleanup']['descendants'], 1)

    def test_already_orphaned_grandchild_is_adopted_by_supervisor(self):
        code = '''import os,time
read,write=os.pipe()
child=os.fork()
if child==0:
    os.close(read)
    if os.fork()==0:
        os.setsid();os.write(write,b'r');os.close(write);time.sleep(5);os._exit(0)
    os._exit(0)
os.close(write);os.read(read,1);os.close(read);os.waitpid(child,0)
print('grandchild-ready',flush=True)
'''
        record, out, _ = self.run_checker(code)
        self.assertEqual(record['status'], 'surviving_descendants')
        self.assertEqual(out, b'grandchild-ready\n')
        self.assertEqual(record['cleanup']['descendants'], 1)

    def test_cancellation_still_reaps_distinct_session(self):
        code = '''import os,time,sys
from pathlib import Path
read,write=os.pipe()
if os.fork()==0:
    os.close(read);os.setsid();os.write(write,b'r');os.close(write);time.sleep(5);os._exit(0)
os.close(write);os.read(read,1);os.close(read)
Path(sys.argv[1]).write_text('ready')
time.sleep(5)
'''
        record, _, _ = self.run_checker(code, cancel=True)
        self.assertEqual(record['status'], 'cancelled')
        self.assertEqual(record['cancellation_signal'], [signal.SIGTERM])
        self.assertEqual(record['cleanup']['descendants'], 1)

    def test_cancellation_after_successful_reap_cannot_report_success(self):
        record, _, _ = self.run_checker('pass', cancel_during_cleanup=True)
        self.assertEqual(record['returncode'], 0)
        self.assertEqual(record['status'], 'cancelled')
        self.assertEqual(record['cancellation_signal'], [signal.SIGTERM])

    def test_capture_grant_requires_live_checker_without_adopted_children(self):
        code = '''import sys
print('CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=1 owned_children=0',flush=True)
assert sys.stdin.buffer.read(10)==b'GPU_READY\\n'
print('granted',flush=True)
'''
        record, out, _ = self.run_checker(code, capture_gate=True)
        self.assertEqual(record['status'], 'exited')
        self.assertTrue(record['capture_gate_granted'])
        self.assertTrue(out.endswith(b'granted\n'))

    def test_adopted_orphan_prevents_capture_grant(self):
        code = '''import os,time,sys
read,write=os.pipe()
child=os.fork()
if child==0:
    os.close(read)
    if os.fork()==0:
        os.setsid();os.write(write,b'r');os.close(write);time.sleep(5);os._exit(0)
    os._exit(0)
os.close(write);os.read(read,1);os.close(read);os.waitpid(child,0)
print('CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=1 owned_children=0',flush=True)
sys.stdin.buffer.read(10)
print('incorrectly-granted',flush=True)
'''
        record, out, _ = self.run_checker(code, capture_gate=True)
        self.assertEqual(record['status'], 'error')
        self.assertFalse(record['capture_gate_granted'])
        self.assertNotIn(b'incorrectly-granted', out)
        self.assertEqual(record['cleanup']['descendants'], 1)

    def test_clean_exit_without_required_gate_is_failure(self):
        record, _, _ = self.run_checker('pass', capture_gate=True)
        self.assertEqual(record['status'], 'missing_capture_gate')

    def test_worker_text_grant_uses_distinct_complete_inventory(self):
        code = '''import sys
print('TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=1 owned_children=0',flush=True)
assert sys.stdin.buffer.read(10)==b'GPU_READY\\n'
print('granted',flush=True)
'''
        record, out, _ = self.run_checker(code, worker_text_gate=True)
        self.assertEqual(record['status'], 'exited')
        self.assertTrue(record['capture_gate_granted'])
        self.assertTrue(out.endswith(b'granted\n'))

    def test_capture_modes_refuse_wrong_inventory_and_cross_suite_marker(self):
        markers = (
            'CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=1 owned_children=0',
            'TEXT_CAPTURE_COMPLETE cases=6 worker_cases=7 own_tasks=1 owned_children=0',
            'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=65 owned_children=0',
        )
        for marker in markers:
            with self.subTest(marker=marker):
                code = f'import sys; print({marker!r},flush=True); sys.stdin.buffer.read(10)'
                record, _, _ = self.run_checker(code, worker_text_gate=True)
                self.assertEqual(record['status'], 'error')
                self.assertFalse(record['capture_gate_granted'])
        code = "import sys; print('TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=1 owned_children=0',flush=True); sys.stdin.buffer.read(10)"
        record, _, _ = self.run_checker(code, capture_gate=True)
        self.assertEqual(record['status'], 'error')
        self.assertFalse(record['capture_gate_granted'])

    def test_worker_text_orphan_prevents_grant(self):
        code = '''import os,sys,time
r,w=os.pipe()
child=os.fork()
if child==0:
    os.close(r)
    if os.fork()==0:
        os.setsid();os.write(w,b'r');os.close(w);time.sleep(5);os._exit(0)
    os._exit(0)
os.close(w);os.read(r,1);os.close(r);os.waitpid(child,0)
print('TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=1 owned_children=0',flush=True)
sys.stdin.buffer.read(10)
print('incorrectly-granted',flush=True)
'''
        record, out, _ = self.run_checker(code, worker_text_gate=True)
        self.assertEqual(record['status'], 'error')
        self.assertFalse(record['capture_gate_granted'])
        self.assertNotIn(b'incorrectly-granted', out)
        self.assertEqual(record['cleanup']['descendants'], 1)


if __name__ == '__main__':
    unittest.main()
