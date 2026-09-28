>>>>> lang=en
## [0.8.0] — 2026-09-28

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- Exact § 3.6 / § 5.2 scalar classification (`01234`, `0_7`, `1_`,
  `2026-09-28` are Strings) and the exact § 3.3 whitespace set.
- Inline compounds: `::` values stay raw Strings; quoted key segments
  are opaque.
- Hover no longer crashes on long non-ASCII strings and resolves nested,
  quoted and escaped keys.
- The spec conformance test covers every corpus category and fails
  without the submodule.

>>>>> lang=ru
## [0.8.0] — 2026-09-28

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — полный
список см. в корневом [`CHANGELOG.md`](../../CHANGELOG.md) (раздел 0.8.0).

- Точная классификация скаляров по § 3.6 / § 5.2 (`01234`, `0_7`, `1_`,
  `2026-09-28` — строки) и точный набор пробелов § 3.3.
- Inline-структуры: значения `::` остаются сырыми строками; сегменты
  ключей в кавычках непрозрачны.
- Hover больше не падает на длинных не-ASCII строках и находит
  вложенные ключи, ключи в кавычках и с экранированием.
- Тест соответствия спецификации покрывает все категории корпуса и
  падает без подмодуля.

>>>>> lang=zh
## [0.8.0] — 2026-09-28

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- 按 § 3.6 / § 5.2 精确分类标量(`01234`、`0_7`、`1_`、`2026-09-28`
  为字符串),并使用 § 3.3 精确的空白集合。
- 内联复合值:`::` 之后的值保持为原始字符串;带引号的键片段不透明。
- Hover 不再因长的非 ASCII 字符串而崩溃,并能解析嵌套键、带引号和带转义的键。
- 规范一致性测试覆盖语料库的所有类别,缺少子模块时会失败。

