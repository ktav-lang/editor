#!/bin/bash
# dev-rebuild.sh - быстрая пересборка ktav-lsp + установка в WebStorm
#
# Используется для итеративной разработки ktav-lsp / ktav без публикации.
# Шаги:
#   1. cargo build --release в editor/lsp
#   2. Копирование собранного бинарника в plugin/lib/bin/{platform}/
#   3. Перезапуск WebStorm
#
# Usage:
#   ./dev-rebuild.sh         — пересобрать LSP + рестарт WebStorm
#   ./dev-rebuild.sh --no-restart  — только пересобрать, без рестарта
#   ./dev-rebuild.sh --plugin       — пересобрать ВСЁ (ktav-lsp + plugin) + рестарт

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LSP_DIR="$PROJECT_ROOT/editor/lsp"
PLUGIN_DIR="$SCRIPT_DIR"

REBUILD_PLUGIN=0
NO_RESTART=0
for arg in "$@"; do
    case "$arg" in
        --plugin) REBUILD_PLUGIN=1 ;;
        --no-restart) NO_RESTART=1 ;;
    esac
done

echo "===== 1. cargo build --release ktav-lsp ====="
cd "$LSP_DIR"
cargo build --release

LSP_BIN="$LSP_DIR/target/release/ktav-lsp.exe"
if [ ! -f "$LSP_BIN" ]; then
    LSP_BIN="$LSP_DIR/target/release/ktav-lsp"
fi
if [ ! -f "$LSP_BIN" ]; then
    echo "ERROR: ktav-lsp not found at $LSP_BIN"
    exit 1
fi
echo "✓ Built: $LSP_BIN ($(stat -c%s "$LSP_BIN" 2>/dev/null || stat -f%z "$LSP_BIN") bytes)"

echo ""
echo "===== 2. Копирую бинарник в plugin/bin/ ====="
mkdir -p "$PLUGIN_DIR/bin/win32-x64"
mkdir -p "$PLUGIN_DIR/bin/x86_64-pc-windows-msvc"
cp "$LSP_BIN" "$PLUGIN_DIR/bin/win32-x64/ktav-lsp.exe"
cp "$LSP_BIN" "$PLUGIN_DIR/bin/x86_64-pc-windows-msvc/ktav-lsp.exe"
echo "✓ Скопирован в plugin/bin/"

if [ $REBUILD_PLUGIN -eq 1 ]; then
    echo ""
    echo "===== 3a. Пересобираю IntelliJ плагин ====="
    cd "$PLUGIN_DIR"
    ./gradlew buildPlugin 2>&1 | grep -E "(error|FAILED|SUCCESSFUL|Repackaged|Archive)" | head -10
fi

# Закрываю WebStorm если запущен
if [ $NO_RESTART -eq 0 ]; then
    echo ""
    echo "===== Закрываю WebStorm ====="
    taskkill //F //IM "webstorm64.exe" 2>/dev/null && echo "✓ Закрыт" || echo "(не запущен)"
    sleep 2
fi

echo ""
echo "===== Установка в WebStorm ====="
WSPLUGINS="C:\Users\Computer\AppData\Roaming\JetBrains\WebStorm2025.3\plugins"
WSCACHE="C:\Users\Computer\AppData\Local\JetBrains\WebStorm2025.3"

if [ $REBUILD_PLUGIN -eq 1 ]; then
    # Полная переустановка плагина
    rm -rf "$WSPLUGINS/ktav-intellij" 2>/dev/null
    rm -f "$WSPLUGINS"/*.zip 2>/dev/null

    LATEST_ZIP=$(ls -t "$PLUGIN_DIR/build/distributions/ktav-intellij-"*.zip | head -1)
    cd "$WSPLUGINS"
    unzip -o "$LATEST_ZIP" > /dev/null
    echo "✓ Плагин переустановлен (версия: $(basename "$LATEST_ZIP"))"
else
    # Только обновляем бинарник (без пересборки плагина)
    if [ -d "$WSPLUGINS/ktav-intellij/lib/bin/win32-x64" ]; then
        cp "$LSP_BIN" "$WSPLUGINS/ktav-intellij/lib/bin/win32-x64/ktav-lsp.exe"
        cp "$LSP_BIN" "$WSPLUGINS/ktav-intellij/lib/bin/x86_64-pc-windows-msvc/ktav-lsp.exe"
        echo "✓ Бинарник обновлён в plugins/ktav-intellij/lib/bin/"
    else
        echo "WARNING: plugin не установлен в Roaming. Запусти с --plugin"
    fi
fi

# Очищаем кэши
rm -rf "$WSCACHE/system" 2>/dev/null
rm -rf "$WSCACHE/caches" 2>/dev/null
mv "$WSCACHE/log/idea.log" "$WSCACHE/log/idea.log.bak" 2>/dev/null
echo "✓ Кэши очищены, лог сохранён в idea.log.bak"

if [ $NO_RESTART -eq 0 ]; then
    echo ""
    echo "===== Запускаю WebStorm ====="
    "C:\Program Files\JetBrains\WebStorm 2025.3.4\bin\webstorm64.exe" >/dev/null 2>&1 &
    disown
    echo "✓ WebStorm запущен"
fi

echo ""
echo "===== Готово ====="
echo "Жди ~30 сек пока WebStorm полностью загрузится"
echo "Логи: $WSCACHE/log/idea.log"
