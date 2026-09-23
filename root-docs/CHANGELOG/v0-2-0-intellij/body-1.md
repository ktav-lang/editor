>>>>> lang=en
### IntelliJ plugin

- Replaced LSP4IJ dependency with an in-house JSON-RPC LSP client
  (no external plugin required).
- Native syntax highlighting via state-machine lexer:
  KEY / KEY_DOT / MARKER_INT / MARKER_FLOAT / DOUBLE_COLON / COLON /
  STRING_VALUE / INT_VALUE / FLOAT_VALUE / BOOLEAN / NULL /
  MULTILINE_OPEN/CLOSE / BRACES / BRACKETS / COMMENT.
- `KtavParserDefinition` (minimal flat parser) — gives PSI tree so
  `ExternalAnnotator` (for LSP diagnostics → tooltip + Problems View)
  works.
- `KtavFoldingBuilder` — folds `{}`, `[]`, `()`, `(())`.
- `KtavBraceMatcher` — paired-brace highlight + auto-close on type.
- `KtavUnicodeAnnotator` — boxed red highlight for non-ASCII chars
  inside keys (analog of VS Code's `editor.unicodeHighlight`).
- `KtavFormattingService` (AsyncDocumentFormattingService) hooks
  Ctrl+Alt+L into LSP `textDocument/formatting`.
- `KtavStartupActivity` syncs already-open `.ktav` files when the
  plugin loads dynamically.
- Bundled cross-platform `ktav-lsp` binaries (Windows x64).
- Plugin distribution ZIP correctly packages binaries via
  `_repackageWithBinaries` task.
- `pluginVerification` pinned to `IC-2024.3 / 2025.1 / 2025.2`
  (avoids `recommended()` failing on metadata-only future releases).
- `untilBuild = provider { null }` — no upper IDE-version pin in
  plugin.xml (was emitting `until-build=""` which the verifier
  rejected).
- Plugin version bumped 0.1.5 → 0.2.0.

>>>>> lang=ru
### IntelliJ plugin

- Зависимость LSP4IJ заменена на собственный JSON-RPC LSP-клиент
  (внешний плагин не требуется).
- Нативная подсветка синтаксиса на конечном автомате-лексере:
  KEY / KEY_DOT / MARKER_INT / MARKER_FLOAT / DOUBLE_COLON / COLON /
  STRING_VALUE / INT_VALUE / FLOAT_VALUE / BOOLEAN / NULL /
  MULTILINE_OPEN/CLOSE / BRACES / BRACKETS / COMMENT.
- `KtavParserDefinition` (минимальный плоский парсер) — даёт PSI-дерево,
  чтобы `ExternalAnnotator` (для диагностики LSP → тултип + Problems
  View) работал.
- `KtavFoldingBuilder` — сворачивает `{}`, `[]`, `()`, `(())`.
- `KtavBraceMatcher` — подсветка парных скобок + автозакрытие при
  вводе.
- `KtavUnicodeAnnotator` — красная рамка для не-ASCII символов внутри
  ключей (аналог `editor.unicodeHighlight` в VS Code).
- `KtavFormattingService` (AsyncDocumentFormattingService) вешает
  Ctrl+Alt+L на LSP `textDocument/formatting`.
- `KtavStartupActivity` синхронизирует уже открытые `.ktav` файлы при
  динамической загрузке плагина.
- В комплект входят кросс-платформенные бинарники `ktav-lsp`
  (Windows x64).
- Дистрибутивный ZIP плагина корректно упаковывает бинарники через
  задачу `_repackageWithBinaries`.
- `pluginVerification` закреплён на `IC-2024.3 / 2025.1 / 2025.2`
  (чтобы `recommended()` не падал на будущих релизах, где есть только
  метаданные).
- `untilBuild = provider { null }` — без верхней привязки версии IDE в
  plugin.xml (раньше выдавал `until-build=""`, который верификатор
  отвергал).
- Версия плагина поднята 0.1.5 → 0.2.0.

>>>>> lang=zh
### IntelliJ plugin

- 用自研的 JSON-RPC LSP 客户端取代 LSP4IJ 依赖(无需外部插件)。
- 基于状态机词法分析器的原生语法高亮:
  KEY / KEY_DOT / MARKER_INT / MARKER_FLOAT / DOUBLE_COLON / COLON /
  STRING_VALUE / INT_VALUE / FLOAT_VALUE / BOOLEAN / NULL /
  MULTILINE_OPEN/CLOSE / BRACES / BRACKETS / COMMENT。
- `KtavParserDefinition`(极简扁平解析器)—— 提供 PSI 树,使
  `ExternalAnnotator`(LSP 诊断 → 工具提示 + Problems View)得以工作。
- `KtavFoldingBuilder` —— 折叠 `{}`、`[]`、`()`、`(())`。
- `KtavBraceMatcher` —— 配对括号高亮 + 输入时自动闭合。
- `KtavUnicodeAnnotator` —— 对键中的非 ASCII 字符加红色方框高亮
  (类似于 VS Code 的 `editor.unicodeHighlight`)。
- `KtavFormattingService`(AsyncDocumentFormattingService)把
  Ctrl+Alt+L 挂到 LSP `textDocument/formatting`。
- `KtavStartupActivity` 在插件动态加载时同步已打开的 `.ktav` 文件。
- 捆绑跨平台 `ktav-lsp` 二进制文件(Windows x64)。
- 插件分发 ZIP 通过 `_repackageWithBinaries` 任务正确打包二进制文件。
- `pluginVerification` 锁定在 `IC-2024.3 / 2025.1 / 2025.2`(避免
  `recommended()` 在仅有元数据的未来版本上失败)。
- `untilBuild = provider { null }` —— plugin.xml 中不设 IDE 版本上限
  (此前会发出 `until-build=""`,被验证器拒绝)。
- 插件版本 0.1.5 → 0.2.0。

