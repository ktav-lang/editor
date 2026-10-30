"""Связать LSP runtime notices с exact SDK source и source/relink artifact."""

import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

REVISION = '5ad91a646a753cc3eecd8751e61cf458dadd9ac4'
GNU = {'LGPL-2.1.txt': '20e50fe7aae3e56378ebf0417d9de904f55a0e61e4df315333e632a4d3555d95', 'GPL-2.0.txt': 'edaef632cbb643e4e7a221717a6c441a4c1a7c918e6e4d56debc3d8739b233f6', 'GPL-3.0.txt': '8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903'}


def read(path, limit=4 * 1024 * 1024):
    with path.open('rb') as source:
        data = source.read(limit + 1)
    if len(data) > limit:
        raise ValueError('LSP runtime input exceeds limit')
    return data


def sources(lib, catalog):
    if catalog.get('source_revision') != REVISION or catalog.get('zig_version') != '0.14.0':
        raise ValueError('unreviewed LSP Zig source identity')
    lib = lib.resolve()
    seen = set()
    for item in catalog['files']:
        relative = PurePosixPath(item['path'])
        if relative.is_absolute() or '..' in relative.parts or '\\' in item['path'] or item['path'] in seen:
            raise ValueError('invalid LSP runtime source path')
        seen.add(item['path'])
        if item['path'] == 'LICENSE':
            source, boundary = lib.parent / 'LICENSE', lib.parent
        elif relative.parts and relative.parts[0] == 'lib':
            source, boundary = lib.joinpath(*relative.parts[1:]), lib
        else:
            raise ValueError('unexpected LSP source location')
        if not source.resolve().is_relative_to(boundary):
            raise ValueError('LSP runtime source escapes SDK')
        data = read(source)
        if hashlib.sha256(data).hexdigest() != item['sha256']:
            raise ValueError('LSP runtime source bytes differ from review')
        if 'license_text' in item:
            if item['license_text'].encode('utf-8') != data:
                raise ValueError('runtime license text differs from original source')
        else:
            comments = re.findall(r'/\*.*?\*/|(?:^[ \t]*//[^\n]*(?:\n|$))+', data.decode('utf-8'), re.M | re.S)
            expected = [text for text in comments if re.search(r'copyright|SPDX-License-Identifier|redistribution|permission is hereby|licens', text, re.I)]
            if not expected or item.get('source_notices') != expected:
                raise ValueError('runtime copyright notices differ from original source')
    return catalog['files']


def startup_terms(catalog, target):
    if target not in {'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu'}:
        raise ValueError('unreviewed LSP runtime target')
    terms = catalog['startup_terms'][target]
    arm = target == 'aarch64-unknown-linux-gnu'
    if terms.get('license') != 'LGPL-2.1-or-later' or terms.get('compiled_linking_exception') is not (not arm) or terms.get('corresponding_source_required') is not arm:
        raise ValueError('startup licensing conditions changed')
    expected = 'lib/libc/glibc/sysdeps/' + ('aarch64' if arm else 'x86_64') + '/start-2.33.S'
    if terms.get('source') != expected:
        raise ValueError('startup source differs from glibc baseline')
    return terms


def prepare(evidence, target):
    root = Path(__file__).resolve().parents[1]
    catalog_bytes = read(root / 'ci/lsp-zig-runtime-catalog.json')
    catalog = json.loads(catalog_bytes)
    terms = startup_terms(catalog, target)
    if len(catalog['files']) != 42:
        raise ValueError('LSP runtime source inventory changed')
    version = subprocess.check_output(['zig', 'version'], text=True).strip()
    if version != '0.14.0':
        raise ValueError('unexpected LSP Zig version')
    environment = json.loads(subprocess.check_output(['zig', 'env']))
    records = sources(Path(environment['lib_dir']), catalog)
    provenance = json.loads(read(evidence / 'source-provenance.json'))
    if not re.fullmatch('[0-9a-f]{40}', provenance['source_ref']):
        raise ValueError('missing immutable LSP source reference')
    documents = json.loads(read(root / 'ci/licenses/gnu-source-texts.json'))['licenses']
    if {item['file'] for item in documents} != set(GNU) or len(documents) != len(GNU):
        raise ValueError('incomplete GNU source license documents')
    texts = []
    for item in documents:
        data = read(root / 'ci/licenses' / item['file'])
        if hashlib.sha256(data).hexdigest() != GNU[item['file']] or item['sha256'] != GNU[item['file']]:
            raise ValueError('original GNU license document changed')
        texts.append({**item, 'text': data.decode('utf-8')})
    bundle = json.loads(read(evidence / 'NOTICE.json', 64 * 1024 * 1024))
    if bundle.get('schema_version') != 1 or 'non_cargo_runtimes' in bundle:
        raise ValueError('unexpected LSP notice preparation state')
    runtime = {'name': 'Zig vendored LLVM libunwind, GNU nonshared runtime and executable startup', 'zig_version': version, 'source_revision': REVISION, 'catalog_sha256': hashlib.sha256(catalog_bytes).hexdigest(), 'files': records, 'credits': catalog['credits'], 'coverage': catalog['coverage'], 'startup_terms': terms, 'original_gnu_license_documents': texts, 'license_document_scope': 'Original source files determine applicable terms; inclusion of a document is not a blanket binary license.'}
    if terms['corresponding_source_required']:
        runtime['corresponding_source'] = {'source_ref': provenance['source_ref'], 'archive': 'ktav-lsp-source-' + target + '.tar.gz', 'conditions': 'Matching source and relink materials must be offered with equivalent access from the same final distribution location; a CI-only artifact is not sufficient.'}
    bundle['non_cargo_runtimes'] = [runtime]
    bundle['coverage'] += ' Includes source-verified GNU/LLVM runtime and startup notices; retained inputs and corresponding-source distribution require independent gates.'
    data = (json.dumps(bundle, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode('utf-8')
    payload = gzip.compress(data, mtime=0)
    if len(data) > 64 * 1024 * 1024 or len(payload) > 4 * 1024 * 1024:
        raise ValueError('LSP runtime notice resource exceeds bounds')
    (evidence / 'NOTICE.json').write_bytes(data)
    (evidence / 'NOTICE.json.gz').write_bytes(payload)
    (evidence / 'lsp-zig-runtime-catalog.json').write_bytes(catalog_bytes)
    notice = ('Ktav LSP includes GNU C Library startup code.\nOriginal per-file conditions and copyright notices are retained in NOTICE.json.\nStartup license: LGPL-2.1-or-later.\nCompiled-linking exception for this architecture: ' + str(terms['compiled_linking_exception']) + '\nSource reference: ' + provenance['source_ref'] + '\n')
    if terms['corresponding_source_required']:
        notice += 'Matching source/relink archive: ' + runtime['corresponding_source']['archive'] + '\nModification for personal use and reverse engineering for debugging such modifications are not contractually prohibited by Ktav.\n'
    original_startup = next(item for item in records if item['path'] == terms['source'])
    notice += '\n' + '\n'.join(original_startup['source_notices']) + '\n'
    notice += '\n' + next(item['text'] for item in texts if item['file'] == 'LGPL-2.1.txt')
    (evidence / 'NOTICE.gnu-startup.txt').write_text(notice, encoding='utf-8')
    print(f'Prepared {len(records)} exact LSP runtime source records for {target}')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: lsp_runtime_notices.py EVIDENCE TARGET')
    prepare(Path(sys.argv[1]), sys.argv[2])
