"""Проверить Mach-O CPU и структурный разбор Darwin linker map."""

from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from check_lsp_macos import cpu, object_inputs


class DarwinNoticeInputs(unittest.TestCase):
    def roots(self, directory):
        root = Path(directory)
        return root / 'cargo', root / 'rustlib', root / 'sdk'

    def map_bytes(self, cargo, sdk, extra=b''):
        return (b'# Object files:\n[  0] linker synthesized\n[  1] ' + str(cargo / 'main.rcgu.o').encode() + b'\n[  2] ' + str(sdk / 'usr/lib/libSystem.tbd').encode() + b'\n# Sections:\n' + extra)

    def test_raw_literal_data_does_not_require_whole_map_utf8(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            records = object_inputs(self.map_bytes(cargo, sdk, b'\xff\xfe\x80'), cargo, rustlib, sdk)
            self.assertEqual([item['kind'] for item in records], ['linker-synthesized', 'rust-object', 'system-stub'])

    def test_external_static_object_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            data = self.map_bytes(cargo, sdk).replace(b'main.rcgu.o', b'foreign-native.o')
            with self.assertRaisesRegex(ValueError, 'external static object'):
                object_inputs(data, cargo, rustlib, sdk)

    def test_duplicate_object_index_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            data = self.map_bytes(cargo, sdk).replace(b'[  2]', b'[  1]')
            with self.assertRaisesRegex(ValueError, 'invalid Darwin map object row'):
                object_inputs(data, cargo, rustlib, sdk)

    def test_unreviewed_sdk_stub_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            data = self.map_bytes(cargo, sdk).replace(b'libSystem.tbd', b'libNewRuntime.tbd')
            with self.assertRaisesRegex(ValueError, 'unreviewed Darwin SDK stub'):
                object_inputs(data, cargo, rustlib, sdk)

    def test_rust_named_object_outside_approved_roots_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            data = self.map_bytes(cargo, sdk).replace(str(cargo / 'main.rcgu.o').encode(), str(Path(directory) / 'elsewhere/main.rcgu.o').encode())
            with self.assertRaisesRegex(ValueError, 'outside reviewed Cargo/sysroot'):
                object_inputs(data, cargo, rustlib, sdk)

    def test_non_rust_member_in_rlib_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo, rustlib, sdk = self.roots(directory)
            data = self.map_bytes(cargo, sdk).replace(b'main.rcgu.o', b'libfake.rlib(native.o)')
            with self.assertRaisesRegex(ValueError, 'non-Rust native member'):
                object_inputs(data, cargo, rustlib, sdk)

    def test_macho_cpu_and_executable_type(self):
        data = bytearray(32)
        data[:4] = b'\xcf\xfa\xed\xfe'
        data[4:8] = (0x0100000C).to_bytes(4, 'little')
        data[12:16] = (2).to_bytes(4, 'little')
        self.assertEqual(cpu(bytes(data)), 0x0100000C)
        data[12:16] = (6).to_bytes(4, 'little')
        with self.assertRaisesRegex(ValueError, 'not an executable'):
            cpu(bytes(data))
        with self.assertRaisesRegex(ValueError, 'thin little-endian Mach-O'):
            cpu(b'not a Mach-O')


if __name__ == '__main__':
    unittest.main()
