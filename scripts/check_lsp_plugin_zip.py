"""Проверить полные LSP payloads непосредственно в plugin ZIP без распаковки."""

import hashlib
import json
from pathlib import Path, PurePosixPath
import stat
import sys
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from lsp_distribution import MAX_SOURCE, TARGETS, verify_payload

MAX_ENTRY = 64 * 1024 * 1024
MAX_ARCHIVE = 256 * 1024 * 1024
PREFIXES = {'vsix': 'extension/bin/', 'intellij': 'ktav-intellij/lib/bin/'}
SOURCE = 'linux-arm64/ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz'


def inventory(archive, prefix):
    records = archive.infolist()
    if len(records) > 30000 or sum(item.file_size for item in records) > MAX_ARCHIVE:
        raise ValueError('archive inventory exceeds limits')
    files = {}
    names = set()
    for item in records:
        value = item.filename
        path = PurePosixPath(value)
        if not value or path.is_absolute() or '..' in path.parts or '\\' in value or ':' in value or path.as_posix() != value.rstrip('/'):
            raise ValueError('unsafe archive path')
        if value in names:
            raise ValueError('duplicate archive member')
        names.add(value)
        if stat.S_ISLNK(item.external_attr >> 16):
            raise ValueError('archive symlink requires review')
        limit = MAX_SOURCE if value == prefix + SOURCE else MAX_ENTRY
        if item.file_size > limit:
            raise ValueError('archive member exceeds size limit')
        if not item.is_dir():
            files[value] = item
    return files


def bounded_read(archive, item, limit):
    if item.file_size > limit:
        raise ValueError('distribution file exceeds size bound')
    with archive.open(item) as stream:
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError('distribution file exceeds size bound')
    return data


def streamed_info(archive, item, limit):
    digest = hashlib.sha256()
    size = 0
    with archive.open(item) as stream:
        while chunk := stream.read(1024 * 1024):
            size += len(chunk)
            if size > limit:
                raise ValueError('distribution file exceeds size bound')
            digest.update(chunk)
    if size != item.file_size:
        raise ValueError('archive member size mismatch')
    return {'bytes': size, 'sha256': digest.hexdigest()}


def verify(path, kind):
    prefix = PREFIXES[kind]
    if path.stat().st_size > MAX_ARCHIVE:
        raise ValueError('plugin ZIP exceeds size limit')
    with zipfile.ZipFile(path) as archive:
        files = inventory(archive, prefix)
        payloads = {platform: {} for platform, _ in TARGETS.values()}
        for filename, item in files.items():
            if not filename.startswith(prefix):
                continue
            relative = filename[len(prefix):].split('/')
            if len(relative) != 2 or relative[0] not in payloads:
                raise ValueError('unexpected plugin platform payload')
            payloads[relative[0]][relative[1]] = item
        manifests = []
        for target, (platform, _) in TARGETS.items():
            members = payloads[platform]
            manifest = verify_payload(
                list(members),
                lambda filename, limit: bounded_read(archive, members[filename], limit),
                lambda filename, limit: streamed_info(archive, members[filename], limit),
            )
            expected_binary = 'ktav-lsp.exe' if 'windows' in target else 'ktav-lsp'
            if manifest['target'] != target or manifest['binary'] != expected_binary:
                raise ValueError('plugin platform or executable lookup mismatch')
            manifests.append(manifest)
        if len({item['source_ref'] for item in manifests}) != 1:
            raise ValueError('plugin requires six matching-source platform payloads')
    return {'schema_version': 1, 'kind': kind, 'source_ref': manifests[0]['source_ref'], 'platforms': manifests, 'verified_source_entries': [prefix + SOURCE]}


if __name__ == '__main__':
    if len(sys.argv) != 3 or sys.argv[2] not in PREFIXES:
        raise SystemExit('usage: check_lsp_plugin_zip.py PLUGIN_ZIP vsix|intellij')
    print(json.dumps(verify(Path(sys.argv[1]), sys.argv[2]), sort_keys=True, indent=2))
