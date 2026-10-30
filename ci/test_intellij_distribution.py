"""Проверить actual ZIP inventory guards без исполнения plugin code."""

import importlib.util
import io
from pathlib import Path
import stat
import tempfile
import unittest
import warnings
import zipfile

spec = importlib.util.spec_from_file_location('intellij_distribution', Path(__file__).with_name('check-intellij-distribution.py'))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class IntelliJDistributionTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.licenses = {'LICENSE-MIT': b'MIT terms', 'LICENSE-APACHE': b'Apache terms'}
        for name, value in self.licenses.items():
            (self.root / name).write_bytes(value)

    def jar(self, extra=(), descriptor=True, classes=True, missing=False, changed=False):
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, 'w') as archive:
            for name, value in self.licenses.items():
                if missing and name == 'LICENSE-MIT':
                    continue
                archive.writestr('META-INF/' + name, b'changed' if changed else value)
            if descriptor:
                archive.writestr('META-INF/plugin.xml', b'<idea-plugin/>')
            if classes:
                archive.writestr('lang/ktav/Example.class', b'\xca\xfe\xba\xbe')
            for name, value in extra:
                archive.writestr(name, value)
        return stream.getvalue()

    def package(self, jar=None, extra=()):
        path = self.root / 'plugin.zip'
        with zipfile.ZipFile(path, 'w') as archive:
            archive.writestr('ktav-intellij/lib/ktav.jar', self.jar() if jar is None else jar)
            for name, value in extra:
                archive.writestr(name, value)
        return path

    def test_own_license_bytes_and_namespace(self):
        result = checker.verify(self.package(), self.root)
        self.assertEqual(len(result['jars']), 1)
        self.assertEqual(result['jars'][0]['classes'], 1)
        self.assertEqual(result['native_lsp_payloads'], [])

    def test_searchable_options_jar_keeps_licenses(self):
        extra = [('ktav-intellij/lib/searchableOptions.jar', self.jar(descriptor=False, classes=False))]
        result = checker.verify(self.package(extra=extra), self.root)
        self.assertEqual(len(result['jars']), 2)

    def test_sdk_class_rejected(self):
        jar = self.jar([('com/intellij/SDK.class', b'\xca\xfe\xba\xbe')])
        with self.assertRaisesRegex(ValueError, 'non-Ktav class'):
            checker.verify(self.package(jar), self.root)

    def test_trove_class_rejected(self):
        jar = self.jar([('gnu/trove/THash.class', b'\xca\xfe\xba\xbe')])
        with self.assertRaisesRegex(ValueError, 'non-Ktav class'):
            checker.verify(self.package(jar), self.root)

    def test_missing_license_rejected(self):
        with self.assertRaisesRegex(ValueError, 'license missing'):
            checker.verify(self.package(self.jar(missing=True)), self.root)

    def test_modified_license_rejected(self):
        with self.assertRaisesRegex(ValueError, 'differs from source'):
            checker.verify(self.package(self.jar(changed=True)), self.root)

    def test_hidden_native_resource_rejected(self):
        with self.assertRaisesRegex(ValueError, 'hidden executable'):
            checker.verify(self.package(self.jar([('resource.bin', b'\x7fELF')])), self.root)

    def test_hidden_archive_rejected(self):
        with self.assertRaisesRegex(ValueError, 'hidden archive'):
            checker.verify(self.package(extra=[('ktav-intellij/resource.bin', b'PK\x03\x04')]), self.root)

    def test_traversal_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unsafe archive path'):
            checker.verify(self.package(extra=[('../outside', b'data')]), self.root)

    def test_symlink_rejected(self):
        link = zipfile.ZipInfo('ktav-intellij/link')
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        with self.assertRaisesRegex(ValueError, 'symlink'):
            checker.verify(self.package(extra=[(link, b'outside')]), self.root)

    def test_duplicate_jar_rejected(self):
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', UserWarning)
            path = self.package(extra=[('ktav-intellij/lib/ktav.jar', self.jar())])
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            checker.verify(path, self.root)

    def test_declared_lsp_is_inventoried_not_claimed_audited(self):
        path = self.package(extra=[('ktav-intellij/lib/bin/linux-x64/ktav-lsp', b'\x7fELF')])
        result = checker.verify(path, self.root)
        self.assertEqual(len(result['native_lsp_payloads']), 1)
        self.assertIn('separate evidence', result['coverage'])

    def test_sdk_native_name_rejected(self):
        path = self.package(extra=[('ktav-intellij/lib/bin/linux-x64/libjvm.so', b'\x7fELF')])
        with self.assertRaisesRegex(ValueError, 'payload name'):
            checker.verify(path, self.root)

    def test_java_class_cannot_hide_as_lsp(self):
        path = self.package(extra=[('ktav-intellij/lib/bin/linux-x64/ktav-lsp', b'\xca\xfe\xba\xbe')])
        with self.assertRaisesRegex(ValueError, 'native LSP format'):
            checker.verify(path, self.root)


if __name__ == '__main__':
    unittest.main()
