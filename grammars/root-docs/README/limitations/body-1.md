>>>>> lang=en
## Known limitations

- Static syntax highlighting only. The grammar does not enforce
  semantic rules (duplicate-name detection, path conflicts, typed
  scalar body validation beyond the regex shape, dotted-key expansion,
  empty-key checks). Those belong to a parser/linter.
- Inline `#` is *not* a comment per the spec, and the grammar honors
  that. A `#` inside a value body is highlighted as part of the string.
- Inside multi-line strings, lines that look like `key: value` are
  **not** parsed as pairs — the `contentName` covers the whole region
  with one string scope. This matches the spec but means a misplaced
  `)` / `))` (one that isn't actually the closer the spec considers
  it) may visually break out of the string region. Authors who need
  to embed both `)` and `))` lines in the same value must split the
  string per § 5.6.1.
- The grammar uses regex-based line classification, not a real parser.
  Pathological inputs (e.g. a key segment with mid-segment `.` that
  would, by the strict grammar, not be a separator) are tokenized
  as if every `.` were a separator. This matches every implementation
  in the wild and is the only reasonable visual behavior.
>>>>> lang=ru
## Известные ограничения

- Только статическая подсветка синтаксиса. Грамматика не проверяет
  семантические правила (обнаружение повторяющихся имён, конфликты путей,
  проверку тела типизированного скаляра сверх формы regex, раскрытие
  точечных ключей, проверку пустых ключей). Это задача парсера/линтера.
- Встроенный `#` по спецификации *не* является комментарием, и грамматика
  это соблюдает. `#` внутри тела значения подсвечивается как часть строки.
- Внутри многострочных строк строки вида `key: value` **не** разбираются
  как пары — `contentName` покрывает всю область одним строковым scope.
  Это соответствует спецификации, но означает, что неуместная `)` / `))`
  (которую спецификация на самом деле не считает закрывающей) может
  визуально вывести подсветку за пределы строковой области. Авторам,
  которым нужно поместить в одно значение и строки `)`, и строки `))`,
  следует разделить строку согласно § 5.6.1.
- Грамматика классифицирует строки с помощью regex, а не настоящего
  парсера. Патологические входные данные (например, сегмент ключа с `.`
  в середине, которая по строгой грамматике не является разделителем)
  токенизируются так, будто каждая `.` — разделитель. Так ведут себя все
  реализации на практике, и это единственное разумное визуальное
  поведение.
>>>>> lang=zh
## 已知限制

- 仅提供静态语法高亮。语法不强制语义规则(重复名称检测、路径冲突、
  超出 regex 形式的类型化标量主体校验、点号键展开、空键检查)。这些属于
  解析器/linter 的职责。
- 按规范,行内 `#` *不是*注释,语法也遵循这一点。值主体中的 `#` 作为
  字符串的一部分高亮。
- 在多行字符串内部,形如 `key: value` 的行**不会**被解析为键值对——
  `contentName` 用一个字符串 scope 覆盖整个区域。这与规范一致,但意味着
  位置不当的 `)` / `))`(规范实际上不视其为闭合符)可能在视觉上跳出
  字符串区域。需要在同一个值中同时嵌入 `)` 行和 `))` 行的作者,必须按
  § 5.6.1 拆分字符串。
- 语法使用基于 regex 的行分类,而不是真正的解析器。病态输入(例如键段
  中间含有 `.`,按严格语法它并非分隔符)会被当作每个 `.` 都是分隔符来
  切分 token。这与现有所有实现一致,也是唯一合理的视觉行为。
