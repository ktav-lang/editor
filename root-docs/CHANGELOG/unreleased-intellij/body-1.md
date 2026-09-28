>>>>> lang=en
### IntelliJ plugin

- The highlighting lexer follows the spec: exact § 3.6 number grammar
  with the § 5.2 leading-zero exception (ASCII digits only — the old
  check used `Char.isDigit()` and accepted other scripts' digits),
  exact keywords, and the exact 25-code-point whitespace set of § 3.3.
- Quoted key segments (§ 5.3.3) are supported in whole-line and inline
  keys; an unterminated quote degrades to a plain value and keeps the
  incremental re-lexing state valid.
- `::` values are Strings inside inline compounds as well; an inline
  scalar with an escape is a String; a literal `:` inside an inline
  value is no longer taken for a separator.

>>>>> lang=ru
### Плагин IntelliJ

- Лексер подсветки следует спецификации: точная грамматика чисел § 3.6
  с исключением § 5.2 о ведущем нуле (только ASCII-цифры — прежняя
  проверка использовала `Char.isDigit()` и принимала цифры других
  письменностей), точные ключевые слова и точный набор из 25 кодовых
  точек пробелов § 3.3.
- Сегменты ключей в кавычках (§ 5.3.3) поддержаны в ключах строк и
  inline; незакрытая кавычка вырождается в обычное значение и не
  портит состояние инкрементального перелексинга.
- Значения `::` — строки и внутри inline-структур; inline-скаляр с
  экранированием — строка; литеральный `:` внутри inline-значения
  больше не принимается за разделитель.

>>>>> lang=zh
### IntelliJ 插件

- 高亮词法分析器遵循规范:严格的 § 3.6 数字语法并加入 § 5.2 前导零例外
  (仅 ASCII 数字——旧检查使用 `Char.isDigit()`,会接受其他文字的数字)、
  精确的关键字,以及 § 3.3 中精确的 25 个空白码位。
- 整行键与内联键都支持带引号的键片段(§ 5.3.3);未闭合的引号退化为
  普通值,并保持增量重新词法分析的状态有效。
- 内联复合值中 `::` 之后的值同样是字符串;含转义的内联标量为字符串;
  内联值中的字面 `:` 不再被当作分隔符。

