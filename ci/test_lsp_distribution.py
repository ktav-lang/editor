"""Проверить обязательные legal/source роли и byte binding distribution payload."""

import gzip
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from check_embedded_notices import MARKER
from lsp_distribution import CORE_CHECKSUM, TARGETS, cpu, file_info, name, stage, verify


def digest(data):
    return hashlib.sha256(data).hexdigest()


def document(data):
    return {'text': data.decode('utf-8'), 'sha256': digest(data)}


class LspDistribution(unittest.TestCase):
    def fixture(self, root, target='aarch64-apple-darwin'):
        evidence = root / 'evidence'
        evidence.mkdir()
        sources = root / 'source'
        sources.mkdir()
        lock = ('version = 4\n[[package]]\nname = "ktav"\nversion = "0.8.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "' + CORE_CHECKSUM + '"\n').encode()
        stdlib = b'<html>Original Rust standard-library copyright.</html>\n'
        bundle = {'schema_version': 1, 'cargo_lock_sha256': digest(lock), 'packages': [{'name': 'ktav-lsp', 'version': '0.8.1', 'source': None}], 'licenses': [document(b'Original package permission and copyright.\n')], 'notices': [document(b'Additional original NOTICE.\n')], 'rust_standard_library': document(stdlib)}
        source_ref = 'a' * 40
        if 'linux' in target:
            arm = target.startswith('aarch64-')
            runtime = {'startup_terms': {'corresponding_source_required': arm, 'compiled_linking_exception': not arm}}
            if arm:
                runtime['corresponding_source'] = {'source_ref': source_ref, 'archive': 'ktav-lsp-source-' + target + '.tar.gz'}
            bundle['non_cargo_runtimes'] = [runtime]
        notice = (json.dumps(bundle, sort_keys=True) + '\n').encode()
        payload = gzip.compress(notice, mtime=0)
        binary = bytearray(128)
        if 'apple' in target:
            binary[:4] = b'\xcf\xfa\xed\xfe'
            binary[4:8] = (0x0100000C if target.startswith('aarch64-') else 0x01000007).to_bytes(4, 'little')
            binary[12:16] = (2).to_bytes(4, 'little')
        elif 'linux' in target:
            binary[:6] = b'\x7fELF\x02\x01'
            binary[16:18] = (3).to_bytes(2, 'little')
            binary[18:20] = (183 if target.startswith('aarch64-') else 62).to_bytes(2, 'little')
            binary[24:32] = (128).to_bytes(8, 'little')
        else:
            binary[:2] = b'MZ'
            binary[60:64] = (64).to_bytes(4, 'little')
            binary[64:68] = b'PE\0\0'
            binary[68:70] = (0xAA64 if target.startswith('aarch64-') else 0x8664).to_bytes(2, 'little')
            binary[86:88] = (2).to_bytes(2, 'little')
        binary = bytes(binary) + MARKER + len(payload).to_bytes(8, 'little') + payload
        executable = root / 'binary'
        executable.write_bytes(binary)
        for filename, data in [('Cargo.lock', lock), ('COPYRIGHT-library.html', stdlib), ('NOTICE.json', notice), ('NOTICE.json.gz', payload)]:
            (evidence / filename).write_bytes(data)
        (evidence / 'source-provenance.json').write_bytes(json.dumps({'source_ref': source_ref}).encode())
        report = {'binary_sha256': digest(binary), 'notice_sha256': digest(notice), 'target': target}
        if 'apple' in target:
            report.update({'dynamic_libraries': ['/usr/lib/libSystem.B.dylib'], 'iconv_symbol_imports': False})
        else:
            runtime_name = 'NOTICE.gnu-startup.txt' if 'linux' in target else 'NOTICE.windows-runtime.txt'
            plain = b'Original runtime copyright and full licence.\n'
            (evidence / runtime_name).write_bytes(plain)
            report['plain_notice_sha256'] = digest(plain)
        filename = 'native-macos-verification.json' if 'apple' in target else 'native-resource-verification.json' if 'linux' in target else 'native-windows-verification.json'
        (evidence / filename).write_bytes(json.dumps(report).encode())
        if target == 'aarch64-unknown-linux-gnu':
            archive = 'ktav-lsp-source-' + target + '.tar.gz'
            archive_data = gzip.compress(b'source binding unit fixture', mtime=0)
            (sources / archive).write_bytes(archive_data)
            binding = {'schema_version': 1, 'target': target, 'source_ref': source_ref, 'binary': {'file': 'ktav-lsp', 'sha256': digest(binary)}, 'notice': {'file': 'NOTICE.json', 'sha256': digest(notice)}, 'corresponding_source': {'file': archive, 'sha256': digest(archive_data)}}
            (evidence / 'native-source-binding.json').write_bytes(json.dumps(binding).encode())
        return evidence, executable, sources

    def test_stage_preserves_each_old_raw_asset_name(self):
        for target, (_, asset) in TARGETS.items():
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                evidence, binary, sources = self.fixture(root, target)
                output = root / 'payload'
                result = stage(evidence, binary, target, sources, output)
                self.assertEqual(result['binary'], asset)
                self.assertEqual((output / asset).read_bytes(), binary.read_bytes())
                self.assertIn(b'Original package permission and copyright.', (output / result['cargo_plain_notice']).read_bytes())
                self.assertEqual(verify(output)['target'], target)

    def test_missing_own_licence_is_rejected_even_if_manifest_is_rewritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root)
            output = root / 'payload'
            manifest = stage(evidence, binary, 'aarch64-apple-darwin', sources, output)
            dropped = manifest['first_party_licenses'].pop()
            (output / dropped).unlink()
            del manifest['files'][dropped]
            (output / (manifest['asset'] + '.distribution.json')).write_bytes(json.dumps(manifest).encode())
            with self.assertRaisesRegex(ValueError, 'legal/source role'):
                verify(output)

    def test_arm_requires_matching_complete_source_asset(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root, 'aarch64-unknown-linux-gnu')
            archive = next(sources.iterdir())
            archive.write_bytes(b'wrong corresponding-source bytes')
            with self.assertRaisesRegex(ValueError, 'not bound to distribution binary'):
                stage(evidence, binary, 'aarch64-unknown-linux-gnu', sources, root / 'payload')
            self.assertFalse((root / 'payload').exists())

    def test_source_obligation_cannot_be_removed_from_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root, 'aarch64-unknown-linux-gnu')
            output = root / 'payload'
            manifest = stage(evidence, binary, 'aarch64-unknown-linux-gnu', sources, output)
            manifest['corresponding_source'] = None
            (output / (manifest['asset'] + '.distribution.json')).write_bytes(json.dumps(manifest).encode())
            with self.assertRaisesRegex(ValueError, 'legal/source role'):
                verify(output)

    def test_changed_binary_or_notice_is_rejected(self):
        for changed in ['binary', 'notice']:
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                evidence, binary, sources = self.fixture(root)
                output = root / 'payload'
                manifest = stage(evidence, binary, 'aarch64-apple-darwin', sources, output)
                (output / manifest[changed]).write_bytes(b'changed')
                with self.assertRaisesRegex(ValueError, 'file identity mismatch'):
                    verify(output)

    def test_exact_compressed_resource_not_only_decoded_json_is_required(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root)
            (evidence / 'NOTICE.json.gz').write_bytes(gzip.compress((evidence / 'NOTICE.json').read_bytes(), compresslevel=1, mtime=123))
            with self.assertRaisesRegex(ValueError, 'resource differs'):
                stage(evidence, binary, 'aarch64-apple-darwin', sources, root / 'payload')

    def test_runtime_plain_notice_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root, 'x86_64-pc-windows-gnullvm')
            (evidence / 'NOTICE.windows-runtime.txt').write_bytes(b'incomplete notice')
            with self.assertRaisesRegex(ValueError, 'not bound to target verification'):
                stage(evidence, binary, 'x86_64-pc-windows-gnullvm', sources, root / 'payload')

    def test_nonempty_output_is_never_deleted_or_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence, binary, sources = self.fixture(root)
            output = root / 'payload'
            output.mkdir()
            sentinel = output / 'user-file'
            sentinel.write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'new empty directory'):
                stage(evidence, binary, 'aarch64-apple-darwin', sources, output)
            self.assertEqual(sentinel.read_bytes(), b'keep')

    def test_safe_names_and_real_size_bounds(self):
        for unsafe in ['../source.tar.gz', '/source.tar.gz', 'nested/source.tar.gz', 'C:source', '.hidden']:
            with self.subTest(name=unsafe), self.assertRaisesRegex(ValueError, 'unsafe distribution'):
                name(unsafe)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'source'
            path.write_bytes(b'1234')
            with self.assertRaisesRegex(ValueError, 'exceeds size bound'):
                file_info(path, 3)

    def test_shared_library_cannot_be_mislabeled_as_lsp_executable(self):
        for target in ['aarch64-unknown-linux-gnu', 'x86_64-pc-windows-gnullvm']:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                _, path, _ = self.fixture(Path(directory), target)
                binary = bytearray(path.read_bytes())
                if 'linux' in target:
                    binary[24:32] = bytes(8)
                else:
                    binary[86:88] = (0x2002).to_bytes(2, 'little')
                with self.assertRaisesRegex(ValueError, 'CPU/format mismatch'):
                    cpu(bytes(binary), target)

    def test_cpu_header_is_not_inferred_from_filename(self):
        with self.assertRaisesRegex(ValueError, 'CPU/format mismatch'):
            cpu(b'not an executable', 'aarch64-apple-darwin')


if __name__ == '__main__':
    unittest.main()
