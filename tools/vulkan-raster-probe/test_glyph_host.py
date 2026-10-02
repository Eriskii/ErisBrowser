"""Loader-directory admission only; no checker, child process or Vulkan calls."""
from __future__ import annotations

import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import run_glyph_host as host


class GlyphHostLoaderTests(unittest.TestCase):
    def test_exact_256_entries_are_bound_in_stable_order(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            # Reverse creation order must not affect the published binding.
            for index in reversed(range(256)):
                (directory / f'lib-{index:03}.so').write_bytes(bytes([index]))
            environment = {'LD_LIBRARY_PATH': str(directory)}
            with mock.patch.object(host, 'digest', wraps=host.digest) as hashed:
                result = host.loader_binding(directory, environment)
            self.assertEqual(hashed.call_count, 256)
            self.assertEqual(result['directory'], str(directory.resolve()))
            self.assertEqual(result['environment']['LD_LIBRARY_PATH'], str(directory))
            self.assertEqual(result['files'], [
                {'name': f'lib-{index:03}.so',
                 'resolved': str((directory / f'lib-{index:03}.so').resolve()),
                 'bytes': 1,
                 'sha256': hashlib.sha256(bytes([index])).hexdigest()}
                for index in range(256)
            ])
            self.assertTrue(all(call.args[1] == host.MAX_BINARY
                                for call in hashed.call_args_list))

    def test_257th_entry_refuses_before_any_file_hashing(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for index in range(257):
                (directory / f'entry-{index:03}').write_bytes(b'')
            with mock.patch.object(host, 'digest', wraps=host.digest) as hashed:
                with self.assertRaisesRegex(ValueError, 'loader-directory entry bound exceeded'):
                    host.loader_binding(directory, {})
            hashed.assert_not_called()


if __name__ == '__main__':
    unittest.main()
