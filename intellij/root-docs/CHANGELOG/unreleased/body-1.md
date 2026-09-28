>>>>> lang=en
## [0.8.0] — 2026-09-28

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- Highlighting lexer: exact § 3.6 / § 5.2 numbers (ASCII digits only),
  § 3.3 whitespace, quoted key segments, raw `::` values inside inline
  compounds.
- The stale bundled `ktav-lsp.exe` copy (built from `ktav` 0.1.5) under
  `src/main/resources/bin/` is removed; the plugin never used it.

>>>>> lang=ru
## [0.8.0] — 2026-09-28

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — полный
список см. в корневом [`CHANGELOG.md`](../../CHANGELOG.md) (раздел 0.8.0).

- Лексер подсветки: точные числа § 3.6 / § 5.2 (только ASCII-цифры),
  пробелы § 3.3, сегменты ключей в кавычках, сырые значения `::` внутри
  inline-структур.
- Устаревшая вложенная копия `ktav-lsp.exe` (собрана из `ktav` 0.1.5) в
  `src/main/resources/bin/` удалена; плагин её не использовал.

>>>>> lang=zh
## [0.8.0] — 2026-09-28

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- 高亮词法分析器:精确的 § 3.6 / § 5.2 数字(仅 ASCII 数字)、§ 3.3 空白、
  带引号的键片段、内联复合值中的原始 `::` 值。
- 移除 `src/main/resources/bin/` 下过时的内置 `ktav-lsp.exe`(由 `ktav`
  0.1.5 构建);插件从未使用它。

