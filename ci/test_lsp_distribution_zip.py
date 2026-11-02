"""Проверить те же обязательные payload roles внутри обоих plugin containers."""

import io
import json
from pathlib import Path
import stat
import sys
import tempfile
import unittest
from unittest.mock import patch
import warnings
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import test_lsp_distribution_organizer as fixtures
from check_lsp_plugin_zip import PREFIXES, SOURCE, inventory, verify
from organize_lsp_distribution import organize


class PluginZipDistribution(unittest.TestCase):
    def build(self, root, kind, change=None, extra=None):
        incoming = fixtures.PluginDistributionOrganization().inputs(root)
        output = root / 'plugin-bin'
        organize(incoming, output)
        if change:
            change(output)
        archive = root / ('plugin.' + ('vsix' if kind == 'vsix' else 'zip'))
        with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as destination:
            for path in output.rglob('*'):
                if path.is_file():
                    destination.write(path, PREFIXES[kind] + path.relative_to(output).as_posix())
            if extra:
                extra(destination)
        return archive

    def test_all_six_payloads_and_source_are_verified_in_both_containers(self):
        for kind in PREFIXES:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as directory:
                report = verify(self.build(Path(directory), kind), kind)
                self.assertEqual(len(report['platforms']), 6)
                self.assertEqual(report['verified_source_entries'], [PREFIXES[kind] + SOURCE])

    def test_missing_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = self.build(Path(directory), 'vsix', lambda output: (output / SOURCE).unlink())
            with self.assertRaisesRegex(ValueError, 'missing or unexpected'):
                verify(archive, 'vsix')

    def test_notice_mutation_is_rejected(self):
        def change(output):
            next(output.rglob('*.NOTICE.json')).write_bytes(b'{}')
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'identity mismatch'):
                verify(self.build(Path(directory), 'intellij', change), 'intellij')

    def test_manifest_cannot_drop_source_role(self):
        def change(output):
            path = next((output / 'linux-arm64').glob('*.distribution.json'))
            manifest = json.loads(path.read_bytes())
            manifest['files'].pop(manifest['corresponding_source'])
            manifest['corresponding_source'] = None
            path.write_text(json.dumps(manifest), encoding='utf-8')
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'required distribution legal/source role'):
                verify(self.build(Path(directory), 'vsix', change), 'vsix')

    def test_unexpected_payload_file_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = self.build(Path(directory), 'vsix', extra=lambda z: z.writestr('extension/bin/linux-x64/extra', b'unknown'))
            with self.assertRaisesRegex(ValueError, 'missing or unexpected'):
                verify(archive, 'vsix')

    def test_unsafe_paths_are_rejected_even_outside_payload(self):
        for filename in ['../escape', '/absolute', 'extension/../escape', 'extension//alias', 'extension\\escape']:
            with self.subTest(filename=filename), tempfile.TemporaryDirectory() as directory:
                archive = self.build(Path(directory), 'vsix', extra=lambda z: z.writestr(filename, b'bad'))
                with self.assertRaisesRegex(ValueError, 'unsafe archive path'):
                    verify(archive, 'vsix')

    def test_duplicate_member_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with warnings.catch_warnings():
                warnings.simplefilter('ignore', UserWarning)
                archive = self.build(Path(directory), 'vsix', extra=lambda z: z.writestr(PREFIXES['vsix'] + SOURCE, b'bad'))
            with self.assertRaisesRegex(ValueError, 'duplicate archive member'):
                verify(archive, 'vsix')

    def test_only_exact_source_path_has_a_larger_inventory_bound(self):
        prefix = PREFIXES['vsix']
        for filename, accepted in [(prefix + SOURCE, True), ('extension/other.tar.gz', False), (prefix + SOURCE.replace('linux-arm64', 'linux-x64'), False)]:
            with self.subTest(filename=filename):
                data = io.BytesIO()
                with zipfile.ZipFile(data, 'w') as archive:
                    archive.writestr(filename, b'12345')
                data.seek(0)
                with zipfile.ZipFile(data) as archive, patch('check_lsp_plugin_zip.MAX_ENTRY', 4):
                    if accepted:
                        self.assertIn(filename, inventory(archive, prefix))
                    else:
                        with self.assertRaisesRegex(ValueError, 'size limit'):
                            inventory(archive, prefix)

    def test_altered_source_binding_is_rejected_even_with_updated_file_hash(self):
        def change(output):
            import hashlib
            manifest_path = next((output / 'linux-arm64').glob('*.distribution.json'))
            manifest = json.loads(manifest_path.read_bytes())
            binding_path = output / 'linux-arm64' / manifest['source_binding']
            binding = json.loads(binding_path.read_bytes())
            binding['binary']['file'] = 'wrong-executable'
            data = json.dumps(binding).encode('utf-8')
            binding_path.write_bytes(data)
            manifest['files'][manifest['source_binding']] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
            manifest_path.write_text(json.dumps(manifest), encoding='utf-8')
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'corresponding-source binding mismatch'):
                verify(self.build(Path(directory), 'intellij', change), 'intellij')

    def test_symlink_is_rejected(self):
        def extra(archive):
            item = zipfile.ZipInfo('extension/link')
            item.create_system = 3
            item.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(item, b'target')
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'archive symlink'):
                verify(self.build(Path(directory), 'vsix', extra=extra), 'vsix')


if __name__ == '__main__':
    unittest.main()
