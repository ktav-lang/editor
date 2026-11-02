"""Доказать relink с изменённым ARM64 startup, не исполняя executable."""

import hashlib
import json
from pathlib import Path
import struct
import sys

from check_embedded_notices import _extract

NOP = b'\x1f\x20\x03\xd5'


def entry_instruction(data):
    if len(data) < 64 or data[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', data, 18)[0] != 183:
        raise ValueError('relink proof requires little-endian ELF64 ARM64')
    entry, phoff = struct.unpack_from('<QQ', data, 24)
    size, count = struct.unpack_from('<HH', data, 54)
    if size != 56 or not count or count > 256 or phoff + size * count > len(data):
        raise ValueError('invalid relink ELF program headers')
    for index in range(count):
        offset = phoff + size * index
        kind, flags, file_offset, address, _, file_size, _, _ = struct.unpack_from('<IIQQQQQQ', data, offset)
        if kind == 1 and flags & 1 and address <= entry and entry + 4 <= address + file_size:
            position = file_offset + entry - address
            if position + 4 > len(data):
                raise ValueError('truncated relink entry code')
            return data[position:position + 4]
    raise ValueError('relink ELF entry is not executable file-backed code')


def verify(original, modified, notice):
    if entry_instruction(original) == NOP or entry_instruction(modified) != NOP:
        raise ValueError('modified startup NOP was not retained at actual ELF entry')
    if original == modified or _extract(original)[0] != notice or _extract(modified)[0] != notice:
        raise ValueError('relink executable or notice identity differs from proof')
    return {'original_sha256': hashlib.sha256(original).hexdigest(), 'modified_sha256': hashlib.sha256(modified).hexdigest(), 'modified_entry_instruction_hex': NOP.hex(), 'same_notice_sha256': hashlib.sha256(notice).hexdigest(), 'modified_startup_retained': True, 'scope': 'Source/relink proof; ARM64 executables were inspected, not executed.'}


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit('usage: check_lsp_relink.py ORIGINAL MODIFIED NOTICE_JSON')
    paths = list(map(Path, sys.argv[1:]))
    for path in paths:
        if not path.is_file() or path.stat().st_size > 128 * 1024 * 1024:
            raise SystemExit('relink proof input exceeds bound')
    print(json.dumps(verify(*(path.read_bytes() for path in paths)), sort_keys=True, indent=2))
