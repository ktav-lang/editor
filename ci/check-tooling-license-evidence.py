"""Проверить точные build/tooling approvals, не выдавая SDK runtime allowance."""

import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import sys
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('sdk_license_evidence', ROOT / 'check-sdk-license-evidence.py')
sdk = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sdk)

VERSIONS = {
    'com.jetbrains:jbr-api': '1.0.0',
    'org.jetbrains.intellij.deps:trove4j': '1.0.20200330',
    'org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin': '2.0.0',
    'org.jetbrains.intellij.platform:org.jetbrains.intellij.platform.gradle.plugin': '2.2.0',
    'idea:ideaIC': '2025.2',
}
BINARIES = {
    'com.jetbrains:jbr-api': ('test-api', '::testRuntimeClasspath', 'f472369968f6692804e528e3a17d3d71386ac502d1312164b6dd670b46d966e3', '4d817ce6fcc231488c4531f010ea3b6b60662bd733a4ac2d5093fafac73ebd19', '984e98c3978c4a937668f5b7dab4c7016ad4b9733150ea3c66a235c1afe9750a', sdk.LICENSE_SHA256),
    'org.jetbrains.intellij.deps:trove4j': ('compiler-tooling', '::kotlinKlibCommonizerClasspath', 'c5fd725bffab51846bf3c77db1383c60aaaebfe1b7fe2f00d23fe1b7df0a439d', '2e0174e80b989eb5deafcfb77407c908a0abe1f67a80306f98aeca63246e5481', '7d4ca9217f2dd7a1992504dab980d1809abb10e0a9e49eabe8990680e3596035', '20e50fe7aae3e56378ebf0417d9de904f55a0e61e4df315333e632a4d3555d95'),
}
MARKERS = {
    'org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin': ('e48cc51526cf45e2dfe7dee2b9a71bbb39cc7742fb2f068fc57fb3a19a09fdd6', 'org.jetbrains.kotlin:kotlin-gradle-plugin:2.0.0'),
    'org.jetbrains.intellij.platform:org.jetbrains.intellij.platform.gradle.plugin': ('1a424f172cb381bc04bd369cc4ea179404ec76df13ffc89b4d667ca1f7f8dd44', 'org.jetbrains.intellij.platform:intellij-platform-gradle-plugin:2.2.0'),
}
SDK_TAR = '30e917c55507443c61674d0bdd5cbb40a5b02c628d5073f8a0028ef49e0c50b5'
SDK_MANIFEST = 'df0bb8a33659a62dd4e55c37d99b9a1cd256090810ff399e63026bb8bc5de427'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(items):
    return json.dumps(items, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')


def scope(row, expected):
    if set(row.get('configurations', [])) != {expected}:
        raise ValueError('tooling entered an unreviewed configuration')


def rows_for(inventory, name, version):
    group, module = name.split(':')
    rows = [row for row in inventory if row.get('group') == group and row.get('name') == module]
    permitted_versions = {version, '2024.3'} if name == 'idea:ideaIC' else {version}
    if any(row.get('version') not in permitted_versions for row in rows):
        raise ValueError('unreviewed tooling version')
    return [row for row in rows if row.get('version') == version]


def verify(inventory, review, policy, distribution):
    if review.get('schema_version') != 1:
        raise ValueError('invalid tooling review schema')
    dependencies = review.get('dependencies', [])
    if len(dependencies) != len(VERSIONS) or {item.get('module_name') for item in dependencies} != set(VERSIONS):
        raise ValueError('unreviewed tooling module set')
    expected_rules = {(re.escape(name), re.escape(sdk.VERSION)) for name in sdk.MODULES} | {(re.escape(name), re.escape(version)) for name, version in sdk.OTHER_MISSING_METADATA}
    missing = [rule for rule in policy['allowedLicenses'] if not rule.get('moduleLicense')]
    if len(missing) != len(expected_rules) or {(rule.get('moduleName'), rule.get('moduleVersion')) for rule in missing} != expected_rules:
        raise ValueError('unbound missing-metadata allowance')
    trove = [rule for rule in policy['allowedLicenses'] if rule.get('moduleName') == re.escape('org.jetbrains.intellij.deps:trove4j')]
    if len(trove) != 1 or trove[0].get('moduleVersion') != re.escape(VERSIONS['org.jetbrains.intellij.deps:trove4j']) or trove[0].get('moduleLicense') != 'GNU LESSER GENERAL PUBLIC LICENSE 2[.]1':
        raise ValueError('unbound LGPL tooling allowance')
    if any(re.search(r'GNU|GPL', rule.get('moduleLicense', '')) and rule not in trove for rule in policy['allowedLicenses']):
        raise ValueError('unreviewed global GPL allowance')
    if distribution.get('schema_version') != 1 or not distribution.get('jars') or not any(jar.get('classes', 0) for jar in distribution['jars']):
        raise ValueError('actual plugin distribution evidence missing')

    for item in dependencies:
        name = item['module_name']
        version = VERSIONS[name]
        if item.get('module_version') != version:
            raise ValueError('tooling review version changed')
        rows = rows_for(inventory, name, version)
        if name in BINARIES:
            kind, expected_scope, binary_hash, source_hash, notice_hash, license_hash = BINARIES[name]
            if item.get('kind') != kind or len(rows) != 1:
                raise ValueError('tooling binary missing or duplicated')
            row = rows[0]
            if row.get('file') != item['file_name'] or row.get('sha256') != binary_hash or item.get('binary_sha256') != binary_hash:
                raise ValueError('tooling binary identity changed')
            scope(row, expected_scope)
            scope(item, expected_scope)
            if item['source_archive']['sha256'] != source_hash or digest(canonical(sorted(item['source_files'], key=lambda source: source['path']))) != notice_hash:
                raise ValueError('tooling source notices changed')
            if item.get('full_license_sha256') != license_hash or digest(item['full_license_text'].encode()) != license_hash:
                raise ValueError('tooling license text changed')
            expected_license = 'Apache-2.0' if kind == 'test-api' else 'LGPL-2.1-only AND LicenseRef-CERN-Trove'
            if item.get('license') != expected_license:
                raise ValueError('unreviewed tooling license selection')
        elif name in MARKERS:
            pom_hash, implementation = MARKERS[name]
            if item.get('kind') != 'plugin-marker' or len(rows) != 1 or rows[0].get('artifact_type') != 'plugin-marker-pom' or rows[0].get('marker_code_artifacts') != 0:
                raise ValueError('plugin marker contains unreviewed code or metadata')
            row = rows[0]
            scope(row, '::buildscript:classpath')
            scope(item, '::buildscript:classpath')
            data = row.get('metadata_text', '').encode('utf-8')
            if row.get('sha256') != pom_hash or item.get('pom_sha256') != pom_hash or digest(data) != pom_hash or data != item['pom_text'].encode('utf-8'):
                raise ValueError('plugin marker POM changed')
            if b'<!DOCTYPE' in data.upper() or b'<!ENTITY' in data.upper():
                raise ValueError('plugin marker XML entities forbidden')
            document = ET.fromstring(data)
            ns = {'m': 'http://maven.apache.org/POM/4.0.0'}
            gav = ':'.join(document.findtext('m:' + field, namespaces=ns).strip() for field in ('groupId', 'artifactId', 'version'))
            children = document.findall('m:dependencies/m:dependency', ns)
            if gav != name + ':' + version or document.findtext('m:packaging', namespaces=ns) != 'pom' or len(children) != 1:
                raise ValueError('plugin marker is not metadata-only delegation')
            target = ':'.join(children[0].findtext('m:' + field, namespaces=ns).strip() for field in ('groupId', 'artifactId', 'version'))
            if target != implementation or item.get('implementation') != implementation:
                raise ValueError('plugin marker implementation changed')
        else:
            if item.get('kind') != 'verifier-sdk' or len(rows) != 2:
                raise ValueError('SDK verifier artifact set changed')
            directory = next((row for row in rows if row.get('file') == item['file_name']), None)
            archive = next((row for row in rows if row.get('file') == item['archive_file_name']), None)
            if not directory or not archive or archive.get('sha256') != SDK_TAR or item.get('archive_sha256') != SDK_TAR:
                raise ValueError('SDK archive identity changed')
            scope(directory, '::intellijPluginVerifierIdes')
            scope(archive, '::intellijPluginVerifierIdesDependency')
            scope(item, '::intellijPluginVerifierIdes')
            if item.get('archive_configurations') != ['::intellijPluginVerifierIdesDependency']:
                raise ValueError('SDK archive entered an unreviewed configuration')
            expected = sorted(item['notice_files'], key=lambda notice: notice['path'])
            actual = sorted([{'path': notice['path'], 'sha256': notice['sha256']} for notice in directory.get('directory_notices', [])], key=lambda notice: notice['path'])
            if len(actual) != 209 or len({notice['path'] for notice in actual}) != 209 or actual != expected or digest(canonical(expected)) != SDK_MANIFEST:
                raise ValueError('SDK license notice inventory changed')
            texts = item['license_texts']
            if len(texts) != 44 or set(texts) != {notice['sha256'] for notice in expected}:
                raise ValueError('SDK license texts missing')
            if any(digest(text.encode('utf-8')) != sha for sha, text in texts.items()):
                raise ValueError('SDK review license text changed')
            for notice in directory['directory_notices']:
                if digest(notice['text'].encode('utf-8')) != notice['sha256'] or texts[notice['sha256']] != notice['text']:
                    raise ValueError('actual SDK license text changed')
    return len(dependencies)


def load(path):
    with path.open('rb') as stream:
        data = stream.read(16 * 1024 * 1024 + 1)
    if len(data) > 16 * 1024 * 1024:
        raise ValueError('tooling evidence exceeds size limit')
    return json.loads(data)


def main():
    if len(sys.argv) != 5:
        raise SystemExit('usage: check-tooling-license-evidence.py INVENTORY REVIEW POLICY DISTRIBUTION_REPORT')
    try:
        inventory_path, review_path, policy_path, distribution_path = map(Path, sys.argv[1:])
        inventory, review, policy, distribution = map(load, (inventory_path, review_path, policy_path, distribution_path))
        archive_name = distribution['archive']
        if PurePosixPath(archive_name).name != archive_name or '\\' in archive_name or ':' in archive_name or not archive_name.endswith('.zip'):
            raise ValueError('invalid distribution evidence path')
        archive_path = inventory_path.parent.parent / 'distributions' / archive_name
        module_spec = importlib.util.spec_from_file_location('intellij_distribution', ROOT / 'check-intellij-distribution.py')
        archive_checker = importlib.util.module_from_spec(module_spec)
        module_spec.loader.exec_module(archive_checker)
        actual = archive_checker.verify(archive_path, ROOT.parent)
        if actual != distribution:
            raise ValueError('actual plugin distribution evidence differs')
        count = verify(inventory, review, policy, distribution)
        (inventory_path.parent / 'reviewed-tooling-licenses.json').write_bytes(review_path.read_bytes())
    except (OSError, ValueError, KeyError, TypeError, AttributeError, ET.ParseError, zipfile.BadZipFile) as error:
        raise SystemExit(str(error)) from error
    print(f'Verified {count} exact build/tooling reviews and actual plugin non-redistribution boundary')


if __name__ == '__main__':
    main()
