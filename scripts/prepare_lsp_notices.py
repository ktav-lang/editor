"""Подготовить source-backed Cargo/std notices для существующего locked LSP source."""

import gzip
import json
from pathlib import Path
import shutil
import subprocess
import sys

from cargo_notice_bundle import _build


def prepare(evidence):
    root = Path.cwd().resolve()
    evidence = evidence.resolve()
    lock = root / 'Cargo.lock'
    if evidence.is_relative_to(root) or (evidence.exists() and (not evidence.is_dir() or any(evidence.iterdir()))):
        raise ValueError('LSP evidence must be empty and outside source')
    if not lock.is_file() or lock.is_symlink():
        raise ValueError('LSP requires the existing regular Cargo.lock')
    original = lock.read_bytes()
    raw = subprocess.check_output(['cargo', 'metadata', '--locked', '--all-features', '--format-version', '1'])
    metadata = json.loads(raw)
    own = [item for item in metadata['packages'] if Path(item['manifest_path']).resolve().parent == root]
    if len(own) != 1 or own[0].get('name') != 'ktav-lsp' or own[0].get('license') != 'MIT OR Apache-2.0':
        raise ValueError('unexpected first-party LSP license')
    for name in ('LICENSE-MIT', 'LICENSE-APACHE'):
        source = root.parent / name
        target = root / name
        if target.is_symlink():
            raise ValueError('LSP license copy must not be a symlink')
        if target.exists():
            if target.read_bytes() != source.read_bytes():
                raise ValueError('LSP own license differs from repository license')
        else:
            shutil.copyfile(source, target)
    for item in metadata['packages']:
        if evidence.is_relative_to(Path(item['manifest_path']).resolve().parent):
            raise ValueError('LSP evidence overlaps a package source')
    evidence.mkdir(parents=True, exist_ok=True)
    (evidence / 'metadata.json').write_bytes(raw)
    (evidence / 'Cargo.lock').write_bytes(original)
    (evidence / 'rustc.txt').write_bytes(subprocess.check_output(['rustc', '--version', '--verbose']))
    sysroot = subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip()
    shutil.copyfile(Path(sysroot) / 'share/doc/rust/COPYRIGHT-library.html', evidence / 'COPYRIGHT-library.html')
    report = subprocess.check_output(['cargo', 'about', 'generate', '--locked', '--all-features', '--fail', '--config', '../ci/about.toml', '--format', 'json'])
    (evidence / 'licenses.json').write_bytes(report)
    data = _build(evidence)
    compressed = gzip.compress(data, mtime=0)
    if len(compressed) > 4 * 1024 * 1024 or lock.read_bytes() != original:
        raise ValueError('LSP notice resource exceeds limit or Cargo.lock changed')
    (evidence / 'NOTICE.json').write_bytes(data)
    (evidence / 'NOTICE.json.gz').write_bytes(compressed)
    print(f'Prepared locked LSP notices: {len(data)} JSON bytes, {len(compressed)} gzip bytes')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: prepare_lsp_notices.py EMPTY_EVIDENCE_DIRECTORY')
    try:
        prepare(Path(sys.argv[1]))
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
