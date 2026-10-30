"""Собрать source/relink материалы без изменения исходного release tree."""

import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request

ZIG_REVISION = '5ad91a646a753cc3eecd8751e61cf458dadd9ac4'
ZIG_SOURCE_SHA = '30811b074ae10baaf9a8ad6494982d860c42eb73904d40116a18a5022721ef04'
LGPL_SHA = '20e50fe7aae3e56378ebf0417d9de904f55a0e61e4df315333e632a4d3555d95'
TARGETS = {'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu'}


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def copy_regular(source, target):
    if source.is_symlink() or not source.is_file():
        raise ValueError('source bundle requires regular files')
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    shutil.copymode(source, target)


def build(evidence, destination, target):
    root = Path.cwd().resolve()
    evidence = evidence.resolve()
    destination = destination.resolve()
    if target not in TARGETS or destination.exists() or destination.is_relative_to(root):
        raise ValueError('invalid new source bundle destination or target')
    destination.mkdir(parents=True)
    bundle = destination / 'lsp-source'
    project = bundle / 'project'
    project.mkdir(parents=True)
    paths = subprocess.check_output(['git', 'ls-files', '-z', '--', 'lsp/']).decode().split('\0')
    paths = {path for path in paths if path}
    paths.update({'lsp/build.rs', 'lsp/LICENSE-MIT', 'lsp/LICENSE-APACHE', 'lsp/Cargo.lock'})
    for path in sorted(paths):
        source = root / path
        relative = Path(path).relative_to('lsp')
        if '..' in relative.parts:
            raise ValueError('unsafe source bundle member')
        copy_regular(source, project / relative)
    if (project / 'Cargo.lock').read_bytes() != (evidence / 'Cargo.lock').read_bytes():
        raise ValueError('source bundle lock differs from build evidence')
    subprocess.run(['cargo', 'vendor', '--locked', '--versioned-dirs', '--manifest-path', str(project / 'Cargo.toml'), str(project / 'vendor')], check=True, stdout=subprocess.PIPE)
    if (project / 'Cargo.lock').read_bytes() != (evidence / 'Cargo.lock').read_bytes():
        raise ValueError('vendoring changed release lock')
    config = project / '.cargo/config.toml'
    if config.exists():
        raise ValueError('source already has Cargo configuration requiring review')
    config.parent.mkdir(exist_ok=True)
    config.write_text('[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor"\n[net]\noffline = true\n', encoding='utf-8')
    legal = bundle / 'legal'
    legal.mkdir()
    for name in ('NOTICE.json', 'NOTICE.json.gz', 'NOTICE.gnu-startup.txt', 'lsp-zig-runtime-catalog.json', 'COPYRIGHT-library.html', 'rustc.txt', 'source-provenance.json'):
        copy_regular(evidence / name, legal / name)
    license_path = root / 'ci/licenses/LGPL-2.1.txt'
    if digest(license_path) != LGPL_SHA:
        raise ValueError('GNU license text differs from reviewed original')
    copy_regular(license_path, legal / 'LGPL-2.1.txt')
    for name, checksum in [('GPL-2.0.txt', 'edaef632cbb643e4e7a221717a6c441a4c1a7c918e6e4d56debc3d8739b233f6'), ('GPL-3.0.txt', '8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903')]:
        source = root / 'ci/licenses' / name
        if digest(source) != checksum:
            raise ValueError('GNU SDK source license differs from reviewed original')
        copy_regular(source, legal / name)
    for name in ('valuable-0.1.1.LICENSE', 'lsp-source-supplements.json', 'gnu-source-texts.json'):
        copy_regular(root / 'ci/licenses' / name, legal / name)
    upstream = bundle / 'upstream'
    upstream.mkdir()
    archive = upstream / 'zig-source.tar.gz'
    with urllib.request.urlopen('https://codeload.github.com/ziglang/zig/tar.gz/' + ZIG_REVISION, timeout=120) as response, archive.open('xb') as output:
        total = 0
        while chunk := response.read(1024 * 1024):
            total += len(chunk)
            if total > 256 * 1024 * 1024:
                raise ValueError('Zig source archive exceeds limit')
            output.write(chunk)
    if digest(archive) != ZIG_SOURCE_SHA:
        raise ValueError('Zig corresponding source archive changed')
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    libraries = sysroot / 'lib/rustlib' / target / 'lib'
    if not libraries.is_dir() or libraries.is_symlink():
        raise ValueError('matching target standard libraries missing')
    for source in sorted(libraries.rglob('*')):
        if source.is_symlink():
            raise ValueError('unreviewed target library symlink')
        if source.is_file():
            copy_regular(source, bundle / 'toolchain/lib/rustlib' / target / 'lib' / source.relative_to(libraries))
    copy_regular(root / 'scripts/rebuild_lsp_source.sh', bundle / 'rebuild.sh')
    copy_regular(root / 'ci/lsp-source-README.txt', bundle / 'README.txt')
    resource = json.loads((evidence / 'native-resource-verification.json').read_text(encoding='utf-8'))
    if resource['notice_sha256'] != digest(legal / 'NOTICE.json'):
        raise ValueError('source package notice differs from actual executable verification')
    record = {'schema_version': 1, 'target': target, 'notice_sha256': resource['notice_sha256'], 'original_binary_sha256': resource['binary_sha256'], 'source_ref': json.loads((legal / 'source-provenance.json').read_text())['source_ref'], 'cargo_lock_sha256': digest(project / 'Cargo.lock'), 'zig_revision': ZIG_REVISION, 'zig_source_sha256': ZIG_SOURCE_SHA, 'rustc': (legal / 'rustc.txt').read_text(), 'coverage': 'Source/relink prototype; publication together with final distribution remains required.'}
    (bundle / 'provenance.json').write_text(json.dumps(record, sort_keys=True, indent=2), encoding='utf-8')
    manifest = []
    for path in sorted(bundle.rglob('*')):
        if path.is_symlink():
            raise ValueError('source archive must not contain unreviewed symlinks')
        if path.is_file():
            manifest.append({'path': path.relative_to(bundle).as_posix(), 'sha256': digest(path), 'bytes': path.stat().st_size, 'mode': 0o755 if path.stat().st_mode & 0o111 else 0o644})
    (bundle / 'MANIFEST.json').write_text(json.dumps(manifest, sort_keys=True, indent=2), encoding='utf-8')
    output_path = destination / ('ktav-lsp-source-' + target + '.tar.gz')
    with output_path.open('xb') as output, gzip.GzipFile(filename='', fileobj=output, mode='wb', mtime=0) as compressed, tarfile.open(fileobj=compressed, mode='w') as tar:
        for path in sorted(bundle.rglob('*')):
            if path.is_file():
                info = tar.gettarinfo(str(path), arcname='lsp-source/' + path.relative_to(bundle).as_posix())
                info.uid = info.gid = info.mtime = 0
                info.uname = info.gname = ''
                info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
                with path.open('rb') as source:
                    tar.addfile(info, source)
    report = {**record, 'archive_sha256': digest(output_path), 'archive_bytes': output_path.stat().st_size, 'files': len(manifest) + 1}
    (evidence / 'corresponding-source.json').write_text(json.dumps(report, sort_keys=True, indent=2), encoding='utf-8')
    binding = {'schema_version': 1, 'target': target, 'source_ref': record['source_ref'], 'binary': {'file': resource['binary'], 'sha256': resource['binary_sha256']}, 'notice': {'file': 'NOTICE.json', 'sha256': resource['notice_sha256']}, 'corresponding_source': {'file': output_path.name, 'sha256': report['archive_sha256']}, 'coverage': 'Exact prototype artifact binding; final distribution must publish all matching materials together.'}
    (evidence / 'native-source-binding.json').write_text(json.dumps(binding, sort_keys=True, indent=2), encoding='utf-8')
    print(json.dumps(report, sort_keys=True))


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit('usage: prepare_lsp_source_bundle.py EVIDENCE NEW_DESTINATION TARGET')
    build(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
