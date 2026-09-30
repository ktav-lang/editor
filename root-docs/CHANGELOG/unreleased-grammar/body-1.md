>>>>> lang=en
### TextMate grammar (VS Code + shared `grammars/`)

- TextMate retains the first-content Object/Array context across lines and
  nested compounds; pair-shaped Array strings no longer open false blocks.
- Bare `#`, multiword keys, trimmed dotted segments and positional quotes
  keep exact key/value scopes. Theme documentation names the actual Boolean
  and Null scopes. A dependency-free exporter produces supported XML
  `.tmLanguage` for Sublime's `source.ktav` syntax.

>>>>> lang=ru
### TextMate grammar (VS Code + shared `grammars/`)

- TextMate сохраняет контекст Object/Array по первой содержательной строке
  между строками и вложенными контейнерами; похожие на пары строки Array
  больше не открывают ложные блоки.
- Некавыченный `#`, многословные ключи, пробелы по краям точечных сегментов
  и позиционные кавычки сохраняют точные scope ключей и значений.
  Документация тем указывает реальные scope Boolean и Null; экспортёр без
  зависимостей создаёт XML `.tmLanguage` для синтаксиса Sublime `source.ktav`.

>>>>> lang=zh
### TextMate grammar (VS Code + shared `grammars/`)

- TextMate 按首个内容行保留跨行及嵌套容器的 Object/Array 上下文；
  形似键值对的 Array 字符串不再错误地打开块。
- 裸 `#`、多词键、点分片段边缘空白与位置性引号保留精确的键/值 scope。
  主题文档列出实际的 Boolean 和 Null scope；无需额外依赖的导出器可生成
  Sublime `source.ktav` 语法支持的 XML `.tmLanguage`。

