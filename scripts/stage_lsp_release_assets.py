"""Сохранить проверенные raw assets и legal/source sidecars для GitHub Release."""

import json
from pathlib import Path
import shutil
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from lsp_distribution import MAX_SOURCE, TARGETS, file_info, verify
from organize_lsp_distribution import empty_output


def stage(incoming, output, source_ref):
    if incoming.is_symlink() or not incoming.is_dir():
        raise ValueError('release input must be a regular directory')
    empty_output(output)
    if output.resolve().is_relative_to(incoming.resolve()):
        raise ValueError('release output overlaps distribution inputs')
    targets = set()
    sources = {}
    for directory in sorted(incoming.iterdir()):
        if directory.is_symlink() or not directory.is_dir():
            raise ValueError('unexpected release input')
        manifest = verify(directory)
        if manifest['target'] in targets or manifest['source_ref'] != source_ref or manifest['binary'] != manifest['asset']:
            raise ValueError('release requires unique matching-source raw platform payloads')
        targets.add(manifest['target'])
        for filename, identity in manifest['files'].items():
            if filename in sources:
                raise ValueError('duplicate release asset filename')
            sources[filename] = (directory / filename, identity)
        filename = manifest['asset'] + '.distribution.json'
        if filename in sources:
            raise ValueError('duplicate release manifest filename')
        path = directory / filename
        sources[filename] = (path, file_info(path, MAX_SOURCE))
    if targets != set(TARGETS):
        raise ValueError('release requires all six platform payloads')
    output.mkdir(parents=True, exist_ok=True)
    for filename, (source, identity) in sources.items():
        destination = output / filename
        with destination.open('xb') as writer, source.open('rb') as reader:
            shutil.copyfileobj(reader, writer)
        if file_info(destination, MAX_SOURCE) != identity:
            raise ValueError('release asset bytes changed while staging')
    return {'source_ref': source_ref, 'files': {filename: identity for filename, (_, identity) in sources.items()}}


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit('usage: stage_lsp_release_assets.py INPUT_PAYLOADS OUTPUT SOURCE_REF')
    print(json.dumps(stage(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]), sort_keys=True, indent=2))
