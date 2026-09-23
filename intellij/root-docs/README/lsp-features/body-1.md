>>>>> lang=en
### LSP features (optional)

When the [LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij)
plugin is installed alongside Ktav, you get live diagnostics, hover,
completion, document symbols, and semantic tokens served by
[`ktav-lsp`](../lsp). Without LSP4IJ the plugin still works in
TextMate-only mode — install LSP4IJ from the Marketplace whenever you
want the richer features.

The server binary is discovered in this order:

1. The explicit path configured under **Settings → Tools → Ktav**.
2. A binary bundled inside the plugin distribution at
   `bin/<platform>-<arch>/ktav-lsp` (not bundled in the current
   release).
3. `ktav-lsp` resolved via your shell `PATH` — install it with
   `cargo install ktav-lsp` (matches the VS Code extension's
   discovery order).

>>>>> lang=ru
### LSP-фичи (опционально)

Если вместе с Ktav установлен плагин
[LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij), вы
получаете live-диагностику, hover, автокомплит, document symbols и
semantic tokens от [`ktav-lsp`](../../lsp). Без LSP4IJ плагин всё
равно работает в режиме TextMate-only — ставьте LSP4IJ из Marketplace,
когда захотите richer features.

Бинарник сервера ищется в таком порядке:

1. Явный путь, заданный в **Settings → Tools → Ktav**.
2. Бинарник, вложенный в дистрибутив плагина по пути
   `bin/<platform>-<arch>/ktav-lsp` (в текущем релизе не вложен).
3. `ktav-lsp`, найденный через `PATH` вашей оболочки — установите
   его командой `cargo install ktav-lsp` (совпадает с порядком
   поиска в расширении VS Code).

>>>>> lang=zh
### LSP 功能(可选)

当与 Ktav 一同安装了
[LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) 插件时,
即可获得由 [`ktav-lsp`](../../lsp) 提供的实时诊断、悬停、补全、
document symbols 和 semantic tokens。未安装 LSP4IJ 时,本插件仍以
TextMate-only 模式正常工作 —— 需要更丰富的功能时再从 Marketplace
安装 LSP4IJ 即可。

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 打包在插件分发包中的二进制
   `bin/<platform>-<arch>/ktav-lsp`(当前版本未打包)。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp` 安装(与 VS Code 扩展的查找顺序一致)。

