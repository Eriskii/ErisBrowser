import gzip
import importlib.util
import io
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('generate_date_zones', Path(__file__).with_name('generate_date_zones.py'))
ZONES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ZONES)


def tar_bytes(entries):
    plain = io.BytesIO()
    with tarfile.open(fileobj=plain, mode='w') as archive:
        for name, data, kind in entries:
            item = tarfile.TarInfo(name)
            item.type = kind
            item.size = len(data) if kind == tarfile.REGTYPE else 0
            archive.addfile(item, io.BytesIO(data) if kind == tarfile.REGTYPE else None)
    return gzip.compress(plain.getvalue())


class DateZoneFixtureTests(unittest.TestCase):
    def test_offline_manifest_sources_fixtures_and_anchors(self):
        manifest, archives = ZONES.verify(ZONES.DIRECTORY)
        self.assertEqual(manifest['version'], '2026d')
        self.assertEqual(len(manifest['fixtures']), 5)
        self.assertEqual(set(archives), set(ZONES.ARCHIVES))
        self.assertEqual(sum(row['bytes'] for row in manifest['fixtures']), 9686)

    def test_unchanged_archive_and_fixture_corruption_reject(self):
        for target in ('upstream/tzdata2026d.tar.gz', 'America/New_York', 'expectations.json'):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary) / 'fixtures'
                shutil.copytree(ZONES.DIRECTORY, directory)
                path = directory / target
                data = bytearray(path.read_bytes())
                data[len(data) // 2] ^= 1
                path.write_bytes(data)
                with self.assertRaises(ValueError):
                    ZONES.verify(directory)

    def test_changed_manifest_cannot_repin_changed_data(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            record = json.loads((ZONES.DIRECTORY / 'manifest.json').read_text())
            record['version'] = 'next'
            (directory / 'manifest.json').write_text(json.dumps(record))
            with self.assertRaisesRegex(ValueError, 'manifest hash'):
                ZONES.verify(directory)

    def test_tar_rejects_path_traversal_duplicate_and_links(self):
        for entries in [
            [('../outside', b'x', tarfile.REGTYPE)],
            [('/outside', b'x', tarfile.REGTYPE)],
            [('dir/name', b'x', tarfile.REGTYPE)],
            [('name', b'x', tarfile.REGTYPE), ('name', b'y', tarfile.REGTYPE)],
            [('name', b'', tarfile.SYMTYPE)],
            [('name', b'', tarfile.LNKTYPE)],
        ]:
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                ZONES.archive_members(tar_bytes(entries))
        self.assertEqual(ZONES.archive_members(tar_bytes([('name', b'x', tarfile.REGTYPE)])), {'name': b'x'})

    def test_tar_expansion_member_and_count_limits(self):
        with self.assertRaisesRegex(ValueError, 'expanded archive'):
            ZONES.archive_members(gzip.compress(b'\0' * (ZONES.MAX_EXPANDED + 1)))
        with self.assertRaises(ValueError):
            ZONES.archive_members(tar_bytes([('name', b'x' * (ZONES.MAX_MEMBER + 1), tarfile.REGTYPE)]))
        with self.assertRaises(ValueError):
            ZONES.archive_members(tar_bytes([(str(n), b'x', tarfile.REGTYPE) for n in range(ZONES.MAX_MEMBERS + 1)]))

    def test_independent_timezone_structure_rejects_truncation_or_leaps(self):
        data = (ZONES.DIRECTORY / 'America/New_York').read_bytes()
        for malformed in (data[:40], data[:-1], b'x' + data[1:], data[:28] + b'\0\0\0\1' + data[32:]):
            with self.assertRaises(ValueError):
                ZONES.tzif_records(malformed)
        _, _, types, footer = ZONES.tzif_records(data)
        self.assertIn((-17762, 0, 0), types)
        self.assertEqual(footer, 'EST5EDT,M3.2.0,M11.1.0')

    def test_independent_negative_epoch_calendar_and_offset_anchors(self):
        self.assertEqual(ZONES.utc_second([1970, 1, 1, 0, 0, 0]), 0)
        self.assertEqual(ZONES.utc_second([1969, 12, 31, 23, 59, 59]), -1)
        self.assertEqual(ZONES.utc_second([2000, 2, 29, 0, 0, 0]), 951782400)
        source = json.loads((ZONES.DIRECTORY / 'expectations.json').read_text())
        self.assertEqual(len(source['transitions']), 10)
        skips = [x for x in source['transitions'] if x['zone'] == 'Pacific/Apia']
        self.assertEqual(skips[0]['after_offset_seconds'] - skips[0]['before_offset_seconds'], 86400)
        self.assertTrue(any(x['before_offset_seconds'] % 60 for x in source['transitions']))

    def test_nonregular_source_rejection(self):
        with tempfile.TemporaryDirectory() as temporary:
            link = Path(temporary) / 'source'
            link.symlink_to(ZONES.DIRECTORY / 'LICENSE')
            with self.assertRaises(ValueError):
                ZONES.read_bounded(link, 4096)


if __name__ == '__main__':
    unittest.main()
