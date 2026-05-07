#!/bin/bash
# Build ktav-lsp binaries for all platforms and place in plugin dirs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LSP_DIR="$SCRIPT_DIR/lsp"
VSCODE_BIN="$SCRIPT_DIR/vscode/bin"
INTELLIJ_BIN="$SCRIPT_DIR/intellij/bin"

# Platforms to build for
TARGETS=(
    "x86_64-pc-windows-msvc:win32-x64:ktav-lsp.exe"
    "aarch64-pc-windows-msvc:win32-arm64:ktav-lsp.exe"
    "x86_64-unknown-linux-gnu:linux-x64:ktav-lsp"
    "aarch64-unknown-linux-gnu:linux-arm64:ktav-lsp"
    "x86_64-apple-darwin:darwin-x64:ktav-lsp"
    "aarch64-apple-darwin:darwin-arm64:ktav-lsp"
)

echo "🔨 Building ktav-lsp for all platforms..."
cd "$LSP_DIR"

for target_spec in "${TARGETS[@]}"; do
    IFS=: read -r cargo_target plugin_dir bin_name <<< "$target_spec"

    echo ""
    echo "📦 Building for $cargo_target → $plugin_dir..."

    # Determine if we need cargo cross (non-native targets)
    if [[ "$cargo_target" == *"linux"* ]] || [[ "$cargo_target" == *"darwin"* ]]; then
        cross build --release --target "$cargo_target" 2>&1 | grep -E "Finished|error" || true
    else
        cargo build --release --target "$cargo_target" 2>&1 | grep -E "Finished|error" || true
    fi

    # Copy to plugin directories
    source_bin="target/$cargo_target/release/$bin_name"
    if [ -f "$source_bin" ]; then
        mkdir -p "$VSCODE_BIN/$plugin_dir" "$INTELLIJ_BIN/$plugin_dir"
        cp "$source_bin" "$VSCODE_BIN/$plugin_dir/"
        cp "$source_bin" "$INTELLIJ_BIN/$plugin_dir/"
        echo "✓ Copied to bin/$plugin_dir/"
    else
        echo "✗ Binary not found: $source_bin"
        exit 1
    fi
done

echo ""
echo "✅ All binaries built and placed in vscode/bin/ and intellij/bin/"
echo ""
echo "📊 Structure:"
ls -lhR "$VSCODE_BIN"
