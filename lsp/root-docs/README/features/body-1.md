>>>>> lang=en
## Features

- **Diagnostics** — every `did_open` / `did_change` re-parses the document
  and reports ktav's structured `ErrorKind`, mapped to a tight byte-span
  range on the offending line(s).
- **Hover** — hover on a `key:` line shows the inferred type and value.
- **Completion** — context-aware after a `:` separator: suggests `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), and the value markers (`:`, `::`).
- **Document symbols** — outline view reflects the parsed object tree;
  scalars become Property/Number/String, objects become Module, arrays
  become Array.
- **Semantic tokens** — token types `comment`, `keyword`, `number`,
  `string`, `property`, `operator`, `null`. Editors can use these instead of (or
  layered over) TextMate grammars for more accurate colouring,
  especially around dotted keys and `::` raw values.
- **Formatting** — `textDocument/formatting` canonically re-indents
  object/array/parenthesised nesting (4 spaces per level), preserving
  blank lines, comments and multi-line string block contents verbatim.

>>>>> lang=ru
## Возможности

- **Диагностики** — каждый `did_open` / `did_change` перепарсивает
  документ и выводит структурированный `ErrorKind` из `ktav`, привязанный
  к точному байтовому диапазону на нужной строке(ах).
- **Hover** — наведение на строку `key:` показывает выведенный тип и
  значение.
- **Автодополнение** — контекстно после разделителя `:`: предлагает
  `null`, `true`, `false`, открывающие скобки (`{`, `[`, `(`, `((`),
  пустые литералы (`{}`, `[]`, `()`) и маркеры значений (`:`, `::`).
- **Document symbols** — outline отражает дерево распарсенного объекта;
  скаляры становятся Property/Number/String, объекты — Module, массивы —
  Array.
- **Semantic tokens** — типы токенов `comment`, `keyword`, `number`,
  `string`, `property`, `operator`, `null`. Редакторы могут использовать их
  вместо (или поверх) TextMate-грамматик для более точной подсветки,
  особенно вокруг точечных ключей и сырых значений `::`.
- **Форматирование** — `textDocument/formatting` канонически
  переотступает вложенность объектов/массивов/скобочных групп (4
  пробела на уровень), сохраняя пустые строки, комментарии и содержимое
  многострочных строковых блоков без изменений.

>>>>> lang=zh
## 功能

- **诊断**:每次 `did_open` / `did_change` 都会重新解析文档,并报告
  `ktav` 的结构化 `ErrorKind`,精确映射到出错行(们)的字节区间。
- **Hover**:在 `key:` 行悬停可显示推断的类型和值。
- **补全**:在 `:` 分隔符之后上下文感知补全:`null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)以及
  值标记(`:`、`::`)。
- **文档符号**:大纲视图反映已解析的对象树;标量为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens**:token 类型 `comment`、`keyword`、`number`、
  `string`、`property`、`operator`、`null`。编辑器可使用它们替代(或叠加于)
  TextMate 语法,尤其在点状键和 `::` 原始值附近获得更准确的着色。
- **格式化**:`textDocument/formatting` 规范化对象/数组/括号分组的
  嵌套缩进(每级 4 个空格),完整保留空行、注释以及多行字符串块的
  内容。

