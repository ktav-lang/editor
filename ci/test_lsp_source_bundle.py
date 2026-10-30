"""Проверить source archive guard на подменах и небезопасных members."""

import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('source_bundle', Path(__file__).resolve().parents[1] / 'scripts/verify_lsp_source_bundle.py')
BUNDLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUNDLE)


class SourceBundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        names = ['project/Cargo.toml', 'project/Cargo.lock', 'project/build.rs', 'project/src/main.rs', 'project/.cargo/config.toml', 'project/LICENSE-MIT', 'project/LICENSE-APACHE', 'legal/LGPL-2.1.txt', 'legal/NOTICE.json', 'legal/NOTICE.json.gz', 'upstream/zig-source.tar.gz', 'rebuild.sh', 'provenance.json', 'README.txt', 'project/vendor/fixture/src/lib.rs', 'toolchain/lib/rustlib/aarch64-unknown-linux-gnu/lib/libfixture.rlib']
        self.files = {name: b'fixture' for name in names}

    def archive(self, changed=None, extra=None, link=False):
        manifest = [{'path': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': 0o644} for name, data in self.files.items()]
        files = {'lsp-source/' + name: data for name, data in self.files.items()}
        files['lsp-source/MANIFEST.json'] = json.dumps(manifest).encode()
        if changed:
            files['lsp-source/' + changed] = b'changed'
        if extra:
            files[extra] = b'extra'
        path = self.root / 'source.tar.gz'
        with tarfile.open(path, 'w:gz') as archive:
            for name, data in files.items():
                member = tarfile.TarInfo(name)
                member.size = len(data)
                archive.addfile(member, io.BytesIO(data))
            if link:
                member = tarfile.TarInfo('lsp-source/link')
                member.type = tarfile.SYMTYPE
                member.linkname = '/outside'
                archive.addfile(member)
        return path

    def test_complete_manifest(self):
        self.assertTrue(BUNDLE.inspect(self.archive())['verified_manifest'])

    def test_changed_source_rejected(self):
        with self.assertRaisesRegex(ValueError, 'differs from manifest'):
            BUNDLE.inspect(self.archive(changed='project/src/main.rs'))

    def test_unlisted_member_rejected(self):
        with self.assertRaisesRegex(ValueError, 'inventory'):
            BUNDLE.inspect(self.archive(extra='lsp-source/extra'))

    def test_traversal_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            BUNDLE.inspect(self.archive(extra='lsp-source/../escape'))

    def test_link_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            BUNDLE.inspect(self.archive(link=True))

    def test_missing_vendor_rejected(self):
        del self.files['project/vendor/fixture/src/lib.rs']
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            BUNDLE.inspect(self.archive())

    def test_missing_target_library_rejected(self):
        del self.files['toolchain/lib/rustlib/aarch64-unknown-linux-gnu/lib/libfixture.rlib']
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            BUNDLE.inspect(self.archive())

    def test_existing_extract_directory_rejected(self):
        with self.assertRaisesRegex(ValueError, 'must be new'):
            BUNDLE.verify_and_extract(self.archive(), self.root)


if __name__ == '__main__':
    unittest.main()
