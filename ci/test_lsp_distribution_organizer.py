"""Проверить перенос complete payloads без потери legal/source материалов."""

import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import test_lsp_distribution as fixtures
from lsp_distribution import TARGETS, stage, verify
from organize_lsp_distribution import organize


class PluginDistributionOrganization(unittest.TestCase):
    def inputs(self, root, omit=None):
        incoming = root / 'downloads'
        incoming.mkdir()
        for index, target in enumerate(TARGETS):
            if target == omit:
                continue
            temporary = root / ('producer-' + str(index))
            temporary.mkdir()
            evidence, binary, source = fixtures.LspDistribution().fixture(temporary, target)
            stage(evidence, binary, target, source, incoming / target)
        return incoming

    def test_all_platforms_keep_exact_binary_notice_and_source_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = self.inputs(root)
            output = root / 'plugin-bin'
            result = organize(incoming, output)
            self.assertEqual(len(result['platforms']), 6)
            for target, (platform, _) in TARGETS.items():
                original = verify(incoming / target)
                copied = verify(output / platform)
                expected_binary = 'ktav-lsp.exe' if 'windows' in target else 'ktav-lsp'
                self.assertEqual(copied['binary'], expected_binary)
                self.assertEqual((output / platform / expected_binary).read_bytes(), (incoming / target / original['binary']).read_bytes())
                for role in ['notice', 'notice_gzip', 'stdlib_report', 'cargo_plain_notice', 'cargo_lock']:
                    self.assertEqual((output / platform / copied[role]).read_bytes(), (incoming / target / original[role]).read_bytes())
                if copied['corresponding_source']:
                    source_name = copied['corresponding_source']
                    self.assertEqual((output / platform / source_name).read_bytes(), (incoming / target / source_name).read_bytes())
                    binding = json.loads((output / platform / copied['source_binding']).read_bytes())
                    self.assertEqual(binding['binary']['file'], expected_binary)
                    self.assertEqual(binding['binary']['sha256'], original['files'][original['binary']]['sha256'])

    def test_missing_platform_fails_before_output_is_created(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = self.inputs(root, omit='aarch64-pc-windows-gnullvm')
            output = root / 'plugin-bin'
            with self.assertRaisesRegex(ValueError, 'six matching-source'):
                organize(incoming, output)
            self.assertFalse(output.exists())

    def test_duplicate_platform_is_not_silently_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = self.inputs(root)
            shutil.copytree(incoming / 'x86_64-apple-darwin', incoming / 'duplicate')
            output = root / 'plugin-bin'
            with self.assertRaisesRegex(ValueError, 'duplicate plugin target'):
                organize(incoming, output)
            self.assertFalse(output.exists())

    def test_different_source_refs_cannot_be_mixed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = self.inputs(root)
            payload = incoming / 'x86_64-apple-darwin'
            manifest_path = next(payload.glob('*.distribution.json'))
            manifest = json.loads(manifest_path.read_bytes())
            manifest['source_ref'] = 'b' * 40
            manifest_path.write_bytes(json.dumps(manifest).encode('utf-8'))
            output = root / 'plugin-bin'
            with self.assertRaisesRegex(ValueError, 'six matching-source'):
                organize(incoming, output)
            self.assertFalse(output.exists())

    def test_existing_plugin_files_are_preserved_on_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = root / 'downloads'
            incoming.mkdir()
            output = root / 'plugin-bin'
            output.mkdir()
            sentinel = output / 'user-file'
            sentinel.write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'new and empty'):
                organize(incoming, output)
            self.assertEqual(sentinel.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
