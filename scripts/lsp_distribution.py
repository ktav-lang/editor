"""Сохранить binary, notices и corresponding source в одном проверяемом payload."""

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import sys
import tomllib

from check_embedded_notices import MARKER, MAX_BINARY, MAX_JSON, _extract, _read

TARGETS = {
    'x86_64-unknown-linux-gnu': ('linux-x64', 'ktav-lsp-linux-amd64'),
    'aarch64-unknown-linux-gnu': ('linux-arm64', 'ktav-lsp-linux-arm64'),
    'x86_64-apple-darwin': ('darwin-x64', 'ktav-lsp-darwin-amd64'),
    'aarch64-apple-darwin': ('darwin-arm64', 'ktav-lsp-darwin-arm64'),
    'x86_64-pc-windows-gnullvm': ('win32-x64', 'ktav-lsp-windows-amd64.exe'),
    'aarch64-pc-windows-gnullvm': ('win32-arm64', 'ktav-lsp-windows-arm64.exe'),
}
MAX_SOURCE = 128 * 1024 * 1024
MAX_PAYLOAD = 256 * 1024 * 1024
CORE_CHECKSUM = '7140dfe2fa6c320e74768cd6a637828c2d26f81746254f4fcd805f3993bd3fd5'


def name(value):
    path = PurePosixPath(value)
    if not value or len(path.parts) != 1 or path.name != value or value.startswith('.') or '\\' in value or ':' in value or not re.fullmatch(r'[A-Za-z0-9_.-]+', value):
        raise ValueError('unsafe distribution filename')
    return value


def file_info(path, limit):
    if path.is_symlink() or not path.is_file():
        raise ValueError('distribution input must be a regular file')
    digest = hashlib.sha256()
    size = 0
    with path.open('rb') as source:
        while chunk := source.read(1024 * 1024):
            size += len(chunk)
            if size > limit:
                raise ValueError('distribution file exceeds size bound')
            digest.update(chunk)
    return {'bytes': size, 'sha256': digest.hexdigest()}


def text(item):
    data = item['text'].encode('utf-8')
    if hashlib.sha256(data).hexdigest() != item['sha256']:
        raise ValueError('original licence text digest mismatch')
    return data


def cpu(binary, target):
    arm = target.startswith('aarch64-')
    if 'linux' in target:
        if len(binary) < 64 or binary[:6] != b'\x7fELF\x02\x01' or int.from_bytes(binary[16:18], 'little') not in {2, 3} or int.from_bytes(binary[18:20], 'little') != (183 if arm else 62) or int.from_bytes(binary[24:32], 'little') == 0:
            raise ValueError('distribution ELF CPU/format mismatch')
    elif 'apple' in target:
        if len(binary) < 32 or binary[:4] != b'\xcf\xfa\xed\xfe' or int.from_bytes(binary[4:8], 'little') != (0x0100000C if arm else 0x01000007) or int.from_bytes(binary[12:16], 'little') != 2:
            raise ValueError('distribution Mach-O CPU/format mismatch')
    else:
        offset = int.from_bytes(binary[60:64], 'little') if len(binary) >= 64 else len(binary)
        characteristics = int.from_bytes(binary[offset + 22:offset + 24], 'little')
        if binary[:2] != b'MZ' or offset > len(binary) - 24 or binary[offset:offset + 4] != b'PE\0\0' or int.from_bytes(binary[offset + 4:offset + 6], 'little') != (0xAA64 if arm else 0x8664) or not characteristics & 2 or characteristics & 0x2000:
            raise ValueError('distribution PE CPU/format mismatch')


def resource(binary, notice, compressed):
    expected = MARKER + len(compressed).to_bytes(8, 'little') + compressed
    if _extract(binary)[0] != notice or binary.count(expected) != 1:
        raise ValueError('distribution resource differs from notice sidecars')


def cargo_plain(bundle):
    sections = [b'Ktav LSP third-party notices.\nConservative Cargo source inventory; not an exact linked-code count.\n']
    for item in [*bundle['licenses'], *bundle['notices']]:
        sections.extend([b'\n========================================\n', text(item)])
    return b'\n'.join(sections)


def core_identity(bundle, lock):
    core = [item for item in tomllib.loads(lock.decode('utf-8'))['package'] if item['name'] == 'ktav']
    own = [item for item in bundle['packages'] if item['name'] == 'ktav-lsp']
    if len(core) != 1 or core[0].get('version') != '0.8.1' or core[0].get('checksum') != CORE_CHECKSUM or len(own) != 1 or own[0].get('version') != '0.8.1' or hashlib.sha256(lock).hexdigest() != bundle['cargo_lock_sha256']:
        raise ValueError('distribution source/core/lock identity changed')


def source_reference(bundle, target, source_ref):
    runtimes = bundle.get('non_cargo_runtimes', [])
    if 'linux' not in target:
        return None
    if len(runtimes) != 1:
        raise ValueError('missing reviewed GNU startup runtime')
    runtime = runtimes[0]
    terms = runtime['startup_terms']
    arm = target == 'aarch64-unknown-linux-gnu'
    if terms.get('corresponding_source_required') is not arm or terms.get('compiled_linking_exception') is not (not arm):
        raise ValueError('GNU startup source obligations differ from target')
    if not arm:
        return None
    expected = 'ktav-lsp-source-' + target + '.tar.gz'
    reference = runtime['corresponding_source']
    if reference.get('archive') != expected or reference.get('source_ref') != source_ref:
        raise ValueError('GNU corresponding-source identity mismatch')
    return expected


def report_identity(report, binary_sha, notice_sha, target):
    if report.get('binary_sha256') != binary_sha or report.get('notice_sha256') != notice_sha or ('target' in report and report['target'] != target):
        raise ValueError('target verification is not bound to distribution bytes')
    if 'apple' in target and (report.get('dynamic_libraries') != ['/usr/lib/libSystem.B.dylib'] or report.get('iconv_symbol_imports') is not False):
        raise ValueError('unreviewed macOS dynamic runtime')
    return report


def provider_report(evidence, binary_sha, notice_sha, target):
    filename = 'native-windows-verification.json' if 'windows' in target else 'native-macos-verification.json' if 'apple' in target else 'native-resource-verification.json'
    return report_identity(json.loads(_read(evidence / filename, MAX_JSON)), binary_sha, notice_sha, target)


def stage(evidence, binary_path, target, source_directory, output):
    if target not in TARGETS:
        raise ValueError('unreviewed LSP distribution target')
    if output.exists() and (output.is_symlink() or not output.is_dir() or any(output.iterdir())):
        raise ValueError('distribution output must be a new empty directory')
    platform, asset = TARGETS[target]
    binary = _read(binary_path, MAX_BINARY)
    cpu(binary, target)
    notice = _read(evidence / 'NOTICE.json', MAX_JSON)
    compressed = _read(evidence / 'NOTICE.json.gz', 4 * 1024 * 1024)
    resource(binary, notice, compressed)
    bundle = json.loads(notice)
    if bundle.get('schema_version') != 1 or not bundle.get('packages'):
        raise ValueError('invalid distribution notice schema')
    lock = _read(evidence / 'Cargo.lock', MAX_JSON)
    core_identity(bundle, lock)
    provenance = json.loads(_read(evidence / 'source-provenance.json', MAX_JSON))
    source_ref = provenance['source_ref']
    if not re.fullmatch(r'[0-9a-f]{40}', source_ref):
        raise ValueError('missing immutable distribution source reference')
    binary_sha = hashlib.sha256(binary).hexdigest()
    notice_sha = hashlib.sha256(notice).hexdigest()
    report = provider_report(evidence, binary_sha, notice_sha, target)
    stdlib = _read(evidence / 'COPYRIGHT-library.html', MAX_JSON)
    if text(bundle['rust_standard_library']) != stdlib:
        raise ValueError('original Rust standard-library report changed')
    data = {
        asset: binary,
        asset + '.NOTICE.json': notice,
        asset + '.NOTICE.json.gz': compressed,
        asset + '.THIRD-PARTY-NOTICES.txt': cargo_plain(bundle),
        asset + '.COPYRIGHT-library.html': stdlib,
        asset + '.Cargo.lock': lock,
    }
    root = Path(__file__).resolve().parents[1]
    for filename in ['LICENSE-MIT', 'LICENSE-APACHE']:
        data[asset + '.' + filename] = _read(root / filename, MAX_JSON)
    runtime_file = 'NOTICE.gnu-startup.txt' if 'linux' in target else 'NOTICE.windows-runtime.txt' if 'windows' in target else None
    if runtime_file:
        runtime_notice = _read(evidence / runtime_file, MAX_JSON)
        if not runtime_notice.strip() or hashlib.sha256(runtime_notice).hexdigest() != report.get('plain_notice_sha256'):
            raise ValueError('plain runtime notice is not bound to target verification')
        data[asset + '.' + runtime_file] = runtime_notice
    archive = source_reference(bundle, target, source_ref)
    source_path = None
    binding_file = None
    if archive:
        source_path = source_directory / name(archive)
        source_info = file_info(source_path, MAX_SOURCE)
        binding = json.loads(_read(evidence / 'native-source-binding.json', MAX_JSON))
        if binding.get('target') != target or binding.get('source_ref') != source_ref or binding['binary']['sha256'] != binary_sha or binding['notice']['sha256'] != notice_sha or binding['corresponding_source'] != {'file': archive, 'sha256': source_info['sha256']}:
            raise ValueError('corresponding source is not bound to distribution binary')
        binding_file = asset + '.source-binding.json'
        rebound = {**binding, 'binary': {'file': asset, 'sha256': binary_sha}, 'notice': {'file': asset + '.NOTICE.json', 'sha256': notice_sha}, 'coverage': 'Matching binary, readable notices and complete source/relink materials are delivered in this payload.'}
        data[binding_file] = (json.dumps(rebound, sort_keys=True, indent=2) + '\n').encode('utf-8')
    files = {name(filename): {'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()} for filename, content in data.items()}
    if archive:
        files[archive] = source_info
    if sum(item['bytes'] for item in files.values()) > MAX_PAYLOAD:
        raise ValueError('LSP distribution payload exceeds bound')
    manifest = {
        'schema_version': 1, 'target': target, 'platform': platform, 'asset': asset,
        'source_ref': source_ref, 'binary': asset, 'notice': asset + '.NOTICE.json',
        'notice_gzip': asset + '.NOTICE.json.gz', 'runtime_notice': asset + '.' + runtime_file if runtime_file else None,
        'stdlib_report': asset + '.COPYRIGHT-library.html', 'cargo_plain_notice': asset + '.THIRD-PARTY-NOTICES.txt',
        'cargo_lock': asset + '.Cargo.lock',
        'first_party_licenses': [asset + '.LICENSE-MIT', asset + '.LICENSE-APACHE'],
        'corresponding_source': archive, 'source_binding': binding_file,
        'files': files, 'verification': report,
    }
    output.mkdir(parents=True, exist_ok=True)
    for filename, content in data.items():
        with (output / filename).open('xb') as destination:
            destination.write(content)
    if source_path:
        shutil.copyfile(source_path, output / archive)
    if 'windows' not in target:
        (output / asset).chmod(0o755)
    with (output / (asset + '.distribution.json')).open('xb') as destination:
        destination.write((json.dumps(manifest, sort_keys=True, indent=2) + '\n').encode('utf-8'))
    verify(output)
    return manifest


def required_files(manifest):
    target, asset = manifest['target'], manifest['asset']
    binary_names = {asset, 'ktav-lsp.exe' if 'windows' in target else 'ktav-lsp'}
    if manifest['binary'] not in binary_names or not re.fullmatch(r'[0-9a-f]{40}', manifest['source_ref']):
        raise ValueError('invalid distribution binary/source identity')
    runtime = asset + '.NOTICE.gnu-startup.txt' if 'linux' in target else asset + '.NOTICE.windows-runtime.txt' if 'windows' in target else None
    source = 'ktav-lsp-source-' + target + '.tar.gz' if target == 'aarch64-unknown-linux-gnu' else None
    fields = {
        'notice': asset + '.NOTICE.json', 'notice_gzip': asset + '.NOTICE.json.gz',
        'stdlib_report': asset + '.COPYRIGHT-library.html',
        'cargo_plain_notice': asset + '.THIRD-PARTY-NOTICES.txt',
        'cargo_lock': asset + '.Cargo.lock', 'runtime_notice': runtime,
        'corresponding_source': source,
        'source_binding': asset + '.source-binding.json' if source else None,
    }
    if any(manifest.get(key) != value for key, value in fields.items()) or manifest.get('first_party_licenses') != [asset + '.LICENSE-MIT', asset + '.LICENSE-APACHE']:
        raise ValueError('required distribution legal/source role was dropped')
    return {manifest['binary'], *manifest['first_party_licenses'], *(value for value in fields.values() if value)}


def verify(directory):
    manifests = list(directory.glob('*.distribution.json'))
    if len(manifests) != 1 or manifests[0].is_symlink():
        raise ValueError('missing or duplicate distribution manifest')
    manifest = json.loads(_read(manifests[0], MAX_JSON))
    target = manifest['target']
    if manifest.get('schema_version') != 1 or target not in TARGETS or (manifest['platform'], manifest['asset']) != TARGETS[target]:
        raise ValueError('invalid LSP distribution identity')
    asset = manifest['asset']
    if set(manifest['files']) != required_files(manifest):
        raise ValueError('distribution file inventory omits required materials')
    expected = {name(item) for item in manifest['files']} | {manifests[0].name}
    if manifests[0].name != asset + '.distribution.json' or {item.name for item in directory.iterdir()} != expected:
        raise ValueError('missing or unexpected distribution file')
    total = 0
    for filename, identity in manifest['files'].items():
        limit = MAX_SOURCE if filename == manifest['corresponding_source'] else MAX_BINARY if filename == manifest['binary'] else MAX_JSON
        actual = file_info(directory / filename, limit)
        if actual != identity:
            raise ValueError('distribution file identity mismatch')
        total += actual['bytes']
    if total > MAX_PAYLOAD:
        raise ValueError('LSP distribution payload exceeds bound')
    binary = _read(directory / name(manifest['binary']), MAX_BINARY)
    cpu(binary, target)
    notice = _read(directory / name(manifest['notice']), MAX_JSON)
    bundle = json.loads(notice)
    core_identity(bundle, _read(directory / name(manifest['cargo_lock']), MAX_JSON))
    report = report_identity(manifest['verification'], hashlib.sha256(binary).hexdigest(), hashlib.sha256(notice).hexdigest(), target)
    if manifest['runtime_notice'] and (manifest['files'][manifest['runtime_notice']]['bytes'] == 0 or manifest['files'][manifest['runtime_notice']]['sha256'] != report.get('plain_notice_sha256')):
        raise ValueError('staged plain runtime notice lost its verification binding')
    if any(manifest['files'][item]['bytes'] == 0 for item in manifest['first_party_licenses']):
        raise ValueError('empty first-party distribution licence')
    resource(binary, notice, _read(directory / name(manifest['notice_gzip']), 4 * 1024 * 1024))
    if _read(directory / name(manifest['stdlib_report']), MAX_JSON) != text(bundle['rust_standard_library']) or _read(directory / name(manifest['cargo_plain_notice']), MAX_JSON) != cargo_plain(bundle):
        raise ValueError('readable original source notices changed')
    source = source_reference(bundle, target, manifest['source_ref'])
    if source != manifest['corresponding_source']:
        raise ValueError('required corresponding source was dropped')
    if source:
        binding = json.loads(_read(directory / name(manifest['source_binding']), MAX_JSON))
        if binding['target'] != target or binding['source_ref'] != manifest['source_ref'] or binding['binary'] != {'file': manifest['binary'], 'sha256': manifest['files'][manifest['binary']]['sha256']} or binding['notice'] != {'file': manifest['notice'], 'sha256': manifest['files'][manifest['notice']]['sha256']} or binding['corresponding_source'] != {'file': source, 'sha256': manifest['files'][source]['sha256']}:
            raise ValueError('staged corresponding-source binding mismatch')
    return manifest


if __name__ == '__main__':
    if len(sys.argv) == 3 and sys.argv[1] == 'verify':
        result = verify(Path(sys.argv[2]))
    elif len(sys.argv) == 7 and sys.argv[1] == 'stage':
        result = stage(Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4], Path(sys.argv[5]), Path(sys.argv[6]))
    else:
        raise SystemExit('usage: lsp_distribution.py stage EVIDENCE BINARY TARGET SOURCE_DIRECTORY OUTPUT | verify PAYLOAD')
    print(json.dumps({'target': result['target'], 'asset': result['asset'], 'files': len(result['files']), 'corresponding_source': result['corresponding_source']}, sort_keys=True, indent=2))
