"""Проверить source-backed разрешения только для точных test SDK artifacts."""

import hashlib
import json
from pathlib import Path
import re
import sys

REVISION = '9ae2edc879df5e9e189483752c4c4ae01650a566'
LICENSE_SHA256 = 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30'
VERSION = '243.21565.193'
MODULES = {
    'com.jetbrains.intellij.platform:test-framework',
    'com.jetbrains.intellij.platform:test-framework-common',
    'com.jetbrains.intellij.platform:test-framework-core',
    'com.jetbrains.intellij.platform:test-framework-team-city',
    'com.jetbrains.intellij.tools:tests-bootstrap',
}


def verify(inventory, review, policy):
    if not isinstance(inventory, list) or review.get('schema_version') != 1:
        raise ValueError('invalid SDK evidence schema')
    if review.get('source_revision') != REVISION:
        raise ValueError('unreviewed SDK source revision')
    text = review.get('upstream_license_text', '').encode('utf-8')
    if hashlib.sha256(text).hexdigest() != LICENSE_SHA256 or review.get('upstream_license_sha256') != LICENSE_SHA256:
        raise ValueError('SDK source license changed')
    dependencies = review.get('dependencies', [])
    if len(dependencies) != len(MODULES) or {item.get('module_name') for item in dependencies} != MODULES:
        raise ValueError('SDK review module set changed')
    expected_rules = {(re.escape(name), re.escape(VERSION)) for name in MODULES}
    actual_rules = [item for item in policy['allowedLicenses'] if not item.get('moduleLicense')]
    if len(actual_rules) != len(expected_rules) or {(item.get('moduleName'), item.get('moduleVersion')) for item in actual_rules} != expected_rules:
        raise ValueError('unbound missing-metadata license allowance')
    for dependency in dependencies:
        name = dependency['module_name']
        if dependency.get('module_version') != VERSION or dependency.get('license') != 'Apache-2.0':
            raise ValueError('unreviewed SDK version or license')
        source = dependency.get('source_archive', {})
        if not re.fullmatch(r'[0-9a-f]{64}', source.get('sha256', '')) or not dependency.get('source_files'):
            raise ValueError('SDK source evidence missing')
        group, module = name.split(':')
        matches = [item for item in inventory if item.get('group') == group and item.get('name') == module and item.get('version') == VERSION and item.get('file') == dependency['file_name']]
        if len(matches) != 1:
            raise ValueError(f'SDK artifact missing or duplicated: {name}')
        artifact = matches[0]
        if artifact.get('sha256') != dependency['binary_sha256']:
            raise ValueError(f'SDK binary changed: {name}')
        if set(artifact.get('configurations', [])) != {'::testRuntimeClasspath'} or dependency.get('configurations') != ['::testRuntimeClasspath']:
            raise ValueError(f'SDK artifact entered a non-test configuration: {name}')
    return len(dependencies)


def main():
    if len(sys.argv) != 4:
        raise SystemExit('usage: check-sdk-license-evidence.py INVENTORY REVIEW POLICY')
    try:
        inventory, review, policy = [json.loads(Path(name).read_text(encoding='utf-8')) for name in sys.argv[1:]]
        count = verify(inventory, review, policy)
        output = Path(sys.argv[1]).parent / 'reviewed-test-sdk.json'
        output.write_bytes(Path(sys.argv[2]).read_bytes())
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise SystemExit(str(error)) from error
    print(f'Verified source-backed licenses for {count} exact test-only SDK artifacts')


if __name__ == '__main__':
    main()
