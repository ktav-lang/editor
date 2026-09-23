>>>>> lang=en
## Features

- **Diagnostics** — every `did_open` / `did_change` re-parses the document
  and surfaces `ktav::Error::Syntax` messages on the offending line.
- **Hover** — hover on a `key:` line shows the inferred type and value.
- **Completion** — context-aware after a `:` separator: suggests `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), and the value markers (`:`, `::`).
- **Document symbols** — outline view reflects the parsed object tree;
  scalars become Property/Number/String, objects become Module, arrays
  become Array.
- **Semantic tokens** — token types `comment`, `keyword`, `number`,
  `string`, `property`, `operator`. Editors can use these instead of (or
  layered over) TextMate grammars for more accurate colouring,
  especially around dotted keys and typed-scalar markers.

>>>>> lang=ru
## Возможности

- **Диагностики** — каждый `did_open` / `did_change` перепарсивает
  документ и выводит сообщения `ktav::Error::Syntax` на нужной строке.
- **Hover** — наведение на строку `key:` показывает выведенный тип и
  значение.
- **Автодополнение** — контекстно после разделителя `:`: предлагает
  `null`, `true`, `false`, открывающие скобки (`{`, `[`, `(`, `((`),
  пустые литералы (`{}`, `[]`, `()`) и маркеры значений (`:`, `::`).
- **Document symbols** — outline отражает дерево распарсенного объекта;
  скаляры становятся Property/Number/String, объекты — Module, массивы —
  Array.
- **Semantic tokens** — типы токенов `comment`, `keyword`, `number`,
  `string`, `property`, `operator`. Редакторы могут использовать их
  вместо (или поверх) TextMate-грамматик для более точной подсветки,
  особенно вокруг точечных ключей и маркеров типизированных скаляров.

>>>>> lang=zh
## 功能

- **诊断**:每次 `did_open` / `did_change` 都会重新解析文档,并在出错
  行上显示 `ktav::Error::Syntax` 消息。
- **Hover**:在 `key:` 行悬停可显示推断的类型和值。
- **补全**:在 `:` 分隔符之后上下文感知补全:`null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)以及
  值标记(`:`、`::`)。
- **文档符号**:大纲视图反映已解析的对象树;标量为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens**:token 类型 `comment`、`keyword`、`number`、
  `string`、`property`、`operator`。编辑器可使用它们替代(或叠加于)
  TextMate 语法,尤其在点状键和类型标记附近获得更准确的着色。

