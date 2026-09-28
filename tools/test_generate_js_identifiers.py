import contextlib
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

import generate_js_identifiers as generator


class IdentifierDataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.inputs = generator.load_inputs(generator.DATA)
        cls.data = cls.inputs['DerivedCoreProperties.txt']

    def copied_inputs(self, directory):
        for name in (*generator.PINNED, 'manifest.json'):
            shutil.copyfile(generator.DATA / name, directory / name)

    def test_exact_pin_final_evidence_and_mutable_license_are_retained(self):
        manifest = json.loads((generator.DATA / 'manifest.json').read_bytes())
        self.assertEqual(manifest['unicode_version'], '18.0.0')
        self.assertEqual(set(self.inputs), set(generator.PINNED))
        self.assertEqual(sum(map(len, self.inputs.values())), 1225451)
        self.assertIn(generator.FINAL_STATEMENT.encode(), self.inputs['ReadMe.txt'])
        self.assertIn(b'preliminary draft page', self.inputs['Unicode18.0.0.html'])
        self.assertIn(b'2026 September 16', self.inputs['Unicode18.0.0.html'])
        self.assertTrue(self.inputs['LICENSE.txt'].startswith(b'UNICODE LICENSE V3\n'))
        self.assertEqual(hashlib.sha256((generator.DATA / 'manifest.json').read_bytes()).hexdigest(),
                         generator.MANIFEST_SHA256)
        for name, (url, size, digest) in generator.PINNED.items():
            entry = next(e for e in manifest['files'] if e['path'] == name)
            self.assertEqual((entry['url'], entry['resolved_url'], entry['bytes'], entry['sha256']),
                             (url, url, size, digest))

    def test_each_corrupted_pinned_file_is_rejected_before_generation(self):
        for name in generator.PINNED:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                self.copied_inputs(directory)
                path = directory / name
                data = path.read_bytes()
                path.write_bytes(bytes([data[0] ^ 1]) + data[1:])
                with self.assertRaisesRegex(ValueError, 'byte integrity mismatch'):
                    generator.load_inputs(directory)

    def test_manifest_repin_url_inventory_or_final_status_cannot_hide_changes(self):
        changes = (
            lambda m: m.update(unicode_version='17.0.0'),
            lambda m: m['files'][0].update(url='https://example.invalid/data'),
            lambda m: m['files'][0].update(sha256='0' * 64),
            lambda m: m['files'].append(m['files'][0]),
            lambda m: m['files'][0].update(path='../DerivedCoreProperties.txt'),
            lambda m: m['final_version_evidence'].update(landing_page_caveat='Final landing page'),
            lambda m: m.update(retrieved_at_utc='1900-01-01T00:00:00+00:00'),
        )
        for change in changes:
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                self.copied_inputs(directory)
                path = directory / 'manifest.json'
                manifest = json.loads(path.read_bytes())
                change(manifest)
                path.write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError, 'manifest byte integrity mismatch'):
                    generator.load_inputs(directory)
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.copied_inputs(directory)
            path = directory / 'DerivedCoreProperties.txt'
            path.write_bytes(path.read_bytes() + b'\n')
            manifest = json.loads((directory / 'manifest.json').read_bytes())
            manifest['files'][0].update(bytes=path.stat().st_size,
                                      sha256=hashlib.sha256(path.read_bytes()).hexdigest())
            (directory / 'manifest.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, 'manifest byte integrity mismatch'):
                generator.load_inputs(directory)

    def test_limits_and_symlinks_reject_without_following_an_alternate_pin(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            self.copied_inputs(directory)
            path = directory / 'ReadMe.txt'
            path.unlink()
            path.symlink_to(generator.DATA / 'ReadMe.txt')
            with self.assertRaisesRegex(ValueError, 'non-symlink'):
                generator.load_inputs(directory)
            manifest = directory / 'manifest.json'
            manifest.write_bytes(b' ' * (generator.MAX_MANIFEST + 1))
            with self.assertRaisesRegex(ValueError, 'byte limit'):
                generator.load_inputs(directory)
        with self.assertRaisesRegex(ValueError, 'byte limit'):
            generator.parse_properties(b' ' * (generator.MAX_FILE + 1))
        with self.assertRaisesRegex(ValueError, 'line length'):
            generator.parse_properties(self.data + b'\n#' + b'x' * generator.MAX_LINE)
        with self.assertRaisesRegex(ValueError, 'line limit'):
            generator.parse_properties(self.data + b'\n' * generator.MAX_LINES)

    def mutate_first_start(self, replacement):
        return self.data.replace(b'0041..005A    ; ID_Start', replacement, 1)

    def test_selected_range_grammar_order_scalar_and_comment_count_validation(self):
        bad = (
            (b'005A..0041    ; ID_Start', 'not Unicode scalars'),
            (b'D800..D819    ; ID_Start', 'not Unicode scalars'),
            (b'110000..110019 ; ID_Start', 'not Unicode scalars'),
            (b'0041...005A   ; ID_Start', 'invalid selected'),
            (b'0041..005A    ; XID_Start', 'invalid selected'),
            (b'0041..0059    ; ID_Start', 'record count mismatch'),
            (b'0041..005A    ; ID_Start ; extra', 'invalid selected'),
        )
        for replacement, error in bad:
            with self.subTest(replacement=replacement):
                with self.assertRaisesRegex(ValueError, error):
                    generator.parse_properties(self.mutate_first_start(replacement))
        first = b'0041..005A    ; ID_Start # L&  [26] LATIN CAPITAL LETTER A..LATIN CAPITAL LETTER Z'
        for extra in (first, b'0030..0039 ; ID_Start # [10] earlier',
                      b'0050..005A ; ID_Start # [11] overlap'):
            with self.assertRaisesRegex(ValueError, 'overlap or are unordered'):
                generator.parse_properties(self.data.replace(first, first + b'\n' + extra, 1))

    def test_property_sections_totals_and_version_cannot_be_silently_skipped(self):
        changes = (
            (b'# DerivedCoreProperties-18.0.0.txt', b'# DerivedCoreProperties-17.0.0.txt', 'header/version'),
            (b'# Total code points: 158739', b'# Total code points: 158738', 'sections/totals'),
            (b'# Total code points: 158739', b'# missing total', 'missing property total'),
            (b'# Derived Property: ID_Start', b'# Derived Property: XID_Start', 'outside its section'),
            (b'# Derived Property: ID_Start', b'# Derived Property: ID_Start\n# Derived Property: ID_Start', 'missing property total'),
        )
        for old, new, error in changes:
            with self.subTest(new=new):
                with self.assertRaisesRegex(ValueError, error):
                    generator.parse_properties(self.data.replace(old, new, 1))
        line = b'00AA          ; ID_Start # Lo       FEMININE ORDINAL INDICATOR\n'
        self.assertIn(line, self.data)
        with self.assertRaisesRegex(ValueError, 'code-point count mismatch'):
            generator.parse_properties(self.data.replace(line, b'', 1))
        with self.assertRaisesRegex(ValueError, 'follows its total'):
            generator.parse_properties(self.data.replace(b'# Total code points: 158739',
                b'# Total code points: 158739\n10FFFD ; ID_Start', 1))

    def test_raw_record_and_merged_range_caps_are_enforced(self):
        with patch.object(generator, 'MAX_RECORDS', 2):
            with self.assertRaisesRegex(ValueError, 'record limit'):
                generator.parse_properties(self.data)
        with patch.object(generator, 'MAX_RANGES', 2):
            with self.assertRaisesRegex(ValueError, 'range limit'):
                generator.parse_properties(self.data)
        with patch.object(generator, 'MAX_UNIQUE_PAGES', 2):
            with self.assertRaisesRegex(ValueError, 'bitmap page limit'):
                generator.render(self.inputs)
        with patch.object(generator, 'MAX_OUTPUT', 10):
            with self.assertRaisesRegex(ValueError, 'module exceeds'):
                generator.render(self.inputs)

    def test_every_codepoint_matches_an_independent_record_bitmap(self):
        # Independent parser: select exact semicolon fields without relying on
        # the generator's regex, section state, merge function or Unicode APIs.
        reference = {name: bytearray(0x110000) for name in ('ID_Start', 'ID_Continue')}
        for line in self.data.decode().splitlines():
            fields = line.partition('#')[0].split(';')
            if len(fields) != 2 or fields[1].strip() not in reference:
                continue
            endpoints = fields[0].strip().split('..')
            first, last = int(endpoints[0], 16), int(endpoints[-1], 16)
            reference[fields[1].strip()][first:last + 1] = b'\1' * (last - first + 1)
        generated = generator.render(self.inputs).decode()
        index_body = generated.split('const PAGE_INDEX: [u8; 4352] = [', 1)[1].split('];', 1)[0]
        indices = [int(value) for value in re.findall(r'\d+', index_body)]
        page_body = generated.split('const IDENTIFIER_PAGES: [[u64; 8]; 132] = [', 1)[1].split('];', 1)[0]
        rows = re.findall(r'\[([^]\[]+)\]', page_body)
        pages = [tuple(int(word, 16) for word in re.findall(r'0x([0-9a-f]+)', row)) for row in rows]
        self.assertEqual(len(indices), 4352)
        self.assertEqual(len(pages), 132)
        self.assertEqual(len(set(pages)), len(pages))
        self.assertTrue(all(len(page) == 8 for page in pages))
        self.assertTrue(all(0 <= index < len(pages) <= 256 for index in indices))
        self.assertEqual(list(dict.fromkeys(indices)), list(range(len(pages))))
        self.assertEqual(len(indices) + len(pages) * 64, 12800)
        # Decode each distinct page independently, then materialize both full
        # property maps by following literal page IDs. This does not invoke
        # the production generator's packing or deduplication helper.
        for property_bit, name, expected_count in ((0, 'ID_Start', 158739),
                                                   (1, 'ID_Continue', 162100)):
            decoded = [bytes((page[(scalar * 2 + property_bit) // 64] >>
                              ((scalar * 2 + property_bit) % 64)) & 1
                             for scalar in range(256)) for page in pages]
            actual = b''.join(decoded[index] for index in indices)
            self.assertEqual(len(actual), 0x110000)
            self.assertEqual(sum(actual), expected_count)
            self.assertEqual(actual, reference[name])
            self.assertFalse(any(actual[0xD800:0xE000]))
        self.assertTrue(reference['ID_Start'][0x037A])  # ID, not XID.
        for scalar in (0x0558, 0x18E00, 0x3D000, 0x3FC3F):
            self.assertTrue(reference['ID_Start'][scalar])
        for scalar in (0x05C8, 0x200C, 0x200D):
            self.assertFalse(reference['ID_Start'][scalar])
            self.assertTrue(reference['ID_Continue'][scalar])

    def test_packing_is_deterministic_and_uses_first_encountered_page_ids(self):
        _, properties = generator.parse_properties(self.data)
        indices, pages = generator.pack_pages(properties)
        self.assertEqual((len(indices), len(pages)), (4352, 132))
        self.assertEqual((indices, pages), generator.pack_pages(dict(reversed(list(properties.items())))))
        self.assertEqual(list(dict.fromkeys(indices)), list(range(132)))
        self.assertTrue(all(len(page) == 64 for page in pages))
        self.assertEqual(len(set(pages)), len(pages))
        for scalar in (0, 0x7F, 0x80, 0xFF, 0x100, 0xD800, 0xDFFF, 0x10FFFF):
            self.assertLess(indices[scalar >> 8], len(pages))
        with patch.object(generator, 'PAGE_COUNT', 4351):
            with self.assertRaisesRegex(ValueError, 'fixed scalar domain'):
                generator.pack_pages(properties)

    def test_deterministic_output_keeps_the_complete_license_and_no_network(self):
        first = generator.render(self.inputs)
        self.assertEqual(first, generator.render(dict(reversed(list(self.inputs.items())))))
        self.assertEqual(first, generator.OUTPUT.read_bytes())
        for line in self.inputs['LICENSE.txt'].decode().rstrip('\n').splitlines():
            self.assertIn(('// ' + line if line else '//').encode() + b'\n', first)
        self.assertNotIn('urllib', generator.__dict__)
        self.assertNotIn('requests', generator.__dict__)
        self.assertNotIn(b'is_alphabetic', first)
        self.assertNotIn(b'is_alphanumeric', first)

    def test_offline_check_is_readonly_and_stale_output_returns_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'table.rs'
            output.write_bytes(generator.render(self.inputs))
            before = output.stat().st_mtime_ns
            args = ['generate_js_identifiers.py', '--check', '--output', str(output)]
            with patch.object(sys, 'argv', args), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(generator.main(), 0)
            self.assertEqual(before, output.stat().st_mtime_ns)
            output.write_bytes(output.read_bytes() + b'// stale\n')
            stale = output.read_bytes()
            with patch.object(sys, 'argv', args), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(generator.main(), 1)
            self.assertEqual(output.read_bytes(), stale)
            args = ['generate_js_identifiers.py', '--output', str(generator.DATA / 'ReadMe.txt')]
            with patch.object(sys, 'argv', args), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(generator.main(), 1)


if __name__ == '__main__':
    unittest.main()
