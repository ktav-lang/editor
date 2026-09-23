>>>>> lang=en
## Token classes

The grammar tags spans with these scope names. Themes that style
these scopes will style Ktav consistently.

| Scope                                              | Matches                                    |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `## …` line comments                       |
| `entity.name.tag.ktav`                             | Key segments (left of `:`)                 |
| `punctuation.accessor.dot.ktav`                    | `.` separating dotted key segments         |
| `punctuation.separator.key-value.ktav`             | The `:` of a plain pair                    |
| `keyword.operator.marker.raw.ktav`                 | `::` (raw-string marker)                   |
| `constant.language.ktav`                           | `null`, `true`, `false` scalars            |
| `constant.numeric.integer.ktav`                    | Bare integer scalar (digits only)          |
| `constant.numeric.float.ktav`                      | Bare decimal scalar (has `.` / exponent)   |
| `string.unquoted.ktav`                             | Ordinary string scalars                    |
| `string.unquoted.raw.ktav`                         | Body after `::`                            |
| `string.quoted.multiline.stripped.ktav`            | Content inside `( … )`                     |
| `string.quoted.multiline.verbatim.ktav`            | Content inside `(( … ))`                   |
| `punctuation.section.braces.begin.ktav`            | `{`                                        |
| `punctuation.section.braces.end.ktav`              | `}`                                        |
| `punctuation.section.brackets.begin.ktav`          | `[`                                        |
| `punctuation.section.brackets.end.ktav`            | `]`                                        |
| `punctuation.section.parens.begin.ktav`            | `(`, `((`                                  |
| `punctuation.section.parens.end.ktav`              | `)`, `))`                                  |

>>>>> lang=ru
## Классы токенов

Грамматика помечает фрагменты следующими именами scope. Темы, которые
стилизуют эти scope, будут единообразно стилизовать Ktav.

| Scope                                              | Что совпадает                              |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | Строчные комментарии `## …`                |
| `entity.name.tag.ktav`                             | Сегменты ключа (слева от `:`)              |
| `punctuation.accessor.dot.ktav`                    | `.`, разделяющая сегменты точечного ключа  |
| `punctuation.separator.key-value.ktav`             | `:` обычной пары                           |
| `keyword.operator.marker.raw.ktav`                 | `::` (маркер raw-строки)                   |
| `constant.language.ktav`                           | Скаляры `null`, `true`, `false`            |
| `constant.numeric.integer.ktav`                    | Голый целочисленный скаляр (только цифры)  |
| `constant.numeric.float.ktav`                      | Голый десятичный скаляр (есть `.` / экспонента) |
| `string.unquoted.ktav`                             | Обычные строковые скаляры                  |
| `string.unquoted.raw.ktav`                         | Тело после `::`                            |
| `string.quoted.multiline.stripped.ktav`            | Содержимое внутри `( … )`                  |
| `string.quoted.multiline.verbatim.ktav`            | Содержимое внутри `(( … ))`                |
| `punctuation.section.braces.begin.ktav`            | `{`                                        |
| `punctuation.section.braces.end.ktav`              | `}`                                        |
| `punctuation.section.brackets.begin.ktav`          | `[`                                        |
| `punctuation.section.brackets.end.ktav`            | `]`                                        |
| `punctuation.section.parens.begin.ktav`            | `(`, `((`                                  |
| `punctuation.section.parens.end.ktav`              | `)`, `))`                                  |

>>>>> lang=zh
## Token 类别

语法用以下 scope 名称标记文本片段。为这些 scope 设置样式的主题会一致地
渲染 Ktav。

| Scope                                              | 匹配内容                                   |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `## …` 行注释                              |
| `entity.name.tag.ktav`                             | 键段(`:` 左侧)                           |
| `punctuation.accessor.dot.ktav`                    | 分隔点号键段的 `.`                         |
| `punctuation.separator.key-value.ktav`             | 普通键值对的 `:`                           |
| `keyword.operator.marker.raw.ktav`                 | `::`(原始字符串标记)                     |
| `constant.language.ktav`                           | `null`、`true`、`false` 标量               |
| `constant.numeric.integer.ktav`                    | 裸整数标量(仅数字)                       |
| `constant.numeric.float.ktav`                      | 裸小数标量(含 `.` / 指数)                |
| `string.unquoted.ktav`                             | 普通字符串标量                             |
| `string.unquoted.raw.ktav`                         | `::` 之后的主体                            |
| `string.quoted.multiline.stripped.ktav`            | `( … )` 内的内容                           |
| `string.quoted.multiline.verbatim.ktav`            | `(( … ))` 内的内容                         |
| `punctuation.section.braces.begin.ktav`            | `{`                                        |
| `punctuation.section.braces.end.ktav`              | `}`                                        |
| `punctuation.section.brackets.begin.ktav`          | `[`                                        |
| `punctuation.section.brackets.end.ktav`            | `]`                                        |
| `punctuation.section.parens.begin.ktav`            | `(`, `((`                                  |
| `punctuation.section.parens.end.ktav`              | `)`, `))`                                  |

