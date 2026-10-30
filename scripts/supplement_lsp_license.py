"""Дополнить один проверенный cfg-only crate, сохранив оригинальный tool report."""

import copy
import hashlib
import json
from pathlib import Path
import tomllib

PACKAGE = 'registry+https://github.com/rust-lang/crates.io-index#valuable@0.1.1'
CHECKSUM = 'ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65'
VCS = '9efc29b6e58cef28f6566a47aa7e142a55fead77'
LICENSE_SHA = 'ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44'


def supplement(metadata, report, lock_bytes, license_path):
    packages = {item['id']: item for item in metadata['packages']}
    reported = {item['package']['id'] for item in report['crates']}
    licensed = {item['crate']['id'] for license in report['licenses'] for item in license['used_by']}
    missing = set(packages) - reported
    if len(packages) != len(metadata['packages']) or len(reported) != len(report['crates']):
        raise ValueError('duplicate package identity in license evidence')
    if not missing and set(packages) == licensed and reported == set(packages):
        return report, []
    if missing != {PACKAGE} or set(packages) - licensed != {PACKAGE} or reported - set(packages) or licensed - set(packages):
        raise ValueError('unreviewed Cargo/license coverage mismatch')
    package = packages[PACKAGE]
    if package.get('name') != 'valuable' or package.get('version') != '0.1.1' or package.get('license') != 'MIT' or package.get('source') != 'registry+https://github.com/rust-lang/crates.io-index':
        raise ValueError('unreviewed cfg-only source package')
    locks = [item for item in tomllib.loads(lock_bytes.decode('utf-8'))['package'] if item['name'] == 'valuable']
    if len(locks) != 1 or locks[0].get('version') != '0.1.1' or locks[0].get('checksum') != CHECKSUM:
        raise ValueError('cfg-only package checksum changed')
    incoming = [dependency for item in metadata['packages'] for dependency in item['dependencies'] if dependency['name'] == 'valuable']
    if len(incoming) != 2 or any(not item.get('optional') or item.get('target') != 'cfg(tracing_unstable)' for item in incoming):
        raise ValueError('cfg-only dependency activation requires review')
    vcs_path = Path(package['manifest_path']).parent / '.cargo_vcs_info.json'
    vcs = json.loads(vcs_path.read_text(encoding='utf-8'))
    if vcs.get('git', {}).get('sha1') != VCS or vcs.get('path_in_vcs') != 'valuable':
        raise ValueError('cfg-only published source VCS identity changed')
    if license_path.is_symlink() or not license_path.is_file() or license_path.stat().st_size > 16 * 1024:
        raise ValueError('invalid reviewed upstream license file')
    license_bytes = license_path.read_bytes()
    if hashlib.sha256(license_bytes).hexdigest() != LICENSE_SHA:
        raise ValueError('reviewed upstream cfg-only license changed')
    provenance = {'package': {'name': 'valuable', 'version': '0.1.1', 'source': package['source']}, 'crate_checksum': CHECKSUM, 'vcs_revision': VCS, 'license': 'MIT', 'license_sha256': LICENSE_SHA,
                  'license_source': 'https://raw.githubusercontent.com/tokio-rs/valuable/' + VCS + '/LICENSE', 'reason': 'Conservative metadata includes optional tracing_unstable-only package excluded by cargo-about; full source-backed terms retained explicitly.'}
    combined = copy.deepcopy(report)
    combined['crates'].append({'package': package, 'license': {'origin': 'reviewed-source-supplement', 'spdx': 'MIT'}})
    combined['licenses'].append({'name': 'MIT', 'id': 'MIT', 'first_of_kind': False, 'text': license_bytes.decode('utf-8'), 'source_path': str(license_path), 'used_by': [{'crate': package, 'path': str(license_path)}]})
    combined['source_license_supplements'] = [provenance]
    combined['report_origin'] = 'cargo-about plus explicit reviewed source supplement; original cargo-about bytes preserved separately'
    return combined, [provenance]


def supplement_fallbacks(metadata, report, lock_bytes, catalog_path):
    catalog = json.loads(catalog_path.read_text(encoding='utf-8'))
    reviewed = {item['name']: item for item in catalog['packages']}
    if len(reviewed) != len(catalog['packages']):
        raise ValueError('duplicate source supplement review')
    packages = {item['id']: item for item in metadata['packages']}
    locks = tomllib.loads(lock_bytes.decode('utf-8'))['package']
    combined = copy.deepcopy(report)
    verified = []
    licenses = []
    for entry in combined['licenses']:
        if entry.get('source_path'):
            licenses.append(entry)
            continue
        if entry.get('id') != 'MIT' or not entry.get('used_by'):
            raise ValueError('unreviewed license fallback')
        for use in entry['used_by']:
            package = packages[use['crate']['id']]
            review = reviewed.get(package['name'])
            if review is None or package['version'] != review['version'] or package.get('license') != review['declared_license'] or package.get('source') != 'registry+https://github.com/rust-lang/crates.io-index':
                raise ValueError('unreviewed fallback package identity')
            locked = [item for item in locks if item['name'] == review['name'] and item['version'] == review['version']]
            if len(locked) != 1 or locked[0].get('checksum') != review['crate_checksum']:
                raise ValueError('fallback source checksum changed')
            root = Path(package['manifest_path']).parent
            vcs = json.loads((root / '.cargo_vcs_info.json').read_text(encoding='utf-8'))
            if vcs != review['vcs']:
                raise ValueError('fallback source VCS changed')
            text = review['license_text']
            if hashlib.sha256(text.encode('utf-8')).hexdigest() != review['license_sha256']:
                raise ValueError('fallback source license changed')
            if review.get('crate_license_path'):
                source = root / review['crate_license_path']
                if source.is_symlink() or not source.resolve().is_relative_to(root.resolve()) or source.read_bytes() != text.encode('utf-8'):
                    raise ValueError('published fallback license differs from review')
            licenses.append({'name': 'MIT', 'id': 'MIT', 'first_of_kind': False, 'text': text,
                             'source_path': str(catalog_path), 'used_by': [use],
                             'origin': 'reviewed-source-supplement'})
            verified.append({key: value for key, value in review.items() if key != 'license_text'})
    combined['licenses'] = licenses
    return combined, verified
