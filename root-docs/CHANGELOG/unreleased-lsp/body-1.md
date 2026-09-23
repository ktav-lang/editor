>>>>> lang=en
- `Cargo.toml`: `ktav = "0.7"` (was `"0.6"`); `rust-version` raised to
  `1.71` (ktav 0.7's own MSRV, was `1.70`).
- **Quoted keys are now understood outside the `ktav` parser too.**
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
  in `server.rs` already uses. This undercounts under both negotiated
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

>>>>> lang=ru
- `Cargo.toml`: `ktav = "0.7"` (было `"0.6"`); `rust-version` поднят до
  `1.71` (собственный MSRV ktav 0.7, было `1.70`).
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
  остальные обработчики в `server.rs`. Это занижает счётчик при обеих
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

>>>>> lang=zh
- `Cargo.toml`:`ktav = "0.7"`(原为 `"0.6"`);`rust-version` 提升至
  `1.71`(ktav 0.7 自身的 MSRV,原为 `1.70`)。
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
  计数)计算,而不是 `server.rs` 其他处理函数都在用的、同样考虑编码
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

