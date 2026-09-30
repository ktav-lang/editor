>>>>> lang=en
## Known limitations

- Static syntax highlighting only. The grammar does not enforce
  semantic rules (duplicate-name detection, path conflicts, numeric
  validity beyond the regex shape, dotted-key expansion, empty-key
  checks). Those belong to a parser/linter.
- Inline `#` is *not* a comment per the spec, and the grammar honors
  that. A `#` inside a value body is highlighted as part of the string.
- Multi-line string content has one string scope; lines such as
  `key: value` are not pairs. A line trimming to exactly `)` closes
  stripped form, and one trimming to exactly `))` closes verbatim
  form (§ 5.6.1). To include one colliding closer as content, switch
  forms; other canonical representability checks still apply.
  If a String has both kinds of colliding line, the canonical
  multi-line writer must reject it with `BothFormsRequired` (§ 5.9.7).
  Adjacent blocks do not concatenate into one String. This is a
  canonical-output restriction, not a ban on every parseable spelling:
  the inline document `{s: ))\n)}` yields one String containing both
  lines, but has no canonical multi-line representation.
- Key dots follow § 4 and § 5.3.2–5.3.3: an unescaped `.` outside a
  quoted segment is a path separator. A literal dot in a bare segment
  requires `\.`; inside a quoted segment it is ordinary content.
  `example.com: 1` is a dotted path; `example\.com: 1` and
  `"example.com": 1` each use one literal key. `x.y\.z: v` has the
  path segments `x` and `y.z`. Regex highlighting is not validation
  of these paths' semantic effects.
>>>>> lang=ru
## Известные ограничения

- Только статическая подсветка синтаксиса. Грамматика не проверяет
  семантические правила (обнаружение повторяющихся имён, конфликты путей,
  проверку чисел сверх формы regex, раскрытие точечных ключей, проверку
  пустых ключей). Это задача парсера/линтера.
- Встроенный `#` по спецификации *не* является комментарием, и грамматика
  это соблюдает. `#` внутри тела значения подсвечивается как часть строки.
- Содержимое многострочной строки имеет один строковый scope; строки
  вида `key: value` не являются парами. Строка, обрезающаяся до `)`,
  закрывает stripped-форму, а до `))` — verbatim-форму (§ 5.6.1).
  Чтобы включить один конфликтующий closer в содержимое, смените форму;
  остальные проверки канонической представимости по-прежнему действуют.
  Если String содержит оба вида конфликтующих строк, канонический
  multiline writer должен отклонить её с `BothFormsRequired` (§ 5.9.7).
  Соседние блоки не объединяются в одну String. Это ограничение
  канонического вывода, а не запрет всех разбираемых записей:
  inline-документ `{s: ))\n)}` даёт одну String с обеими строками,
  но не имеет канонического многострочного представления.
- Точки в ключах следуют § 4 и § 5.3.2–5.3.3: неэкранированная `.`
  вне сегмента в кавычках — разделитель пути. Для буквальной точки
  в некавыченном сегменте нужна `\.`; в сегменте в кавычках она является
  содержимым. `example.com: 1` задаёт точечный путь; `example\.com: 1`
  и `"example.com": 1` используют один буквальный ключ. `x.y\.z: v`
  задаёт сегменты пути `x` и `y.z`. Regex-подсветка не проверяет
  семантические последствия этих путей.
>>>>> lang=zh
## 已知限制

- 仅提供静态语法高亮。语法不强制语义规则(重复名称检测、路径冲突、
  超出 regex 形式的数字有效性校验、点号键展开、空键检查)。这些属于
  解析器/linter 的职责。
- 按规范,行内 `#` *不是*注释,语法也遵循这一点。值主体中的 `#` 作为
  字符串的一部分高亮。
- 多行字符串内容使用一个字符串 scope;形如 `key: value` 的行不是
  键值对。修剪后恰为 `)` 的行关闭 stripped 形式,恰为 `))` 的行关闭
  verbatim 形式(§ 5.6.1)。要将一种冲突的闭合符作为内容,需切换形式;
  其他规范可表示性检查仍然适用。若同一 String 含两种冲突行,
  规范多行 writer 必须以 `BothFormsRequired` 拒绝它(§ 5.9.7)。
  相邻块不会拼接成一个 String。这是规范输出的限制,并非禁止所有
  可解析写法:inline 文档 `{s: ))\n)}` 产生含两种行的一个 String,
  但不存在规范多行表示。
- 键中的点遵循 § 4 和 § 5.3.2–5.3.3:引号片段外未转义的 `.`
  是路径分隔符。裸片段中的字面点需要 `\.`;引号片段中的点则是普通
  内容。`example.com: 1` 是点分路径;`example\.com: 1` 和
  `"example.com": 1` 都使用一个字面键。`x.y\.z: v` 的路径片段为
  `x` 和 `y.z`。Regex 高亮不会验证这些路径的语义效果。
