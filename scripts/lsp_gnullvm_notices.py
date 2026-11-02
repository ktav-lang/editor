"""Подготовить Windows runtime notices из проверенных оригинальных исходников."""

import gzip
import hashlib
import json
from pathlib import Path
import re
import sys
import urllib.error
import urllib.request

SDK_SHA256 = 'fd5462b7f5d93b2ff981fcd3f9c7e427e47f3203c64ea91c27bcc444eebd74cd'
REVISIONS = {
    'mingw-w64/mingw-w64': '0d6dfbc348dfad714761c0a7151615767f96bcb5',
    'llvm/llvm-project': '7a941dfb2877a0b8cf30323fefa85ad019e0b0f5',
}
TARGETS = {'x86_64-pc-windows-gnullvm', 'aarch64-pc-windows-gnullvm'}
DATA_ONLY = {'mingw-w64-crt/crt/cinitexe.c', 'mingw-w64-crt/misc/mingw_matherr.c'}
MAX_FILE = 4 * 1024 * 1024
MAX_TOTAL = 64 * 1024 * 1024


def original(url):
    if not any(url.startswith('https://raw.githubusercontent.com/' + repo + '/' + revision + '/') for repo, revision in REVISIONS.items()):
        raise ValueError('unreviewed runtime source URL')
    for attempt in range(3):
        try:
            with urllib.request.urlopen(url, timeout=90) as response:
                data = response.read(MAX_FILE + 1)
            break
        except urllib.error.HTTPError as error:
            if error.code not in {500, 502, 503, 504} or attempt == 2:
                raise
        except (TimeoutError, urllib.error.URLError):
            if attempt == 2:
                raise
    if len(data) > MAX_FILE:
        raise ValueError('runtime source exceeds size bound')
    return data


def catalogue(root):
    data = (root / 'ci/lsp-gnullvm-runtime-catalog.json').read_bytes()
    if len(data) > MAX_FILE:
        raise ValueError('runtime catalogue exceeds bound')
    result = json.loads(data)
    if result.get('schema_version') != 1 or result.get('sdk_archive_sha256') != SDK_SHA256 or result.get('sdk_version') != '20261020':
        raise ValueError('unreviewed Windows runtime SDK')
    if result.get('mingw_revision') != REVISIONS['mingw-w64/mingw-w64'] or result.get('llvm_revision') != REVISIONS['llvm/llvm-project']:
        raise ValueError('unreviewed Windows runtime source revisions')
    if set(result['targets']) != TARGETS or len(result['source_files']) != 55 or len(result['original_documents']) != 6:
        raise ValueError('Windows runtime source inventory changed')
    return result, hashlib.sha256(data).hexdigest()


def verify_sources(catalog, read=original):
    seen = set()
    total = 0
    for item in [*catalog['source_files'], *catalog['original_documents']]:
        url = item['source']
        if url in seen:
            raise ValueError('duplicate runtime source record')
        seen.add(url)
        data = read(url)
        total += len(data)
        if len(data) > MAX_FILE or total > MAX_TOTAL or hashlib.sha256(data).hexdigest() != item['sha256']:
            raise ValueError('runtime source bytes differ from review')
        if 'text' in item:
            if item['text'].encode('utf-8') != data:
                raise ValueError('runtime legal document differs from original')
            continue
        comments = re.findall(r'/\*.*?\*/|(?:^[ \t]*//[^\n]*(?:\n|$))+', data.decode('utf-8'), re.M | re.S)
        expected = [text for text in comments if re.search(r'copyright|licens|public domain|permission|redistribution', text, re.I)]
        if item.get('source_notices') != expected or (not expected and item['path'] not in DATA_ONLY):
            raise ValueError('runtime source notices differ from original')


def augment(bundle, catalog, target, digest):
    if target not in TARGETS or bundle.get('schema_version') != 1 or 'non_cargo_runtimes' in bundle:
        raise ValueError('unexpected Windows notice preparation state')
    runtime = {
        'name': 'LLVM-MinGW executable startup, MinGW runtime and LLVM libunwind',
        'target': target,
        'sdk_archive_sha256': SDK_SHA256,
        'sdk_recipe_revision': catalog['sdk_recipe_revision'],
        'mingw_revision': catalog['mingw_revision'],
        'llvm_revision': catalog['llvm_revision'],
        'catalog_sha256': digest,
        'source_files': catalog['source_files'],
        'original_documents': catalog['original_documents'],
        'coverage': catalog['coverage'],
        'license_document_scope': 'Original source terms determine applicability. Including the SDK runtime legal collection does not classify every SDK component as linked code.',
    }
    return {**bundle, 'non_cargo_runtimes': [runtime], 'coverage': bundle['coverage'] + ' Includes reviewed Windows GNU/LLVM source notices; retained linker inputs require a separate final-binary gate.'}


def plain_notice(catalog):
    sections = ['Ktav LSP Windows GNU/LLVM runtime notices.\nOriginal upstream terms follow unchanged.\n']
    for item in catalog['original_documents']:
        sections.extend(['\n===== ' + item['source'] + ' =====\n', item['text']])
    for item in catalog['source_files']:
        sections.extend(['\n===== ' + item['source'] + ' =====\n', '\n'.join(item['source_notices'])])
    return '\n'.join(sections).encode('utf-8')


def prepare(evidence, target):
    root = Path(__file__).resolve().parents[1]
    catalog, digest = catalogue(root)
    verify_sources(catalog)
    notice = evidence / 'NOTICE.json'
    data = notice.read_bytes()
    if len(data) > MAX_TOTAL:
        raise ValueError('Cargo/std notice exceeds size bound')
    bundle = augment(json.loads(data), catalog, target, digest)
    combined = (json.dumps(bundle, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode('utf-8')
    payload = gzip.compress(combined, mtime=0)
    if len(combined) > MAX_TOTAL or len(payload) > MAX_FILE:
        raise ValueError('Windows runtime notice resource exceeds bound')
    original_notice = evidence / 'NOTICE.cargo-std.json'
    if original_notice.exists():
        raise ValueError('Windows runtime notices already prepared')
    original_notice.write_bytes(data)
    notice.write_bytes(combined)
    (evidence / 'NOTICE.json.gz').write_bytes(payload)
    (evidence / 'NOTICE.windows-runtime.txt').write_bytes(plain_notice(catalog))
    (evidence / 'lsp-gnullvm-runtime-catalog.json').write_bytes((root / 'ci/lsp-gnullvm-runtime-catalog.json').read_bytes())
    print(f'Prepared {len(catalog["source_files"])} source-backed Windows runtime records for {target}')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: lsp_gnullvm_notices.py EVIDENCE TARGET')
    prepare(Path(sys.argv[1]), sys.argv[2])
