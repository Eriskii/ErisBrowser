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
        ('zero-scope-suppresses-pixels-and-restores-root', 0, 8),
    )
    rows = [ADAPTER]
    for name, scratch, size in cases:
        for fmt in ('Bgra8Unorm', 'Rgba8Unorm'):
            rows.append(f'PASS opacity-{name} format={fmt} scratch_bytes={scratch} '
                        f'compared_bytes={size} exact=true reference=literal')
    rows.append(ADAPTER)
    for cut in range(1, 9):
        rows.append(f'PASS cancellation checkpoint={cut} flush_submissions=1 '
                    'retired=true reusable=false scopes=3')
    reuse = (
        ('a', 'Bgra8Unorm', False), ('b', 'Bgra8Unorm', True),
        ('c', 'Bgra8Unorm', True), ('a', 'Bgra8Unorm', True),
        ('resized', 'Bgra8Unorm', False), ('resized', 'Rgba8Unorm', False),
        ('a', 'Rgba8Unorm', False), ('b', 'Rgba8Unorm', True),
        ('c', 'Rgba8Unorm', True), ('a', 'Rgba8Unorm', True),
        ('resized', 'Rgba8Unorm', False), ('resized', 'Bgra8Unorm', False),
        ('resized', 'Bgra8Unorm', False), ('resized', 'Bgra8Unorm', True),
    )
    for name, fmt, reused in reuse:
        planned = 2164 if name == 'resized' else 2148
        rows.append(f'PASS reuse-opacity-reuse-{name} format={fmt} width=4 height=2 '
                    f'reused={str(reused).lower()} planned_bytes={planned} compared_bytes=32 '
                    'reference=literal exact=true')
    rows.append('COMPLETE adapter=0 literal_frames=18 literal_bytes=376 refusals=3 '
                'reuse_frames=14 reuse_bytes=448 allocations=7 reuses=7 evictions=7 '
                'offscreen=true acquired_surface=false exact=true')
    return ('\n'.join(rows) + '\n').encode()


class OpacityProtocolTests(unittest.TestCase):
    def setUp(self):
        # Negative cases must begin from an accepted complete transcript.
        self.good = transcript()
        result = validate_run(self.good, [ADAPTER], 0)
        self.assertEqual((result['literal_bytes'], result['reuse_bytes']), (376, 448))

    def test_success_totals_cannot_replace_missing_duplicate_or_reordered_work(self):
        rows = self.good.splitlines(keepends=True)
        for bad in (rows[:3] + rows[4:], rows[:3] + [rows[2]] + rows[4:],
                    rows[:3] + [rows[4], rows[3]] + rows[5:],
                    rows[:20] + rows[21:], rows + [b'PASS extra\n'], rows[:-1]):
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
            (b'evictions=7', b'evictions=6'), (b'refusals=3', b'refusals=2'),
            (b'acquired_surface=false', b'acquired_surface=true'),
            (b'adapter=0', b'adapter=1'),
        )
        for old, new in changes:
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate_run(self.good.replace(old, new, 1), [ADAPTER], 0)

    def test_second_enumeration_cannot_change_device_or_add_unlisted_adapters(self):
        rows = self.good.splitlines(keepends=True)
        self.assertEqual(rows[19], (ADAPTER + '\n').encode())
        changed = list(rows)
        changed[19] = rows[19].replace(b'device=0x5678', b'device=0x5679')
        for bad in (b''.join(changed), b''.join(rows[:19] + rows[20:])):
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


if __name__ == '__main__':
    unittest.main()
