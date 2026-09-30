>>>>> lang=en
## [0.8.0] — 2026-09-30

All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
plugin) move to 0.8.0 in step with the `ktav` crate and the
specification — see the root [`CHANGELOG.md`](../CHANGELOG.md)
(0.8.0 section) for the full list.

- TextMate grammar: exact § 3.6 / § 5.2 number scopes, surrogate-pair
  aware `\uXXXX`, raw `::` values and quoted keys inside inline objects,
  and a fix for classification that failed off column 0.
- A real tokenizer test (`npm run test:unit`) now runs the grammar
  over the vectors.
- The packaged extension now includes its production dependencies
  (`vscode-languageclient`); the 0.6.1 VSIX was built without them.
- TextMate grammar: a top-level single-line inline object or array is
  highlighted; a corpus-wide tokenizer test covers every valid fixture.
- TextMate keeps implicit root Array context after scalar items and nested
  compounds. Legal hash/multiword keys and positional quotes retain exact
  key/value spans, covered by stateful tokenizer regressions.
- Works in Restricted Mode (`ktav.server.path` from an untrusted
  workspace is ignored), and Ktav files no longer box non-ASCII text
  such as Cyrillic keys or `§`, including a tab restored at window start.
- New default palette: green keys with a soft underline (theme colour
  `ktav.keyUnderline`), bold quoted-key content between magenta quotes,
  gray `##` comments; the paired quote of a quoted key is highlighted
  like a matching bracket.

>>>>> lang=ru
## [0.8.0] — 2026-09-30

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — полный
список см. в корневом [`CHANGELOG.md`](../../CHANGELOG.md) (раздел 0.8.0).

- Грамматика TextMate: точные scope чисел § 3.6 / § 5.2, `\uXXXX` с
  учётом суррогатных пар, сырые значения `::` и ключи в кавычках внутри
  inline-объектов, а также исправление классификации, не работавшей вне
  колонки 0.
- Настоящий тест токенизатора (`npm run test:unit`) теперь прогоняет
  грамматику по этим векторам.
- Упакованное расширение теперь включает production-зависимости
  (`vscode-languageclient`); VSIX 0.6.1 был собран без них.
- Грамматика TextMate: однострочный inline-объект или массив верхнего
  уровня теперь подсвечивается; тест токенизатора по всему корпусу
  покрывает каждую valid-фикстуру.
- TextMate сохраняет контекст неявного корневого Array после скалярных
  элементов и вложенных контейнеров. Допустимые ключи с `#`, многословные
  ключи и позиционные кавычки сохраняют точные границы ключей и значений;
  это проверяют регрессии токенизатора с состоянием.
- Работает в Restricted Mode (`ktav.server.path` из недоверенного
  workspace игнорируется), а файлы Ktav больше не обводят рамкой не-ASCII
  текст вроде кириллических ключей или `§`, в том числе во вкладке,
  восстановленной при старте окна.
- Новая палитра по умолчанию: зелёные ключи с мягким подчёркиванием
  (цвет темы `ktav.keyUnderline`), жирное содержимое ключей в кавычках
  между пурпурными кавычками, серые комментарии `##`; парная кавычка
  ключа подсвечивается как парная скобка.

>>>>> lang=zh
## [0.8.0] — 2026-09-30

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- TextMate 语法:精确的 § 3.6 / § 5.2 数字 scope、识别代理对的
  `\uXXXX`、内联对象中的原始 `::` 值和带引号的键,并修复了在非第 0 列时
  失效的分类。
- 新增真实的分词器测试(`npm run test:unit`),用这些向量运行语法。
- 打包后的扩展现在包含其生产依赖(`vscode-languageclient`);0.6.1 的
  VSIX 构建时缺少它们。
- TextMate 语法:顶层单行内联对象或数组现在会被高亮;覆盖整个语料库的
  分词器测试涵盖每个 valid 样例。
- TextMate 在标量元素及嵌套容器之后保留隐式根 Array 上下文。
  合法的 `#`/多词键及位置性引号保留精确键/值范围，并有带状态分词回归覆盖。
- 可在受限模式下工作(忽略来自不受信任工作区的 `ktav.server.path`),Ktav
  文件不再为西里尔字母键或 `§` 等非 ASCII 文本加框,包括窗口启动时恢复的标签页。
- 新的默认配色:绿色键带柔和下划线(主题颜色 `ktav.keyUnderline`),带引号键的
  内容加粗、引号为品红色,`##` 注释为灰色;带引号键的配对引号会像匹配括号一样高亮。

