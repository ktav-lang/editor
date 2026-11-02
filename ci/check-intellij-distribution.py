"""Проверить Ktav licenses и отсутствие IDE SDK code в actual plugin ZIP."""

import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import stat
import sys
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from check_lsp_plugin_zip import MAX_SOURCE, verify as verify_lsp_zip

MAX_ENTRY = 64 * 1024 * 1024
MAX_ARCHIVE = 256 * 1024 * 1024
NATIVE_PREFIXES = (b'\x7fELF', b'MZ', b'\x00asm', b'!<arch>\n', b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca')
PLATFORMS = {'linux-x64', 'linux-arm64', 'darwin-x64', 'darwin-arm64', 'win32-x64', 'win32-arm64'}


def entries(archive, verified_source_entries=()):
    records = archive.infolist()
    if len(records) > 30000 or sum(item.file_size for item in records) > MAX_ARCHIVE:
        raise ValueError('archive inventory exceeds limits')
    names = set()
    for item in records:
        name = item.filename
        path = PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or '\\' in name or ':' in name:
            raise ValueError('unsafe archive path')
        if name in names:
            raise ValueError('duplicate archive member')
        names.add(name)
        if stat.S_ISLNK(item.external_attr >> 16):
            raise ValueError('archive symlink requires review')
        limit = MAX_SOURCE if name in verified_source_entries else MAX_ENTRY
        if item.file_size > limit:
            raise ValueError('archive member exceeds size limit')
        if not item.is_dir():
            yield item


def read(archive, item):
    with archive.open(item) as stream:
        data = stream.read(MAX_ENTRY + 1)
    if len(data) > MAX_ENTRY:
        raise ValueError('archive member exceeds read limit')
    return data


def verify(path, project):
    if path.stat().st_size > MAX_ARCHIVE:
        raise ValueError('plugin ZIP exceeds size limit')
    licenses = {name: (project / name).read_bytes() for name in ('LICENSE-MIT', 'LICENSE-APACHE')}
    jars = []
    native = []
    plugin_descriptors = 0
    with zipfile.ZipFile(path) as outer:
        has_manifests = any(item.filename.endswith('.distribution.json') for item in outer.infolist())
        lsp = verify_lsp_zip(path, 'intellij') if has_manifests else None
        verified_sources = lsp['verified_source_entries'] if lsp else ()
        for item in entries(outer, verified_sources):
            name = item.filename
            if not name.startswith('ktav-intellij/'):
                raise ValueError('unexpected plugin root')
            if name in verified_sources:
                continue
            data = read(outer, item)
            if name.endswith('.jar'):
                with zipfile.ZipFile(io.BytesIO(data)) as jar:
                    members = list(entries(jar))
                    own_classes = 0
                    for member in members:
                        value = read(jar, member)
                        if member.filename.endswith('.class'):
                            if not member.filename.startswith('lang/ktav/') or not value.startswith(b'\xca\xfe\xba\xbe'):
                                raise ValueError(f'non-Ktav class bundled: {member.filename}')
                            own_classes += 1
                        elif value.startswith(NATIVE_PREFIXES) or value.startswith(b'PK\x03\x04') or member.filename.endswith(('.so', '.dll', '.dylib', '.jar', '.a')):
                            raise ValueError('hidden executable or archive in plugin JAR')
                        if member.filename == 'META-INF/plugin.xml':
                            plugin_descriptors += 1
                    member_names = {member.filename for member in members}
                    for license_name, expected in licenses.items():
                        if 'META-INF/' + license_name not in member_names:
                            raise ValueError('Ktav license missing from plugin JAR')
                        if jar.read('META-INF/' + license_name) != expected:
                            raise ValueError('packaged Ktav license differs from source')
                    jars.append({'path': name, 'sha256': hashlib.sha256(data).hexdigest(), 'classes': own_classes})
            elif data.startswith(NATIVE_PREFIXES) or name.endswith(('.so', '.dll', '.dylib', '.exe', '.class', '.a')):
                parts = PurePosixPath(name).parts
                if len(parts) != 5 or parts[:3] != ('ktav-intellij', 'lib', 'bin') or parts[3] not in PLATFORMS:
                    raise ValueError('unexpected native or SDK payload')
                expected = 'ktav-lsp.exe' if parts[3].startswith('win32-') else 'ktav-lsp'
                if parts[4] != expected:
                    raise ValueError('unexpected native payload name')
                signatures = (b'\x7fELF',) if parts[3].startswith('linux-') else ((b'MZ',) if parts[3].startswith('win32-') else (b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf'))
                if not data.startswith(signatures):
                    raise ValueError('unexpected native LSP format')
                native.append({'path': name, 'sha256': hashlib.sha256(data).hexdigest()})
            elif data.startswith(b'PK\x03\x04'):
                raise ValueError('hidden archive in plugin ZIP')
    if not jars or plugin_descriptors != 1 or not any(item['classes'] for item in jars):
        raise ValueError('missing or duplicate Ktav plugin descriptor/classes')
    return {
        'schema_version': 1,
        'archive': path.name,
        'archive_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'coverage': 'Ktav JAR license bytes and exclusion of IDE SDK/tooling JVM classes. Native LSP code and license obligations require separate evidence.',
        'jars': jars,
        'native_lsp_payloads': native,
        'lsp_distribution': lsp,
        'licenses': {name: hashlib.sha256(value).hexdigest() for name, value in licenses.items()},
    }


def main():
    if len(sys.argv) != 4:
        raise SystemExit('usage: check-intellij-distribution.py DISTRIBUTION_DIRECTORY PROJECT_ROOT REPORT')
    try:
        archives = list(Path(sys.argv[1]).glob('*.zip'))
        if len(archives) != 1:
            raise ValueError('expected exactly one IntelliJ distribution ZIP')
        result = verify(archives[0], Path(sys.argv[2]))
        output = Path(sys.argv[3])
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, sort_keys=True, indent=2) + '\n', encoding='utf-8')
    except (OSError, ValueError, KeyError, zipfile.BadZipFile) as error:
        raise SystemExit(str(error)) from error
    print('Verified actual IntelliJ ZIP: Ktav licenses retained; only Ktav JAR classes and declared native LSP paths')


if __name__ == '__main__':
    main()
