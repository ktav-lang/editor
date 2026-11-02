import json
import re
import sys
import zipfile
from pathlib import Path


def check(names, packages):
    failures = []
    paths = [name for name in names if name.startswith('extension/node_modules/')]
    entries = {f'extension/{path}/': data for path, data in packages.items() if path.startswith('node_modules/')}
    for path in paths:
        matches = [prefix for prefix in entries if path.startswith(prefix)]
        if not matches:
            failures.append(f'untracked dependency in VSIX: {path}')
            continue
        owner = max(matches, key=len)
        if entries[owner].get('dev') is True:
            failures.append(f'development dependency in VSIX: {path}')
    for prefix, data in entries.items():
        if data.get('dev') is True or not any(path.startswith(prefix) for path in paths):
            continue
        license_files = [path[len(prefix):] for path in paths if path.startswith(prefix)]
        if not any(re.fullmatch(r'(?i)(license|copying)(\.[a-z0-9]+)?', name) for name in license_files):
            failures.append(f'no package license file in VSIX: {prefix}')
    return failures


def main():
    packages = json.loads(Path('vscode/package-lock.json').read_text(encoding='utf-8'))['packages']
    with zipfile.ZipFile(sys.argv[1]) as archive:
        names = archive.namelist()
    failures = check(names, packages)
    assert check(['extension/node_modules/x/index.js'], {'node_modules/x': {'dev': True}})
    assert check(['extension/node_modules/y/index.js'], {})
    assert check(['extension/node_modules/x/index.js'], {'node_modules/x': {}})
    assert not check(['extension/node_modules/x/LICENSE'], {'node_modules/x': {}})
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)
    print('Checked VSIX dependencies and license files')


if __name__ == '__main__':
    main()
