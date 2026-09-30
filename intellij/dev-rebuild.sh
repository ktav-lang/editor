#!/usr/bin/env bash
# Build the current worktree's LSP and update only the selected Ktav plugin.
# Close the selected IDE yourself before installing; this helper never kills processes.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
LSP_DIR="$REPO_ROOT/lsp"
PLUGINS_DIR="${KTAV_PLUGINS_DIR:-}"
IDE_EXECUTABLE="${KTAV_IDE_EXECUTABLE:-}"
CACHE_DIR="${KTAV_IDE_CACHE_DIR:-}"
PLATFORM="${KTAV_PLATFORM:-}"
PLUGIN_ZIP="${KTAV_PLUGIN_ZIP:-}"
REBUILD_PLUGIN=0
NO_RESTART=0

usage() {
    cat <<'USAGE'
Usage: dev-rebuild.sh --plugins-dir PATH --platform PLATFORM [options]
  --plugins-dir PATH     Selected IDE's plugins directory (KTAV_PLUGINS_DIR)
  --platform PLATFORM    win32-x64/arm64, linux-x64/arm64, darwin-x64/arm64
                         (KTAV_PLATFORM); builds the corresponding Rust target
  --plugin               Build and replace only plugins/ktav-intellij (Python 3 required)
  --plugin-zip PATH       Exact output archive if multiple versions exist
                         (KTAV_PLUGIN_ZIP; used with --plugin)
  --ide-executable PATH   Launch this IDE after installation (KTAV_IDE_EXECUTABLE)
  --cache-dir PATH        Report this IDE's log path only (KTAV_IDE_CACHE_DIR)
  --no-restart            Do not launch an IDE, even if one is configured
  --help                 Show this help without changing files
Close the selected IDE manually first. No processes are killed and no IDE
caches, logs, unrelated plugins or ZIP archives are deleted. Repository paths
are resolved from this script, not the caller's working directory.
USAGE
}
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
while (($#)); do
    case "$1" in
        --plugin) REBUILD_PLUGIN=1; shift ;;
        --no-restart) NO_RESTART=1; shift ;;
        --help) usage; exit 0 ;;
        --plugins-dir|--platform|--plugin-zip|--ide-executable|--cache-dir)
            (($# >= 2)) || fail "Missing value for $1"
            case "$1" in
                --plugins-dir) PLUGINS_DIR="$2" ;;
                --platform) PLATFORM="$2" ;;
                --plugin-zip) PLUGIN_ZIP="$2" ;;
                --ide-executable) IDE_EXECUTABLE="$2" ;;
                --cache-dir) CACHE_DIR="$2" ;;
            esac
            shift 2 ;;
        *) fail "Unknown argument: $1" ;;
    esac
done

[[ -n "$PLUGINS_DIR" ]] || fail 'Set --plugins-dir or KTAV_PLUGINS_DIR'
[[ -d "$PLUGINS_DIR" ]] || fail "Plugins directory does not exist: $PLUGINS_DIR"
# Resolve user-supplied relative paths before changing working directory.
PLUGINS_DIR="$(cd -- "$PLUGINS_DIR" && pwd)"
if [[ -n "$PLUGIN_ZIP" ]]; then
    [[ -f "$PLUGIN_ZIP" ]] || fail "Archive does not exist: $PLUGIN_ZIP"
    PLUGIN_ZIP="$(cd -- "$(dirname -- "$PLUGIN_ZIP")" && pwd)/$(basename -- "$PLUGIN_ZIP")"
fi
if [[ -n "$IDE_EXECUTABLE" && "$NO_RESTART" == 0 ]]; then
    [[ -f "$IDE_EXECUTABLE" ]] || fail "IDE executable does not exist: $IDE_EXECUTABLE"
    IDE_EXECUTABLE="$(cd -- "$(dirname -- "$IDE_EXECUTABLE")" && pwd)/$(basename -- "$IDE_EXECUTABLE")"
fi
case "$PLATFORM" in
    win32-x64) TARGET=x86_64-pc-windows-msvc; SUFFIX=.exe ;;
    win32-arm64) TARGET=aarch64-pc-windows-msvc; SUFFIX=.exe ;;
    linux-x64) TARGET=x86_64-unknown-linux-gnu; SUFFIX= ;;
    linux-arm64) TARGET=aarch64-unknown-linux-gnu; SUFFIX= ;;
    darwin-x64) TARGET=x86_64-apple-darwin; SUFFIX= ;;
    darwin-arm64) TARGET=aarch64-apple-darwin; SUFFIX= ;;
    *) fail 'Set --platform or KTAV_PLATFORM to a supported platform' ;;
esac
INSTALLED_PLUGIN="$PLUGINS_DIR/ktav-intellij"
[[ ! -L "$INSTALLED_PLUGIN" ]] || fail 'Refusing to modify a symlinked plugin'
if [[ "$REBUILD_PLUGIN" == 0 ]]; then
    [[ -d "$INSTALLED_PLUGIN/lib/bin/$PLATFORM" ]] || fail 'Ktav plugin is not installed; use --plugin'
    [[ ! -L "$INSTALLED_PLUGIN/lib" && ! -L "$INSTALLED_PLUGIN/lib/bin" && ! -L "$INSTALLED_PLUGIN/lib/bin/$PLATFORM" ]] || fail 'Refusing to modify symlinked plugin binaries'
fi
if [[ "$REBUILD_PLUGIN" == 1 ]]; then
    command -v python3 >/dev/null || fail 'Python 3 is required for plugin ZIP installation'
fi

(cd -- "$LSP_DIR" && cargo build --locked --release --target "$TARGET" --target-dir "$LSP_DIR/target")
LSP_BIN="$LSP_DIR/target/$TARGET/release/ktav-lsp$SUFFIX"
[[ -f "$LSP_BIN" ]] || fail "Built binary not found: $LSP_BIN"
mkdir -p -- "$SCRIPT_DIR/bin/$PLATFORM" "$SCRIPT_DIR/bin/$TARGET"
cp -- "$LSP_BIN" "$SCRIPT_DIR/bin/$PLATFORM/ktav-lsp$SUFFIX"
cp -- "$LSP_BIN" "$SCRIPT_DIR/bin/$TARGET/ktav-lsp$SUFFIX"

if [[ "$REBUILD_PLUGIN" == 1 ]]; then
    (cd -- "$SCRIPT_DIR" && ./gradlew buildPlugin --no-daemon)
    if [[ -z "$PLUGIN_ZIP" ]]; then
        shopt -s nullglob
        archives=("$SCRIPT_DIR"/build/distributions/ktav-intellij-*.zip)
        [[ ${#archives[@]} == 1 ]] || fail 'Select the exact built archive with --plugin-zip'
        PLUGIN_ZIP="${archives[0]}"
    fi
    staging="$(mktemp -d "$PLUGINS_DIR/.ktav-install.XXXXXX")"
    cleanup() {
        # Roll back if replacement failed after moving the old Ktav installation.
        if [[ -d "$staging/previous" && ! -e "$INSTALLED_PLUGIN" ]]; then
            mv -- "$staging/previous" "$INSTALLED_PLUGIN"
        fi
        rm -rf -- "$staging"
    }
    trap cleanup EXIT
    python3 - "$PLUGIN_ZIP" "$staging" <<'PY'
import pathlib, stat, sys, zipfile

with zipfile.ZipFile(sys.argv[1]) as archive:
    members = archive.infolist()
    if not members:
        raise SystemExit("ERROR: Empty plugin archive")
    for member in members:
        name = member.filename
        if (not name.startswith("ktav-intellij/") or "\\" in name
                or any(part in (".", "..") for part in name.split("/"))
                or stat.S_ISLNK(member.external_attr >> 16)):
            raise SystemExit(f"ERROR: Unsafe plugin archive path: {name}")
    archive.extractall(sys.argv[2])
    for member in members:
        mode = (member.external_attr >> 16) & 0o777
        if member.create_system == 3 and mode:
            pathlib.Path(sys.argv[2], member.filename).chmod(mode)
PY
    [[ -d "$staging/ktav-intellij/lib" && ! -L "$staging/ktav-intellij" ]] || fail 'Archive has no Ktav plugin'
    if [[ -e "$INSTALLED_PLUGIN" ]]; then
        mv -- "$INSTALLED_PLUGIN" "$staging/previous"
    fi
    mv -- "$staging/ktav-intellij" "$INSTALLED_PLUGIN"
else
    [[ ! -L "$INSTALLED_PLUGIN/lib/bin/$PLATFORM/ktav-lsp$SUFFIX" ]] || fail 'Refusing to overwrite a symlinked binary'
    cp -- "$LSP_BIN" "$INSTALLED_PLUGIN/lib/bin/$PLATFORM/ktav-lsp$SUFFIX"
    # Update an existing legacy triple directory, without creating unrelated paths.
    if [[ -d "$INSTALLED_PLUGIN/lib/bin/$TARGET" ]]; then
        [[ ! -L "$INSTALLED_PLUGIN/lib/bin/$TARGET" && ! -L "$INSTALLED_PLUGIN/lib/bin/$TARGET/ktav-lsp$SUFFIX" ]] || fail 'Refusing to overwrite symlinked binaries'
        cp -- "$LSP_BIN" "$INSTALLED_PLUGIN/lib/bin/$TARGET/ktav-lsp$SUFFIX"
    fi
fi

printf 'Updated Ktav plugin: %s\n' "$INSTALLED_PLUGIN"
if [[ -n "$CACHE_DIR" ]]; then
    printf 'IDE log (unchanged): %s/log/idea.log\n' "$CACHE_DIR"
fi
if [[ "$NO_RESTART" == 0 && -n "$IDE_EXECUTABLE" ]]; then
    "$IDE_EXECUTABLE" >/dev/null 2>&1 &
    printf 'Launched selected IDE: %s\n' "$IDE_EXECUTABLE"
fi
