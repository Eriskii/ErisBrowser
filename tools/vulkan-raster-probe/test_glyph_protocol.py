"""Synthetic transcript/binding checks only: no child, renderer, font or Vulkan calls."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

import glyph_protocol as protocol

FIXTURES = protocol.DEFAULT_FIXTURES
ADAPTERS = [
    'ADAPTER 0 vendor=0x1234 device=0x1 type=DiscreteGpu backend=Vulkan name="synthetic" driver="fake" info="protocol-only"',
    'ADAPTER 1 vendor=0x0 device=0x2 type=Cpu backend=Vulkan name="synthetic-cpu" driver="fake" info="protocol-only"',
]


def encoded(lines):
    return ('\n'.join(lines) + '\n').encode()


def replace_field(line, name, value):
    words = line.split(' ')
    matches = [i for i, word in enumerate(words) if word.startswith(name + '=')]
    if len(matches) != 1:
        raise AssertionError('synthetic mutation target is not unique')
    words[matches[0]] = name + '=' + str(value)
    return ' '.join(words)


class Transcript:
    """Deliberately invented in-bound counters, not glyph observations or goldens."""
    def __init__(self):
        literal = json.loads((FIXTURES / 'literal-fixtures.json').read_text())['pixel_cases']
        font = json.loads((FIXTURES / 'font-inventory.json').read_text())['cases']
        self.cases = [(c, 'literal-mask') for c in literal] + [(c, 'font-reference') for c in font]
        self.meta = []
        for case in font:
            text = [c['text'] for c in case['commands'] if c['kind'] == 'Text']
            count = sum(len(t) for t in text)
            area = case['frame']['width'] * case['frame']['height']
            cpu = area * sum(c['kind'] != 'Text' for c in case['commands']) + 1
            self.meta.append(f"FONT_META {case['name']} bytes={sum(len(t.encode('utf-8')) for t in text)} scalars={count} visited=1 occurrences=1 masks=1 coverage=1 cold_work=1 scratch=20 rows=1 cpu_upper={cpu} gpu_upper={area * 8 + 256} cold_warm=true")
        self.ready, self.passes = [], []
        self.storage = self.compared = 0
        for case, population in self.cases:
            name = case['name']; w = case['frame']['width']; h = case['frame']['height']
            size = w * h * 4
            work = ((w + 7) // 8) * ((h + 7) // 8) * 64
            storage = size * 2 + 256
            self.storage += storage; self.compared += size
            self.ready.append(f'GLYPH_READY {name} population={population} width={w} height={h} compared_bytes={size} draws=1 invocations={work} gpu_buffers={storage}')
            self.passes.append(f'GLYPH_PASS {name} population={population} width={w} height={h} compared_bytes={size} exact=true')
        self.prepared = f'GLYPH_PREPARED cases=26 literal=12 font=14 retained_gpu_bytes={self.storage} retained_reference_bytes={self.compared} worker_cases=0'

    def preparation(self):
        return self.meta + self.ready + [self.prepared]

    def gpu(self, adapters=ADAPTERS, index=0):
        return self.preparation() + adapters + self.passes + [f'GLYPH_COMPLETE adapter={index} cases=26 literal=12 font=14 exact=true custom_wgsl=true']

    def cpu(self):
        return self.preparation() + ['GLYPH_CPU_COMPLETE cases=26 literal=12 font=14 cpu_passes=28 exact=true']


def baseline(directory):
    cases = json.loads((FIXTURES / 'font-inventory.json').read_text())['cases']
    rows = []
    for case in cases:
        data = bytes(case['frame']['width'] * case['frame']['height'] * 4)
        name = case['name'] + '.rgb'
        (directory / name).write_bytes(data)
        rows.append({'path': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    manifest = {'schema': 1, 'parent': protocol.PARENT, 'encoding': 'u32le-00RRGGBB',
                'cases': 14, 'bytes': 344064, 'files': rows}
    return manifest, save_manifest(directory, manifest)


def save_manifest(directory, manifest):
    data = (json.dumps(manifest, indent=2) + '\n').encode()
    (directory / 'freeze.json').write_bytes(data)
    return hashlib.sha256(data).hexdigest()


class GlyphProtocolTests(unittest.TestCase):
    def setUp(self):
        self.transcript = Transcript()

    def bad(self, lines):
        with self.assertRaises(ValueError):
            protocol.validate_run(encoded(lines), ADAPTERS, 0, FIXTURES)

    def test_complete_gpu_and_cpu_populations_remain_distinct(self):
        result = protocol.validate_run(encoded(self.transcript.gpu()), ADAPTERS, 0, FIXTURES)
        self.assertEqual((result['literal_cases'], result['font_cases'], result['compared_bytes']), (12, 14, 344956))
        self.assertEqual(result['gpu_comparisons'], 26)
        self.assertIn('observed bounded', result['metadata'][0]['claim'])
        cpu = protocol.validate_cpu(encoded(self.transcript.cpu()), FIXTURES)
        self.assertEqual((cpu['cpu_font_passes'], cpu['gpu_comparisons']), (28, 0))
        self.assertEqual((cpu['cpu_font_compared_bytes'], cpu['literal_cpu_comparisons']), (688128, 0))
        self.assertEqual(cpu['retained_reference_bytes'], 344956)

    def test_listing_is_exact_ordered_and_portable(self):
        self.assertEqual(protocol.validate_listing(encoded(ADAPTERS)), ADAPTERS)
        self.assertEqual(protocol.validate_listing(encoded(ADAPTERS[:1])), ADAPTERS[:1])
        devices = [ADAPTERS[0].replace('ADAPTER 0 ', f'ADAPTER {i} ') for i in range(16)]
        self.assertEqual(protocol.validate_listing(encoded(devices)), devices)
        for bad in [[], ADAPTERS[::-1], ADAPTERS + ADAPTERS[:1], ADAPTERS + ['extra'], devices + [devices[-1].replace('15 ', '16 ')]]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                protocol.validate_listing(encoded(bad))
        one = protocol.validate_run(encoded(self.transcript.gpu(ADAPTERS[:1])), ADAPTERS[:1], 0, FIXTURES)
        self.assertEqual(one['adapter'], 0)

    def test_each_font_metadata_omission_and_duplicate_fails(self):
        original = self.transcript.gpu()
        for i in range(14):
            with self.subTest(i=i):
                self.bad(original[:i] + original[i + 1:])
                self.bad(original[:i] + [original[i]] + original[i:])
        lines = original.copy(); lines[0], lines[1] = lines[1], lines[0]; self.bad(lines)

    def test_each_prepared_frame_omission_and_duplicate_fails(self):
        original = self.transcript.gpu()
        for i in range(14, 40):
            with self.subTest(i=i):
                self.bad(original[:i] + original[i + 1:])
                self.bad(original[:i] + [original[i]] + original[i:])
        lines = original.copy(); lines[14], lines[15] = lines[15], lines[14]; self.bad(lines)

    def test_each_gpu_pass_omission_duplicate_and_identity_fails(self):
        original = self.transcript.gpu()
        for i in range(43, 69):
            with self.subTest(i=i):
                self.bad(original[:i] + original[i + 1:])
                self.bad(original[:i] + [original[i]] + original[i:])
                lines = original.copy(); lines[i] = replace_field(lines[i], 'compared_bytes', 0); self.bad(lines)
        lines = original.copy(); lines[43], lines[44] = lines[44], lines[43]; self.bad(lines)

    def test_phase_order_and_cpu_gpu_completion_are_not_interchangeable(self):
        self.bad(self.transcript.cpu())
        with self.assertRaises(ValueError):
            protocol.validate_cpu(encoded(self.transcript.gpu()), FIXTURES)
        lines = self.transcript.gpu(); self.bad(ADAPTERS + lines[:41] + lines[43:])
        lines = self.transcript.gpu(); lines[41] = ADAPTERS[1]; self.bad(lines)
        self.bad(self.transcript.gpu(index=1))

    def test_utf8_byte_and_scalar_counts_come_from_inputs(self):
        for field in ('bytes', 'scalars'):
            lines = self.transcript.gpu(); lines[5] = replace_field(lines[5], field, 1)
            self.bad(lines)
        lines = self.transcript.gpu(); lines[0] = lines[0].replace('font-regular-kern-av-to', 'font-bold-translucent'); self.bad(lines)

    def test_metadata_limits_and_accounting_relations(self):
        mutations = [('visited', 4097), ('occurrences', 2), ('masks', 257), ('coverage', 262145),
                     ('cold_work', 0), ('cold_work', 3), ('scratch', 1048593), ('scratch', 3),
                     ('rows', 65537), ('rows', 1025), ('cpu_upper', 0), ('cpu_upper', 1000001),
                     ('gpu_upper', 0), ('gpu_upper', 1048577), ('cold_warm', 'false')]
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                lines = self.transcript.gpu(); lines[0] = replace_field(lines[0], field, value); self.bad(lines)

    def test_frame_sizes_and_dynamic_gpu_bounds(self):
        mutations = [('width', 7), ('height', 0), ('compared_bytes', 0), ('draws', 0), ('draws', 258),
                     ('invocations', 0), ('invocations', 65), ('invocations', 4000064),
                     ('gpu_buffers', 0), ('gpu_buffers', 1048580), ('gpu_buffers', 449)]
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                lines = self.transcript.gpu(); lines[14] = replace_field(lines[14], field, value); self.bad(lines)
        lines = self.transcript.gpu(); lines[26] = replace_field(lines[26], 'gpu_buffers', 49412); self.bad(lines)

    def test_prepared_and_final_summaries_must_match_all_rows(self):
        for field, value in [('cases', 25), ('literal', 11), ('font', 13), ('worker_cases', 1),
                             ('retained_gpu_bytes', self.transcript.storage + 4), ('retained_reference_bytes', 344955)]:
            lines = self.transcript.gpu(); lines[40] = replace_field(lines[40], field, value); self.bad(lines)
        for field, value in [('cases', 25), ('literal', 13), ('font', 12), ('exact', 'false'), ('custom_wgsl', 'false')]:
            lines = self.transcript.gpu(); lines[-1] = replace_field(lines[-1], field, value); self.bad(lines)
        lines = self.transcript.cpu(); lines[-1] = replace_field(lines[-1], 'cpu_passes', 27)
        with self.assertRaises(ValueError):
            protocol.validate_cpu(encoded(lines), FIXTURES)

    def test_noncanonical_truncated_and_extra_bytes_fail(self):
        raw = encoded(self.transcript.gpu())
        for data in [raw[:-1], raw + b'\n', raw + b'extra\n', b'\xff\n' + raw,
                     raw.replace(b'\n', b'\r\n'), b'x' * (protocol.MAX_OUTPUT + 1)]:
            with self.subTest(size=len(data)), self.assertRaises(ValueError):
                protocol.validate_run(data, ADAPTERS, 0, FIXTURES)
        lines = self.transcript.gpu(); lines[0] = replace_field(lines[0], 'bytes', '0008'); self.bad(lines)

    def test_fixture_json_bytes_are_pinned(self):
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td)
            for name in ('literal-fixtures.json', 'font-inventory.json'):
                shutil.copyfile(FIXTURES / name, directory / name)
            self.assertEqual(len(protocol.load_fixtures(directory)['cases']), 26)
            with (directory / 'font-inventory.json').open('ab') as out:
                out.write(b' ')
            with self.assertRaises(ValueError):
                protocol.load_fixtures(directory)

    def test_parent_baseline_valid_and_manifest_pin_required(self):
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td); _, pin = baseline(directory)
            self.assertEqual(protocol.check_baselines(directory, pin, FIXTURES)['bytes'], 344064)
            for wrong in ('', '0' * 64, pin[:-1]):
                with self.assertRaises(ValueError):
                    protocol.check_baselines(directory, wrong, FIXTURES)

    def test_parent_baseline_all_files_and_lengths_are_bound(self):
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td); manifest, pin = baseline(directory)
            name = manifest['files'][0]['path']; original = (directory / name).read_bytes()
            for data in (original[:-1], original + b'\0', b'\1' + original[1:]):
                (directory / name).write_bytes(data)
                with self.assertRaises(ValueError):
                    protocol.check_baselines(directory, pin, FIXTURES)
            (directory / name).write_bytes(original)
            manifest['files'][1] = manifest['files'][0].copy()
            with self.assertRaises(ValueError):
                protocol.check_baselines(directory, save_manifest(directory, manifest), FIXTURES)

    def test_parent_baseline_semantic_identity_encoding_and_high_byte(self):
        for field, value in [('parent', '0' * 40), ('encoding', 'RGB24'), ('cases', 13), ('bytes', 344063)]:
            with tempfile.TemporaryDirectory() as td, self.subTest(field=field):
                directory = Path(td); manifest, _ = baseline(directory); manifest[field] = value
                with self.assertRaises(ValueError):
                    protocol.check_baselines(directory, save_manifest(directory, manifest), FIXTURES)
        with tempfile.TemporaryDirectory() as td:
            directory = Path(td); manifest, _ = baseline(directory); row = manifest['files'][0]
            data = bytearray((directory / row['path']).read_bytes()); data[3] = 1
            (directory / row['path']).write_bytes(data); row['sha256'] = hashlib.sha256(data).hexdigest()
            with self.assertRaises(ValueError):
                protocol.check_baselines(directory, save_manifest(directory, manifest), FIXTURES)

    def test_parent_baseline_path_escape_and_symlink_refused(self):
        with tempfile.TemporaryDirectory() as td, tempfile.TemporaryDirectory() as outside:
            directory = Path(td); manifest, pin = baseline(directory); row = manifest['files'][0]
            path = directory / row['path']; external = Path(outside) / row['path']; path.rename(external); path.symlink_to(external)
            with self.assertRaises(ValueError):
                protocol.check_baselines(directory, pin, FIXTURES)
            row['path'] = '../escape.rgb'
            with self.assertRaises(ValueError):
                protocol.check_baselines(directory, save_manifest(directory, manifest), FIXTURES)


if __name__ == '__main__':
    unittest.main()
