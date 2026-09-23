>>>>> lang=en
## [0.3.1] — 2026-05-10

Sync to `ktav 0.3.1` and spec `0.1.1`. Adds top-level Array support
in the document-symbols outline.

### Changed

- `build_symbols` now recognises a top-level Array root (per spec
  § 5.0.1) and renders its items as `[0]`, `[1]`, … outline entries.
  Object roots are unchanged.
- `Cargo.toml`: `ktav = "0.3.1"` (was `"0.3.0"`).

### Tests

- New: `document_symbols_built_from_top_level_array_of_scalars`,
  `document_symbols_top_level_array_of_objects_have_children` (in
  `tests/integration.rs`).
- New: `top_level_array_of_scalars_preserved`,
  `top_level_array_of_objects_preserved` (in
  `tests/format_pipeline.rs`).
- Updated: pre-existing `MissingSeparatorSpace` / `EmptyKey` /
  Cyrillic byte-column fixtures use a leading anchor pair so the
  malformed-pair branch is still exercised under spec 0.1.1.

>>>>> lang=ru
## [0.3.1] — 2026-05-10

Синхронизация с `ktav 0.3.1` и spec `0.1.1`. В outline символов
документа добавлена поддержка массива верхнего уровня.

### Изменено

- `build_symbols` теперь распознаёт корень-массив верхнего уровня
  (spec § 5.0.1) и отображает его элементы как записи outline `[0]`,
  `[1]`, … Корни-объекты не изменились.
- `Cargo.toml`: `ktav = "0.3.1"` (было `"0.3.0"`).

### Тесты

- Новые: `document_symbols_built_from_top_level_array_of_scalars`,
  `document_symbols_top_level_array_of_objects_have_children` (в
  `tests/integration.rs`).
- Новые: `top_level_array_of_scalars_preserved`,
  `top_level_array_of_objects_preserved` (в
  `tests/format_pipeline.rs`).
- Обновлены: существующие фикстуры `MissingSeparatorSpace` / `EmptyKey` /
  с кириллическими byte-колонками используют ведущую пару-анкер, чтобы
  ветка malformed-pair по-прежнему покрывалась при spec 0.1.1.

>>>>> lang=zh
## [0.3.1] — 2026-05-10

同步到 `ktav 0.3.1` 与 spec `0.1.1`。文档符号大纲新增顶层 Array 支持。

### 变更

- `build_symbols` 现在识别顶层 Array 根(spec § 5.0.1),并将其各项渲染为
  大纲条目 `[0]`、`[1]`、…。对象根不变。
- `Cargo.toml`:`ktav = "0.3.1"`(原为 `"0.3.0"`)。

### 测试

- 新增:`document_symbols_built_from_top_level_array_of_scalars`、
  `document_symbols_top_level_array_of_objects_have_children`(位于
  `tests/integration.rs`)。
- 新增:`top_level_array_of_scalars_preserved`、
  `top_level_array_of_objects_preserved`(位于
  `tests/format_pipeline.rs`)。
- 更新:既有 `MissingSeparatorSpace` / `EmptyKey` / 西里尔字节列固件改用
  前置锚定对,确保 malformed-pair 分支在 spec 0.1.1 下仍被覆盖。

