"""Reject incomplete or falsely healthy opacity transcripts; never starts a GPU."""
import unittest

from run_opacity_host import validate_listing, validate_run

ADAPTER = ('ADAPTER 0 vendor=0x1234 device=0x5678 type=Cpu backend=Vulkan '
           'name="fixture" driver="fixture-driver" info="source-only"')


def transcript():
    # Independent fixed transcript data, not imported from the parser's roster.
    cases = (
        ('halfway-red-65-over-black', 8, 12),
        ('opaque-backed-overlap-and-after-pop', 96, 64),
        ('nested-opaque-blue-over-red', 32, 16),
        ('quarter-group-with-translucent-image-on-white', 16, 12),
        ('coverage-times-alpha-truncates-before-group-pop', 16, 12),
        ('fixed-escape-stays-inside-opaque-backing', 80, 48),
        ('unit-scope-preserves-current-direct-rgb8-target', 0, 4),
        ('fractional-caller-clip-integer-origin-containment', 8, 12),
        ('multiple-rect-group-without-first-covering-backing', 24, 16),
        ('translucent-image-only-group', 8, 8),
        ('zero-scope-suppresses-pixels-and-restores-root', 0, 8),
        ('non-grid-point-one-white-over-black', 8, 4),
        ('transparent-black-alpha55-three-quarters-root-rounding', 8, 4),
        ('inner-rounding-survives-outer-pop', 16, 4),
        ('transparent-parent-retains-partial-alpha', 16, 4),
        ('opaque-islands-with-transparent-hole', 24, 12),
        ('transparent-glyph-coverage-without-backing', 16, 12),
        ('zero-alpha-image-preserves-root', 8, 8),
        ('decimal-point-one-is-not-nearest-grid', 8, 4),
        ('decimal-point-seven-needs-source-product-rounding', 8, 4),
        ('smallest-subnormal', 8, 4),
        ('largest-subnormal', 8, 4),
        ('smallest-normal', 8, 4),
        ('tiny-identity-threshold', 8, 4),
        ('next-above-identity-threshold', 8, 4),
        ('largest-value-below-unit-retains-layer-semantics', 8, 4),
        ('raw-nested-point-one-over-point-seven', 16, 4),
        ('raw-inner-point-one-under-grid-half', 16, 4),
        ('grid-inner-half-under-raw-point-one', 16, 4),
    )
    rows = [ADAPTER]
    for name, scratch, size in cases:
        for fmt in ('Bgra8Unorm', 'Rgba8Unorm'):
            rows.append(f'PASS opacity-{name} format={fmt} scratch_bytes={scratch} '
                        f'compared_bytes={size} exact=true reference=literal')
    reuse = (
        ('a', 'Bgra8Unorm', False), ('b', 'Bgra8Unorm', True),
        ('c', 'Bgra8Unorm', True), ('a', 'Bgra8Unorm', True),
        ('resized', 'Bgra8Unorm', False), ('resized', 'Rgba8Unorm', False),
        ('a', 'Rgba8Unorm', False), ('b', 'Rgba8Unorm', True),
        ('c', 'Rgba8Unorm', True), ('a', 'Rgba8Unorm', True),
        ('resized', 'Rgba8Unorm', False), ('resized', 'Bgra8Unorm', False),
        ('resized', 'Bgra8Unorm', False), ('resized', 'Bgra8Unorm', True),
    )
    for prefix in ('opacity-reuse', 'opacity-transparent-reuse', 'opacity-full-reuse'):
        rows.append(ADAPTER)
        for cut in range(1, 9):
            rows.append(f'PASS cancellation checkpoint={cut} flush_submissions=1 '
                        'retired=true reusable=false scopes=3')
        for name, fmt, reused in reuse:
            planned = 2164 if name == 'resized' else 2148
            rows.append(f'PASS reuse-{prefix}-{name} format={fmt} width=4 height=2 '
                        f'reused={str(reused).lower()} planned_bytes={planned} compared_bytes=32 '
                        'reference=literal exact=true')
    rows.append('COMPLETE adapter=0 literal_frames=58 literal_bytes=608 refusals=0 '
                'reuse_frames=14 reuse_bytes=448 allocations=7 reuses=7 evictions=7 '
                'transparent_reuse_frames=14 transparent_reuse_bytes=448 transparent_allocations=7 '
                'transparent_reuses=7 transparent_evictions=7 '
                'full_reuse_frames=14 full_reuse_bytes=448 full_allocations=7 '
                'full_reuses=7 full_evictions=7 '
                'offscreen=true acquired_surface=false exact=true')
    return ('\n'.join(rows) + '\n').encode()


class OpacityProtocolTests(unittest.TestCase):
    def setUp(self):
        # Negative cases must begin from an accepted complete transcript.
        self.good = transcript()
        result = validate_run(self.good, [ADAPTER], 0)
        self.assertEqual((result['literal_bytes'], result['reuse_bytes'],
                          result['transparent_reuse_bytes'], result['full_reuse_bytes']), (608, 448, 448, 448))

    def test_success_totals_cannot_replace_missing_duplicate_or_reordered_work(self):
        rows = self.good.splitlines(keepends=True)
        for bad in (rows[:3] + rows[4:], rows[:3] + [rows[2]] + rows[4:],
                    rows[:3] + [rows[4], rows[3]] + rows[5:],
                    rows[:60] + rows[61:], rows[:83] + rows[84:],
                    rows[:106] + rows[107:], rows[:82] + rows[105:],
                    rows[:105] + rows[128:], rows + [b'PASS extra\n'], rows[:-1]):
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate_run(b''.join(bad), [ADAPTER], 0)

    def test_reference_routes_bytes_and_retirement_must_all_match(self):
        changes = (
            (b'exact=true', b'exact=false'), (b'reference=literal', b'reference=CPU'),
            (b'scratch_bytes=8', b'scratch_bytes=0'),
            (b'compared_bytes=12', b'compared_bytes=8'),
            (b'scopes=3', b'scopes=2'), (b'retired=true', b'retired=false'),
            (b'reusable=false', b'reusable=true'),
            (b'flush_submissions=1', b'flush_submissions=0'),
            (b'checkpoint=8', b'checkpoint=9'),
            (b'planned_bytes=2164', b'planned_bytes=2148'),
            (b'reused=true', b'reused=false'),
            (b'evictions=7', b'evictions=6'), (b'refusals=0', b'refusals=1'),
            (b'transparent_reuse_frames=14', b'transparent_reuse_frames=13'),
            (b'transparent_reuse_bytes=448', b'transparent_reuse_bytes=447'),
            (b'transparent_allocations=7', b'transparent_allocations=6'),
            (b'transparent_reuses=7', b'transparent_reuses=6'),
            (b'transparent_evictions=7', b'transparent_evictions=6'),
            (b'PASS reuse-opacity-transparent-reuse-a', b'PASS reuse-opacity-reuse-a'),
            (b'full_reuse_frames=14', b'full_reuse_frames=13'),
            (b'full_reuse_bytes=448', b'full_reuse_bytes=447'),
            (b'full_allocations=7', b'full_allocations=6'),
            (b'full_reuses=7', b'full_reuses=6'),
            (b'full_evictions=7', b'full_evictions=6'),
            (b'PASS reuse-opacity-full-reuse-a', b'PASS reuse-opacity-transparent-reuse-a'),
            (b'acquired_surface=false', b'acquired_surface=true'),
            (b'adapter=0', b'adapter=1'),
        )
        for old, new in changes:
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate_run(self.good.replace(old, new, 1), [ADAPTER], 0)

    def test_reuse_enumerations_cannot_change_device_or_add_unlisted_adapters(self):
        rows = self.good.splitlines(keepends=True)
        for index in (59, 82, 105):
            self.assertEqual(rows[index], (ADAPTER + '\n').encode())
            changed = list(rows)
            changed[index] = rows[index].replace(b'device=0x5678', b'device=0x5679')
            for bad in (b''.join(changed), b''.join(rows[:index] + rows[index + 1:])):
                with self.assertRaises(ValueError):
                    validate_run(bad, [ADAPTER], 0)
        for selected in (-1, 1, True):
            with self.assertRaises(ValueError):
                validate_run(self.good, [ADAPTER], selected)
        for listing in (b'', (ADAPTER + '\n' + ADAPTER + '\n').encode(),
                        (ADAPTER.replace('backend=Vulkan', 'backend=Gl') + '\n').encode(),
                        (ADAPTER + '\nCOMPLETE\n').encode(),
                        ('\n'.join(ADAPTER.replace('ADAPTER 0 ', f'ADAPTER {i} ')
                                   for i in range(17)) + '\n').encode()):
            with self.subTest(listing=listing), self.assertRaises(ValueError):
                validate_listing(listing)

    def test_trailing_garbage_partial_records_and_non_utf8_cannot_be_success(self):
        for bad in (self.good[:-1], self.good + b'garbage', self.good + b'\n',
                    self.good.replace(b'\n', b'\r\n', 1), self.good + b'\0\n',
                    self.good + b'\xff\n', b'x' * (2 * 1024 * 1024 + 1)):
            with self.subTest(size=len(bad)), self.assertRaises(ValueError):
                validate_run(bad, [ADAPTER], 0)

    def test_four_full_adapter_enumerations_fit_the_derived_record_bound(self):
        adapters = [ADAPTER.replace('ADAPTER 0 ', f'ADAPTER {i} ') for i in range(16)]
        listing = ('\n'.join(adapters) + '\n').encode()
        self.assertEqual(validate_listing(listing), adapters)
        full = self.good.replace((ADAPTER + '\n').encode(), listing)
        full = full.replace(b'COMPLETE adapter=0 ', b'COMPLETE adapter=15 ')
        self.assertEqual(len(full.splitlines()), 189)
        result = validate_run(full, adapters, 15)
        self.assertEqual((result['literal_frames'], result['reuse_frames'],
                          result['transparent_reuse_frames'], result['full_reuse_frames']), (58, 14, 14, 14))
        with self.assertRaises(ValueError):
            validate_run(full + b'PASS extra\nPASS extra\nPASS extra\nPASS extra\n', adapters, 15)


if __name__ == '__main__':
    unittest.main()
