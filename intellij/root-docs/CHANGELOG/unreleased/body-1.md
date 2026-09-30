>>>>> lang=en
## [0.8.0] — 2026-09-30

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- Incremental highlighting preserves root/container context and whole Array
  Strings, including lone paren closers and multiword bare keys with quotes.
- Folding uses lexer tokens, keeping raw values and multiline bodies opaque.
- Diagnostics are project/client/version/session-owned; delayed publications
  and other project closures cannot overwrite current results. Split-editor
  highlighters are cleared through each actual owning model.
- Highlighting lexer: exact § 3.6 / § 5.2 numbers (ASCII digits only),
  § 3.3 whitespace, quoted key segments, raw `::` values inside inline
  compounds.
- The stale bundled `ktav-lsp.exe` copy (built from `ktav` 0.1.5) under
  `src/main/resources/bin/` is removed; the plugin never used it.
- Marketplace/Settings descriptions no longer say `#` for comment
  toggle (it's `##`) or claim LSP4IJ / a non-bundled binary are
  needed — the plugin has its own built-in LSP client and bundles a
  per-platform `ktav-lsp` binary.

>>>>> lang=ru
## [0.8.0] — 2026-09-30

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — полный
список см. в корневом [`CHANGELOG.md`](../../CHANGELOG.md) (раздел 0.8.0).

- Инкрементальная подсветка сохраняет контекст корня/контейнеров и целые
  строки Array, включая отдельные закрывающие парены и многословные
  некавыченные ключи с кавычками.
- Folding использует токены лексера; raw-значения и многострочные тела непрозрачны.
- Диагностика привязана к проекту/клиенту/версии/сессии: задержанные публикации
  и закрытие другого проекта не затирают текущий результат. Highlighter
  каждого split-editor удаляется через его реальный MarkupModel.
- Лексер подсветки: точные числа § 3.6 / § 5.2 (только ASCII-цифры),
  пробелы § 3.3, сегменты ключей в кавычках, сырые значения `::` внутри
  inline-структур.
- Устаревшая вложенная копия `ktav-lsp.exe` (собрана из `ktav` 0.1.5) в
  `src/main/resources/bin/` удалена; плагин её не использовал.
- В текстах Marketplace/Settings больше не упоминается `#` для
  переключения комментария (это `##`) и не утверждается, что нужен
  LSP4IJ или что бинарник не вложен — у плагина свой встроенный
  LSP-клиент, а бинарник `ktav-lsp` вложен под каждую платформу.

>>>>> lang=zh
## [0.8.0] — 2026-09-30

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- 增量高亮保留根/容器上下文及完整 Array 字符串，包括独立的右圆括号和
  含引号的多词裸键。
- Folding 使用词法标记，raw 值及多行正文保持不透明。
- 诊断绑定项目/客户端/版本/会话，延迟发布及其他项目关闭不会覆盖当前
  结果；每个 split-editor 的 highlighter 通过实际所属 MarkupModel 清除。
- 高亮词法分析器:精确的 § 3.6 / § 5.2 数字(仅 ASCII 数字)、§ 3.3 空白、
  带引号的键片段、内联复合值中的原始 `::` 值。
- 移除 `src/main/resources/bin/` 下过时的内置 `ktav-lsp.exe`(由 `ktav`
  0.1.5 构建);插件从未使用它。
- Marketplace/Settings 文案不再将注释切换写成 `#`(应为 `##`),也不再
  声称需要 LSP4IJ 或未内置二进制文件 —— 插件拥有自己内置的 LSP 客户端,
  并为每个平台内置了 `ktav-lsp` 二进制文件。

