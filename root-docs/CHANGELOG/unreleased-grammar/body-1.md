>>>>> lang=en
### TextMate grammar (VS Code + shared `grammars/`)

- Quoted key segments: the dotted-key pattern shared by every
  `pair-*` / `inline-pair` rule now accepts `"..."`, `'...'` or
  `` `...` `` as an alternative to a bare segment at each segment
  boundary, respecting the positional rule (`don't: 1` is unaffected).
  `key-name` sub-highlights each quoted form with its own scope
  (`string.quoted.{double,single,backtick}.key.ktav`).
- `\uXXXX` escape recognised in `key-name` and `inline-scalar-body`
  (`constant.character.escape.unicode.ktav`); `inline-scalar-body`'s
  named-escape character class also gained `\.` `\:` `\"` `\'`
  `` \` `` — present in the spec since 0.6.0/0.7.0 but missing from
  this grammar's escape-highlighting list until now.
- `grammars/ktav.tmLanguage.json` is the source of truth;
  `vscode/syntaxes/ktav.tmLanguage.json` is a generated mirror kept in
  sync by `vscode/scripts/sync-grammars.js` (run via `npm run
  sync-grammars` / `compile` / `vscode:prepublish`, and explicitly in
  the release workflow before packaging). Both files are updated here
  through that script so they stay byte-identical.

>>>>> lang=ru
### TextMate grammar (VS Code + shared `grammars/`)

- Квотированные сегменты ключа: паттерн точечных ключей, общий для
  всех правил `pair-*` / `inline-pair`, теперь принимает `"..."`,
  `'...'` или `` `...` `` как альтернативу голому сегменту на каждой
  границе сегментов, с соблюдением позиционного правила (`don't: 1` не
  затронут). `key-name` подсвечивает каждую квотированную форму своим
  scope'ом (`string.quoted.{double,single,backtick}.key.ktav`).
- Экранирование `\uXXXX` распознаётся в `key-name` и
  `inline-scalar-body` (`constant.character.escape.unicode.ktav`);
  класс именованных escape-символов `inline-scalar-body` также
  пополнился `\.` `\:` `\"` `\'` `` \` `` — они есть в спецификации с
  0.6.0/0.7.0, но до сих пор отсутствовали в списке
  escape-подсветки этой грамматики.
- `grammars/ktav.tmLanguage.json` — источник правды;
  `vscode/syntaxes/ktav.tmLanguage.json` — сгенерированное зеркало,
  синхронизируемое `vscode/scripts/sync-grammars.js` (запускается
  через `npm run
  sync-grammars` / `compile` / `vscode:prepublish`, а также явно в
  релизном workflow перед упаковкой). Оба файла обновлены здесь через
  этот скрипт, чтобы оставаться байт-в-байт одинаковыми.

>>>>> lang=zh
### TextMate grammar (VS Code + shared `grammars/`)

- 带引号的键片段:所有 `pair-*` / `inline-pair` 规则共享的点分键
  模式,现在在每个片段边界处都接受 `"..."`、`'...'` 或 `` `...` ``
  作为裸片段的替代形式,并遵守位置规则(`don't: 1` 不受影响)。
  `key-name` 为每种带引号形式用各自的 scope 子高亮
  (`string.quoted.{double,single,backtick}.key.ktav`)。
- `key-name` 和 `inline-scalar-body` 现在识别 `\uXXXX` 转义
  (`constant.character.escape.unicode.ktav`);`inline-scalar-body` 的
  命名转义字符类也补入了 `\.` `\:` `\"` `\'` `` \` `` —— 规范自
  0.6.0/0.7.0 起就有它们,但本语法的转义高亮列表一直缺失。
- `grammars/ktav.tmLanguage.json` 是唯一事实来源;
  `vscode/syntaxes/ktav.tmLanguage.json` 是由
  `vscode/scripts/sync-grammars.js` 同步生成的镜像(经 `npm run
  sync-grammars` / `compile` / `vscode:prepublish` 运行,并在发布
  workflow 打包前显式执行)。两个文件此处均通过该脚本更新,以保持
  字节级一致。

