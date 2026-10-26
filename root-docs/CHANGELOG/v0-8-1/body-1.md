>>>>> lang=en
## [0.8.1] — Unreleased

- Align the LSP, VS Code and IntelliJ versions on 0.8.1. The server pins
  `ktav = "=0.8.1"`; the spec submodule pins `v0.8.1`, not the 0.9 work.
- The reference parser now rejects explicit roots left open at EOF.
  LSP regressions pin the diagnostic range from the opener through EOF
  for LF, CRLF and lone CR, including a root with a closed nested value.
- Only lowercase `0x`/`0o`/`0b` prefixes are numeric: `0x1A` is an Integer,
  while `0X1A`, `0O17`, `0B101` and signed uppercase forms remain Strings.
  TextMate and IntelliJ regression vectors preserve that distinction.
- Conformance guards track all 340 spec fixtures across five categories:
  228 valid, 90 invalid, 5 unrepresentable, 4 parseable-unrepresentable
  and 13 strict-lossy. IDE lexer suites cover all 228 valid fixtures;
  raw invalid UTF-8 is checked at the corpus boundary, not sent as LSP text.
- Update installation guides and component changelogs in English, Russian
  and Chinese. Source-edit and the 0.9 error model are not part of 0.8.1.

>>>>> lang=ru
## [0.8.1] — Не выпущено

- Версии LSP, VS Code и IntelliJ синхронизированы на 0.8.1. Сервер закрепляет
  `ktav = "=0.8.1"`, подмодуль spec — `v0.8.1`, а не наработки 0.9.
- Эталонный парсер теперь отклоняет явные корни, оставленные открытыми в EOF.
  Регрессии LSP фиксируют диапазон от открывающего символа до EOF для LF,
  CRLF и одиночного CR, включая корень с закрытым вложенным значением.
- Числовыми являются только префиксы `0x`/`0o`/`0b`: `0x1A` — Integer,
  а `0X1A`, `0O17`, `0B101` и формы с заглавным префиксом и знаком — String.
  Регрессионные векторы TextMate и IntelliJ сохраняют это различие.
- Guards охватывают все 340 фикстур пяти категорий: 228 valid, 90 invalid,
  5 unrepresentable, 4 parseable-unrepresentable и 13 strict-lossy.
  Наборы IDE-лексеров охватывают все 228 valid-фикстур; невалидный UTF-8
  проверяется на границе corpus, а не передаётся как текст LSP.
- Инструкции установки и changelog компонентов обновлены на EN/RU/ZH.
  Source-edit и модель ошибок 0.9 не входят в 0.8.1.

>>>>> lang=zh
## [0.8.1] — 未发布

- LSP、VS Code 和 IntelliJ 版本同步至 0.8.1。服务器固定
  `ktav = "=0.8.1"`，spec 子模块固定在 `v0.8.1`，不引入 0.9 的工作。
- 参考解析器拒绝到 EOF 仍未闭合的显式根容器。LSP 回归测试固定从
  起始符到 EOF 的诊断范围，覆盖 LF、CRLF、单独 CR，以及已闭合的嵌套值。
- 只有小写 `0x`/`0o`/`0b` 前缀是数值：`0x1A` 是 Integer，
  `0X1A`、`0O17`、`0B101` 及带符号的大写前缀形式仍为 String。
  TextMate 和 IntelliJ 回归向量保持这一区别。
- 一致性检查覆盖五类共 340 个规范样例：228 valid、90 invalid、
  5 unrepresentable、4 parseable-unrepresentable、13 strict-lossy。
  IDE 词法测试覆盖全部 228 个有效样例；无效 UTF-8 在语料边界检查，
  不作为 LSP 文本发送。
- 安装指南和组件变更日志同步更新英语、俄语和中文。
  Source-edit 和 0.9 错误模型不属于 0.8.1。

