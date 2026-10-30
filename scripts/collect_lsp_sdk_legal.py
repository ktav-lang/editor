"""Сохранить actual SDK legal documents и candidates без license inference."""

import base64
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

LEGAL = re.compile(r'(?:licen[sc]e|eula|redist.*list|third.?party.*notice|acknowledg)', re.I)


def collect(evidence):
    candidates = []
    identity = {}
    if platform.system() == 'Darwin':
        developer = Path(subprocess.check_output(['xcode-select', '-p'], text=True).strip())
        sdk = Path(subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip())
        identity = {'developer_directory': str(developer), 'sdk_directory': str(sdk)}
        resources = developer.parent / 'Resources'
        if resources.is_dir():
            candidates.append(('Xcode-Resources', resources))
        for name in ('SDKSettings.json', 'SDKSettings.plist'):
            source = sdk / name
            if source.is_file():
                (evidence / name).write_bytes(source.read_bytes())
    elif platform.system() == 'Windows':
        vswhere = Path(os.environ['ProgramFiles(x86)']) / 'Microsoft Visual Studio/Installer/vswhere.exe'
        installations = json.loads(subprocess.check_output([str(vswhere), '-all', '-format', 'json']).decode('utf-8-sig'))
        identity = {'installation_candidates': [{'version': item.get('installationVersion'), 'path': item['installationPath']} for item in installations]}
        for number, item in enumerate(installations):
            root = Path(item['installationPath'])
            for relative in ('Licenses', 'licenses', 'VC/Redist', 'Common7/IDE/Resources'):
                source = root / relative
                if source.is_dir():
                    candidates.append(('VS-' + str(number) + '-' + relative.replace('/', '-'), source))
    else:
        raise ValueError('SDK legal inventory supports only Windows/macOS')
    records = []
    selected = []
    total = 0
    seen = set()
    for label, root in candidates:
        for source in sorted(root.rglob('*')):
            if not source.is_file() or source.is_symlink() or not LEGAL.search(source.name):
                continue
            if source.suffix.lower() not in {'.txt', '.rtf', '.html', '.htm', '.md', '.xml', ''}:
                continue
            resolved = source.resolve()
            if resolved in seen:
                continue
            seen.add(resolved)
            selected.append((label + '/' + source.relative_to(root).as_posix(), source, source.stat().st_size))
    index = [{'location': location, 'bytes': size} for location, source, size in selected]
    (evidence / 'sdk-legal-index.json').write_text(json.dumps({'identity_candidates': identity, 'documents': index, 'candidate_bytes': sum(item['bytes'] for item in index)}, sort_keys=True, indent=2), encoding='utf-8')
    for location, source, size in selected:
        if size > 4 * 1024 * 1024 or total + size > 32 * 1024 * 1024:
            raise ValueError(f'SDK legal inventory exceeds document bound: {location}, file_bytes={size}, preceding_bytes={total}')
        data = source.read_bytes()
        total += len(data)
        records.append({'location': location, 'sha256': hashlib.sha256(data).hexdigest(), 'raw_bytes_base64': base64.b64encode(data).decode('ascii')})
    (evidence / 'sdk-legal-documents.json').write_text(json.dumps({'identity_candidates': identity, 'documents': records, 'document_bytes': total, 'coverage': 'Original installed SDK documents for review, not proof of selected linker or redistribution compatibility.'}, sort_keys=True, indent=2), encoding='utf-8')
    print(json.dumps({'sdk_legal_documents': len(records), 'document_bytes': total}))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: collect_lsp_sdk_legal.py EVIDENCE_DIRECTORY')
    collect(Path(sys.argv[1]))
