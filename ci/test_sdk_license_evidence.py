"""Проверить привязку SDK license approvals к bytes, version и scope."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('sdk_license_evidence', root / 'check-sdk-license-evidence.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class SdkLicenseEvidenceTest(unittest.TestCase):
    def setUp(self):
        self.review = json.loads((root / 'reviewed-intellij-test-licenses.json').read_text(encoding='utf-8'))
        self.policy = json.loads((root / 'allowed-licenses.json').read_text(encoding='utf-8'))
        self.inventory = []
        for dependency in self.review['dependencies']:
            group, name = dependency['module_name'].split(':')
            self.inventory.append({
                'group': group, 'name': name, 'version': dependency['module_version'],
                'file': dependency['file_name'], 'sha256': dependency['binary_sha256'],
                'configurations': ['::testRuntimeClasspath'],
            })

    def test_exact_reviewed_artifacts(self):
        self.assertEqual(checker.verify(self.inventory, self.review, self.policy), 5)

    def test_binary_change_rejected(self):
        self.inventory[0]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'binary changed'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_runtime_scope_rejected(self):
        self.inventory[0]['configurations'].append('::runtimeClasspath')
        with self.assertRaisesRegex(ValueError, 'non-test'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_renamed_test_scope_rejected(self):
        self.inventory[0]['configurations'] = ['::contestRuntimeClasspath']
        with self.assertRaisesRegex(ValueError, 'non-test'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_version_change_rejected(self):
        self.inventory[0]['version'] = '243.99999.1'
        with self.assertRaisesRegex(ValueError, 'missing or duplicated'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_missing_artifact_rejected(self):
        self.inventory.pop()
        with self.assertRaisesRegex(ValueError, 'missing or duplicated'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_duplicate_artifact_rejected(self):
        self.inventory.append(copy.deepcopy(self.inventory[0]))
        with self.assertRaisesRegex(ValueError, 'missing or duplicated'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_license_change_rejected(self):
        self.review['upstream_license_text'] += 'changed'
        with self.assertRaisesRegex(ValueError, 'license changed'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_source_revision_change_rejected(self):
        self.review['source_revision'] = '0' * 40
        with self.assertRaisesRegex(ValueError, 'source revision'):
            checker.verify(self.inventory, self.review, self.policy)

    def test_unbound_missing_metadata_allowance_rejected(self):
        self.policy['allowedLicenses'].append({'moduleName': '.*'})
        with self.assertRaisesRegex(ValueError, 'unbound'):
            checker.verify(self.inventory, self.review, self.policy)


if __name__ == '__main__':
    unittest.main()
