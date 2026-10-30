"""Проверить Mach-O notice resource и границу Cargo/system SDK inputs."""

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

from check_embedded_notices import MAX_BINARY, MAX_JSON, _extract, _read

TARGET_CPUS = {'x86_64-apple-darwin': 0x01000007, 'aarch64-apple-darwin': 0x0100000C}
SYSTEM_STUBS = {
    'libiconv.tbd', 'libSystem.tbd', 'libc.tbd', 'libm.tbd',
    'system/libcommonCrypto.tbd', 'system/libdispatch.tbd',
    'system/libdyld.tbd', 'system/libsystem_c.tbd',
    'system/libsystem_kernel.tbd', 'system/libsystem_m.tbd',
    'system/libsystem_malloc.tbd', 'system/libsystem_platform.tbd',
    'system/libsystem_pthread.tbd', 'system/libunwind.tbd',
}


def cpu(data):
    if len(data) < 32 or data[:4] != b'\xcf\xfa\xed\xfe':
        raise ValueError('expected thin little-endian Mach-O 64 executable')
    if int.from_bytes(data[12:16], 'little') != 2:
        raise ValueError('Mach-O is not an executable')
    return int.from_bytes(data[4:8], 'little')


def object_inputs(data, cargo_root, rustlib_root, sdk_root):
    if data.count(b'# Object files:') != 1:
        raise ValueError('missing or duplicate Darwin map object section')
    section = data.split(b'# Object files:', 1)[1].split(b'# Sections:', 1)
    if len(section) != 2:
        raise ValueError('missing Darwin map sections boundary')
    result = []
    ids = set()
    for line in section[0].splitlines():
        if not line.strip():
            continue
        match = re.fullmatch(rb'\[\s*(\d+)\]\s+(.+)', line)
        if not match or int(match[1]) in ids:
            raise ValueError('invalid Darwin map object row')
        ids.add(int(match[1]))
        name = match[2].decode('utf-8')
        if name == 'linker synthesized':
            result.append({'kind': 'linker-synthesized'})
            continue
        if name.endswith('.tbd'):
            path = Path(name).resolve()
            boundary = (sdk_root / 'usr/lib').resolve()
            if not path.is_relative_to(boundary) or path.relative_to(boundary).as_posix() not in SYSTEM_STUBS:
                raise ValueError('unreviewed Darwin SDK stub')
            result.append({'kind': 'system-stub', 'path': path.relative_to(boundary).as_posix()})
            continue
        archive = name.split('.rlib(', 1)[0] + '.rlib' if '.rlib(' in name else None
        if archive:
            if not name.endswith('.rcgu.o)'):
                raise ValueError('non-Rust native member in Darwin rlib input')
            path = Path(archive).resolve()
        else:
            if not name.endswith('.rcgu.o'):
                raise ValueError('external static object in Darwin LSP')
            path = Path(name).resolve()
        boundary = next((item for item in [cargo_root.resolve(), rustlib_root.resolve()] if path.is_relative_to(item)), None)
        if boundary is None:
            raise ValueError('Darwin Rust object outside reviewed Cargo/sysroot roots')
        result.append({'kind': 'rust-object', 'path': path.relative_to(boundary).as_posix(), 'origin': 'cargo' if boundary == cargo_root.resolve() else 'rust-sysroot'})
    if ids != set(range(len(result))) or sum(item['kind'] == 'linker-synthesized' for item in result) != 1 or not any(item['kind'] == 'rust-object' for item in result):
        raise ValueError('incomplete Darwin map input inventory')
    return result


def check(evidence, binary_path, target):
    if target not in TARGET_CPUS:
        raise ValueError('unreviewed Darwin LSP target')
    binary = _read(binary_path, MAX_BINARY)
    if cpu(binary) != TARGET_CPUS[target]:
        raise ValueError('Darwin LSP CPU differs from target')
    notice = _read(evidence / 'NOTICE.json', MAX_JSON)
    if _extract(binary)[0] != notice:
        raise ValueError('Darwin embedded notices differ from prepared source report')
    bundle = json.loads(notice)
    if bundle.get('schema_version') != 1 or not bundle.get('packages'):
        raise ValueError('missing Darwin Cargo/std notice coverage')
    identity = json.loads(_read(evidence / 'release-before-tests.json', MAX_JSON))
    link_map = _read(evidence / 'native-link.map', MAX_JSON)
    if identity['binary_sha256'] != hashlib.sha256(binary).hexdigest() or identity['map_sha256'] != hashlib.sha256(link_map).hexdigest():
        raise ValueError('Darwin tests changed release binary/map identity')
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    sdk = Path(subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip())
    inputs = object_inputs(link_map, binary_path.parent / 'deps', sysroot / 'lib/rustlib' / target / 'lib', sdk)
    report = {
        'target': target,
        **identity,
        'notice_sha256': hashlib.sha256(notice).hexdigest(),
        'objects': inputs,
        'scope': 'Cargo/std resource and static-input boundary only. System dynamic-library conditions and Apple SDK terms remain a separate review; no blanket distribution approval.',
    }
    (evidence / 'native-macos-verification.json').write_text(json.dumps(report, sort_keys=True, indent=2), encoding='utf-8')
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit('usage: check_lsp_macos.py EVIDENCE BINARY TARGET')
    check(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
