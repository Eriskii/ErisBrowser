"""Bounded fake CPU-only child checks; no Vulkan library or GPU execution."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock
import run_host as runner

@unittest.skipUnless(sys.platform.startswith('linux'),'Linux WNOWAIT supervisor')
class SupervisorTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='raster supervisor ');self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
    def run_fake(self,source,**kw):
        path=self.root/'fake child';path.write_text('#!'+sys.executable+'\n'+source+'\n');path.chmod(0o700)
        return runner.run_process(path,[],'case',self.root/'logs',os.environ.copy(),**kw)
    def test_exit_is_observed_without_releasing_identity_before_group_cleanup(self):
        real=runner._stop_group;seen=[]
        def cleanup(proc):
            seen.append(proc.returncode)
            return real(proc)
        with mock.patch.object(runner,'_stop_group',cleanup):result=self.run_fake("print('exact')",timeout=2)
        self.assertEqual(seen,[None]);self.assertEqual(result.record['status'],'exited');self.assertEqual(result.record['returncode'],0);self.assertEqual(result.stdout,b'exact\n')
    def test_silent_timeout_is_killed_and_reaped(self):
        result=self.run_fake('import time;time.sleep(60)',timeout=0.1)
        self.assertEqual(result.record['status'],'timeout');self.assertEqual(result.record['returncode'],-9)
    def test_combined_output_limit_is_retained_not_exceeded(self):
        result=self.run_fake("import os;os.write(2,b'x'*8192)",timeout=2,output_limit=256)
        self.assertEqual(result.record['status'],'output_limit');self.assertEqual(len(result.stdout)+len(result.stderr),256)
    def test_stdout_stderr_are_drained_before_success(self):
        result=self.run_fake("import os;os.write(1,b'out');os.write(2,b'err')",timeout=2)
        self.assertEqual(result.record['status'],'exited');self.assertEqual((result.stdout,result.stderr),(b'out',b'err'))
    def test_exit_with_descendant_holding_pipes_does_not_terminate_monitor_early(self):
        result=self.run_fake("import subprocess,sys;subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)']);print('leader exited')",timeout=0.2)
        self.assertEqual(result.record['status'],'timeout');self.assertEqual(result.record['returncode'],0)
        self.assertEqual(result.stdout,b'leader exited\n')
if __name__=='__main__':unittest.main()
