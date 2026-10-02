"""Synthetic protocol/control checks only; no process, socket, desktop or GPU."""
from copy import deepcopy
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_raster_zoom_host as zoom
from test_native_raster_wide_host import population as wide_population

CONTROLLER = Path('/fake/hyprctl')


def population(case='zoom-in-verified'):
    key, bits, verify = zoom.CASES[case]
    with zoom.case_scope(case):
        data, control = wide_population('wide-verified' if verify else 'wide-normal')
    marker = f'native zoom prepared: serial=4 zoom_bits={bits}\n'.encode()
    data = data.replace(b'presenter: native shader frame presented;', marker + b'presenter: native shader frame presented;')
    command = {'argv': [str(CONTROLLER), 'dispatch', 'sendshortcut', f'CTRL,{key},pid:71'],
               'status': 'exited', 'returncode': 0, 'stdout': 'ok\n', 'stderr': '',
               'stdout_bytes': 3, 'stdout_sha256': zoom.wide.sha(b'ok\n'),
               'stderr_bytes': 0, 'stderr_sha256': zoom.wide.sha(b'')}
    control['zoom'] = {'case': case, 'zoom_bits': bits, 'dispatch_complete': True, 'command': command}
    control['commands'] = [command]
    return data, control


class ZoomTests(unittest.TestCase):
    def test_three_exact_cases_and_two_complete_acquired_frames(self):
        total = 0
        for case, (_, bits, verify) in zoom.CASES.items():
            data, control = population(case)
            checked = zoom.validate_zoom(b'', data, case, control, CONTROLLER)
            self.assertEqual(checked['observed_zoom_by_serial'], {4: bits})
            self.assertEqual(checked['verification_requested'], verify)
            total += checked['acquired_texture_compared_bytes']
        self.assertEqual(total, 9_011_200)

    def test_wrong_missing_duplicate_or_late_zoom_proof_refuses(self):
        data, control = population()
        marker = b'native zoom prepared: serial=4 zoom_bits=1066192077\n'
        without = data.replace(marker, b'')
        candidates = [without, data.replace(marker, marker * 2),
                      data.replace(marker, marker.replace(b'1066192077', b'1063675494')),
                      data.replace(marker, marker.replace(b'serial=4', b'serial=5')),
                      without + marker, data.replace(marker, b'native zoom broken\n')]
        for changed in candidates:
            with self.subTest(changed=changed[-100:]), self.assertRaises(ValueError):
                zoom.validate_zoom(b'', changed, 'zoom-in-verified', control, CONTROLLER)

    def test_before_scene_zoom_proof_refuses(self):
        data, control = population()
        lines = data.splitlines(keepends=True)
        scene = next(i for i, line in enumerate(lines) if line.startswith(b'native scene prepared:'))
        lines[scene], lines[scene + 1] = lines[scene + 1], lines[scene]
        with self.assertRaises(ValueError):
            zoom.validate_zoom(b'', b''.join(lines), 'zoom-in-verified', control, CONTROLLER)

    def test_normal_cannot_carry_reference_or_verification(self):
        verified, verified_control = population()
        normal, normal_control = population('zoom-in-normal')
        with self.assertRaises(ValueError):
            zoom.validate_zoom(b'', normal.replace(b'reference=false', b'reference=true'),
                               'zoom-in-normal', normal_control, CONTROLLER)
        with self.assertRaises(ValueError):
            zoom.validate_zoom(b'', verified, 'zoom-in-normal', verified_control, CONTROLLER)

    def test_shortcut_must_be_exactly_one_successful_owned_pid_command(self):
        data, original = population()
        changes = [lambda r: r['zoom'].update(dispatch_complete=False),
                   lambda r: r['zoom']['command']['argv'].__setitem__(-1, 'CTRL,equal,pid:72'),
                   lambda r: r['zoom']['command'].update(returncode=1),
                   lambda r: r['zoom']['command'].update(stdout='not ok\n'),
                   lambda r: r['commands'].append(deepcopy(r['zoom']['command'])),
                   lambda r: r['commands'].append({'argv': [str(CONTROLLER), 'dispatch', 'sendshortcut', 'CTRL,r,pid:71']}),
                   lambda r: r['zoom'].update(zoom_bits=1063675494)]
        for change in changes:
            control = deepcopy(original)
            change(control)
            with self.subTest(change=change), self.assertRaises(ValueError):
                zoom.validate_zoom(b'', data, 'zoom-in-verified', control, CONTROLLER)

    def test_wrong_waiting_zoom_and_dirty_cleanup_refuse(self):
        data, control = population()
        with self.assertRaises(ValueError):
            zoom.validate_zoom(b'', data.replace(b'page; zoom_bits=1066192077;', b'page; zoom_bits=1063675494;'),
                               'zoom-in-verified', control, CONTROLLER)
        control['browser_reaped'] = False
        with self.assertRaises(ValueError):
            zoom.validate_zoom(b'', data, 'zoom-in-verified', control, CONTROLLER)

    def test_control_substitution_restores_helpers_and_targets_only_live_child(self):
        original_reason = zoom.wide.PHYSICAL
        configured = []
        def configure(identity, controller, env, record, persist, **kwargs):
            configured.append(kwargs)
            record['control_complete'] = True
        result = SimpleNamespace(record={'status': 'exited', 'returncode': 0}, stdout=b'ok\n', stderr=b'')
        record = {'commands': []}
        with patch.object(zoom.base, 'configure_owned', configure), \
             patch.object(zoom.base, 'ensure_live_child') as live, \
             patch.object(zoom.base, 'controller_call', return_value=result) as call:
            with zoom.case_scope('zoom-out-verified', control=True):
                zoom.base.configure_owned({'pid': 71}, CONTROLLER, {}, record, lambda: None, requested_size=zoom.wide.SIZE)
            self.assertIs(zoom.base.configure_owned, configure)
            self.assertEqual(live.call_count, 2)
            call.assert_called_once_with(CONTROLLER, ['dispatch', 'sendshortcut', 'CTRL,minus,pid:71'], {}, 0.5)
        self.assertEqual(zoom.wide.PHYSICAL, original_reason)
        self.assertEqual(configured, [{'requested_size': (1280, 880)}])
        self.assertTrue(record['control_complete'])
        self.assertTrue(record['zoom']['dispatch_complete'])

    def test_failed_controller_does_not_open_gate_and_restores_helpers(self):
        original_reason = zoom.wide.PHYSICAL
        record = {'commands': []}
        result = SimpleNamespace(record={'status': 'exited', 'returncode': 0}, stdout=b'error\n', stderr=b'')
        with patch.object(zoom.base, 'configure_owned'), patch.object(zoom.base, 'ensure_live_child'), \
             patch.object(zoom.base, 'controller_call', return_value=result):
            with self.assertRaises(ValueError):
                with zoom.case_scope('zoom-in-verified', control=True):
                    zoom.base.configure_owned({'pid': 71}, CONTROLLER, {}, record, lambda: None)
        self.assertFalse(record['control_complete'])
        self.assertFalse(record['zoom']['dispatch_complete'])
        self.assertEqual(zoom.wide.PHYSICAL, original_reason)


if __name__ == '__main__':
    unittest.main()
