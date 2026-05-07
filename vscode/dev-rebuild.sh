#!/bin/bash
# dev-rebuild.sh - быстрая пересборка ktav-lsp + VSCode/VSCodium плагина
#
# Шаги:
#   1. cargo build --release ktav-lsp
#   2. Копирование бинарника в editor/vscode/bin/win32-x64/
#   3. npm run compile + vsce package → ktav-X.Y.Z.vsix
#   4. codium --install-extension <vsix> --force
#   5. (опц.) Перезапуск VSCodium

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LSP_DIR="$PROJECT_ROOT/editor/lsp"
VSC_DIR="$SCRIPT_DIR"

NO_RESTART=0
for arg in "$@"; do
    case "$arg" in
        --no-restart) NO_RESTART=1 ;;
    esac
done

echo "===== 1. cargo build --release ktav-lsp ====="
cd "$LSP_DIR"
cargo build --release

LSP_BIN="$LSP_DIR/target/release/ktav-lsp.exe"
[ -f "$LSP_BIN" ] || LSP_BIN="$LSP_DIR/target/release/ktav-lsp"
echo "✓ Built: $LSP_BIN"

echo ""
echo "===== 2. Копирую бинарник в vscode/bin/ ====="
mkdir -p "$VSC_DIR/bin/win32-x64"
cp "$LSP_BIN" "$VSC_DIR/bin/win32-x64/ktav-lsp.exe"
echo "✓ Скопирован"

echo ""
echo "===== 3. npm install + compile + vsce package ====="
cd "$VSC_DIR"

# Install runtime deps (vscode-languageclient et al.) so vsce bundles them.
# We need a real node_modules tree — without it the extension throws
# "Cannot find module 'vscode-languageclient/node'" at activation.
if [ ! -d "node_modules/vscode-languageclient" ]; then
    npm install 2>&1 | tail -3
fi

npm run compile 2>&1 | tail -3

# Pack with deps (default). NOT --no-dependencies, otherwise the
# extension can't resolve runtime imports.
npx vsce package 2>&1 | tail -5
LATEST_VSIX=$(ls -t ktav-*.vsix | head -1)
echo "✓ VSIX: $LATEST_VSIX"

# Закроем VSCodium перед установкой
if [ $NO_RESTART -eq 0 ]; then
    echo ""
    echo "===== Закрываю VSCodium ====="
    taskkill //F //IM "Code - OSS.exe" 2>/dev/null && echo "✓ Закрыт" || echo "(не запущен)"
    sleep 2
fi

echo ""
echo "===== 4. Установка в VSCodium ====="
codium --install-extension "$VSC_DIR/$LATEST_VSIX" --force 2>&1 | tail -5

# Удалить старую версию (0.1.0) если осталась
OLD_DIR=$(ls -d ~/.vscode-oss/extensions/ktav-lang.ktav-* 2>/dev/null | grep -v "0.1.5" | head -1 || true)
if [ -n "$OLD_DIR" ]; then
    rm -rf "$OLD_DIR"
    echo "✓ Удалена старая версия: $(basename "$OLD_DIR")"
fi

if [ $NO_RESTART -eq 0 ]; then
    echo ""
    echo "===== Запускаю VSCodium ====="
    codium >/dev/null 2>&1 &
    disown
    echo "✓ VSCodium запущен"
fi

echo ""
echo "===== Готово ====="
echo "Версия: $LATEST_VSIX"
