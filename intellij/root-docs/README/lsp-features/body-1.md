>>>>> lang=en
### LSP features

The plugin talks to [`ktav-lsp`](../lsp) through its own built-in LSP
client — no separate LSP plugin (e.g. LSP4IJ) is required. It gives you
live diagnostics, hover, completion, document symbols, semantic tokens
and formatting out of the box.

The server binary is discovered in this order:

1. The explicit path configured under **Settings → Tools → Ktav**.
2. The binary bundled inside the plugin distribution at
   `lib/bin/<platform>-<arch>/ktav-lsp`, one per supported platform.
3. `ktav-lsp` resolved via your shell `PATH` — install it with
   `cargo install ktav-lsp` (matches the VS Code extension's
   discovery order).

>>>>> lang=ru
### LSP-фичи

Плагин общается с [`ktav-lsp`](../../lsp) через собственный встроенный
LSP-клиент — отдельный LSP-плагин (например, LSP4IJ) не требуется. Из
коробки доступны live-диагностика, hover, автокомплит, document
symbols, semantic tokens и форматирование.

Бинарник сервера ищется в таком порядке:

1. Явный путь, заданный в **Settings → Tools → Ktav**.
2. Бинарник, вложенный в дистрибутив плагина по пути
   `lib/bin/<platform>-<arch>/ktav-lsp` — по одному на каждую
   поддерживаемую платформу.
3. `ktav-lsp`, найденный через `PATH` вашей оболочки — установите
   его командой `cargo install ktav-lsp` (совпадает с порядком
   поиска в расширении VS Code).

>>>>> lang=zh
### LSP 功能

插件通过自带的内置 LSP 客户端与 [`ktav-lsp`](../../lsp) 通信 ——
无需安装单独的 LSP 插件(例如 LSP4IJ)。开箱即用地提供实时诊断、
悬停、补全、document symbols、semantic tokens 和格式化。

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 打包在插件分发包中的二进制
   `lib/bin/<platform>-<arch>/ktav-lsp`,每个受支持平台各一份。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp` 安装(与 VS Code 扩展的查找顺序一致)。

