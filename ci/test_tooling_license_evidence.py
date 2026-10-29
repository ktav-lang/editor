"""Проверить version/hash/scope и условия ограниченных tooling approvals."""

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('tooling_evidence', root / 'check-tooling-license-evidence.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class ToolingEvidenceTest(unittest.TestCase):
    def setUp(self):
        self.review = json.loads((root / 'reviewed-intellij-tooling-licenses.json').read_text(encoding='utf-8'))
        self.policy = json.loads((root / 'allowed-licenses.json').read_text(encoding='utf-8'))
        self.inventory = []
        for item in self.review['dependencies']:
            group, name = item['module_name'].split(':')
            row = {'group': group, 'name': name, 'version': item['module_version'], 'file': item['file_name'], 'configurations': item['configurations'][:]}
            if item['kind'] in ('test-api', 'compiler-tooling'):
                row['sha256'] = item['binary_sha256']
            elif item['kind'] == 'plugin-marker':
                row.update(sha256=item['pom_sha256'], metadata_text=item['pom_text'], artifact_type='plugin-marker-pom', marker_code_artifacts=0)
            else:
                row['directory_notices'] = [{'path': notice['path'], 'sha256': notice['sha256'], 'text': item['license_texts'][notice['sha256']]} for notice in item['notice_files']]
                self.inventory.append({'group': group, 'name': name, 'version': item['module_version'], 'file': item['archive_file_name'], 'sha256': item['archive_sha256'], 'configurations': item['archive_configurations'][:]})
            self.inventory.append(row)
        self.distribution = {'schema_version': 1, 'jars': [{'classes': 82}]}

    def row(self, name):
        group, module = name.split(':')
        return next(item for item in self.inventory if item['group'] == group and item['name'] == module and not item['file'].endswith('.tar.gz'))

    def item(self, name):
        return next(item for item in self.review['dependencies'] if item['module_name'] == name)

    def verify(self):
        return checker.verify(self.inventory, self.review, self.policy, self.distribution)

    def test_exact_tooling_reviews(self):
        self.assertEqual(self.verify(), 5)

    def test_changed_binary_rejected(self):
        self.row('com.jetbrains:jbr-api')['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'binary identity'):
            self.verify()

    def test_unreviewed_version_rejected(self):
        self.row('com.jetbrains:jbr-api')['version'] = '1.0.1'
        with self.assertRaisesRegex(ValueError, 'tooling version'):
            self.verify()

    def test_trove_runtime_scope_rejected(self):
        self.row('org.jetbrains.intellij.deps:trove4j')['configurations'] = ['::runtimeClasspath']
        with self.assertRaisesRegex(ValueError, 'unreviewed configuration'):
            self.verify()

    def test_jbr_additional_scope_rejected(self):
        self.row('com.jetbrains:jbr-api')['configurations'].append('::compileClasspath')
        with self.assertRaisesRegex(ValueError, 'unreviewed configuration'):
            self.verify()

    def test_source_notice_changed_rejected(self):
        self.item('com.jetbrains:jbr-api')['source_files'][0]['notice'] += 'changed'
        with self.assertRaisesRegex(ValueError, 'source notices'):
            self.verify()

    def test_license_text_change_rejected(self):
        self.item('org.jetbrains.intellij.deps:trove4j')['full_license_text'] += 'changed'
        with self.assertRaisesRegex(ValueError, 'license text'):
            self.verify()

    def test_marker_code_rejected(self):
        self.row('org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin')['marker_code_artifacts'] = 1
        with self.assertRaisesRegex(ValueError, 'unreviewed code'):
            self.verify()

    def test_changed_marker_bytes_rejected(self):
        self.row('org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin')['metadata_text'] += 'changed'
        with self.assertRaisesRegex(ValueError, 'POM changed'):
            self.verify()

    def revised_marker(self, name, text):
        row, item = self.row(name), self.item(name)
        sha = hashlib.sha256(text.encode()).hexdigest()
        row['metadata_text'], row['sha256'] = text, sha
        item['pom_text'], item['pom_sha256'] = text, sha
        return patch.dict(checker.MARKERS, {name: (sha, checker.MARKERS[name][1])})

    def test_marker_jar_packaging_rejected_even_after_hash_review(self):
        name = 'org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin'
        text = self.item(name)['pom_text'].replace('<packaging>pom</packaging>', '<packaging>jar</packaging>')
        with self.revised_marker(name, text), self.assertRaisesRegex(ValueError, 'metadata-only'):
            self.verify()

    def test_marker_delegate_change_rejected_even_after_hash_review(self):
        name = 'org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin'
        text = self.item(name)['pom_text'].replace('<artifactId>kotlin-gradle-plugin</artifactId>', '<artifactId>other-plugin</artifactId>')
        with self.revised_marker(name, text), self.assertRaisesRegex(ValueError, 'implementation changed'):
            self.verify()

    def test_sdk_archive_change_rejected(self):
        next(item for item in self.inventory if item['file'].endswith('.tar.gz'))['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'SDK archive identity'):
            self.verify()

    def test_missing_sdk_notice_rejected(self):
        self.row('idea:ideaIC')['directory_notices'].pop()
        with self.assertRaisesRegex(ValueError, 'notice inventory'):
            self.verify()

    def test_changed_actual_sdk_notice_rejected(self):
        self.row('idea:ideaIC')['directory_notices'][0]['text'] += 'changed'
        with self.assertRaisesRegex(ValueError, 'actual SDK license text'):
            self.verify()

    def test_sdk_runtime_scope_rejected(self):
        self.row('idea:ideaIC')['configurations'].append('::runtimeClasspath')
        with self.assertRaisesRegex(ValueError, 'unreviewed configuration'):
            self.verify()

    def test_missing_distribution_evidence_rejected(self):
        self.distribution = {}
        with self.assertRaisesRegex(ValueError, 'distribution evidence'):
            self.verify()

    def test_global_gpl_allowance_rejected(self):
        self.policy['allowedLicenses'].append({'moduleLicense': 'GNU.*'})
        with self.assertRaisesRegex(ValueError, 'global GPL'):
            self.verify()

    def test_unknown_metadata_allowance_rejected(self):
        self.policy['allowedLicenses'].append({'moduleName': '.*'})
        with self.assertRaisesRegex(ValueError, 'unbound'):
            self.verify()

    def test_duplicate_binary_rejected(self):
        self.inventory.append(copy.deepcopy(self.row('com.jetbrains:jbr-api')))
        with self.assertRaisesRegex(ValueError, 'missing or duplicated'):
            self.verify()


if __name__ == '__main__':
    unittest.main()
