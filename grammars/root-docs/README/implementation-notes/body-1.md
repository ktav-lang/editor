>>>>> lang=en
## Notes on the implementation

- Marker disambiguation. Inside `pair`, alternatives are ordered
  `pair-raw` (`::`) → empty/open compound and multi-line forms →
  `pair-value` (`:` fallback). After a plain `:`, a bare body is
  classified by lexical form (§ 3.6 / § 5.2): an integer literal →
  `constant.numeric.integer`, a float literal → `constant.numeric.float`,
  anything else — including redundant-leading-zero decimals such as
  `01234` — → `string.unquoted` (matching the spec's mandatory
  space-after-separator rule, § 5.3 / § 6.10).
- The classification lives in the directly scanned pattern, not in a
  rule reached through `captures`: `^` / `$` there anchor to the line,
  not to the capture, so a capture-level check silently fails off
  column 0.
- Compound closers are anchored to standalone lines: `^\s*\)\s*$`,
  `^\s*\)\)\s*$`, `^\s*\}\s*$`, `^\s*\]\s*$`. A line like `) x` or
  `))suffix` does not close the block — it is content (in a multi-line
  string) or a syntax error (in an Object/Array context, which the
  grammar leaves unhighlighted).
- Array context is tracked through dedicated `array-*` repository
  rules included only inside `[ … ]` regions, so item-form lines
  (`:: foo`, bare scalars) light up only there.
- Multi-line string content is highlighted as a single string scope —
  no inner classification is applied, matching the spec's "raw
  content" semantics (§ 5.6).

>>>>> lang=ru
## Заметки о реализации

- Разрешение маркеров. Внутри `pair` альтернативы упорядочены так:
  `pair-raw` (`::`) → пустые/открывающие составные и многострочные формы →
  `pair-value` (запасной вариант `:`). После обычного `:` голое тело
  классифицируется по лексической форме (§ 3.6 / § 5.2): целый литерал →
  `constant.numeric.integer`, литерал с плавающей точкой →
  `constant.numeric.float`, всё остальное, включая десятичные с
  избыточным ведущим нулём вроде `01234`, → `string.unquoted` (в
  соответствии с обязательным правилом пробела после разделителя в
  спецификации, § 5.3 / § 6.10).
- Классификация живёт в напрямую сканируемом шаблоне, а не в правиле,
  подключённом через `captures`: там `^` / `$` привязываются к строке, а
  не к захвату, поэтому проверка на уровне захвата молча не срабатывает
  вне колонки 0.
- Закрывающие скобки составных значений привязаны к отдельным строкам:
  `^\s*\)\s*$`, `^\s*\)\)\s*$`, `^\s*\}\s*$`, `^\s*\]\s*$`. Строка вида
  `) x` или `))suffix` не закрывает блок — это содержимое (в многострочной
  строке) или синтаксическая ошибка (в контексте Object/Array, которую
  грамматика оставляет без подсветки).
- Контекст массива отслеживается через отдельные правила репозитория
  `array-*`, подключаемые только внутри областей `[ … ]`, поэтому строки
  в форме элементов (`:: foo`, голые скаляры) подсвечиваются только там.
- Содержимое многострочной строки подсвечивается одним строковым scope —
  внутренняя классификация не применяется, что соответствует семантике
  «сырого содержимого» из спецификации (§ 5.6).

>>>>> lang=zh
## 实现说明

- 标记消歧。在 `pair` 内部,备选项按以下顺序排列:`pair-raw`(`::`)→
  空/开放的复合与多行形式 → `pair-value`(`:` 兜底)。在普通 `:` 之后,
  裸主体按词法形式分类(§ 3.6 / § 5.2):整数字面量 →
  `constant.numeric.integer`,浮点字面量 → `constant.numeric.float`,
  其他一切(包括 `01234` 这类带冗余前导零的十进制数)→
  `string.unquoted`(与规范中分隔符后必须有空格的规则一致,
  § 5.3 / § 6.10)。
- 分类位于直接扫描的模式中,而不是经由 `captures` 引入的规则:那里的
  `^` / `$` 锚定的是行而不是捕获,因此捕获级检查在非第 0 列时会静默失效。
- 复合值的闭合符锚定在独立的行上:`^\s*\)\s*$`、`^\s*\)\)\s*$`、
  `^\s*\}\s*$`、`^\s*\]\s*$`。形如 `) x` 或 `))suffix` 的行不会闭合块
  ——它是内容(在多行字符串中)或语法错误(在 Object/Array 上下文中,
  语法对此不做高亮)。
- 数组上下文通过专门的 `array-*` 仓库规则跟踪,这些规则只在 `[ … ]`
  区域内引入,因此元素形式的行(`:: foo`、裸标量)只在那里高亮。
- 多行字符串内容以单一字符串 scope 高亮——不做内部分类,与规范的
  "原始内容"语义一致(§ 5.6)。

