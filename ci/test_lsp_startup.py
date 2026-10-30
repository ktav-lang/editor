"""Проверить fail-closed inventory retained startup inputs."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('startup', Path(__file__).resolve().parents[1] / 'scripts/check_lsp_startup.py')
STARTUP = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(STARTUP)


def fixture():
    lines = [f'/cache/{archive}(/cache/{member}):(.text)' for archive, member in sorted(STARTUP.EXPECTED)]
    lines.extend(['/cache/Scrt1.o:(.text)', '/cache/Scrt1.o:(.note.ABI-tag)', '/cache/Scrt1.o:(.eh_frame+0x0)', '/build/main.rcgu.o:(.text)'])
    return '\n'.join(lines)


class StartupTests(unittest.TestCase):
    def test_exact_known_sections(self):
        path, sections, members = STARTUP.analyze_map(fixture())
        self.assertEqual(path, '/cache/Scrt1.o')
        self.assertEqual(sections, ['.eh_frame', '.note.ABI-tag', '.text'])
        self.assertEqual(len(members), 8)

    def test_new_archive_rejected(self):
        with self.assertRaisesRegex(ValueError, 'archive'):
            STARTUP.analyze_map(fixture() + '\n/cache/libgcc.a(new.o):(.text)')

    def test_new_direct_object_rejected(self):
        with self.assertRaisesRegex(ValueError, 'direct startup'):
            STARTUP.analyze_map(fixture() + '\n/cache/crtn.o:(.text)')

    def test_retained_data_rejected(self):
        with self.assertRaisesRegex(ValueError, 'retained startup section'):
            STARTUP.analyze_map(fixture() + '\n/cache/Scrt1.o:(.rodata)')

    def test_map_symbol_rejected(self):
        with self.assertRaisesRegex(ValueError, 'data retained'):
            STARTUP.analyze_map(fixture() + '\n_IO_stdin_used')

    def test_elf_symbol_rejected(self):
        with self.assertRaisesRegex(ValueError, 'final ELF'):
            STARTUP.verify_binary_symbols('42: 0000000000001000 4 OBJECT GLOBAL DEFAULT 12 _IO_stdin_used')

    def test_no_lto_object_rejected(self):
        with self.assertRaisesRegex(ValueError, 'direct startup'):
            STARTUP.analyze_map(fixture().replace('/build/main.rcgu.o:(.text)', ''))


if __name__ == '__main__':
    unittest.main()
