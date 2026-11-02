#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
target="${1:?usage: bash rebuild.sh TARGET}"
case "$target" in
  x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu) ;;
  *) printf '%s\n' 'unsupported source/relink target' >&2; exit 1 ;;
esac
if [[ "$(zig version)" != '0.14.0' ]]; then
  printf '%s\n' 'Zig 0.14.0 is required' >&2
  exit 1
fi
python3 - "$root" "$target" <<'PY'
import hashlib
import json
from pathlib import Path
import sys
import tarfile

root, target = Path(sys.argv[1]), sys.argv[2]
record = json.loads((root / 'provenance.json').read_text())
if record['target'] != target:
    raise SystemExit('source package target differs from supplied Rust libraries')
archive = root / 'upstream/zig-source.tar.gz'
if hashlib.sha256(archive.read_bytes()).hexdigest() != record['zig_source_sha256']:
    raise SystemExit('original Zig source archive differs from provenance')
source = root / 'upstream' / ('zig-' + record['zig_revision'])
if not source.exists():
    with tarfile.open(archive, 'r:gz') as incoming:
        prefix = 'zig-' + record['zig_revision'] + '/lib/'
        members = [item for item in incoming if item.name.startswith(prefix)]
        incoming.extractall(root / 'upstream', members=members, filter='data')
if not (source / 'lib').is_dir():
    raise SystemExit('Zig source SDK is incomplete')
PY
export ZIG_LIB_DIR="$root/upstream/zig-5ad91a646a753cc3eecd8751e61cf458dadd9ac4/lib"
export KTAV_NATIVE_NOTICES="$root/legal/NOTICE.json.gz"
unset RUSTFLAGS CARGO_ZIGBUILD_ZIG_COMMAND CARGO_ZIGBUILD_ZIG_COMMAND_ARGS
export CARGO_ENCODED_RUSTFLAGS="$(printf '%s\037%s' --sysroot "$root/toolchain")"
cd -- "$root/project"
# Изменения SDK должны вызывать повторную линковку.
touch -- build.rs
cargo +1.99.1 zigbuild --locked --offline --release --target "$target.2.17" --target-dir "$root/target-relinked"
