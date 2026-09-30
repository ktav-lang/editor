>>>>> lang=en
## [0.8.0] — 2026-09-28

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- Source builds require Rust 1.88 for the locked dependency graph;
  CI builds all targets on that declared minimum as well as stable.
- Completion keeps separator whitespace and raw marker placement valid,
  including UTF-8/UTF-16 positions, BOM and LF/CR/CRLF. Pair suggestions do
  not rewrite Array strings, multiline bodies or existing values.
- Document symbol declaration ranges contain their values, nested children and
  compound/multiline closers. Separate selections retain exact first-source
  navigation anchors, including reopened dotted prefixes and array items.
- Exact § 3.6 / § 5.2 scalar classification (`01234`, `0_7`, `1_`,
  `2026-09-28` are Strings) and the exact § 3.3 whitespace set.
- Inline compounds: `::` values stay raw Strings; quoted key segments
  are opaque.
- Hover no longer crashes on long non-ASCII strings and resolves nested,
  quoted and escaped keys.
- The spec conformance test covers every corpus category and fails
  without the submodule.
- Semantic tokens: multi-line string blocks (`(` / `((`) no longer leak
  ordinary line-shape rules onto their content lines, and an inline key
  escaping a structural byte after a comma no longer desyncs the
  scanner. A new corpus-wide test guards both.
- LF, CR and CRLF documents behave the same in every handler; the
  formatter no longer treats a single `#` as a comment or `key:: ((` as a
  block opener; a leading BOM is ignored; symbols get ranges for inline
  keys. Corpus tests now also cover symbols, hover, tokens and
  formatting — see the root [`CHANGELOG.md`](../CHANGELOG.md).

>>>>> lang=ru
## [0.8.0] — 2026-09-28

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — полный
список см. в корневом [`CHANGELOG.md`](../../CHANGELOG.md) (раздел 0.8.0).

- Закреплённый граф зависимостей требует Rust 1.88 для сборки исходников;
  CI собирает все targets на объявленном минимуме и сохраняет stable-проверки.
- Completion сохраняет допустимые пробелы разделителя и позицию raw-маркера
  с учётом UTF-8/UTF-16, BOM и LF/CR/CRLF. Pair-подстановки не переписывают
  строки Array, многострочное содержимое или уже введённые значения.
- Диапазоны объявлений DocumentSymbol включают значения, дочерние символы
  и закрывающие скобки контейнеров/многострочных строк. Отдельные selection
  сохраняют точную первую навигационную точку, включая повторные dotted-префиксы
  и элементы массивов.
- Точная классификация скаляров по § 3.6 / § 5.2 (`01234`, `0_7`, `1_`,
  `2026-09-28` — строки) и точный набор пробелов § 3.3.
- Inline-структуры: значения `::` остаются сырыми строками; сегменты
  ключей в кавычках непрозрачны.
- Hover больше не падает на длинных не-ASCII строках и находит
  вложенные ключи, ключи в кавычках и с экранированием.
- Тест соответствия спецификации покрывает все категории корпуса и
  падает без подмодуля.
- Semantic tokens: многострочные строковые блоки (`(` / `((`) больше не
  протекают обычными построчными правилами в свои строки содержимого, а
  экранирование структурного байта после запятой в inline-ключе больше
  не десинхронизирует сканер. Новый тест по всему корпусу защищает оба
  случая.
- Документы с LF, CR и CRLF ведут себя одинаково во всех обработчиках;
  форматтер больше не считает одиночный `#` комментарием, а `key:: ((` —
  открывающей строкой блока; ведущий BOM игнорируется; у symbols есть
  диапазоны для inline-ключей. Тесты корпуса теперь покрывают и symbols,
  hover, токены и форматирование — см. корневой
  [`CHANGELOG.md`](../../CHANGELOG.md).

>>>>> lang=zh
## [0.8.0] — 2026-09-28

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- 锁定依赖图的源码构建需要 Rust 1.88；CI 在声明的最低版本及 stable 上
  构建和检查所有 targets。
- 补全正确保留分隔空白及 raw 标记位置，支持 UTF-8/UTF-16、BOM 和
  LF/CR/CRLF；键值对建议不会改写 Array 字符串、多行正文或已存在值。
- 文档符号声明范围包含值、子符号及容器/多行闭合符。独立 selection 保留
  精确的首次导航定位，涵盖重新打开的点状前缀与数组项。
- 按 § 3.6 / § 5.2 精确分类标量(`01234`、`0_7`、`1_`、`2026-09-28`
  为字符串),并使用 § 3.3 精确的空白集合。
- 内联复合值:`::` 之后的值保持为原始字符串;带引号的键片段不透明。
- Hover 不再因长的非 ASCII 字符串而崩溃,并能解析嵌套键、带引号和带转义的键。
- 规范一致性测试覆盖语料库的所有类别,缺少子模块时会失败。
- Semantic tokens:多行字符串块(`(` / `((`)的内容行不再受普通逐行规则
  影响;inline 键中逗号之后的结构字节转义也不再使扫描器失步。新增覆盖
  整个语料库的测试防止两者回归。
- LF、CR 和 CRLF 文档在所有处理器中表现一致;格式化器不再把单个 `#`
  当作注释,也不再把 `key:: ((` 当作块起始行;行首 BOM 被忽略;符号对
  内联键有了范围。语料库测试现在还覆盖符号、hover、token 和格式化——见
  根目录 [`CHANGELOG.md`](../../CHANGELOG.md)。

