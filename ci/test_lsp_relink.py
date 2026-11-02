"""Проверить ELF entry proof на данных, не исполняя fixtures."""

import gzip
import importlib.util
from pathlib import Path
import struct
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1] / 'scripts'
for name in ('check_embedded_notices', 'check_lsp_relink'):
    spec = importlib.util.spec_from_file_location(name, ROOT / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
PROOF = sys.modules['check_lsp_relink']
NOTICE = b'{"schema_version":1}'


def elf(instruction):
    payload = gzip.compress(NOTICE, mtime=0)
    resource = b'KTAV-NOTICES-GZIP-v1\0' + len(payload).to_bytes(8, 'little') + payload
    data = bytearray(128 + 4 + len(resource))
    data[:7] = b'\x7fELF\x02\x01\x01'
    struct.pack_into('<H', data, 18, 183)
    struct.pack_into('<QQ', data, 24, 0x400080, 64)
    struct.pack_into('<HH', data, 54, 56, 1)
    struct.pack_into('<IIQQQQQQ', data, 64, 1, 5, 0, 0x400000, 0, len(data), len(data), 4096)
    data[128:132] = instruction
    data[132:] = resource
    return bytes(data)


class RelinkTests(unittest.TestCase):
    def test_actual_entry_changed_and_notice_retained(self):
        self.assertTrue(PROOF.verify(elf(b'\x1d\x00\x80\xd2'), elf(PROOF.NOP), NOTICE)['modified_startup_retained'])

    def test_changed_bytes_outside_entry_are_not_proof(self):
        original = elf(b'\x1d\x00\x80\xd2')
        with self.assertRaisesRegex(ValueError, 'actual ELF entry'):
            PROOF.verify(original, original + PROOF.NOP, NOTICE)

    def test_original_already_nop_is_not_proof(self):
        with self.assertRaisesRegex(ValueError, 'actual ELF entry'):
            PROOF.verify(elf(PROOF.NOP), elf(PROOF.NOP), NOTICE)

    def test_wrong_notice_rejected(self):
        with self.assertRaisesRegex(ValueError, 'notice identity'):
            PROOF.verify(elf(b'\x1d\x00\x80\xd2'), elf(PROOF.NOP), b'changed')

    def test_wrong_cpu_rejected(self):
        data = bytearray(elf(PROOF.NOP))
        struct.pack_into('<H', data, 18, 62)
        with self.assertRaisesRegex(ValueError, 'ARM64'):
            PROOF.entry_instruction(bytes(data))


if __name__ == '__main__':
    unittest.main()
