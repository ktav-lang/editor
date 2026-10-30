"""Проверить actual source archive до распаковки и relink."""

import hashlib
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile


def inspect(archive_path):
    with tarfile.open(archive_path, 'r:gz') as archive:
        members = archive.getmembers()
        if len(members) > 100000:
            raise ValueError('source archive member limit exceeded')
        names = set()
        total = 0
        for member in members:
            path = PurePosixPath(member.name)
            total += member.size
            if not member.isfile() or path.is_absolute() or '..' in path.parts or '\\' in member.name or not path.parts or path.parts[0] != 'lsp-source' or path.as_posix() != member.name or member.name in names or total > 2 * 1024 * 1024 * 1024:
                raise ValueError('unsafe source archive member')
            names.add(member.name)
        manifest_member = archive.getmember('lsp-source/MANIFEST.json')
        if manifest_member.size > 32 * 1024 * 1024:
            raise ValueError('source manifest size limit exceeded')
        manifest = json.load(archive.extractfile(manifest_member))
        expected = {item['path']: item for item in manifest}
        if len(expected) != len(manifest) or names != {'lsp-source/' + name for name in expected} | {'lsp-source/MANIFEST.json'}:
            raise ValueError('source archive inventory differs from manifest')
        for name, entry in expected.items():
            member = archive.getmember('lsp-source/' + name)
            if member.size != entry['bytes'] or member.mode != entry['mode']:
                raise ValueError('source file size or mode differs from manifest')
            checksum = hashlib.sha256()
            with archive.extractfile(member) as source:
                for chunk in iter(lambda: source.read(1024 * 1024), b''):
                    checksum.update(chunk)
            if checksum.hexdigest() != entry['sha256']:
                raise ValueError('source file checksum differs from manifest')
        required = {'project/Cargo.toml', 'project/Cargo.lock', 'project/build.rs', 'project/src/main.rs', 'project/.cargo/config.toml', 'project/LICENSE-MIT', 'project/LICENSE-APACHE', 'legal/LGPL-2.1.txt', 'legal/NOTICE.json', 'legal/NOTICE.json.gz', 'legal/NOTICE.gnu-startup.txt', 'legal/lsp-zig-runtime-catalog.json', 'upstream/zig-source.tar.gz', 'rebuild.sh', 'provenance.json', 'README.txt'}
        if not required <= set(expected) or not any(name.startswith('project/vendor/') for name in expected) or not any(name.startswith('toolchain/lib/rustlib/') and name.endswith('.rlib') for name in expected):
            raise ValueError('incomplete corresponding source materials')
        return {'files': len(expected), 'verified_manifest': True}


def verify_and_extract(archive_path, destination):
    result = inspect(archive_path)
    if destination.exists():
        raise ValueError('source verification destination must be new')
    destination.mkdir(parents=True)
    with tarfile.open(archive_path, 'r:gz') as archive:
        archive.extractall(destination, filter='data')
    return result


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: verify_lsp_source_bundle.py ARCHIVE NEW_DESTINATION')
    print(json.dumps(verify_and_extract(*map(Path, sys.argv[1:])), sort_keys=True))
