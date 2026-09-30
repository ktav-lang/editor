>>>>> lang=en
## Features

- Native syntax highlighting for `.ktav` files (own lexer, no TextMate).
  Incremental highlighting preserves the first-content root kind and nested
  Object/Array context; quotes inside multiword bare keys remain literal.
- Folding for multiline compounds and strings uses the same lexer tokens,
  leaving raw scalars and multiline string bodies opaque.
- Comment toggle (`Ctrl/Cmd+/`) prepends `## ` per the Ktav spec.
- Bracket matching and auto-closing for `{}` `[]` `()`.
- File icon and File → New → Ktav file (icon TODO; uses the platform
  default text-file icon for now).

>>>>> lang=ru
## Возможности

- Нативная подсветка синтаксиса для `.ktav` файлов (собственный лексер,
  без TextMate). Инкрементальная подсветка сохраняет тип корня по первой
  содержательной строке и контекст вложенных Object/Array; кавычки внутри
  многословных некавыченных ключей остаются обычными символами.
- Сворачивание многострочных контейнеров и строк использует те же токены
  лексера; raw-значения и содержимое многострочных строк остаются непрозрачными.
- Переключение комментария (`Ctrl/Cmd+/`) добавляет `## ` согласно
  спецификации Ktav.
- Парные скобки и автозакрытие для `{}` `[]` `()`.
- Иконка файла и File → New → Ktav file (иконка TODO; пока используется
  стандартная иконка текстового файла платформы).

>>>>> lang=zh
## 功能

- 原生语法高亮支持 `.ktav` 文件(自有词法分析器,不依赖 TextMate)。
  增量高亮保留首个内容行确定的根类型和嵌套 Object/Array 上下文；
  多词裸键内部的引号仍视为普通字符。
- 多行容器和字符串的折叠使用相同的词法标记，不解析 raw 标量或多行字符串正文。
- 注释切换(`Ctrl/Cmd+/`)按 Ktav 规范在行首添加 `## `。
- `{}` `[]` `()` 的括号匹配与自动闭合。
- 文件图标与 File → New → Ktav file(图标 TODO;暂时使用平台
  默认的文本文件图标)。

