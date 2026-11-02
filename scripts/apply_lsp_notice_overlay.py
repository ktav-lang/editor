"""Идемпотентно добавить только существующий metadata retention hook."""

import hashlib
import json
from pathlib import Path
import sys

DECLARATION = b'include!(concat!(env!("OUT_DIR"), "/native_notices.rs"));'
USE = b'let _ = std::hint::black_box(KTAV_NATIVE_NOTICES);'
ENTRY = b'async fn main() {'


def apply(path):
    path = Path(path)
    if path.is_symlink() or not path.is_file():
        raise ValueError('LSP entry must be a regular source file')
    with path.open('rb') as source:
        original = source.read(1024 * 1024 + 1)
    if len(original) > 1024 * 1024 or original.count(ENTRY) != 1:
        raise ValueError('LSP entry requires source review')
    state = (original.count(DECLARATION), original.count(USE))
    if state == (0, 0):
        if b'native_notices.rs' in original or b'KTAV_NATIVE_NOTICES' in original:
            raise ValueError('unrecognized LSP notice hook requires review')
        packaged = DECLARATION + b'\n\n' + original.replace(ENTRY, ENTRY + b'\n    ' + USE, 1)
        path.write_bytes(packaged)
        status = 'added'
    elif state == (1, 1):
        if original.index(DECLARATION) > original.index(ENTRY) or original.index(USE) < original.index(ENTRY):
            raise ValueError('LSP notice hook location requires review')
        packaged = original
        status = 'already-present'
    else:
        raise ValueError('partial or duplicate LSP notice hook requires review')
    return {'original_main_sha256': hashlib.sha256(original).hexdigest(), 'packaged_main_sha256': hashlib.sha256(packaged).hexdigest(), 'notice_overlay': status, 'changes': 'Read-only metadata retention only; protocol and public API unchanged.'}


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: apply_lsp_notice_overlay.py LSP_MAIN_SOURCE')
    print(json.dumps(apply(Path(sys.argv[1])), sort_keys=True, indent=2))
