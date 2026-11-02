"""Проверить exact-source compiler grants без запуска toolchain."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('compiler_notices', ROOT / 'scripts/rust_compiler_notices.py')
NOTICES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NOTICES)
RUSTC = 'rustc 1.99.1 (fixture)\ncommit-hash: ' + NOTICES.RUST_REVISION + '\nrelease: 1.99.1\n'


class CompilerNoticeTests(unittest.TestCase):
    def test_complete_original_source_and_legal_materials(self):
        record = NOTICES.collect(ROOT, RUSTC)
        profile = json.loads((ROOT / 'ci/rust-compiler-notices.json').read_text(encoding='utf-8'))
        self.assertEqual(record['compiler_builtins_license'], 'MIT AND Apache-2.0 WITH LLVM-exception')
        self.assertEqual(record['libm_license'], 'MIT')
        self.assertEqual(record['rust_revision'], NOTICES.RUST_REVISION)
        self.assertEqual(len(record['materials']), 14)
        self.assertEqual(sum(item['legal_document'] for item in record['materials']), 4)
        for expected, actual in zip(profile['materials'], record['materials']):
            self.assertEqual(actual, {**expected, 'text': (ROOT / expected['path']).read_bytes().decode('utf-8')})

    def test_libm_and_legacy_attribution_is_preserved(self):
        materials = NOTICES.collect(ROOT, RUSTC)['materials']
        text = '\n'.join(item['text'] for item in materials if item['legal_document'])
        self.assertIn('Copyright (c) 2018 Jorge Aparicio', text)
        self.assertIn('Copyright © 2005-2020 Rich Felker, et al.', text)
        self.assertIn('Copyright (c) 2009-2016', text)
        self.assertIn('LLVM Exceptions to the Apache 2.0 License', text)

    def test_unknown_toolchain_and_same_version_wrong_commit_are_rejected(self):
        for rustc in ['', 'rustc 1.99.1', RUSTC.replace('1.99.1', '1.88.0'), RUSTC.replace(NOTICES.RUST_REVISION, '0' * 40)]:
            with self.subTest(rustc=rustc):
                with self.assertRaisesRegex(ValueError, 'compiler source identity'):
                    NOTICES.collect(ROOT, rustc)

    def test_changed_profile_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'ci').mkdir()
            (root / 'ci/rust-compiler-notices.json').write_text('{}')
            with self.assertRaisesRegex(ValueError, 'profile differs from review'):
                NOTICES.collect(root, RUSTC)

    def test_missing_profile_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ValueError, 'missing or symlinked'):
                NOTICES.collect(Path(temporary), RUSTC)

    def test_changed_and_missing_original_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'ci').mkdir()
            profile = (ROOT / 'ci/rust-compiler-notices.json').read_bytes()
            (root / 'ci/rust-compiler-notices.json').write_bytes(profile)
            item = json.loads(profile)['materials'][0]
            original = root / item['path']
            original.parent.mkdir(parents=True)
            original.write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError, 'original identity differs'):
                NOTICES.collect(root, RUSTC)
            original.unlink()
            with self.assertRaisesRegex(ValueError, 'missing or symlinked'):
                NOTICES.collect(root, RUSTC)

    def test_collection_is_read_only_and_deterministic(self):
        before = (ROOT / 'ci/rust-compiler-notices.json').read_bytes()
        self.assertEqual(NOTICES.collect(ROOT, RUSTC), NOTICES.collect(ROOT, RUSTC))
        self.assertEqual((ROOT / 'ci/rust-compiler-notices.json').read_bytes(), before)

    def test_oversized_input_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / 'large.txt'
            source.write_bytes(b'12345')
            with self.assertRaisesRegex(ValueError, 'exceeds bound'):
                NOTICES.read(source, 4)


if __name__ == '__main__':
    unittest.main()
