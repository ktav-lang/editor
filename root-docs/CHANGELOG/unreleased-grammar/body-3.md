>>>>> lang=en
- Quoted key segments are now three tokens: the opening and closing quote
  (`punctuation.definition.string.begin/end.ktav`) around the content, so
  the paired quotes can be coloured. `ktav-lsp` marks quoted key content
  with a new `quoted` semantic-token modifier and leaves the quotes to the
  grammar.
- VS Code: the extension works in Restricted Mode (`ktav.server.path` from
  an untrusted workspace is ignored), Ktav files no longer box non-ASCII
  text such as Cyrillic keys or `§`, and the default palette makes bare
  keys, quoted keys, their quotes and `##` comments (gray) distinct.
- Word selection and occurrence highlighting stop at quote characters, so
  double-clicking a quoted key selects its text without the quotes.
- VS Code: keys are green with a soft underline (theme colour
  `ktav.keyUnderline`) so they never read as values, and
  the paired quote of a quoted key is highlighted like a matching bracket
  when the cursor touches either quote.
- VS Code: a Ktav editor restored at window start in Restricted Mode no
  longer keeps the non-ASCII boxes VS Code drew before the extension's
  defaults applied; the extension re-opens it in place once, as switching
  tabs would.

>>>>> lang=ru
- Сегменты ключа в кавычках теперь три токена: открывающая и закрывающая
  кавычки (`punctuation.definition.string.begin/end.ktav`) вокруг
  содержимого, чтобы парные кавычки можно было подсветить. `ktav-lsp`
  помечает содержимое ключа в кавычках новым модификатором семантических
  токенов `quoted`, а кавычки оставляет грамматике.
- VS Code: расширение работает в Restricted Mode (`ktav.server.path` из
  недоверенного workspace игнорируется), файлы Ktav больше не обводят
  рамкой не-ASCII текст вроде кириллических ключей или `§`, а палитра по
  умолчанию различает обычные ключи, ключи в кавычках, их кавычки и
  комментарии `##` (серые).
- Выделение слова и подсветка вхождений останавливаются на кавычках:
  двойной щелчок по ключу в кавычках выделяет текст без кавычек.
- VS Code: ключи зелёные с мягким подчёркиванием (цвет темы
  `ktav.keyUnderline`), чтобы не путаться со значениями, а
  парная кавычка ключа подсвечивается как парная скобка, когда курсор стоит
  у любой из кавычек.
- VS Code: редактор Ktav, восстановленный при старте окна в Restricted
  Mode, больше не сохраняет рамки вокруг не-ASCII текста, которые VS Code нарисовал до
  применения настроек расширения: расширение один раз переоткрывает его на
  месте, как при переключении вкладки.

>>>>> lang=zh
- 带引号的键段现在是三个 token:内容两侧的开引号和闭引号(`punctuation.definition.string.begin/end.ktav`),以便为成对的引号着色。`ktav-lsp` 用新的 `quoted` 语义 token 修饰符标记带引号键的内容,引号交给语法处理。
- VS Code:扩展可在受限模式下工作(忽略来自不受信任工作区的 `ktav.server.path`),Ktav 文件不再为西里尔字母键或 `§` 等非 ASCII 文本加框,默认配色区分裸键、带引号的键及其引号和 `##` 注释(灰色)。
- 单词选择和出现位置高亮在引号处停止,双击带引号的键只选中其文本、不含引号。
- VS Code:键为绿色并带柔和下划线(主题颜色 `ktav.keyUnderline`),不会被误认为值;光标位于带引号键的任一引号旁时,其配对引号会像匹配括号一样高亮。
- VS Code:受限模式下窗口启动时恢复的 Ktav 编辑器不再保留 VS Code 在扩展默认设置生效前绘制的非 ASCII 方框;扩展会像切换标签页一样就地重新打开它一次。

