"""Перенести все legal/source материалы в прежние plugin binary directories."""

import copy
import hashlib
import json
from pathlib import Path
import shutil
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from lsp_distribution import TARGETS, verify


def empty_output(path):
    if path.is_symlink() or (path.exists() and (not path.is_dir() or any(path.iterdir()))):
        raise ValueError('plugin binary output must be new and empty')


def copy_platform(source, output):
    manifest = verify(source)
    empty_output(output)
    if output.resolve().is_relative_to(source.resolve()):
        raise ValueError('plugin output overlaps distribution input')
    client_name = 'ktav-lsp.exe' if 'windows' in manifest['target'] else 'ktav-lsp'
    previous_name = manifest['binary']
    if client_name != previous_name and client_name in manifest['files']:
        raise ValueError('duplicate canonical plugin executable')
    updated = copy.deepcopy(manifest)
    identity = updated['files'].pop(previous_name)
    updated['files'][client_name] = identity
    updated['binary'] = client_name
    binding_name = manifest['source_binding']
    binding_data = None
    if binding_name:
        binding = json.loads((source / binding_name).read_bytes())
        binding['binary'] = {'file': client_name, 'sha256': identity['sha256']}
        binding_data = (json.dumps(binding, sort_keys=True, indent=2) + '\n').encode('utf-8')
        updated['files'][binding_name] = {'bytes': len(binding_data), 'sha256': hashlib.sha256(binding_data).hexdigest()}
    output.mkdir(parents=True, exist_ok=True)
    for filename in manifest['files']:
        if filename == binding_name:
            continue
        destination = output / (client_name if filename == previous_name else filename)
        if destination.exists():
            raise ValueError('plugin destination already exists')
        shutil.copyfile(source / filename, destination)
    if binding_data is not None:
        with (output / binding_name).open('xb') as destination:
            destination.write(binding_data)
    if 'windows' not in manifest['target']:
        (output / client_name).chmod(0o755)
    with (output / (manifest['asset'] + '.distribution.json')).open('xb') as destination:
        destination.write((json.dumps(updated, sort_keys=True, indent=2) + '\n').encode('utf-8'))
    checked = verify(output)
    if checked['files'][client_name] != manifest['files'][previous_name]:
        raise ValueError('plugin executable bytes changed during organization')
    return checked


def organize(incoming, output):
    if incoming.is_symlink() or not incoming.is_dir():
        raise ValueError('distribution input must be a regular directory')
    empty_output(output)
    if output.resolve().is_relative_to(incoming.resolve()):
        raise ValueError('plugin output overlaps distribution inputs')
    inputs = {}
    source_refs = set()
    for directory in sorted(incoming.iterdir()):
        if directory.is_symlink() or not directory.is_dir():
            raise ValueError('unexpected non-directory distribution input')
        manifest = verify(directory)
        if manifest['target'] in inputs:
            raise ValueError('duplicate plugin target distribution')
        inputs[manifest['target']] = directory
        source_refs.add(manifest['source_ref'])
    if set(inputs) != set(TARGETS) or len(source_refs) != 1:
        raise ValueError('plugin requires six matching-source platform payloads')
    output.mkdir(parents=True, exist_ok=True)
    records = []
    for target, (platform, _) in TARGETS.items():
        manifest = copy_platform(inputs[target], output / platform)
        records.append({'target': target, 'platform': platform, 'binary_sha256': manifest['files'][manifest['binary']]['sha256'], 'source_archive': manifest['corresponding_source']})
    if {path.name for path in output.iterdir()} != {value[0] for value in TARGETS.values()}:
        raise ValueError('unexpected plugin platform directory')
    return {'source_ref': next(iter(source_refs)), 'platforms': records}


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: organize_lsp_distribution.py DOWNLOADED_PAYLOADS PLUGIN_BIN_DIRECTORY')
    print(json.dumps(organize(Path(sys.argv[1]), Path(sys.argv[2])), sort_keys=True, indent=2))
