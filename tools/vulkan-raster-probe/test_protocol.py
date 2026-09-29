"""No child processes, GPU objects or adapter enumeration in these checks."""
import json
from pathlib import Path
import unittest
import run_host as runner
DEVICE='ADAPTER 0 vendor=0x1 device=0x2 type=Cpu backend=Vulkan name="fake" driver="fake" info="offline"'
def result(lines,**fields):
    return runner.RunResult({'status':'exited','returncode':0,**fields}, ('\n'.join(lines)+'\n').encode(),b'')
class ProtocolTests(unittest.TestCase):
    def test_protocol_matches_independently_frozen_counts(self):
        frozen=json.loads((Path(__file__).parent/'evidence/image-protocol.json').read_text())
        self.assertEqual(runner.EXPECTED_PASSES,[tuple(row) for row in frozen['original_tuples']+frozen['new_tuples']])
        self.assertEqual(len(runner.EXPECTED_PASSES),21)
        self.assertEqual(sum(row[-1] for row in runner.EXPECTED_PASSES),921272)
    def test_complete_exact_protocol(self):
        self.assertEqual(runner.validate_listing(result([DEVICE])),[DEVICE])
        runner.validate_adapter(result([DEVICE,*runner.expected_lines(),'COMPLETE adapter=0 fixtures=21 exact=true custom_wgsl=true']),[DEVICE],0)
    def test_exit_zero_cannot_hide_missing_changed_or_reordered_case(self):
        good=[DEVICE,*runner.expected_lines(),'COMPLETE adapter=0 fixtures=21 exact=true custom_wgsl=true']
        bad=[good[:-1],good[:1],good+[good[-1]],[good[0],good[2],good[1],*good[3:]],[s.replace('compared_bytes=307200','compared_bytes=307199') for s in good],[s.replace('backend=Vulkan','backend=Gl') for s in good],[good[1],good[0],*good[2:]]]
        for lines in bad:
            with self.assertRaises(ValueError):runner.validate_adapter(result(lines),[DEVICE],0)
    def test_every_image_record_is_required_and_exact(self):
        good=[DEVICE,*runner.expected_lines(),'COMPLETE adapter=0 fixtures=21 exact=true custom_wgsl=true']
        for index in range(8,22):
            with self.assertRaises(ValueError):runner.validate_adapter(result(good[:index]+good[index+1:]),[DEVICE],0)
            for field in ['draws','invocations','gpu_buffers','compared_bytes']:
                prefix,value=good[index].split(field+'=',1)
                count,suffix=value.split(' ',1)
                changed=good.copy();changed[index]=prefix+field+'='+str(int(count)+1)+' '+suffix
                with self.assertRaises(ValueError):runner.validate_adapter(result(changed),[DEVICE],0)
        with self.assertRaises(ValueError):
            runner.validate_adapter(result([DEVICE,*runner.expected_lines()[:7],'COMPLETE adapter=0 fixtures=7 exact=true custom_wgsl=true']),[DEVICE],0)
    def test_timeout_and_failed_exit_are_not_passes(self):
        for state,code in [('timeout',0),('output_limit',0),('exited',1)]:
            with self.assertRaises(ValueError):runner.validate_listing(result([DEVICE],status=state,returncode=code))
if __name__=='__main__':unittest.main()
