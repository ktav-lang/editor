>>>>> lang=en
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
- Number scopes follow § 3.6 / § 5.2 exactly (whole-line pair values,
  array items and inline values): redundant leading zeros (`01234`,
  `0_7`, `01.5`), misplaced underscores (`1_`, `1__0`, `0x_1`),
  upper-case base prefixes (`0X1A`) and dates or dotted runs
  (`2026-09-28`, `127.0.0.1`) are strings, not numbers.
- `\uXXXX`: a high+low surrogate pair is one escape token; a lone
  surrogate or a malformed `\u` gets
  `invalid.illegal.escape.unicode.ktav` (§ 3.7.1).
- Inline objects: a `::` value is always `string.unquoted.raw.ktav`
  (never number or keyword); quoted key segments are recognised at any
  indentation and after `{` or `,`.
- Fixed a structural defect: `^` / `$` inside a rule reached through
  `captures` anchor to the line, not to the capture, so the
  number/keyword/quoted-key classification silently failed whenever the
  value did not start at column 0. The classification is now part of
  the directly scanned pattern.
- A tokenizer test (`vscode/src/test/unit/grammar-tokens.test.ts`, on
  `vscode-textmate` + `vscode-oniguruma`) runs the real grammar over
  these vectors in whole-line, array-item and inline contexts.
- A document that is a single-line inline object or array at the top
  level (no leading key) was not highlighted at all; the root patterns
  now include `top-level-inline-object` / `top-level-inline-array`. A
  corpus-wide tokenizer test (`corpus-coverage.test.ts`) runs the grammar
  over every valid fixture: no `invalid.*` scope, and the number, boolean
  and null scope counts match the fixture's expected value.

>>>>> lang=ru
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
- Scope чисел точно следуют § 3.6 / § 5.2 (значения пар в строке,
  элементы массива и inline-значения): избыточные ведущие нули
  (`01234`, `0_7`, `01.5`), неверно поставленные подчёркивания (`1_`,
  `1__0`, `0x_1`), префиксы систем счисления в верхнем регистре
  (`0X1A`), даты и цепочки через точку (`2026-09-28`, `127.0.0.1`) —
  строки, а не числа.
- `\uXXXX`: пара старший+младший суррогат — один токен экранирования;
  одиночный суррогат или некорректный `\u` получает
  `invalid.illegal.escape.unicode.ktav` (§ 3.7.1).
- Inline-объекты: значение после `::` всегда
  `string.unquoted.raw.ktav` (никогда число или ключевое слово);
  сегменты ключей в кавычках распознаются при любом отступе и после
  `{` или `,`.
- Исправлен структурный дефект: `^` / `$` внутри правила, подключённого
  через `captures`, привязываются к строке, а не к захвату, поэтому
  классификация число/ключевое слово/ключ в кавычках молча не
  срабатывала, если значение начиналось не с колонки 0. Теперь
  классификация входит в напрямую сканируемый шаблон.
- Тест токенизатора (`vscode/src/test/unit/grammar-tokens.test.ts` на
  `vscode-textmate` + `vscode-oniguruma`) прогоняет настоящую грамматику
  по этим векторам в контекстах строки, элемента массива и inline.
- Документ, целиком являющийся однострочным inline-объектом или
  массивом верхнего уровня (без ключа), вообще не подсвечивался; корневые
  patterns теперь включают `top-level-inline-object` /
  `top-level-inline-array`. Тест токенизатора по всему корпусу
  (`corpus-coverage.test.ts`) прогоняет грамматику по каждой valid-фикстуре:
  ни одного scope `invalid.*`, а число scope чисел, булевых и null
  совпадает с ожидаемым значением фикстуры.

>>>>> lang=zh
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
- 数字 scope 严格遵循 § 3.6 / § 5.2(整行键值对、数组元素和内联值):
  冗余前导零(`01234`、`0_7`、`01.5`)、位置错误的下划线(`1_`、`1__0`、
  `0x_1`)、大写进制前缀(`0X1A`)以及日期或点分串(`2026-09-28`、
  `127.0.0.1`)都是字符串,而不是数字。
- `\uXXXX`:高位加低位代理对是一个转义 token;孤立代理项或格式错误的
  `\u` 获得 `invalid.illegal.escape.unicode.ktav`(§ 3.7.1)。
- 内联对象:`::` 之后的值始终为 `string.unquoted.raw.ktav`(绝不是数字或
  关键字);带引号的键片段在任意缩进以及 `{` 或 `,` 之后都能识别。
- 修复结构性缺陷:经由 `captures` 引入的规则中的 `^` / `$` 锚定的是行而
  不是捕获,因此只要值不从第 0 列开始,数字/关键字/带引号键的分类就会
  静默失效。现在分类已并入直接扫描的模式。
- 新增分词器测试(`vscode/src/test/unit/grammar-tokens.test.ts`,基于
  `vscode-textmate` + `vscode-oniguruma`),在整行、数组元素和内联上下文
  中用真实语法跑这些向量。
- 顶层就是单行内联对象或数组(没有前导键)的文档此前完全不被高亮;
  根 patterns 现在包含 `top-level-inline-object` /
  `top-level-inline-array`。新增覆盖整个语料库的分词器测试
  (`corpus-coverage.test.ts`),对每个 valid 样例运行该语法:不出现
  `invalid.*` scope,数字、布尔和 null 的 scope 数量与样例的期望值一致。

