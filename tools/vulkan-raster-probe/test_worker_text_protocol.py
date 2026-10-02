"""Synthetic EWB1/protocol tests; no workers, font rasterization or GPU calls.

Text/style/scope/image assumptions come from the frozen source fixtures. The
finite positions, mask counters and white reference bytes below are artificial
protocol inputs, never claimed as observed layout, font masks or pixel goldens.
"""
from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct
import tempfile
import unittest

import worker_text_protocol as protocol

ADAPTERS = [
    'ADAPTER 0 vendor=0x1234 device=0x1 type=DiscreteGpu backend=Vulkan name="synthetic" driver="fake" info="protocol-only"',
    'ADAPTER 1 vendor=0x0 device=0x2 type=Cpu backend=Vulkan name="synthetic-cpu" driver="fake" info="protocol-only"',
]


def u32(value):
    return struct.pack('<I', value)


def string(value):
    raw = value.encode('utf-8')
    return u32(len(raw)) + raw


def command_bytes(command):
    kind = command['kind']
    if kind == 'PushClip':
        return b'\0' + struct.pack('<4f', *command['rect'])
    if kind in ('PopClip', 'PushFixed', 'PopFixed', 'PopOpacity'):
        return bytes([{'PopClip': 1, 'PushFixed': 2, 'PopFixed': 3, 'PopOpacity': 5}[kind]])
    if kind == 'PushOpacity':
        return b'\4' + struct.pack('<f', command['opacity'])
    if kind == 'Rect':
        return (b'\6' + struct.pack('<4f', *command['rect']) + bytes(command['color'])
                + struct.pack('<f', command['radius']))
    if kind == 'Text':
        return (b'\7' + struct.pack('<3f', command['x'], command['y'], command['size'])
                + bytes(command['color']) + bytes(command['flags']) + string(command['text']))
    if kind == 'Image':
        return b'\10' + struct.pack('<4f', *command['rect']) + string(command['key'])
    if kind == 'Line':
        return (b'\11' + struct.pack('<5f', *(command[k] for k in ('x1', 'y1', 'x2', 'y2', 'width')))
                + bytes(command['color']))
    raise AssertionError('unknown synthetic command')


def snapshot_bytes(snapshot):
    result = (b'EWB1' + struct.pack('<IIQQBfd', snapshot['width'], snapshot['height'],
              snapshot['generation'], snapshot['edit'], snapshot['task'],
              snapshot['content_height'], snapshot['load_ms'])
              + string(snapshot['title']) + string(snapshot['url']))
    result += u32(len(snapshot['commands'])) + b''.join(map(command_bytes, snapshot['commands']))
    result += u32(len(snapshot['keys']))
    for key, source in snapshot['keys']:
        result += string(key) + u32(source)
    result += u32(len(snapshot['sources']))
    for width, height, rgba in snapshot['sources']:
        result += u32(width) + u32(height) + u32(len(rgba)) + bytes(rgba)
    return result + u32(len(snapshot['diagnostics'])) + b''.join(map(string, snapshot['diagnostics']))


def encoded(lines):
    return ('\n'.join(lines) + '\n').encode()


def field(line, name, value):
    changed, count = re.subn(rf'(?<!\S){re.escape(name)}=\S+', f'{name}={value}', line)
    if count != 1:
        raise AssertionError(f'expected one {name} field')
    return changed


class Transcript:
    def __init__(self, directory=protocol.DEFAULT_FIXTURES):
        self.directory = Path(directory).resolve()
        self.cases = json.loads((self.directory / 'inventory.json').read_bytes())['cases']
        self.snapshots = {}
        for case in self.cases:
            frame, request = case['frame'], case['request']
            commands, text_index, image_index, rect_index = [], 0, 0, 0
            for kind in case['drawing_order']:
                if kind == 'Rect':
                    commands.append({'kind': kind, 'rect': [0, 0, frame['width'], frame['height']],
                                     'color': [255] * 4, 'radius': 2 if rect_index else 0})
                    rect_index += 1
                elif kind == 'Text':
                    expected = case['texts'][text_index]
                    text_index += 1
                    # Scope state is independent of the decoder under test.
                    if expected['clipped']:
                        commands.append({'kind': 'PushClip', 'rect': [1, 25, 10, 10]})
                    if expected['fixed']:
                        # Reset a deliberately restrictive ancestor clip.
                        commands.extend([{'kind': 'PushClip', 'rect': [1, 25, 10, 10]},
                                         {'kind': 'PushFixed'}])
                    commands.append({'kind': kind, 'x': 2.25, 'y': 3.5,
                                     'size': expected['size'], 'color': expected['rgba'].copy(),
                                     'flags': [int(expected[k]) for k in ('bold', 'italic', 'monospace')],
                                     'text': expected['text']})
                    if expected['fixed']:
                        commands.extend([{'kind': 'PopFixed'}, {'kind': 'PopClip'}])
                    if expected['clipped']:
                        commands.append({'kind': 'PopClip'})
                elif kind == 'Image':
                    commands.append({'kind': kind, 'rect': [image_index, 1, 2, 1],
                                     'key': case['images'][0]['key']})
                    image_index += 1
                elif kind == 'Line':
                    commands.append({'kind': kind, 'x1': 1.25, 'y1': 20.5, 'x2': 12.25,
                                     'y2': 20.5, 'width': 1, 'color': [0, 0, 128, 128]})
                else:
                    raise AssertionError('unexpected fixture primitive')
            sources = sorted(case['images'], key=lambda image: image['key'])
            self.snapshots[case['name']] = {
                'width': frame['width'], 'height': frame['height'],
                'generation': request['generation'], 'edit': 0, 'task': 0,
                'content_height': frame['height'], 'load_ms': 0.25,
                'title': request['expected_title'],
                'url': (self.directory / case['html_file']).as_uri(),
                'commands': commands, 'keys': [(image['key'], i) for i, image in enumerate(sources)],
                'sources': [(image['width'], image['height'], image['rgba'].copy()) for image in sources],
                'diagnostics': ['synthetic isolation status; not a captured worker'],
            }

    def snapshot_line(self, case):
        data = snapshot_bytes(self.snapshots[case['name']])
        return f"SNAPSHOT {case['name']} bytes={len(data)} hex={data.hex()}"

    def ready_line(self, case):
        snapshot = self.snapshots[case['name']]
        commands = snapshot['commands']
        texts = [c for c in commands if c['kind'] == 'Text']
        area = snapshot['width'] * snapshot['height']
        scalars = sum(len(c['text']) for c in texts)
        counts = dict(width=snapshot['width'], height=snapshot['height'], commands=len(commands),
                      image_entries=len(snapshot['keys']), text_runs=len(texts),
                      bytes=sum(len(c['text'].encode()) for c in texts), scalars=scalars)
        fallback = case['expected_route'] == 'cpu-fallback'
        index = next((i for i, c in enumerate(commands) if c['kind'] == 'Rect' and c['radius']), None)
        if fallback:
            metadata = {key: 'none' for key in protocol.PLAN_FIELDS}
        else:
            nontext = len(commands) - len(texts)
            # Artificial one-cell masks and finite work/storage counters. This
            # exercises relations and caps, not actual bundled-font metrics.
            masks = len({(c['flags'][2], c['flags'][1], c['flags'][0], c['size'], char)
                         for c in texts for char in c['text']})
            image_count = sum(c['kind'] == 'Image' for c in commands)
            rgba = sum(w * h * 4 for w, h, _ in snapshot['sources'])
            lowered = nontext + scalars
            draws = 1 + scalars + sum(c['kind'] in ('Rect', 'Image', 'Line') for c in commands)
            upper = area * 8 + rgba + 4 * (masks + scalars + image_count *
                    (snapshot['width'] + snapshot['height'])) + 256 * (lowered + 1)
            metadata = dict(lowered_commands=lowered, unique_sources=len(snapshot['sources']),
                            referenced_sources=len(snapshot['sources']), total_rgba_bytes=rgba,
                            referenced_rgba_bytes=rgba, missing_images=0, occurrences=scalars,
                            masks=masks, coverage=masks, cold_work=masks, scratch=20, rows=scalars,
                            cpu_upper=nontext * area + scalars, gpu_upper=upper, draws=draws,
                            invocations=((snapshot['width'] + 7) // 8) *
                                        ((snapshot['height'] + 7) // 8) * 64 + (draws - 1) * 64,
                            gpu_buffers=area * 8 + draws * 256 + rgba + 4 * (masks + scalars))
        base = ' '.join(f'{key}={counts[key]}' for key in protocol.BASE_FIELDS)
        plan = ' '.join(f'{key}={metadata[key]}' for key in protocol.PLAN_FIELDS)
        return (f"TEXT_READY {case['name']} path={case['expected_route']} "
                f"reason={case['expected_fallback'] or 'none'} command_index={index if fallback else 'none'} "
                f'{base} {plan} reference_bytes={area * 4} cpu_passes=2 cold_warm=true '
                f'reference=canvas-snapshot pixels_le_hex={"ffffff00" * area}')

    def preparation(self):
        lines = []
        for case in self.cases:
            lines.extend([self.snapshot_line(case), self.ready_line(case)])
        return lines + ['TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=1 owned_children=0']

    def cpu_lines(self):
        return self.preparation() + ['TEXT_CPU_COMPLETE cases=7 gpu_admitted=6 fallback=1 cpu_passes=14 reference_bytes=358400 gpu_comparisons=0 differential=true']

    def gpu_lines(self, adapters=ADAPTERS, index=0):
        return (self.preparation() + adapters +
                [f"TEXT_PASS {c['name']} width=160 height=80 compared_bytes=51200 exact=true"
                 for c in self.cases if c['expected_route'] == 'gpu'] +
                [f'WORKER_TEXT_COMPLETE adapter={index} cases=7 gpu=6 fallback=1 gpu_compared_bytes=307200 fallback_reference_bytes=51200 differential=true custom_wgsl=true'])


class WorkerTextProtocolTests(unittest.TestCase):
    def setUp(self):
        self.transcript = Transcript()

    def cpu(self, lines):
        return protocol.validate_cpu(encoded(lines), self.transcript.directory)

    def gpu(self, lines):
        return protocol.validate_run(encoded(lines), ADAPTERS, 0, self.transcript.directory)

    def reject_cpu(self, lines):
        with self.assertRaises(ValueError):
            self.cpu(lines)

    def test_complete_cpu_and_gpu_transcripts_retain_identity_and_differential_scope(self):
        cpu = self.cpu(self.transcript.cpu_lines())
        gpu = self.gpu(self.transcript.gpu_lines())
        for result in (cpu, gpu):
            self.assertEqual((result['cases'], result['worker_cases'], result['gpu_admitted'], result['fallback']), (7, 7, 6, 1))
            self.assertEqual(result['cpu_passes'], 14)
            self.assertEqual(result['reference_bytes'], 358400)
            self.assertEqual(result['cpu_compared_bytes'], 716800)
            self.assertIn('not independent golden', result['reference_kind'])
            for case, row in zip(self.transcript.cases, result['frames']):
                data = snapshot_bytes(self.transcript.snapshots[case['name']])
                self.assertEqual(row['snapshot_sha256'], hashlib.sha256(data).hexdigest())
                self.assertEqual(row['snapshot_bytes'], len(data))
                self.assertEqual(row['diagnostics'], self.transcript.snapshots[case['name']]['diagnostics'])
        self.assertEqual(cpu['gpu_comparisons'], 0)
        self.assertEqual((gpu['gpu_comparisons'], gpu['gpu_compared_bytes'], gpu['fallback_reference_bytes']), (6, 307200, 51200))

    def test_every_snapshot_ready_pair_is_required_once_in_frozen_order(self):
        original = self.transcript.cpu_lines()
        for i in range(14):
            with self.subTest(record=i, mutation='omit'):
                self.reject_cpu(original[:i] + original[i + 1:])
            with self.subTest(record=i, mutation='duplicate'):
                self.reject_cpu(original[:i] + [original[i]] + original[i:])
            swapped = original.copy()
            j = (i + 2) % 14
            swapped[i], swapped[j] = swapped[j], swapped[i]
            with self.subTest(record=i, mutation='swap'):
                self.reject_cpu(swapped)

    def test_cpu_gpu_and_other_suite_phases_cannot_be_substituted(self):
        original = self.transcript.cpu_lines()
        for line in ('CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks=1 owned_children=0',
                     'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=0 owned_children=0',
                     'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=65 owned_children=0',
                     'TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks=1 owned_children=1'):
            changed = original.copy(); changed[14] = line
            self.reject_cpu(changed)
        self.reject_cpu(original[:-1])
        self.reject_cpu(original + [original[-1]])
        self.reject_cpu(self.transcript.gpu_lines())
        with self.assertRaises(ValueError):
            self.gpu(original)
        changed = self.transcript.gpu_lines()
        changed[14], changed[15] = changed[15], changed[14]
        with self.assertRaises(ValueError):
            self.gpu(changed)

    def test_snapshot_bytes_require_exact_length_magic_and_full_consumption(self):
        case = self.transcript.cases[0]
        raw = snapshot_bytes(self.transcript.snapshots[case['name']])
        for data in (b'NOPE' + raw[4:], raw[:-1], raw + b'\0', b'EWB1', raw[:32]):
            changed = self.transcript.cpu_lines()
            changed[0] = f"SNAPSHOT {case['name']} bytes={len(data)} hex={data.hex()}"
            with self.subTest(length=len(data)):
                self.reject_cpu(changed)
        changed = self.transcript.cpu_lines()
        changed[0] = field(changed[0], 'bytes', len(raw) + 1)
        self.reject_cpu(changed)

    def test_snapshot_request_metadata_and_finite_values_are_checked(self):
        base = copy.deepcopy(self.transcript)
        name = self.transcript.cases[0]['name']
        for key, value in [('generation', 0), ('edit', 1), ('task', 1), ('width', 159),
                           ('title', 'different'), ('url', 'file:///other'),
                           ('content_height', float('nan')), ('load_ms', float('inf'))]:
            self.transcript = copy.deepcopy(base)
            self.transcript.snapshots[name][key] = value
            with self.subTest(key=key):
                self.reject_cpu(self.transcript.cpu_lines())

    def test_every_text_content_style_color_and_boolean_flag_is_frozen(self):
        base = copy.deepcopy(self.transcript)
        name = self.transcript.cases[1]['name']
        for key, value in [('text', 'different'), ('size', 17), ('color', [0, 0, 0, 255]),
                           ('flags', [1, 0, 0]), ('flags', [2, 0, 0]), ('x', float('nan'))]:
            self.transcript = copy.deepcopy(base)
            command = next(c for c in self.transcript.snapshots[name]['commands'] if c['kind'] == 'Text')
            command[key] = value
            with self.subTest(key=key, value=value):
                self.reject_cpu(self.transcript.cpu_lines())

    def test_finite_dynamic_text_origins_and_diagnostics_are_observations(self):
        name = self.transcript.cases[0]['name']
        snapshot = self.transcript.snapshots[name]
        next(c for c in snapshot['commands'] if c['kind'] == 'Text').update(x=45.125, y=33.5)
        snapshot['diagnostics'] = ['sandbox isolation active', 'synthetic informational status']
        result = self.cpu(self.transcript.cpu_lines())
        self.assertEqual(result['frames'][0]['diagnostics'], snapshot['diagnostics'])

    def test_fixed_clip_restoration_and_typed_balanced_scopes(self):
        base = copy.deepcopy(self.transcript)
        name = self.transcript.cases[3]['name']
        for mutation in ('remove-fixed', 'unrestrictive', 'mismatched-pop', 'unclosed', 'too-deep'):
            self.transcript = copy.deepcopy(base)
            commands = self.transcript.snapshots[name]['commands']
            if mutation == 'remove-fixed':
                commands[:] = [c for c in commands if c['kind'] not in ('PushFixed', 'PopFixed')]
            elif mutation == 'unrestrictive':
                next(c for c in commands if c['kind'] == 'PushClip')['rect'] = [0, 24, 160, 80]
            elif mutation == 'mismatched-pop':
                next(c for c in commands if c['kind'] == 'PopFixed')['kind'] = 'PopClip'
            elif mutation == 'unclosed':
                commands.append({'kind': 'PushFixed'})
            else:
                commands[:0] = [{'kind': 'PushFixed'}] * 33
                commands.extend([{'kind': 'PopFixed'}] * 33)
            with self.subTest(mutation=mutation):
                self.reject_cpu(self.transcript.cpu_lines())

    def test_text_image_line_paint_order_is_not_filtered_or_rearranged(self):
        base = copy.deepcopy(self.transcript)
        for case_index in (4, 5):
            self.transcript = copy.deepcopy(base)
            commands = self.transcript.snapshots[self.transcript.cases[case_index]['name']]['commands']
            commands[1], commands[2] = commands[2], commands[1]
            self.reject_cpu(self.transcript.cpu_lines())

    def test_exact_decoded_images_dimensions_keys_bytes_and_references(self):
        base = copy.deepcopy(self.transcript)
        name = self.transcript.cases[4]['name']
        for mutation in ('rgba', 'width', 'key', 'missing-source', 'reference', 'source-id'):
            self.transcript = copy.deepcopy(base)
            snapshot = self.transcript.snapshots[name]
            if mutation == 'rgba':
                snapshot['sources'][0][2][7] = 255
            elif mutation == 'width':
                snapshot['sources'][0] = (1, 2, snapshot['sources'][0][2])
            elif mutation == 'key':
                snapshot['keys'][0] = ('other.png', 0)
            elif mutation == 'missing-source':
                snapshot['sources'].clear()
            elif mutation == 'source-id':
                snapshot['keys'][0] = ('two-pixels.png', 1)
            else:
                next(c for c in snapshot['commands'] if c['kind'] == 'Image')['key'] = 'absent.png'
            with self.subTest(mutation=mutation):
                self.reject_cpu(self.transcript.cpu_lines())

    def test_fallback_cannot_claim_partial_plan_or_point_to_later_text(self):
        original = self.transcript.cpu_lines()
        for key, value in [('path', 'gpu'), ('reason', 'none'), ('command_index', 'none'),
                           ('command_index', 2), ('command_index', 999), ('masks', 0),
                           ('draws', 1), ('gpu_buffers', 102656)]:
            changed = original.copy(); changed[13] = field(changed[13], key, value)
            with self.subTest(key=key):
                self.reject_cpu(changed)

    def test_snapshot_derived_counts_utf8_and_referenced_image_metadata(self):
        original = self.transcript.cpu_lines()
        for index, key, value in [(1, 'commands', 0), (1, 'text_runs', 1),
                                  (1, 'bytes', 8), (1, 'scalars', 9),
                                  (9, 'image_entries', 0), (9, 'unique_sources', 0),
                                  (9, 'referenced_sources', 0), (9, 'total_rgba_bytes', 7),
                                  (9, 'referenced_rgba_bytes', 0), (9, 'missing_images', 1)]:
            changed = original.copy(); changed[index] = field(changed[index], key, value)
            with self.subTest(key=key):
                self.reject_cpu(changed)

    def test_plan_limits_and_cold_metadata_cannot_contradict_each_other(self):
        original = self.transcript.cpu_lines()
        changes = [('masks', 0), ('coverage', 262145), ('cold_work', 0),
                   ('scratch', 3), ('rows', 65537), ('lowered_commands', 257),
                   ('occurrences', 4097), ('cpu_upper', 1), ('gpu_upper', 1048577),
                   ('gpu_upper', 1048576), ('gpu_buffers', 1), ('draws', 258),
                   ('invocations', 65), ('invocations', 4000064), ('masks', 'none')]
        for key, value in changes:
            changed = original.copy(); changed[1] = field(changed[1], key, value)
            with self.subTest(key=key, value=value):
                self.reject_cpu(changed)

    def test_reference_byte_counts_high_byte_and_reference_kind_are_strict(self):
        original = self.transcript.cpu_lines()
        for key, value in [('reference_bytes', 51199), ('cpu_passes', 1), ('cold_warm', 'false'),
                           ('reference', 'independent-golden'), ('pixels_le_hex', 'ffffff01' * 12800),
                           ('pixels_le_hex', 'ffffff00' * 12799)]:
            changed = original.copy(); changed[1] = field(changed[1], key, value)
            with self.subTest(key=key):
                self.reject_cpu(changed)
        changed = original.copy(); changed[1] = field(changed[1], 'pixels_le_hex', '12345600' * 12800)
        self.assertNotEqual(self.cpu(original)['frames'][0]['reference_sha256'],
                            self.cpu(changed)['frames'][0]['reference_sha256'])

    def test_cpu_footer_counts_are_exact_and_never_gpu_comparisons(self):
        original = self.transcript.cpu_lines()
        for key, value in [('cases', 6), ('gpu_admitted', 7), ('fallback', 0),
                           ('cpu_passes', 7), ('reference_bytes', 307200),
                           ('gpu_comparisons', 6), ('differential', 'false')]:
            changed = original.copy(); changed[-1] = field(changed[-1], key, value)
            with self.subTest(key=key):
                self.reject_cpu(changed)

    def test_gpu_pass_order_adapter_inventory_and_totals(self):
        original = self.transcript.gpu_lines()
        variants = [original[:-2] + original[-1:], original + [original[-1]],
                    original[:15] + ADAPTERS[::-1] + original[17:]]
        swapped = original.copy(); swapped[17], swapped[18] = swapped[18], swapped[17]
        variants.append(swapped)
        for key, value in [('adapter', 1), ('gpu_compared_bytes', 358400),
                           ('fallback_reference_bytes', 0), ('custom_wgsl', 'false')]:
            changed = original.copy(); changed[-1] = field(changed[-1], key, value)
            variants.append(changed)
        for key, value in [('compared_bytes', 51204), ('width', 159), ('exact', 'false')]:
            changed = original.copy(); changed[17] = field(changed[17], key, value)
            variants.append(changed)
        for changed in variants:
            with self.subTest(index=variants.index(changed)), self.assertRaises(ValueError):
                self.gpu(changed)
        for index in (-1, 2):
            with self.assertRaises(ValueError):
                protocol.validate_run(encoded(original), ADAPTERS, index)

    def test_stdout_framing_encoding_bounds_and_noncanonical_fields(self):
        data = encoded(self.transcript.cpu_lines())
        for bad in (b'', data[:-1], data + b'\n', data.replace(b'\n', b'\r\n'),
                    data + b'\0', b'\xff\n', b'x' * (2 * 1024 * 1024 + 1),
                    b'x' * 150001 + b'\n', b'x\n' * 65):
            with self.subTest(length=len(bad)), self.assertRaises(ValueError):
                protocol.validate_cpu(bad)
        for value in ('0160', '+160', '160.0', '-1', '10000000000'):
            changed = self.transcript.cpu_lines(); changed[1] = field(changed[1], 'width', value)
            self.reject_cpu(changed)

    def test_fixture_hash_binding_rejects_input_substitution_and_missing_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / 'fixtures'
            shutil.copytree(self.transcript.directory, directory)
            self.assertEqual(protocol.load_fixtures(directory)['binding'],
                             protocol.load_fixtures(self.transcript.directory)['binding'])
            path = directory / 'inventory.json'
            original = path.read_bytes()
            path.write_bytes(original + b' ')
            with self.assertRaises(ValueError):
                protocol.load_fixtures(directory)
            path.write_bytes(original)
            path.unlink()
            with self.assertRaises(OSError):
                protocol.load_fixtures(directory)
            freeze = directory / 'freeze.json'
            freeze.write_bytes(freeze.read_bytes() + b' ')
            with self.assertRaises(ValueError):
                protocol.load_fixtures(directory)


if __name__ == '__main__':
    unittest.main()
