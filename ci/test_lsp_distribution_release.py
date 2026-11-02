"""Проверить полную доставку raw binaries, notices и matching source."""

from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import test_lsp_distribution_organizer as fixtures
from lsp_distribution import verify
from stage_lsp_release_assets import stage


class ReleasePayloadStaging(unittest.TestCase):
    def test_every_producer_file_is_preserved_byte_for_byte(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = fixtures.PluginDistributionOrganization().inputs(root)
            source_ref = verify(next(incoming.iterdir()))['source_ref']
            output = root / 'release-assets'
            result = stage(incoming, output, source_ref)
            expected = set()
            for producer in incoming.iterdir():
                for path in producer.iterdir():
                    expected.add(path.name)
                    self.assertEqual((output / path.name).read_bytes(), path.read_bytes())
            self.assertEqual(set(result['files']), expected)
            self.assertEqual({path.name for path in output.iterdir()}, expected)
            self.assertIn('ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz', expected)

    def test_wrong_source_ref_fails_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = fixtures.PluginDistributionOrganization().inputs(root)
            output = root / 'release-assets'
            with self.assertRaisesRegex(ValueError, 'matching-source'):
                stage(incoming, output, 'f' * 40)
            self.assertFalse(output.exists())

    def test_missing_platform_fails_before_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = fixtures.PluginDistributionOrganization().inputs(root, omit='aarch64-apple-darwin')
            source_ref = verify(next(incoming.iterdir()))['source_ref']
            output = root / 'release-assets'
            with self.assertRaisesRegex(ValueError, 'all six'):
                stage(incoming, output, source_ref)
            self.assertFalse(output.exists())

    def test_existing_output_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = root / 'incoming'
            incoming.mkdir()
            output = root / 'release-assets'
            output.mkdir()
            sentinel = output / 'existing'
            sentinel.write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'new and empty'):
                stage(incoming, output, 'a' * 40)
            self.assertEqual(sentinel.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
