"""Проверить exact SDK copyright и различия startup permissions."""

import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('runtime', Path(__file__).resolve().parents[1] / 'scripts/lsp_runtime_notices.py')
RUNTIME = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNTIME)


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.lib = self.root / 'lib'
        (self.lib / 'source').mkdir(parents=True)
        self.data = b'/* Copyright fixture. Licensed fixture. */\nint fixture;\n'
        (self.lib / 'source/file.c').write_bytes(self.data)
        self.record = {'path': 'lib/source/file.c', 'sha256': hashlib.sha256(self.data).hexdigest(), 'source_notices': ['/* Copyright fixture. Licensed fixture. */']}
        self.catalog = {'zig_version': '0.14.0', 'source_revision': RUNTIME.REVISION, 'files': [self.record], 'startup_terms': {
            'aarch64-unknown-linux-gnu': {'source': 'lib/libc/glibc/sysdeps/aarch64/start-2.33.S', 'license': 'LGPL-2.1-or-later', 'compiled_linking_exception': False, 'corresponding_source_required': True},
            'x86_64-unknown-linux-gnu': {'source': 'lib/libc/glibc/sysdeps/x86_64/start-2.33.S', 'license': 'LGPL-2.1-or-later', 'compiled_linking_exception': True, 'corresponding_source_required': False},
        }}

    def test_original_source_and_copyright(self):
        self.assertEqual(RUNTIME.sources(self.lib, self.catalog), [self.record])

    def test_changed_source_rejected(self):
        (self.lib / 'source/file.c').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'bytes differ'):
            RUNTIME.sources(self.lib, self.catalog)

    def test_changed_copyright_text_rejected(self):
        self.record['source_notices'] = ['Copyright changed']
        with self.assertRaisesRegex(ValueError, 'copyright notices differ'):
            RUNTIME.sources(self.lib, self.catalog)

    def test_missing_copyright_rejected(self):
        self.record['source_notices'] = []
        with self.assertRaisesRegex(ValueError, 'copyright notices differ'):
            RUNTIME.sources(self.lib, self.catalog)

    def test_path_escape_rejected(self):
        self.record['path'] = 'lib/../outside'
        with self.assertRaisesRegex(ValueError, 'invalid'):
            RUNTIME.sources(self.lib, self.catalog)

    def test_arm_exception_cannot_be_inferred_from_x64(self):
        self.catalog['startup_terms']['aarch64-unknown-linux-gnu']['compiled_linking_exception'] = True
        with self.assertRaisesRegex(ValueError, 'licensing conditions'):
            RUNTIME.startup_terms(self.catalog, 'aarch64-unknown-linux-gnu')

    def test_arm_source_requirement_cannot_be_removed(self):
        self.catalog['startup_terms']['aarch64-unknown-linux-gnu']['corresponding_source_required'] = False
        with self.assertRaisesRegex(ValueError, 'licensing conditions'):
            RUNTIME.startup_terms(self.catalog, 'aarch64-unknown-linux-gnu')

    def test_x64_compiled_permission_preserved(self):
        self.assertTrue(RUNTIME.startup_terms(self.catalog, 'x86_64-unknown-linux-gnu')['compiled_linking_exception'])


if __name__ == '__main__':
    unittest.main()
