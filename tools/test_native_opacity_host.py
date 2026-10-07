#!/usr/bin/env python3
"""Pure log-oracle tests; no importer, subprocess, window, shader or GPU run."""
import copy
import json
from pathlib import Path
import unittest

import native_opacity_host as host


ADAPTER = ('presenter: Vulkan adapter="test" type=Cpu driver="none" format=Rgba8Unorm '
           'color_space=Srgb alpha=Opaque mode=Fifo; raster=native-shaders with complete CPU admission fallback')


def start(reference=True):
    result = host.Observation(reference)
    result.line(ADAPTER)
    result.line('native wide gate: document released for owned pid=1 size=1280x880 prefix_bytes=0 '
                + 'prefix_sha256=' + '0' * 64 + ' html_sha256=' + '0' * 64)
    return result


def scene(state, serial, numerator=256, reference=True, scratch=host.SCRATCH):
    flag = str(reference).lower()
    state.line(f'native scene prepared: serial={serial} snapshot_generation=1 page_commands=12 images=1 '
               f'loading=false phases=3 lowered_commands=40 rounded_masks=1 raster_invocations=1400000 '
               f'total_invocations=2526400 reference={flag}')
    state.line(f'native opacity prepared: serial={serial} groups=2 scratch_bytes={scratch} opacity_sum={numerator}')
    if serial == 1:
        state.line(f'presenter: native shader frame presented; size=1280x880 serial=1 CPU_upload=false reference={flag}')


def verified(state, serial):
    return state.line(f'presenter: native shader acquired-texture verified; serial={serial} generation=1 '
                      f'viewport_revision=1 size=1280x880 compared_bytes={host.PIXEL_BYTES} exact=true (before compositor)')


def summary(state, frames=4, serial=4):
    state.line(f'presenter: Vulkan acquired-texture verification passed; route=native-raster frames={frames} '
               f'compared_bytes={frames * host.PIXEL_BYTES} last_serial={serial} (before compositor)')


def normal_receipt():
    prefix = (ADAPTER + '\npresenter: native raster admission fallback; complete CPU upload; reason='
              + host.wide.PHYSICAL + '\n').encode()
    release = {'prefix_bytes': len(prefix), 'prefix_sha256': host.sha(prefix),
               'html_sha256': host.sha(host.HTML.read_bytes()), 'physical_size': list(host.SIZE)}
    marker = (f'native wide gate: document released for owned pid=71 size=1280x880 prefix_bytes={len(prefix)} '
              f'prefix_sha256={release["prefix_sha256"]} html_sha256={release["html_sha256"]}\n').encode()
    suffix = (f'native scene prepared: serial=1 snapshot_generation=1 page_commands=12 images=1 loading=false '
              f'phases=3 lowered_commands=40 rounded_masks=1 raster_invocations=1400000 total_invocations=2526400 reference=false\n'
              f'native opacity prepared: serial=1 groups=2 scratch_bytes={host.SCRATCH} opacity_sum=256\n'
              'presenter: native shader frame presented; size=1280x880 serial=1 CPU_upload=false reference=false\n').encode()
    data = prefix + marker + suffix
    state = host.Observation(False)
    for line in data.decode().splitlines():
        state.line(line)
    bodies = {'/admitted.html': host.HTML.read_bytes(), '/vulkan-opacity.png': host.PNG.read_bytes()}
    record = {'success': True, 'control_complete': True, 'browser_reaped': True, 'browser_returncode': 0,
              'controlled_size': list(host.SIZE), 'verified_mode': False, 'identity': {'pid': 71},
              'release': release, 'commands': [], 'escapes': [], 'accepted_connections': 2,
              'fixtures': {path: host.sha(body) for path, body in bodies.items()}, 'requests': [],
              'observations': json.loads(json.dumps(state.complete()))}
    with host.fixture_routes():
        for path, body in bodies.items():
            response = host.wide.response_for(path, bodies)
            record['requests'].append({'path': path, 'method': 'GET', 'header_bytes': 80,
                                       'body_bytes': len(body), 'body_sha256': host.sha(body),
                                       'response_complete': True, 'response_bytes': len(response),
                                       'response_sha256': host.sha(response)})
    return data, record


class OpacityLogTests(unittest.TestCase):
    def test_exact_worker_diagnostics_do_not_hide_page_errors(self):
        state = start()
        state.line('[page] Page process 71: Landlock ABI 6 and seccomp: no direct resource or socket access')
        state.line('[page] Resource broker 72: committed URL and fetch policy enforced outside renderer')
        for message in ['[page] TypeError: click failed', '[page] Page process stopped',
                        '[page] Page process 0: Landlock ABI 6 and seccomp: no direct resource or socket access']:
            with self.assertRaises(ValueError):
                state.line(message)

    def test_loading_resize_is_permitted_only_before_document_release(self):
        state = host.Observation(True)
        for size in ['1180x880', '842x1390', '839x1390', '852x1400', '1280x880']:
            state.line('presenter: native raster admission fallback; complete CPU upload; reason='
                       'native scene awaits a current loaded page; size=' + size)
        with self.assertRaises(ValueError):
            start().line('presenter: native raster admission fallback; complete CPU upload; reason='
                         'native scene awaits a current loaded page; size=1180x880')
        for reason in ['planner-budget; size=1180x880',
                       'native scene awaits a current loaded page; size=16385x880']:
            with self.assertRaises(ValueError):
                state.line('presenter: native raster admission fallback; complete CPU upload; reason=' + reason)

    def test_four_frames_include_both_states(self):
        state = start()
        for serial, phase in enumerate([256, 256, 320, 320], 1):
            scene(state, serial, phase)
            verified(state, serial)
        summary(state)
        self.assertEqual(state.complete()['compared_bytes'], 18_022_400)

    def test_normal_has_group_route_without_reference(self):
        state = start(False)
        scene(state, 1, reference=False)
        self.assertEqual(state.complete()['compared_bytes'], 0)

    def test_quota_without_changed_state_is_not_success(self):
        state = start()
        for serial in range(1, 5):
            scene(state, serial)
            verified(state, serial)
        summary(state)
        with self.assertRaises(ValueError):
            state.complete()

    def test_missing_or_zero_scratch_cannot_claim_route(self):
        state = start()
        with self.assertRaises(ValueError):
            scene(state, 1, scratch=0)
        with self.assertRaises(ValueError):
            verified(state, 1)

    def test_unknown_serial_and_duplicate_verification_refuse(self):
        state = start()
        with self.assertRaises(ValueError):
            verified(state, 1)
        scene(state, 1)
        verified(state, 1)
        with self.assertRaises(ValueError):
            verified(state, 1)

    def test_normal_reference_and_late_fallback_refuse(self):
        with self.assertRaises(ValueError):
            scene(start(False), 1)
        state = start(False)
        scene(state, 1, reference=False)
        with self.assertRaises(ValueError):
            state.line('presenter: native raster admission fallback; complete CPU upload; reason=planner-budget; size=1280x880')

    def test_changed_then_old_state_refuses(self):
        state = start()
        for serial, phase in enumerate([256, 320, 256, 320], 1):
            scene(state, serial, phase)
            verified(state, serial)
        summary(state)
        with self.assertRaises(ValueError):
            state.complete()

    def test_incomplete_summary_and_overquota_refuse(self):
        state = start()
        for serial, phase in enumerate([256, 320, 320, 320], 1):
            scene(state, serial, phase)
            verified(state, serial)
        with self.assertRaises(ValueError):
            summary(state, frames=3)
        scene(state, 5, 320)
        with self.assertRaises(ValueError):
            verified(state, 5)

    def test_reference_free_frames_after_quota_are_not_extra_verifications(self):
        state = start()
        for serial, phase in enumerate([256, 320, 320, 320], 1):
            scene(state, serial, phase)
            verified(state, serial)
        scene(state, 5, 320, reference=False)
        with self.assertRaises(ValueError):
            verified(state, 5)
        summary(state)
        self.assertEqual(len(state.complete()['verified']), 4)

    def test_complete_normal_receipt_preserves_fixed_response_bytes(self):
        data, record = normal_receipt()
        self.assertEqual(host.validate(b'', data, False, record, Path('/held/hyprctl'))['compared_bytes'], 0)

    def test_release_requires_latest_loading_size(self):
        data, record = normal_receipt()
        self.assertEqual(host.validate(b'', data, False, record, Path('/held/hyprctl'))['compared_bytes'], 0)
        offset = record['release']['prefix_bytes']
        marker, suffix = data[offset:].split(b'\n', 1)
        prefix = (data[:offset] + b'presenter: native raster admission fallback; complete CPU upload; reason='
                  b'native scene awaits a current loaded page; size=1180x880\n')
        record['release'].update(prefix_bytes=len(prefix), prefix_sha256=host.sha(prefix))
        marker = (f'native wide gate: document released for owned pid=71 size=1280x880 prefix_bytes={len(prefix)} '
                  f'prefix_sha256={host.sha(prefix)} html_sha256={record["release"]["html_sha256"]}').encode()
        with self.assertRaisesRegex(ValueError, 'delayed opacity document release binding'):
            host.validate(b'', prefix + marker + b'\n' + suffix, False, record, Path('/held/hyprctl'))

    def test_retained_response_changes_fail_closed(self):
        data, record = normal_receipt()
        for mutate in [lambda r: r['requests'].reverse(),
                       lambda r: r['requests'][1].update(response_complete=False),
                       lambda r: r['requests'][0].update(response_sha256='0' * 64),
                       lambda r: r['fixtures'].update({'/admitted.html': '0' * 64}),
                       lambda r: r.update(verified_mode=True)]:
            changed = copy.deepcopy(record)
            mutate(changed)
            with self.assertRaises(ValueError):
                host.validate(b'', data, False, changed, Path('/held/hyprctl'))


if __name__ == '__main__':
    unittest.main()
