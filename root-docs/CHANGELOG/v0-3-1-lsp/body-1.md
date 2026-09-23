>>>>> lang=en
### LSP server (`ktav-lsp`)

- **Top-level Array** (spec § 5.0.1) is now first-class in
  `build_symbols`: when the parsed root is an Array, items render as
  `[0]`, `[1]`, … entries in the document outline (mirroring how
  nested arrays already render). For object items the symbol's range
  points at the line of the item's first key; for bare-scalar items
  the range covers the item's own line.
- `reindent` is verified to preserve bare top-level Array form — the
  formatter does not synthesise `[ ... ]` brackets at the document
  root. Pinned by new test cases in `tests/format_pipeline.rs`.
- The 0.3.0 `name: (value)` → `name:: (value)` auto-disambiguation in
  `reindent` is preserved.
- Existing diagnostic / pinning tests updated to use a leading anchor
  pair (`anchor: 1\n…`) for inputs that were previously
  bare-pair-shaped at the document start — under spec 0.1.1 those
  bare lines now parse as top-level Array string items, so the
  fixtures are anchored explicitly to keep the malformed-pair branch
  exercised.

>>>>> lang=ru
### LSP server (`ktav-lsp`)

- **Массив верхнего уровня** (spec § 5.0.1) теперь стал полноправным
  в `build_symbols`: когда корень разобран как массив, элементы
  отображаются записями `[0]`, `[1]`, … в структуре документа (так же,
  как уже отображаются вложенные массивы). Для элементов-объектов
  диапазон символа указывает на строку первого ключа элемента; для
  голых скаляров диапазон покрывает собственную строку элемента.
- Проверено, что `reindent` сохраняет голую форму массива верхнего
  уровня — форматтер не синтезирует скобки `[ ... ]` в корне
  документа. Зафиксировано новыми тест-кейсами в
  `tests/format_pipeline.rs`.
- Автоматическая дезамбигуация 0.3.0 `name: (value)` →
  `name:: (value)` в `reindent` сохранена.
- Существующие тесты диагностики и закрепления обновлены: входы,
  которые ранее имели форму голой пары в начале документа, теперь
  получают ведущую пару-якорь (`anchor: 1\n…`) — по spec 0.1.1 эти
  голые строки разбираются как строковые элементы массива верхнего
  уровня, поэтому фикстуры явно заякорены, чтобы ветка malformed-pair
  оставалась покрытой.

>>>>> lang=zh
### LSP server (`ktav-lsp`)

- **顶层数组**(spec § 5.0.1)现在在 `build_symbols` 中是一等公民:
  当解析根节点为数组时,各元素在大纲中渲染为 `[0]`、`[1]`、…… 条目
  (与嵌套数组既有的渲染方式一致)。对象元素的符号范围指向该元素第一
  个键所在行;裸标量元素的范围则覆盖该元素自身所在行。
- 已验证 `reindent` 能保留裸顶层数组形态 —— 格式化器不会在文档根部
  凭空合成 `[ ... ]` 方括号。由 `tests/format_pipeline.rs` 中的新测试
  用例锁定。
- 0.3.0 中 `name: (value)` → `name:: (value)` 的自动消歧在
  `reindent` 中得以保留。
- 现有诊断/锁定测试已更新为使用前置锚定对(`anchor: 1\n…`)—— 针对
  那些此前在文档开头呈裸键值形态的输入:在 spec 0.1.1 下,这些裸行
  现在会解析为顶层数组的字符串条目,因此夹具被显式锚定,以保持
  malformed-pair 分支仍然被覆盖。

