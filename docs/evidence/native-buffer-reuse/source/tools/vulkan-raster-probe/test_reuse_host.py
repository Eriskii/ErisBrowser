import unittest

from run_reuse_host import expected_sequence, validate_run


def transcript():
    lines = ['ADAPTER fixture']
    for cut in range(1, 7):
        lines.append(f'PASS cancellation checkpoint={cut} flush_submissions=1 retired=true reusable=false scopes=3')
    for name, fmt, width, height, reused in expected_sequence():
        lines.append(f'PASS reuse-{name} format={fmt} width={width} height={height} '
                     f'reused={str(reused).lower()} planned_bytes=4096 compared_bytes={width * height * 4} '
                     'reference=literal exact=true')
    lines.append('COMPLETE adapter=0 frames=28 allocations=13 reuses=15 evictions=13 formats=2 '
                 'compared_bytes=1560 offscreen=true acquired_surface=false exact=true')
    return '\n'.join(lines) + '\n'


class ReuseProtocolTests(unittest.TestCase):
    def test_complete_ordered_evidence(self):
        result = validate_run(transcript().encode(), ['ADAPTER fixture'], 0)
        self.assertEqual(result['gpu_compared_bytes'], 1560)
        self.assertEqual(result['reuses'], 15)

    def test_missing_duplicate_reordered_or_extra_records_fail(self):
        lines = transcript().splitlines()
        variants = [lines[:3] + lines[4:], lines[:3] + [lines[2]] + lines[3:],
                    lines[:7] + [lines[8], lines[7]] + lines[9:], lines + ['unexpected']]
        for bad in variants:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate_run('\n'.join(bad), ['ADAPTER fixture'], 0)

    def test_cache_retirement_format_accounting_and_terminal_claims_are_checked(self):
        replacements = [('reusable=false', 'reusable=true'),
                        ('flush_submissions=1', 'flush_submissions=0'),
                        ('scopes=3', 'scopes=2'), ('reused=true', 'reused=false'),
                        ('format=Rgba8Unorm', 'format=Bgra8Unorm'),
                        ('compared_bytes=48', 'compared_bytes=44'),
                        ('planned_bytes=4096', 'planned_bytes=1'),
                        ('evictions=13', 'evictions=12'), ('adapter=0', 'adapter=1')]
        for old, new in replacements:
            with self.subTest(old=old), self.assertRaises(ValueError):
                validate_run(transcript().replace(old, new, 1), ['ADAPTER fixture'], 0)


if __name__ == '__main__':
    unittest.main()
