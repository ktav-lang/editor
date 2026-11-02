"""Проверить legacy source и уже встроенный hook без изменения поведения."""

from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from apply_lsp_notice_overlay import DECLARATION, ENTRY, USE, apply


class LspNoticeOverlay(unittest.TestCase):
    def test_legacy_source_gets_only_existing_retention_lines(self):
        original = b'use example;\n#[tokio::main]\nasync fn main() {\n    original_protocol();\n}\n'
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_bytes(original)
            result = apply(path)
            expected = DECLARATION + b'\n\n' + original.replace(ENTRY, ENTRY + b'\n    ' + USE, 1)
            self.assertEqual(path.read_bytes(), expected)
            self.assertEqual(result['notice_overlay'], 'added')

    def test_existing_hook_is_byte_exact_and_repeatable(self):
        original = DECLARATION + b'\n\n#[tokio::main]\nasync fn main() {\n    ' + USE + b'\n    original_protocol();\n}\n'
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_bytes(original)
            first = apply(path)
            second = apply(path)
            self.assertEqual(path.read_bytes(), original)
            self.assertEqual(first, second)
            self.assertEqual(first['original_main_sha256'], first['packaged_main_sha256'])

    def test_partial_hook_is_rejected_without_source_changes(self):
        for original in [DECLARATION + b'\nasync fn main() {}\n', b'async fn main() { ' + USE + b' }\n']:
            with self.subTest(source=original), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / 'main.rs'
                path.write_bytes(original)
                with self.assertRaisesRegex(ValueError, 'partial or duplicate'):
                    apply(path)
                self.assertEqual(path.read_bytes(), original)

    def test_duplicate_entry_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_bytes(b'async fn main() {}\nasync fn main() {}\n')
            with self.assertRaisesRegex(ValueError, 'entry requires source review'):
                apply(path)

    def test_unknown_hook_representation_is_not_blindly_duplicated(self):
        original = b'include!("native_notices.rs");\nasync fn main() {}\n'
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_bytes(original)
            with self.assertRaisesRegex(ValueError, 'unrecognized'):
                apply(path)
            self.assertEqual(path.read_bytes(), original)

    def test_hook_outside_entry_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'main.rs'
            path.write_bytes(DECLARATION + b'\nfn helper() { ' + USE + b' }\nasync fn main() {}\n')
            with self.assertRaisesRegex(ValueError, 'location requires review'):
                apply(path)


if __name__ == '__main__':
    unittest.main()
