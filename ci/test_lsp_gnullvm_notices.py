"""Регрессионные проверки source-backed Windows runtime notices."""

import copy
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from check_lsp_gnullvm import machine, retained_objects, sdk_inputs
from lsp_gnullvm_notices import augment, catalogue, original, plain_notice, verify_sources


class WindowsRuntimeNotices(unittest.TestCase):
    def fixture(self, data=b'/* Copyright Example. Permission to redistribute. */\nint x;\n'):
        url = 'https://raw.githubusercontent.com/mingw-w64/mingw-w64/0d6dfbc348dfad714761c0a7151615767f96bcb5/example.c'
        item = {'path': 'example.c', 'source': url, 'sha256': hashlib.sha256(data).hexdigest(), 'source_notices': ['/* Copyright Example. Permission to redistribute. */']}
        return {'source_files': [item], 'original_documents': []}, lambda _: data

    def test_transient_transport_failure_is_retried(self):
        url = 'https://raw.githubusercontent.com/mingw-w64/mingw-w64/0d6dfbc348dfad714761c0a7151615767f96bcb5/example.c'
        with patch('lsp_gnullvm_notices.urllib.request.urlopen', side_effect=[urllib.error.URLError('TLS timeout'), io.BytesIO(b'original source')]) as request:
            self.assertEqual(original(url), b'original source')
            self.assertEqual(request.call_count, 2)

    def test_http_access_denial_is_not_retried(self):
        url = 'https://raw.githubusercontent.com/mingw-w64/mingw-w64/0d6dfbc348dfad714761c0a7151615767f96bcb5/example.c'
        with patch('lsp_gnullvm_notices.urllib.request.urlopen', side_effect=urllib.error.HTTPError(url, 403, 'Forbidden', None, None)) as request:
            with self.assertRaises(urllib.error.HTTPError):
                original(url)
            self.assertEqual(request.call_count, 1)

    def test_exact_source_notice(self):
        catalog, read = self.fixture()
        verify_sources(catalog, read)

    def test_modified_source_bytes_rejected(self):
        catalog, _ = self.fixture()
        with self.assertRaisesRegex(ValueError, 'bytes differ'):
            verify_sources(catalog, lambda _: b'changed source')

    def test_missing_copyright_rejected(self):
        catalog, read = self.fixture()
        catalog['source_files'][0]['source_notices'] = []
        with self.assertRaisesRegex(ValueError, 'notices differ'):
            verify_sources(catalog, read)

    def test_unknown_unlicensed_source_rejected(self):
        catalog, read = self.fixture(b'int x;\n')
        catalog['source_files'][0]['source_notices'] = []
        with self.assertRaisesRegex(ValueError, 'notices differ'):
            verify_sources(catalog, read)

    def test_original_legal_document_must_match_bytes(self):
        data = b'Original permission and copyright.\n'
        catalog = {'source_files': [], 'original_documents': [{'source': 'document', 'sha256': hashlib.sha256(data).hexdigest(), 'text': 'Changed permission.\n'}]}
        with self.assertRaisesRegex(ValueError, 'document differs'):
            verify_sources(catalog, lambda _: data)

    def test_sdk_identity_is_not_an_allowlist_fallback(self):
        root = Path(__file__).resolve().parents[1]
        catalog, _ = catalogue(root)
        catalog['sdk_archive_sha256'] = '0' * 64
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            (temporary / 'ci').mkdir()
            (temporary / 'ci/lsp-gnullvm-runtime-catalog.json').write_text(json.dumps(catalog), encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'unreviewed Windows runtime SDK'):
                catalogue(temporary)

    def test_plain_notice_keeps_every_original_document(self):
        catalog, _ = catalogue(Path(__file__).resolve().parents[1])
        result = plain_notice(catalog)
        for document in catalog['original_documents']:
            self.assertIn(document['text'].encode('utf-8'), result)
        self.assertIn(b'2009-2014 by the contributors', result)
        self.assertIn(b'1997, 1998, 1999, 2000, 2001 by Lucent', result)

    def test_unknown_target_and_double_augmentation_rejected(self):
        catalog, digest = catalogue(Path(__file__).resolve().parents[1])
        bundle = {'schema_version': 1, 'coverage': 'Cargo/std.'}
        with self.assertRaisesRegex(ValueError, 'preparation state'):
            augment(bundle, catalog, 'x86_64-pc-windows-msvc', digest)
        already = augment(copy.deepcopy(bundle), catalog, 'x86_64-pc-windows-gnullvm', digest)
        with self.assertRaisesRegex(ValueError, 'preparation state'):
            augment(already, catalog, 'x86_64-pc-windows-gnullvm', digest)

    def test_unknown_map_input_is_not_silently_dropped(self):
        sample = b'00001000 00001000 4     new-runtime.o:(.text)\n00002000 00002000 4     own.rcgu.o:(.text)\n'
        self.assertEqual(retained_objects(sample), {'new-runtime.o'})

    def test_duplicate_and_foreign_sdk_input_rejected(self):
        item = {'path': 'link/llvm-mingw-20261020-ucrt-x86_64/x86_64-w64-mingw32/lib/libunwind.a', 'sha256': 'a' * 64, 'size': 12}
        with self.assertRaisesRegex(ValueError, 'duplicate native SDK'):
            sdk_inputs([item, item])
        with self.assertRaisesRegex(ValueError, 'outside selected'):
            sdk_inputs([{**item, 'path': 'unexpected/libunwind.a'}])

    def test_actual_pe_machine_is_read(self):
        data = bytearray(70)
        data[:2] = b'MZ'
        data[60:64] = (64).to_bytes(4, 'little')
        data[64:68] = b'PE\0\0'
        data[68:70] = (0xAA64).to_bytes(2, 'little')
        self.assertEqual(machine(bytes(data)), 0xAA64)
        data[60:64] = (4096).to_bytes(4, 'little')
        with self.assertRaisesRegex(ValueError, 'invalid Windows LSP PE header'):
            machine(bytes(data))


if __name__ == '__main__':
    unittest.main()
