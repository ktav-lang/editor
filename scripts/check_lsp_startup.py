"""Проверить actual retained startup sections, не исполняя ELF."""

import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

EXPECTED = {
    ('libunwind.a', 'UnwindLevel1.o'), ('libunwind.a', 'UnwindLevel1-gcc-ext.o'),
    ('libunwind.a', 'UnwindRegistersSave.o'), ('libunwind.a', 'UnwindRegistersRestore.o'),
    ('libunwind.a', 'libunwind.o'), ('libc_nonshared.a', 'stat64-2.32.o'),
    ('libc_nonshared.a', 'fstat64-2.32.o'), ('libc_nonshared.a', 'elf-init-2.33.o'),
}


def analyze_map(text):
    members = {(PurePosixPath(a).name, PurePosixPath(m).name) for a, m in re.findall(r'([^\s()]+[.]a)\(([^()\n]+)\)', text)}
    if members != EXPECTED:
        raise ValueError('unreviewed LSP static archive inputs')
    objects = set(re.findall(r'[^\s()]+[.]o(?=:\()', text))
    startup = {path for path in objects if not path.endswith('.rcgu.o')}
    if not objects - startup or len(startup) != 1 or PurePosixPath(next(iter(startup))).name != 'Scrt1.o':
        raise ValueError('unreviewed LSP direct startup input')
    path = next(iter(startup))
    sections = {name.split('+', 1)[0] for name in re.findall(re.escape(path) + r':\(([^)]+)\)', text)}
    if not {'.text', '.note.ABI-tag'} <= sections or not sections <= {'.text', '.note.ABI-tag', '.eh_frame'}:
        raise ValueError('unreviewed retained startup section')
    if re.search(r'\b_IO_stdin_used\b', text):
        raise ValueError('non-exempt startup data retained in linker map')
    return path, sorted(sections), sorted(members)


def verify_binary_symbols(text):
    if re.search(r'\b_IO_stdin_used\b', text):
        raise ValueError('non-exempt startup symbol retained in final ELF')


def verify(binary, evidence):
    root = Path.cwd().resolve()
    text = (evidence / 'native-link.map').read_text(encoding='utf-8')
    startup, sections, members = analyze_map(text)
    startup = Path(startup)
    if not startup.is_file() or not startup.resolve().is_relative_to(root / '.zig-cache'):
        raise ValueError('startup object must belong to this build cache')
    object_sections = subprocess.check_output(['readelf', '-SW', str(startup)])
    object_symbols = subprocess.check_output(['readelf', '-sW', str(startup)])
    binary_symbols = subprocess.check_output(['readelf', '-sW', str(binary)])
    verify_binary_symbols(binary_symbols.decode('utf-8'))
    for name, data in [('startup-object-sections.txt', object_sections), ('startup-object-symbols.txt', object_symbols), ('final-elf-symbols.txt', binary_symbols)]:
        (evidence / name).write_bytes(data)
    record = {'archive_members': members, 'retained_startup_sections': sections, 'final_io_stdin_used_absent': True, 'coverage': 'Retained-input proof only; source/license and SDK header notices require separate review.'}
    (evidence / 'startup-retention.json').write_text(json.dumps(record, sort_keys=True, indent=2), encoding='utf-8')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: check_lsp_startup.py BINARY EVIDENCE_DIRECTORY')
    verify(*map(Path, sys.argv[1:]))
