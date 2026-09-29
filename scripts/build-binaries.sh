#!/bin/bash
# Build ktav-lsp binaries for all platforms and place in plugin dirs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd -P)"
LSP_DIR="$ROOT_DIR/lsp"

TMP_DIR="$(cd "${TMPDIR:-/tmp}" && pwd -P)"
STAGE="$(mktemp -d "${TMP_DIR%/}/build-binaries.XXXXXX")"
STAGE="$(cd "$STAGE" && pwd -P)"
publishing=0
committed=0
vscode_touched=0
intellij_touched=0

cleanup() {
    local status=$? plugin destination
    trap - EXIT
    if (( publishing && ! committed )); then
        for plugin in vscode intellij; do
            if [[ "$plugin" == vscode && "$vscode_touched" == 0 ||
                  "$plugin" == intellij && "$intellij_touched" == 0 ]]; then
                continue
            fi
            destination="$ROOT_DIR/$plugin/bin"
            if [[ -e "$destination" || -L "$destination" ]]; then
                if ! mv "$destination" "$STAGE/discard-$plugin"; then
                    echo "Rollback failed; backups retained in $STAGE" >&2
                    return 1
                fi
            fi
            if [[ -d "$STAGE/old-$plugin" ]]; then
                if ! mv "$STAGE/old-$plugin" "$destination"; then
                    echo "Rollback failed; backups retained in $STAGE" >&2
                    return 1
                fi
            fi
        done
    fi
    # Delete only our resolved, direct child of the scratch directory.
    if [[ ! -L "$STAGE" && -d "$STAGE" &&
          "$STAGE" == "${TMP_DIR%/}"/build-binaries.* &&
          "$(cd "$STAGE/.." && pwd -P)" == "$TMP_DIR" ]]; then
        rm -rf -- "$STAGE" || return 1
    else
        echo "Refusing unsafe scratch cleanup: $STAGE" >&2
        return 1
    fi
    return "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

# Platforms to build for
TARGETS=(
    "x86_64-pc-windows-msvc:win32-x64:ktav-lsp.exe"
    "aarch64-pc-windows-msvc:win32-arm64:ktav-lsp.exe"
    "x86_64-unknown-linux-gnu:linux-x64:ktav-lsp"
    "aarch64-unknown-linux-gnu:linux-arm64:ktav-lsp"
    "x86_64-apple-darwin:darwin-x64:ktav-lsp"
    "aarch64-apple-darwin:darwin-arm64:ktav-lsp"
)

echo "Building ktav-lsp for all platforms..."
cd "$LSP_DIR"

for target_spec in "${TARGETS[@]}"; do
    IFS=: read -r cargo_target plugin_dir bin_name <<< "$target_spec"

    echo ""
    echo "Building for $cargo_target -> $plugin_dir..."

    # Determine if we need cargo cross (non-native targets)
    if [[ "$cargo_target" == *"linux"* ]] || [[ "$cargo_target" == *"darwin"* ]]; then
        cross build --locked --release --target "$cargo_target" --target-dir "$STAGE/target"
    else
        cargo build --locked --release --target "$cargo_target" --target-dir "$STAGE/target"
    fi

    # A fresh target directory excludes artifacts from previous builds.
    source_bin="$STAGE/target/$cargo_target/release/$bin_name"
    if [[ ! -f "$source_bin" || ! -s "$source_bin" || -L "$source_bin" ]]; then
        echo "Binary missing or invalid: $source_bin" >&2
        exit 1
    fi
    mkdir -p "$STAGE/vscode/$plugin_dir" "$STAGE/intellij/$plugin_dir"
    cp "$source_bin" "$STAGE/vscode/$plugin_dir/$bin_name"
    cp "$source_bin" "$STAGE/intellij/$plugin_dir/$bin_name"
done

# Publish complete sets only; restore both old sets if a move fails.
for plugin in vscode intellij; do
    mkdir -p "$ROOT_DIR/$plugin"
    destination="$ROOT_DIR/$plugin/bin"
    if [[ -L "$destination" || ( -e "$destination" && ! -d "$destination" ) ]]; then
        echo "Invalid plugin binary directory: $destination" >&2
        exit 1
    fi
    # TMPDIR may be on another filesystem; keep complete backups first.
    if [[ -d "$destination" ]]; then
        cp -a "$destination" "$STAGE/old-$plugin"
    fi
done
publishing=1
for plugin in vscode intellij; do
    destination="$ROOT_DIR/$plugin/bin"
    if [[ "$plugin" == vscode ]]; then
        vscode_touched=1
    else
        intellij_touched=1
    fi
    if [[ -d "$destination" ]]; then
        mv "$destination" "$STAGE/replaced-$plugin"
    fi
    mv "$STAGE/$plugin" "$destination"
done
committed=1

echo "All binaries built and placed in vscode/bin/ and intellij/bin/"
