>>>>> lang=en
## What you get as a Ktav user

LSP features depend on the editor's client. The IntelliJ plugin
integrates live diagnostics and whole-file formatting only; it does
not integrate hover, completion, document symbols or semantic tokens.
Its syntax highlighting is provided by a native lexer.

- **Syntax highlighting** — keys, scalars, the literal-string marker (`::`), multi-line strings, comments
- **Bracket matching & auto-close** — `{}` `[]` `()`
- **Comment toggle** — `Ctrl/Cmd+/` → `## comment`
- **Live diagnostics** (with the LSP) — every `MissingSeparatorSpace`, duplicate key, dotted-prefix conflict surfaces as a red squiggle on the offending line, with the same message the parser emits
- **Hover info** (with the LSP) — dotted path of the key under cursor, inferred type of the value
- **Completion** (with the LSP) — keywords (`null` / `true` / `false`), the literal-string marker (`::`), compound openers
- **Document symbols** (with the LSP) — outline reflecting the parsed `Value::Object` structure

>>>>> lang=ru
## Что получает пользователь Ktav

LSP-функции зависят от клиента редактора. Плагин IntelliJ интегрирует
только live-диагностику и форматирование всего файла; hover,
автокомплит, document symbols и semantic tokens не интегрированы.
Подсветку синтаксиса обеспечивает нативный лексер.

- **Подсветка синтаксиса** — ключи, скаляры, raw-string-маркер (`::`),
  многострочные строки, комментарии
- **Парные скобки + автозакрытие** — `{}` `[]` `()`
- **Переключение комментария** — `Ctrl/Cmd+/` → `## comment`
- **Live-диагностика** (с LSP) — каждый `MissingSeparatorSpace`,
  дубликат ключа, конфликт dotted-префикса всплывает красной
  подчёркивающей линией на проблемной строке, с тем же сообщением,
  что выдаёт парсер
- **Hover-подсказки** (с LSP) — dotted-путь ключа под курсором,
  выведенный тип значения
- **Автокомплит** (с LSP) — ключевые слова (`null` / `true` / `false`),
  raw-string-маркер (`::`), открытие compound'ов
- **Document symbols** (с LSP) — outline отражает структуру
  `Value::Object`

>>>>> lang=zh
## Ktav 用户能得到什么

LSP 功能取决于编辑器的客户端。IntelliJ 插件仅集成实时诊断和
整文件格式化,未集成悬停、补全、文档符号或语义令牌。
其语法高亮由原生词法分析器提供。

- **语法高亮** —— 键、标量、原始字符串标记(`::`)、多行字符串、注释
- **括号匹配 + 自动闭合** —— `{}` `[]` `()`
- **注释切换** —— `Ctrl/Cmd+/` → `## comment`
- **实时诊断**(配合 LSP)—— 每一个 `MissingSeparatorSpace`、重复键、
  dotted-前缀冲突都会以红色波浪线显示在出错行上,信息与解析器发出的
  完全一致
- **悬停提示**(配合 LSP)—— 光标处键的 dotted 路径、值的推断类型
- **补全**(配合 LSP)—— 关键字(`null` / `true` / `false`)、原始字符串标记(`::`)、复合体开括号
- **文档符号**(配合 LSP)—— outline 反映 `Value::Object` 的结构

