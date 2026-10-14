"""Frozen synthetic protocol controls; never GPU rendering evidence."""
import unittest

import run_box_core_host as host

ADAPTERS = ['ADAPTER 0 vendor=0x10de device=0x1 type=DiscreteGpu backend=Vulkan name="synthetic" driver="test" info="test"']
# Independent literal rows, rather than deriving expectations from host.CASES.
ROWS = [
    ('core-radius1', 512, 513, 4, 2046, 1024, 791114, 791114, 5, 811008),
    ('core-alpha128', 512, 513, 4, 2046, 1024, 791114, 791114, 5, 811008),
    ('core-opacity-half', 512, 513, 4, 2046, 1024, 791114, 1316426, 7, 2000000),
    ('core-alpha128-opacity-half', 512, 513, 4, 2046, 1024, 791114, 1316426, 7, 2000000),
    ('core-suppressed', 512, 513, 4, 2046, 1024, 791114, 1316426, 7, 300000),
    ('core-full-viewport', 1280, 1024, 6, 4604, 2048, 3938888, 3938888, 7, 3969024),
    ('core-fractional-clip', 514, 514, 2, 1025, 513, 791073, 791073, 3, 1000000),
    ('core-seven-tiles', 1280, 205, 7, 226063, 613, 1032096, 1032096, 8, 1000000),
    ('old-cap-new-admission', 1280, 205, 6, 2966, 410, 790588, 790588, 7, 1000000),
    ('core-page-padding-box', 512, 513, 4, 2046, 1024, 791114, 1053770, 6, 1077248),
]


def synthetic():
    lines = list(ADAPTERS)
    for name, width, height, masks, coverage, rows, work, cpu, lowered, invocations in ROWS:
        conversion = ((width + 7) // 8) * ((height + 7) // 8) * 64
        padded = ((width * 4 + 255) // 256) * 256 * height
        planned = width * height * 4 + padded + 16 + 256
        for fmt in ('Bgra8Unorm', 'Rgba8Unorm'):
            lines.append(
                f'PASS {name} format={fmt} width={width} height={height} draws=1 '
                f'total_invocations={invocations} conversion_invocations={conversion} '
                f'planned_bytes={planned} compared_bytes={width * height * 4} cores=1 '
                f'masks={masks} coverage={coverage} rows={rows} box_work={work} cpu_work={cpu} '
                f'lowered={lowered} exact=true reference=literal canvas=true packed_native=true '
                'rounded_color_core=true')
    lines.append('COMPLETE adapter=0 fixtures=10 frames=20 compared_bytes=29405216 exact=true '
                 'rounded_color_core=true custom_rectangle_glyph_shaders=true offscreen=true acquired_surface=false')
    return '\n'.join(lines) + '\n'


class BoxCoreProtocol(unittest.TestCase):
    def test_complete_population_has_frozen_counts_and_includes_near_cap_total(self):
        result = host.validate_run(synthetic(), ADAPTERS, 0)
        self.assertEqual(result, host.validate_run(synthetic().encode(), ADAPTERS, 0))
        self.assertEqual(result, {'fixtures': 10, 'frames': 20, 'formats': 2,
                                 'gpu_compared_bytes': 29405216,
                                 'independent_literal_bytes': 29405216,
                                 'real_page_fixtures': 1, 'acquired_surface': False})
        self.assertIn('total_invocations=3969024 conversion_invocations=1310720', synthetic())

    def test_missing_duplicate_foreign_or_extra_records_are_refused(self):
        lines = synthetic().splitlines()
        candidates = ['\n'.join(lines[:1] + lines[2:]),
                      '\n'.join(lines[:2] + [lines[1]] + lines[3:]),
                      synthetic().replace('core-radius1', 'unknown'),
                      synthetic() + 'debug\n',
                      synthetic().replace('Bgra8Unorm', 'Bgra8UnormSrgb'), b'\xff']
        for candidate in candidates:
            with self.subTest(candidate=str(candidate)[:60]), self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)

    def test_every_literal_counter_and_completion_marker_is_bound(self):
        changes = [('width=512', 'width=511'), ('height=513', 'height=512'),
                   ('compared_bytes=1050624', 'compared_bytes=1050623'), ('cores=1', 'cores=0'),
                   ('masks=4', 'masks=3'), ('coverage=2046', 'coverage=2045'), ('rows=1024', 'rows=1023'),
                   ('box_work=791114', 'box_work=791113'), ('cpu_work=791114', 'cpu_work=791113'),
                   ('lowered=5', 'lowered=4'), ('total_invocations=811008', 'total_invocations=811007'),
                   ('exact=true', 'exact=false'), ('reference=literal', 'reference=renderer'),
                   ('canvas=true', 'canvas=false'), ('packed_native=true', 'packed_native=false'),
                   ('rounded_color_core=true', 'rounded_color_core=false'), ('fixtures=10', 'fixtures=9'),
                   ('frames=20', 'frames=19'), ('compared_bytes=29405216', 'compared_bytes=29405215'),
                   ('custom_rectangle_glyph_shaders=true', 'custom_rectangle_glyph_shaders=false'),
                   ('acquired_surface=false', 'acquired_surface=true')]
        for old, new in changes:
            candidate = synthetic().replace(old, new, 1)
            self.assertNotEqual(candidate, synthetic(), old)
            with self.subTest(old=old), self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)

    def test_resource_and_adapter_mismatches_are_refused(self):
        changes = [('draws=1', 'draws=0'), ('draws=1', 'draws=258'),
                   ('total_invocations=2000000', 'total_invocations=4000001'),
                   ('total_invocations=2000000', 'total_invocations=1'),
                   ('conversion_invocations=266240', 'conversion_invocations=266239'),
                   ('planned_bytes=2101520', 'planned_bytes=1'),
                   ('planned_bytes=2101520', 'planned_bytes=16777217'),
                   ('adapter=0 fixtures', 'adapter=1 fixtures'),
                   ('name="synthetic"', 'name="changed"')]
        for old, new in changes:
            candidate = synthetic().replace(old, new, 1)
            self.assertNotEqual(candidate, synthetic(), old)
            with self.subTest(old=old), self.assertRaises(ValueError):
                host.validate_run(candidate, ADAPTERS, 0)
        for selected in (-1, 1):
            with self.assertRaises(ValueError):
                host.validate_run(synthetic(), ADAPTERS, selected)


if __name__ == '__main__':
    unittest.main()
