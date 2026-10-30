"""Проверить связь Windows binary, notices и фактически выбранного SDK runtime."""

import hashlib
import json
from pathlib import Path
import re
import sys

from check_embedded_notices import MAX_BINARY, MAX_JSON, _extract, _read
from lsp_gnullvm_notices import augment, catalogue, plain_notice


def retained_objects(data):
    result = set()
    for line in data.splitlines():
        match = re.match(rb'^[0-9a-fA-F]+\s+[0-9a-fA-F]+\s+\d+\s{2,}(.+):\([^\r\n]*\)\s*$', line)
        if match and b'.rcgu.o' not in match[1]:
            result.add(match[1].decode('utf-8').rsplit('/', 1)[-1])
    return result


def machine(data):
    if len(data) < 64 or data[:2] != b'MZ':
        raise ValueError('Windows LSP is not a PE executable')
    offset = int.from_bytes(data[60:64], 'little')
    if offset > len(data) - 6 or data[offset:offset + 4] != b'PE\0\0':
        raise ValueError('invalid Windows LSP PE header')
    return int.from_bytes(data[offset + 4:offset + 6], 'little')


def sdk_inputs(items):
    result = []
    seen = set()
    for item in items:
        name = item['path']
        if not name.endswith(('.a', '/crt2.o', '/crtbegin.o', '/crtend.o')):
            continue
        marker = 'llvm-mingw-20261020-ucrt-x86_64/'
        if name.count(marker) != 1:
            raise ValueError('native input outside selected GNU LLVM SDK')
        relative = name.split(marker, 1)[1]
        if relative in seen:
            raise ValueError('duplicate native SDK input')
        seen.add(relative)
        result.append({**item, 'path': relative})
    return sorted(result, key=lambda item: item['path'])


def check(evidence, binary_path, target):
    root = Path(__file__).resolve().parents[1]
    catalog, digest = catalogue(root)
    if target not in catalog['targets']:
        raise ValueError('unreviewed Windows LSP target')
    binary = _read(binary_path, MAX_BINARY)
    expected_machine = 0x8664 if target == 'x86_64-pc-windows-gnullvm' else 0xAA64
    if machine(binary) != expected_machine:
        raise ValueError('Windows LSP CPU differs from its target')
    notice = _read(evidence / 'NOTICE.json', MAX_JSON)
    original = json.loads(_read(evidence / 'NOTICE.cargo-std.json', MAX_JSON))
    expected = augment(original, catalog, target, digest)
    if json.loads(notice) != expected or _extract(binary)[0] != notice:
        raise ValueError('Windows native notices differ from reviewed sources')
    plain = _read(evidence / 'NOTICE.windows-runtime.txt', MAX_JSON)
    if plain != plain_notice(catalog):
        raise ValueError('Windows plain notices differ from original legal documents')
    if _read(evidence / 'lsp-gnullvm-runtime-catalog.json', MAX_JSON) != (root / 'ci/lsp-gnullvm-runtime-catalog.json').read_bytes():
        raise ValueError('staged Windows runtime catalogue changed')
    identity = json.loads(_read(evidence / 'release-before-tests.json', MAX_JSON))
    link_map = _read(evidence / 'native-link.map', MAX_JSON)
    if identity['binary_sha256'] != hashlib.sha256(binary).hexdigest() or identity['map_sha256'] != hashlib.sha256(link_map).hexdigest():
        raise ValueError('Windows release identity changed after notice preparation')
    linked = json.loads(_read(evidence / 'link-input-fingerprints.json', MAX_JSON))
    if linked['release_identity'] != identity:
        raise ValueError('Windows linker inputs are not bound to release binary')
    profile = catalog['targets'][target]
    if sdk_inputs(linked['inputs']) != sorted(profile['sdk_inputs'], key=lambda item: item['path']):
        raise ValueError('Windows SDK archive/direct-input bytes changed')
    if retained_objects(link_map) != set(profile['retained_objects']):
        raise ValueError('Windows retained runtime objects differ from source review')
    report = {
        'target': target,
        'binary_sha256': identity['binary_sha256'],
        'map_sha256': identity['map_sha256'],
        'notice_sha256': hashlib.sha256(notice).hexdigest(),
        'plain_notice_sha256': hashlib.sha256(plain).hexdigest(),
        'catalog_sha256': digest,
        'retained_runtime_objects': len(profile['retained_objects']),
        'source_records': len(catalog['source_files']),
        'scope': 'Exact experimental Windows LSP resource and runtime-input binding; no historical artifact or other binding compatibility claim.',
    }
    (evidence / 'native-windows-verification.json').write_text(json.dumps(report, sort_keys=True, indent=2), encoding='utf-8')
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit('usage: check_lsp_gnullvm.py EVIDENCE BINARY TARGET')
    check(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
