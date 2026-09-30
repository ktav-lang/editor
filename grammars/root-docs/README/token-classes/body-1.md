>>>>> lang=en
## Token classes

The grammar tags spans with these scope names. Themes that style
these scopes will style Ktav consistently.

| Scope                                              | Matches                                    |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `## …` line comments                       |
| `entity.name.tag.ktav`                             | Bare key segments (left of `:`)            |
| `string.quoted.double.key.ktav`                    | Double-quoted key segments                 |
| `string.quoted.single.key.ktav`                    | Single-quoted key segments                 |
| `string.quoted.backtick.key.ktav`                  | Backtick-quoted key segments               |
| `punctuation.accessor.dot.ktav`                    | `.` separating dotted key segments         |
| `punctuation.separator.key-value.ktav`             | The `:` of a plain pair                    |
| `keyword.operator.marker.raw.ktav`                 | `::` (raw-string marker)                   |
| `constant.language.boolean.ktav`                   | `true`, `false` scalars                    |
| `constant.language.null.ktav`                      | `null` scalars                             |
| `constant.numeric.integer.ktav`                    | Integer literal (§ 3.6; no redundant leading zero) |
| `constant.numeric.float.ktav`                      | Float literal (§ 3.6; has `.` / exponent)  |
| `constant.numeric.ktav`                            | Whole-line Array item number literals      |
| `invalid.illegal.escape.unicode.ktav`              | Lone surrogate or malformed `\uXXXX`        |
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
| `entity.name.tag.ktav`                             | Некавыченные сегменты ключа (слева от `:`) |
| `string.quoted.double.key.ktav`                    | Сегменты ключа в двойных кавычках          |
| `string.quoted.single.key.ktav`                    | Сегменты ключа в одинарных кавычках        |
| `string.quoted.backtick.key.ktav`                  | Сегменты ключа в обратных кавычках         |
| `punctuation.accessor.dot.ktav`                    | `.`, разделяющая сегменты точечного ключа  |
| `punctuation.separator.key-value.ktav`             | `:` обычной пары                           |
| `keyword.operator.marker.raw.ktav`                 | `::` (маркер raw-строки)                   |
| `constant.language.boolean.ktav`                   | Скаляры `true`, `false`                    |
| `constant.language.null.ktav`                      | Скаляры `null`                             |
| `constant.numeric.integer.ktav`                    | Целый литерал (§ 3.6; без избыточного ведущего нуля) |
| `constant.numeric.float.ktav`                      | Литерал с плавающей точкой (§ 3.6; есть `.` / экспонента) |
| `constant.numeric.ktav`                            | Числовые литералы строк-элементов Array    |
| `invalid.illegal.escape.unicode.ktav`              | Одиночный суррогат или некорректный `\uXXXX` |
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
| `entity.name.tag.ktav`                             | 裸键段（`:` 左侧）                         |
| `string.quoted.double.key.ktav`                    | 双引号键段                                 |
| `string.quoted.single.key.ktav`                    | 单引号键段                                 |
| `string.quoted.backtick.key.ktav`                  | 反引号键段                                 |
| `punctuation.accessor.dot.ktav`                    | 分隔点号键段的 `.`                         |
| `punctuation.separator.key-value.ktav`             | 普通键值对的 `:`                           |
| `keyword.operator.marker.raw.ktav`                 | `::`(原始字符串标记)                     |
| `constant.language.boolean.ktav`                   | `true`、`false` 标量                       |
| `constant.language.null.ktav`                      | `null` 标量                                |
| `constant.numeric.integer.ktav`                    | 整数字面量(§ 3.6;无冗余前导零)         |
| `constant.numeric.float.ktav`                      | 浮点字面量(§ 3.6;含 `.` / 指数)        |
| `constant.numeric.ktav`                            | 整行 Array 元素的数字字面量                |
| `invalid.illegal.escape.unicode.ktav`              | 孤立代理项或格式错误的 `\uXXXX`           |
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

