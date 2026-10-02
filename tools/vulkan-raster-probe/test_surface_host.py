"""Synthetic protocol/host tests; these are never GPU rendering evidence."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import run_surface_host as host

ADAPTERS = ['ADAPTER 0 vendor=0x10de device=0x1 type=DiscreteGpu backend=Vulkan name="synthetic" driver="test" info="test"']


def synthetic():
    lines = list(ADAPTERS)
    total = 0
    for name, (width, height, reference) in host.CASES.items():
        conversion = ((width + 7) // 8) * ((height + 7) // 8) * 64
        padded = ((width * 4 + 255) // 256) * 256 * height
        planned = width * height * 4 + padded + 16 + 256
        for fmt in host.FORMATS:
            lines.append(f'PASS {name} format={fmt} width={width} height={height} draws=1 '
                         f'raster_invocations={conversion} conversion_invocations={conversion} '
                         f'planned_bytes={planned} compared_bytes={width * height * 4} reference={reference} exact=true')
            total += width * height * 4
    lines.append(f'COMPLETE adapter=0 cases=11 formats=2 compared_bytes={total} '
                 'offscreen=true acquired_surface=false exact=true')
    return '\n'.join(lines) + '\n'


class SurfaceProtocol(unittest.TestCase):
    def test_complete_synthetic_population_is_attributed_separately(self):
        result = host.validate_run(synthetic(), ADAPTERS, 0)
        self.assertEqual(result, host.validate_run(synthetic().encode(), ADAPTERS, 0))
        self.assertEqual(result['cases'], 11)
        self.assertEqual(result['formats'], 2)
        self.assertEqual(result['Canvas_differential_bytes'], 1180 * 880 * 8)
        self.assertEqual(result['independent_literal_bytes'] + result['Canvas_differential_bytes'],
                         result['gpu_compared_bytes'])
        self.assertFalse(result['acquired_surface'])

    def test_missing_duplicate_or_unknown_case_is_refused(self):
        lines = synthetic().splitlines()
        candidates = ['\n'.join(lines[:1] + lines[2:]),
                      '\n'.join(lines[:2] + [lines[1]] + lines[3:]),
                      synthetic().replace('round-corner', 'unknown')]
        for candidate in candidates:
            with self.subTest(candidate=candidate[:60]), self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)

    def test_metadata_completion_and_inventory_mutations_are_refused(self):
        changes = [('width=2', 'width=3'), ('compared_bytes=16', 'compared_bytes=15'),
                   ('reference=literal-coverage', 'reference=Canvas-differential'),
                   ('conversion_invocations=64', 'conversion_invocations=0'),
                   ('raster_invocations=64', 'raster_invocations=4000000'),
                   ('planned_bytes=800', 'planned_bytes=1'),
                   ('draws=1', 'draws=258'), ('exact=true', 'exact=false'),
                   ('acquired_surface=false', 'acquired_surface=true'),
                   ('adapter=0 cases', 'adapter=1 cases'),
                   ('name="synthetic"', 'name="changed"')]
        for old, new in changes:
            candidate = synthetic().replace(old, new, 1)
            self.assertNotEqual(candidate, synthetic(), old)
            with self.subTest(old=old), self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)

    def test_extra_output_and_unrecognized_formats_are_refused(self):
        for candidate in [synthetic() + 'debug\n', synthetic().replace('Bgra8Unorm', 'Bgra8UnormSrgb'), b'\xff']:
            with self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)

    def test_invalid_timeout_and_existing_output_stop_before_launch(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(host, 'run_supervised') as launch:
            output = Path(tmp)
            for timeout in [float('nan'), float('inf'), 0, 121]:
                with self.assertRaises(ValueError):
                    host.run_host(Path('missing'), output / 'new', None, timeout)
            with self.assertRaises(ValueError):
                host.run_host(Path('missing'), output, None)
            launch.assert_not_called()

    def test_mocked_supervisor_bytes_complete_the_host_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / 'checker'
            binary.write_bytes(b'synthetic checker identity')

            def supervised(command, output, name, env, timeout):
                (output / f'{name}.supervisor.stderr.log').write_bytes(b'')
                record = {'status': 'exited', 'returncode': 0,
                          'cleanup': {'complete': True, 'descendants': 0},
                          'stderr_sha256': host.hashlib.sha256(b'').hexdigest(),
                          'capture_gate_required': False, 'capture_gate_granted': False}
                stdout = ('\n'.join(ADAPTERS) + '\n').encode() if name == 'enumeration' else synthetic().encode()
                return record, stdout

            with patch.object(host, 'run_supervised', side_effect=supervised) as launch:
                result = host.run_host(binary, root / 'result', None)
                self.assertTrue(result['success'], result.get('error'))
                self.assertEqual(result['adapter_count'], 1)
                self.assertEqual(launch.call_count, 2)
                self.assertEqual(result['runs'][1]['validation']['cases'], 11)

    def test_checker_or_cleanup_failure_stops_before_adapter_execution(self):
        for status, code, clean, descendants in [('timeout', None, True, 0),
                                                ('exited', 1, True, 0),
                                                ('exited', 0, False, 0),
                                                ('exited', 0, True, 1)]:
            with self.subTest(status=status, code=code, clean=clean, descendants=descendants), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                binary = root / 'checker'
                binary.write_bytes(b'synthetic checker identity')
                record = {'status': status, 'returncode': code,
                          'cleanup': {'complete': clean, 'descendants': descendants}}
                with patch.object(host, 'run_supervised', return_value=(record, b'')) as launch:
                    result = host.run_host(binary, root / 'result', None)
                    self.assertFalse(result['success'])
                    self.assertEqual(launch.call_count, 1)


if __name__ == '__main__':
    unittest.main()
