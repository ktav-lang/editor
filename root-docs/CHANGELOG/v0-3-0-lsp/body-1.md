>>>>> lang=en
### LSP server (`ktav-lsp`)

- **`build_symbols` rewritten as O(N) single-pass scanner** — the
  IDE document outline previously stalled the language server for
  ~11.5 seconds on a 500 KiB document because every top-level key
  triggered a full-document text scan. The new scanner walks the
  text once, recording `(virtual_depth, key, line_range)` for every
  pair, and the DFS over the parsed `Value` advances a sequential
  cursor through the hits. Wall-clock: 11.5 s → 13.2 ms (871×
  faster). Outline-aware editors (JetBrains, VSCode) no longer
  hang on large config files.

- **Format pipeline: line-based reindent runs unconditionally**
  instead of gating on a successful parse. With the `ktav` 0.3.0
  parser strictness rejecting inline `(value)`, the previous
  "format only when parses" gate locked users out of formatting
  exactly when formatting would have repaired the issue.
  `canonicalise_paren_scalar` rewrites `key: (value)` →
  `key:: (value)` on save. 22-test integration suite added in
  `tests/format_pipeline.rs` covering indentation normalisation,
  blank-line preservation, comment preservation, multi-line
  stripped/verbatim forms (byte-exact), auto-fix of inline paren
  scalars, edge cases (CRLF, no trailing newline, empty doc).

>>>>> lang=ru
### LSP server (`ktav-lsp`)

- **`build_symbols` переписан как однопроходный сканер O(N)** — ранее
  структура документа в IDE останавливала языковой сервер примерно на
  11.5 секунды на документе в 500 KiB, потому что каждый ключ верхнего
  уровня запускал полный скан текста. Новый сканер проходит текст один
  раз, записывая `(virtual_depth, key, line_range)` для каждой пары, а
  DFS по разобранному `Value` двигает последовательный курсор по
  попаданиям. Замер времени: 11.5 с → 13.2 мс (в 871× быстрее).
  Редакторы со структурой документа (JetBrains, VSCode) больше не
  зависают на больших конфигурационных файлах.

- **Конвейер форматирования: построчный reindent работает
  безусловно** вместо условия «только при успешном парсе». При
  строгости парсера `ktav` 0.3.0, отвергающем инлайн `(value)`,
  прежний режим «форматируем только когда парсится» отрезал
  пользователей от форматирования ровно тогда, когда форматирование
  как раз исправило бы проблему. `canonicalise_paren_scalar`
  переписывает `key: (value)` → `key:: (value)` при сохранении. В
  `tests/format_pipeline.rs` добавлен интеграционный набор из 22
  тестов: нормализация отступов, сохранение пустых строк, сохранение
  комментариев, многострочные stripped/verbatim формы (байт-в-байт),
  автофикс инлайн-скобочных скаляров, краевые случаи (CRLF, отсутствие
  завершающего перевода строки, пустой документ).

>>>>> lang=zh
### LSP server (`ktav-lsp`)

- **`build_symbols` 重写为 O(N) 单遍扫描器** —— 此前,IDE 的文档大纲
  会让语言服务器在 500 KiB 的文档上卡住约 11.5 秒,因为每个顶层键都
  会触发一次全文扫描。新扫描器只遍历文本一次,为每一对记录
  `(virtual_depth, key, line_range)`,而对解析后 `Value` 的 DFS 则用
  一个顺序游标推进这些命中。实测耗时:11.5 秒 → 13.2 毫秒(快 871×)。
  支持大纲的编辑器(JetBrains、VSCode)不再在大配置文件上卡死。

- **格式化流水线:基于行的 reindent 无条件运行**,而不是以解析成功为
  前提。`ktav` 0.3.0 解析器严格拒绝内联 `(value)`,此前「仅在解析
  成功时格式化」的开关,恰恰在格式化本可以修复问题的时候把用户挡在
  门外。`canonicalise_paren_scalar` 会在保存时把 `key: (value)` 改写
  为 `key:: (value)`。`tests/format_pipeline.rs` 中新增了 22 个测试的
  集成套件,覆盖缩进规范化、空行保留、注释保留、多行
  stripped/verbatim 形式(字节级一致)、内联圆括号标量的自动修复,
  以及边界情况(CRLF、无结尾换行、空文档)。

