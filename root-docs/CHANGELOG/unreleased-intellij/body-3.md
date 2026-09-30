>>>>> lang=en
- The highlighting lexer follows the spec: exact § 3.6 number grammar
  with the § 5.2 leading-zero exception (ASCII digits only — the old
  check used `Char.isDigit()` and accepted other scripts' digits),
  exact keywords, and the exact 25-code-point whitespace set of § 3.3.
- Quoted key segments (§ 5.3.3) are supported in whole-line and inline
  keys; an unterminated quote degrades to a plain value and keeps the
  incremental re-lexing state valid.
- `::` values are Strings inside inline compounds as well; an inline
  scalar with an escape is a String; a literal `:` inside an inline
  value is no longer taken for a separator.
- Fixed stale Marketplace/Settings copy: the comment-toggle description
  said `#` instead of `##` (`plugin.xml`, `build.gradle.kts`); the
  Settings → Tools → Ktav help text and `KtavLanguage`/`KtavConfigurable`
  KDoc claimed LSP features need the separate LSP4IJ plugin and that no
  binary is bundled — both were true only before this plugin grew its
  own built-in LSP client and per-platform bundled binaries. The
  Marketplace description's example also had `# ...` inline comments,
  which are not valid Ktav (`#` is ordinary content outside a leading
  `##`) — rewritten with standalone `##` lines.
- Lexer: the body of a multi-line `(` / `((` block is now opaque
  (`MULTILINE_TEXT`, closed by `)` / `))`) — before, `{`, `[` and
  `key: value` lines inside it were lexed as structure and a bare `(`
  array item was a bad character. A `\r` before `\n` is whitespace, so
  CRLF documents keep `true` and numbers typed.
- `KtavCorpusCoverageTest` runs the lexer over every valid fixture (no
  gaps or overlaps in the token stream; number, keyword and null counts
  match the fixture's expected value). Three older `KtavLexerTest`
  assertions compared against the platform's `TokenType.BAD_CHARACTER`
  instead of the plugin's own and could never fail — fixed.
- LSP synchronization now keeps one subscription per project/document
  session: shared documents update every owner independently, restored
  tabs use the normal open path, and close/reopen sends current content
  in a new session. Project disposal removes its listeners and closes
  its client without publishing an unfinished initialization or leaving
  a late-started server process alive.
  Closing the transport also fails pending requests and atomically
  rejects late request registration instead of waiting for a timeout.
- Formatting checks document identity, synchronized version/stamp,
  subscription and client ownership both before sending and during the
  final editor-thread write. Stale, cancelled and expired results do not
  overwrite edits or reopened documents; the applied text synchronizes
  to every owner and has a separate Undo step. Regression coverage uses
  real IntelliJ Documents and platform formatting callbacks, plus gated
  stdio child processes for delayed responses and disposal races; an
  opt-in smoke exercises the actual `ktav-lsp` binary.

>>>>> lang=ru
- Лексер подсветки следует спецификации: точная грамматика чисел § 3.6
  с исключением § 5.2 о ведущем нуле (только ASCII-цифры — прежняя
  проверка использовала `Char.isDigit()` и принимала цифры других
  письменностей), точные ключевые слова и точный набор из 25 кодовых
  точек пробелов § 3.3.
- Сегменты ключей в кавычках (§ 5.3.3) поддержаны в ключах строк и
  inline; незакрытая кавычка вырождается в обычное значение и не
  портит состояние инкрементального перелексинга.
- Значения `::` — строки и внутри inline-структур; inline-скаляр с
  экранированием — строка; литеральный `:` внутри inline-значения
  больше не принимается за разделитель.
- Исправлены устаревшие тексты Marketplace/Settings: описание
  переключения комментария называло `#` вместо `##` (`plugin.xml`,
  `build.gradle.kts`); текст справки Settings → Tools → Ktav и
  KDoc `KtavLanguage`/`KtavConfigurable` утверждали, что LSP-функции
  требуют отдельный плагин LSP4IJ и что бинарник не вложен — оба
  утверждения были верны только до того, как плагин обзавёлся
  собственным встроенным LSP-клиентом и вложенными бинарниками под
  каждую платформу. Пример в описании для Marketplace также содержал
  inline-комментарии `# ...`, которые не являются валидным Ktav (`#` —
  обычный символ вне ведущего `##`) — переписан на отдельные строки
  `##`.
- Лексер: тело многострочного блока `(` / `((` теперь непрозрачно
  (`MULTILINE_TEXT`, закрывается `)` / `))`) — раньше строки `{`, `[` и
  `key: value` внутри него лексились как структура, а одиночный элемент
  массива `(` давал bad character. `\r` перед `\n` — пробел, поэтому в
  документах с CRLF `true` и числа сохраняют тип.
- `KtavCorpusCoverageTest` прогоняет лексер по каждой valid-фикстуре (без
  дыр и перекрытий в потоке токенов; число чисел, ключевых слов и null
  совпадает с ожидаемым значением фикстуры). Три старых ассерта
  `KtavLexerTest` сравнивали с `TokenType.BAD_CHARACTER` платформы, а не
  с собственным типом плагина, и не могли упасть — исправлено.
- LSP-синхронизация теперь хранит одну подписку на сессию документа в
  каждом проекте: общие документы обновляют каждого владельца независимо,
  восстановленные вкладки используют обычный путь открытия, а повторное
  открытие отправляет текущий текст в новой сессии. Закрытие проекта
  удаляет его слушателей и закрывает клиент, не публикуя незавершённую
  инициализацию и не оставляя поздно запущенный процесс сервера.
  Закрытие транспорта также завершает ожидающие запросы с ошибкой и
  атомарно отклоняет позднюю регистрацию запросов вместо ожидания таймаута.
- Форматирование проверяет документ, синхронизированную версию/stamp,
  подписку и принадлежность клиента как перед отправкой, так и при
  итоговой записи в потоке редактора. Устаревшие, отменённые и просроченные
  результаты не затирают правки или повторно открытые документы; новый
  текст синхронизируется с каждым владельцем и имеет отдельный шаг Undo.
  Регрессионное покрытие использует реальные IntelliJ Documents и
  callback-и платформенного форматирования, а также дочерние stdio-процессы
  с управляемой задержкой для гонок ответа и закрытия; отдельный smoke
  запускается с настоящим бинарником `ktav-lsp`.

>>>>> lang=zh
- 高亮词法分析器遵循规范:严格的 § 3.6 数字语法并加入 § 5.2 前导零例外
  (仅 ASCII 数字——旧检查使用 `Char.isDigit()`,会接受其他文字的数字)、
  精确的关键字,以及 § 3.3 中精确的 25 个空白码位。
- 整行键与内联键都支持带引号的键片段(§ 5.3.3);未闭合的引号退化为
  普通值,并保持增量重新词法分析的状态有效。
- 内联复合值中 `::` 之后的值同样是字符串;含转义的内联标量为字符串;
  内联值中的字面 `:` 不再被当作分隔符。
- 修复了 Marketplace/Settings 中过时的文案:注释切换的描述写的是
  `#` 而非 `##`(`plugin.xml`、`build.gradle.kts`);Settings → Tools
  → Ktav 的帮助文本以及 `KtavLanguage`/`KtavConfigurable` 的 KDoc
  声称 LSP 功能需要单独安装 LSP4IJ 插件、且未内置二进制文件 ——
  这两点只在插件拥有自己内置的 LSP 客户端和按平台打包的二进制文件
  之前才成立。Marketplace 描述中的示例也含有 `# ...` 这类行内注释,
  这在 Ktav 中并不合法(`#` 若不在行首 `##` 之后即为普通内容)——
  已改写为独立的 `##` 行。
- 词法分析器:多行 `(` / `((` 块的正文现在是不透明的(`MULTILINE_TEXT`,
  由 `)` / `))` 关闭)——此前其中的 `{`、`[` 和 `key: value` 行会被当作
  结构处理,数组中单独的 `(` 条目会成为非法字符。`\n` 之前的 `\r` 视为
  空白,因此 CRLF 文档中的 `true` 和数字保持其类型。
- `KtavCorpusCoverageTest` 对每个 valid 样例运行词法分析器(token 流无空洞
  也无重叠;数字、关键字和 null 的数量与样例期望值一致)。三条较早的
  `KtavLexerTest` 断言比较的是平台的 `TokenType.BAD_CHARACTER` 而不是插件
  自己的类型,永远不会失败——已修复。
- LSP 同步现在按项目/文档会话保留一个订阅:共享文档分别更新每个拥有者,
  恢复的标签页使用正常的打开流程,重新打开时在新会话中发送当前内容。
  项目关闭会移除其监听器并关闭客户端,不会发布尚未完成初始化的客户端,
  也不会留下延迟启动的服务器进程。
  传输关闭时还会使等待中的请求失败,并原子地拒绝迟到的请求注册,
  而不是等待超时。
- 格式化在发送前和编辑器线程中的最终写入时,均检查文档身份、已同步
  版本/修改戳、订阅和客户端归属。旧的、已取消或过期的结果不会覆盖编辑
  或重新打开的文档;应用后的文本会同步到每个拥有者,并有独立的 Undo 步骤。
  回归覆盖使用真实 IntelliJ Documents 和平台格式化回调,以及可控制响应
  延迟的 stdio 子进程来验证响应和关闭竞态;可选的 smoke 使用真实
  `ktav-lsp` 二进制文件。

