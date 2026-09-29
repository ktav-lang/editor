>>>>> lang=en
- `Cargo.toml`: `ktav = "0.8"` (was `"0.6"`, the version in the last
  released v0.6.1); `rust-version` raised `1.70` → `1.71`.
- **Quoted keys were already understood by the `ktav` parser in 0.7.0;**
  this release extends support to the LSP's own scanners.
  `ktav::parse`-based diagnostics/symbols already handled 0.7 syntax
  correctly with no code changes (see below), but the LSP's *own*
  line-based classifier (`tokens::classify_line`, `split_dotted`,
  used for semantic tokens, hover and completion) and the document-
  outline scanner (`symbols::collect_key_hits`) each hand-roll their
  own colon/dot scan over raw text and did not know a `:` or `.`
  inside `"..."` / `'...'` / `` `...` `` is ordinary content. A quoted
  key containing a structural byte — e.g. `"a:b": 1` or
  `a."b.c".d: 1` — would have had its colon/dot misread, corrupting
  semantic highlighting, completion context detection, and the
  document-symbol outline's key boundaries. Fixed by adding a
  quote-aware separator scan (`tokens::find_key_separator`, replacing
  `tokens::find_unescaped`) and making `tokens::split_dotted`
  quote-opaque; `symbols.rs`'s independent duplicate scanner was
  removed in favour of importing the same two functions from
  `tokens`, which now lives up to its own "single source of truth"
  doc comment. A quote character NOT at a segment's first position
  (`don't: 1`) is unaffected, matching § 5.3.3's positional rule.
- Fixed a pre-existing span-encoding bug in `textDocument/formatting`,
  found while auditing the byte-offset `Span` contract for this
  bump: the whole-document replace edit's end `Position.character`
  was computed with `str::chars().count()` (Unicode scalar count)
  instead of the same encoding-aware conversion every other handler
  in `lsp/src/server/mod.rs` already uses. This undercounts under both negotiated
  encodings whenever the last line has non-ASCII content (UTF-8:
  undercounts byte length for any multi-byte character; UTF-16:
  undercounts for any astral-plane / surrogate-pair character),
  which could leave trailing bytes of the last line unreplaced by a
  format edit. Extracted into `end_of_document()`, covered by new
  tests.
- No code changes were needed for the new `ErrorKind` variants:
  `diagnostics::parse_for_diagnostics` already drives everything off
  `ErrorKind::span()` / `::line()` / `Display` generically, so
  `UnterminatedQuotedKey` gets a correct, tight diagnostic for free.
  `Error::InvalidUtf8` cannot occur through this crate's `&str`-based
  entry points (a Rust `&str` is valid UTF-8 by construction — the
  variant only fires for the byte-level `ktav::from_file`, which this
  LSP never calls); verified, no live code path to fix.
- `reindent::canonicalise_paren_scalar`: removed dead reasoning about
  the `:i` / `:f` typed markers, dropped from the spec back in 0.5.0.
  The special case was already unreachable (any `:i` / `:f` sequence
  already fails the "separator must be followed by whitespace" check
  for an unrelated reason), so removing it changes no behaviour.
  Renamed/rewrote the tests built on the removed markers in both
  `reindent.rs` and `tests/format_pipeline.rs` (including the
  `port:i 8080` / `timeout:f 5.0` sample lines in
  `complex_document_canonicalised`, now plain pairs). The `::` raw
  marker is untouched — it is still current syntax.
- Scalar classification (`tokens::classify_value`) now implements § 3.6
  exactly plus the § 5.2 redundant-leading-zero exception: `01234`, `00`,
  `-045`, `0_7`, `01.5`, `05e3` are Strings; so are `1_`, `1__0`,
  `0x_1`, `0X1A`, `1.`, `.5`, `2026-09-28` and IPv4-like runs, while
  `0`, `0.5`, `0e0`, `0x1_A` stay numbers. The old heuristic accepted a
  sign or an underscore anywhere. Integers beyond the i64 range still
  highlight as numbers (the domain check belongs to the parser).
- Trimming uses the exact 25-code-point whitespace set of § 3.3
  (previously only space, tab and CR were trimmed at the end of a
  line), so a trailing NBSP no longer turns `true` into a string.
- Inline compounds: a value after `::` is a raw String and is never
  highlighted as a number or keyword; quoted key segments
  (`{"a,b": 1}`, `{'x:y': 2}`) are opaque to `,` `:` and brackets; a
  value containing an escape is a String (§ 3.7).
- Hover: fixed a crash — a string value longer than 80 bytes with a
  multi-byte character at the cut point panicked, and because the
  release profile aborts on panic it terminated the server. Hover now
  resolves the full key path, so keys nested in objects and in arrays
  of objects, quoted keys (`"a.b"`) and escaped keys (`a\.b`) show
  their value; the labels read `integer` / `float` (there have been no
  typed markers since 0.5).
- `tests/spec_conformance.rs` validates the corpus manifest (schema,
  categories, fixture counts), also runs `parseable-unrepresentable`
  and `strict-lossy` (through `ktav::parse_strict`), checks the
  `unrepresentable` oracles, and fails instead of silently passing
  when the spec submodule is missing.
- Semantic tokens: a multi-line string block (`(` / `((`) had no
  cross-line state, so `classify_line` re-ran on every content line in
  isolation — a line that merely looked like a comment, a pair or a
  lone closer was highlighted as one, and the verbatim terminator `))`
  fell through to a String token instead of an Operator. Fixed with a
  small `MultiForm` state carried across lines in `semantic_tokens`;
  content lines now emit one trimmed String token each, and `(` / `((`
  / `)` / `))` markers are always Operator. `hover` and `completion`
  gained the same guard (`tokens::line_is_multiline_content`) so they
  no longer misread a line inside an open block as a real `key:` pair.
- Semantic tokens: an inline-object key escaping a structural byte
  right after a comma (e.g. `{x: 0, \[a: 1}`) desynced the inline
  scanner — the lone `\` became a PROPERTY token, the escaped `[`
  opened a bogus nested array, and the real value collapsed into one
  String. `emit_inline`'s key-run scanner now treats `\X` as one
  escaped unit, matching the value-run scanner's existing behaviour.
- `tests/spec_conformance.rs` gained a corpus-wide regression guard:
  for every `valid/**.ktav` fixture, the number of Number/Bool/Null
  *leaves* in the parsed `Value` tree must equal the number of
  Number/Keyword/"null" *tokens* `semantic_tokens` emits (five
  magnitude-overflow-to-String fixtures are pinned exceptions, since
  highlighting is lexical and does not enforce i64/f64 range).
- Known limitation: a document whose only line terminator is a lone
  CR (§ 3.2) is still split on LF by the position mapping shared by
  all handlers; a systemic fix is not part of this release.

>>>>> lang=ru
- `Cargo.toml`: `ktav = "0.8"` (было `"0.6"`, версия последнего
  выпущенного релиза v0.6.1); `rust-version` повышен с `1.70` до `1.71`.
- **Квотированные ключи теперь понимаются и вне парсера `ktav`.**
  Диагностика и символы на базе `ktav::parse` и без правок корректно
  обрабатывали синтаксис 0.7 (см. ниже), но собственный построчный
  классификатор LSP (`tokens::classify_line`, `split_dotted`,
  используемый для семантических токенов, hover и автодополнения) и
  сканер структуры документа (`symbols::collect_key_hits`) каждый
  по-своему сканировали двоеточия и точки в сыром тексте и не знали, что
  `:` или `.` внутри `"..."` / `'...'` / `` `...` `` — обычное
  содержимое.
  Квотированный ключ, содержащий структурный байт, — например
  `"a:b": 1` или `a."b.c".d: 1` — получил бы неверно прочитанные
  двоеточие и точку, ломая семантическую подсветку, определение контекста
  автодополнения и границы ключей в структуре документа. Исправлено
  добавлением кавычек-осведомлённого скана разделителя
  (`tokens::find_key_separator`, заменяющего `tokens::find_unescaped`)
  и переводом `tokens::split_dotted` в режим непрозрачности для
  кавычек; независимый сканер дубликатов в `symbols.rs` удалён в
  пользу импорта тех же двух функций из `tokens`, которые теперь
  оправдывают свой doc-комментарий «единственный источник правды».
  Кавычка НЕ на первой позиции сегмента (`don't: 1`) не затронута —
  это соответствует позиционному правилу § 5.3.3.
- Исправлен давний баг кодирования диапазонов в
  `textDocument/formatting`, найденный при аудите контракта `Span` с
  байтовыми смещениями для этого бампа: конечная
  `Position.character` правки замены всего документа вычислялась через
  `str::chars().count()` (число скаляров Unicode) вместо того же
  преобразования с учётом кодировки, которое уже используют все
  остальные обработчики в `lsp/src/server/mod.rs`. Это занижает счётчик при обеих
  согласованных кодировках, когда последняя строка содержит не-ASCII
  (UTF-8: занижает байтовую длину для любого многобайтового символа;
  UTF-16: занижает для любого астрального символа / суррогатной
  пары), из-за чего хвостовые байты последней строки могли остаться
  незаменёнными после форматирования. Вынесено в `end_of_document()`,
  покрыто новыми тестами.
- Для новых вариантов `ErrorKind` правки кода не потребовалось:
  `diagnostics::parse_for_diagnostics` уже работает через
  `ErrorKind::span()` / `::line()` / `Display` дженерически, поэтому
  `UnterminatedQuotedKey` получает корректную точную диагностику
  бесплатно. `Error::InvalidUtf8` не может возникнуть через
  `&str`-API этого крейта (Rust-овый `&str` валиден в UTF-8 по
  построению — вариант срабатывает только для байтового
  `ktav::from_file`, которого этот LSP никогда не вызывает); проверено,
  живого пути для исправления нет.
- `reindent::canonicalise_paren_scalar`: удалено мёртвое рассуждение о
  типизированных маркерах `:i` / `:f`, исключённых из спецификации ещё
  в 0.5.0. Особая ветка была уже недостижима (любая последовательность
  `:i` / `:f` и так проваливает проверку «после разделителя должен
  идти пробел» по другой причине), так что её удаление не меняет
  поведения. Переименованы/переписаны тесты, построенные на удалённых
  маркерах, в `reindent.rs` и `tests/format_pipeline.rs` (включая
  строки-примеры `port:i 8080` / `timeout:f 5.0` в
  `complex_document_canonicalised`, теперь обычные пары). Сырой маркер
  `::` не тронут — это по-прежнему актуальный синтаксис.
- Классификация скаляров (`tokens::classify_value`) теперь точно
  реализует § 3.6 и исключение § 5.2 об избыточном ведущем нуле:
  `01234`, `00`, `-045`, `0_7`, `01.5`, `05e3` — строки; также строки
  `1_`, `1__0`, `0x_1`, `0X1A`, `1.`, `.5`, `2026-09-28` и цепочки вида
  IPv4, а `0`, `0.5`, `0e0`, `0x1_A` остаются числами. Прежняя
  эвристика допускала знак или подчёркивание где угодно. Целые за
  пределами i64 по-прежнему подсвечиваются как числа (проверка
  диапазона — дело парсера).
- Обрезка использует точный набор из 25 кодовых точек пробелов § 3.3
  (раньше в конце строки обрезались только пробел, табуляция и CR),
  поэтому конечный NBSP больше не превращает `true` в строку.
- Inline-структуры: значение после `::` — сырая строка и никогда не
  подсвечивается как число или ключевое слово; сегменты ключей в
  кавычках (`{"a,b": 1}`, `{'x:y': 2}`) непрозрачны для `,` `:` и
  скобок; значение с экранированием — строка (§ 3.7).
- Hover: исправлено падение — строковое значение длиннее 80 байт с
  многобайтовым символом в точке обрезки вызывало panic, а так как
  release-профиль прерывает процесс при panic, сервер завершался.
  Теперь hover находит полный путь ключа: вложенные в объекты и в
  массивы объектов ключи, ключи в кавычках (`"a.b"`) и с
  экранированием (`a\.b`) показывают значение; подписи — `integer` /
  `float` (типизированных маркеров нет с 0.5).
- `tests/spec_conformance.rs` проверяет manifest корпуса (схему,
  категории, число фикстур), прогоняет также `parseable-unrepresentable`
  и `strict-lossy` (через `ktav::parse_strict`), проверяет оракулы
  `unrepresentable` и падает, а не проходит молча, если подмодуль spec
  отсутствует.
- Semantic tokens: у многострочного строкового блока (`(` / `((`) не
  было состояния между строками, поэтому `classify_line` заново
  разбирал каждую строку содержимого в одиночку — строка, лишь похожая
  на комментарий, пару или одиночную закрывающую скобку, подсвечивалась
  как таковая, а закрывающая `))` verbatim-формы вместо Operator
  становилась токеном String. Исправлено небольшим состоянием
  `MultiForm`, переносимым между строками в `semantic_tokens`: строки
  содержимого теперь дают ровно один обрезанный токен String, а маркеры
  `(` / `((` / `)` / `))` всегда Operator. `hover` и `completion`
  получили ту же защиту (`tokens::line_is_multiline_content`) и больше
  не принимают строку внутри открытого блока за настоящую пару `key:`.
- Semantic tokens: экранирование структурного байта сразу после запятой
  в ключе inline-объекта (например, `{x: 0, \[a: 1}`) десинхронизировало
  inline-сканер — одиночный `\` становился токеном PROPERTY, экранированный
  `[` открывал фиктивный вложенный массив, а реальное значение схлопывалось
  в одну строку. Сканер ключа в `emit_inline` теперь трактует `\X` как одну
  экранированную единицу — так же, как уже делает сканер значений.
- `tests/spec_conformance.rs` получил сквозной регрессионный тест по
  корпусу: для каждой фикстуры `valid/**.ktav` число Number/Bool/Null
  *листьев* в распарсенном дереве `Value` должно совпадать с числом
  токенов Number/Keyword/"null", которые выдаёт `semantic_tokens` (пять
  фикстур с переполнением величины в String — зафиксированные
  исключения: подсветка лексическая и не проверяет диапазон i64/f64).
- Известное ограничение: документ, где единственный разделитель строк —
  одиночный CR (§ 3.2), по-прежнему делится по LF в общем для всех
  обработчиков отображении позиций; системное исправление не входит в
  этот релиз.

>>>>> lang=zh
- `Cargo.toml`:`ktav = "0.8"`(原为 `"0.6"`,即上一个已发布版本
  v0.6.1 使用的版本);`rust-version` 从 `1.70` 提升至 `1.71`。
- **带引号的键现在在 `ktav` 解析器之外也能被正确理解。** 基于
  `ktav::parse` 的诊断/符号无需任何代码改动即可正确处理 0.7 语法
  (见下文),但 LSP 自有的按行分类器(`tokens::classify_line`、
  `split_dotted`,用于语义高亮、悬停提示和自动补全)与文档结构扫描器
  (`symbols::collect_key_hits`)各自手工扫描原始文本中的冒号和点,
  不知道 `"..."` / `'...'` / `` `...` `` 内的 `:` 或 `.` 只是普通
  内容。含结构字节的带引号键 —— 例如 `"a:b": 1` 或 `a."b.c".d: 1`
  —— 其冒号/点会被误读,进而破坏语义高亮、补全上下文检测以及文档
  符号大纲的键边界。修复方式是新增感知引号的冒号扫描
  (`tokens::find_key_separator`,取代 `tokens::find_unescaped`),并让
  `tokens::split_dotted` 对引号不透明;`symbols.rs` 中独立的重复
  扫描器已删除,改为从 `tokens` 导入同样的两个函数,它们现在终于
  配得上自己「单一事实来源」的文档注释。不在片段首位的引号
  (`don't: 1`)不受影响,符合 § 5.3.3 的位置规则。
- 修复了 `textDocument/formatting` 中一个既有的跨度编码 bug,是在
  为本次升级审计字节偏移 `Span` 契约时发现的:整档替换编辑的结束
  `Position.character` 此前用 `str::chars().count()`(Unicode 标量
  计数)计算,而不是 `lsp/src/server/mod.rs` 其他处理函数都在用的、同样考虑编码
  的转换。只要最后一行含非 ASCII 内容,两种协商编码下都会少算
  (UTF-8:任何多字节字符都会少算字节长度;UTF-16:任何星界平面/代理
  对字符都会少算),可能导致格式编辑后最后一行尾部字节未被替换。
  相关逻辑已提取到 `end_of_document()`,并有新测试覆盖。
- 新的 `ErrorKind` 变体无需改动代码:
  `diagnostics::parse_for_diagnostics` 已经通过 `ErrorKind::span()` /
  `::line()` / `Display` 通用地驱动一切,因此 `UnterminatedQuotedKey`
  免费获得正确而精确的诊断。`Error::InvalidUtf8` 不会经由本 crate 的
  `&str` 入口出现(Rust 的 `&str` 按构造即为合法 UTF-8 —— 该变体
  只为字节级入口 `ktav::from_file` 触发,而本 LSP 从不调用它);
  已验证无现存代码路径需要修复。
- `reindent::canonicalise_paren_scalar`:删除了关于 `:i` / `:f` 类型
  标记的失效推理,这些标记早在 0.5.0 就已从规范中移除。该特殊分支
  本就不可达(任何 `:i` / `:f` 序列已经会因为另一个原因 —— 「分隔符
  后必须跟空格」检查 —— 失败),删除它不改变任何行为。基于已删除
  标记编写的测试在 `reindent.rs` 和 `tests/format_pipeline.rs` 中已
  改名/重写(包括 `complex_document_canonicalised` 中的示例行
  `port:i 8080` / `timeout:f 5.0`,现在只是普通键值对)。`::` 原始
  标记未动 —— 它仍是现行语法。
- 标量分类(`tokens::classify_value`)现在严格实现 § 3.6,并加入 § 5.2
  的冗余前导零例外:`01234`、`00`、`-045`、`0_7`、`01.5`、`05e3` 为
  字符串;`1_`、`1__0`、`0x_1`、`0X1A`、`1.`、`.5`、`2026-09-28` 以及
  类 IPv4 的串也是字符串,而 `0`、`0.5`、`0e0`、`0x1_A` 仍是数字。旧的
  启发式允许符号或下划线出现在任意位置。超出 i64 范围的整数仍按数字
  高亮(范围检查属于解析器)。
- 裁剪使用 § 3.3 中精确的 25 个空白码位(此前行尾只裁剪空格、制表符
  和 CR),因此行尾的 NBSP 不再让 `true` 变成字符串。
- 内联复合值:`::` 之后的值是原始字符串,绝不按数字或关键字高亮;
  带引号的键片段(`{"a,b": 1}`、`{'x:y': 2}`)对 `,`、`:` 和括号不透明;
  含转义的值为字符串(§ 3.7)。
- Hover:修复崩溃——长度超过 80 字节且截断点处是多字节字符的字符串值
  会 panic,而 release 配置在 panic 时中止进程,导致服务器退出。现在
  hover 会解析完整的键路径,因此嵌套在对象和对象数组中的键、带引号的键
  (`"a.b"`)和带转义的键(`a\.b`)都能显示其值;标签为 `integer` /
  `float`(自 0.5 起不再有类型标记)。
- `tests/spec_conformance.rs` 校验语料库 manifest(结构、类别、样例数量),
  另外运行 `parseable-unrepresentable` 和 `strict-lossy`(通过
  `ktav::parse_strict`),检查 `unrepresentable` 的 oracle,并且在缺少 spec
  子模块时失败而不是静默通过。
- Semantic tokens:多行字符串块(`(` / `((`)此前没有跨行状态,导致
  `classify_line` 对每一行内容都单独重新解析——某行内容只是碰巧像注释、
  键值对或单独的闭合符,就会被当作对应类型高亮,而 verbatim 形式的
  闭合符 `))` 也会误判为 String token 而非 Operator。现通过在
  `semantic_tokens` 中跨行携带的小型 `MultiForm` 状态修复:块内容行
  现在各自产生一个裁剪后的 String token,`(` / `((` / `)` / `))`
  标记始终为 Operator。`hover` 与 `completion` 也获得了相同的防护
  (`tokens::line_is_multiline_content`),不会再把开放块内的一行误判为
  真正的 `key:` 键值对。
- Semantic tokens:内联对象的键在逗号之后紧跟转义结构字节(例如
  `{x: 0, \[a: 1}`)会使内联扫描器失步——单独的 `\` 变成 PROPERTY
  token,被转义的 `[` 会打开一个虚假的嵌套数组,真正的值则被折叠进
  一个字符串。`emit_inline` 的键扫描器现在将 `\X` 视为一个整体的转义
  单元,与值扫描器现有的处理方式一致。
- `tests/spec_conformance.rs` 新增了覆盖整个语料库的回归测试:对每个
  `valid/**.ktav` 样例,已解析 `Value` 树中 Number/Bool/Null *叶子*
  的数量必须与 `semantic_tokens` 输出的 Number/Keyword/"null" *token*
  数量一致(五个数值溢出为 String 的样例为固定的例外——高亮是词法层面
  的,不检查 i64/f64 的取值范围)。
- 已知限制:仅以单独 CR(§ 3.2)作为行终止符的文档,仍会在所有处理器
  共用的位置映射中按 LF 拆分;系统性修复不在本次发布范围内。

