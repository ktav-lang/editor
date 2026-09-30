>>>>> lang=en
### LSP features

The plugin talks to [`ktav-lsp`](../lsp) through its own built-in LSP
client — no separate LSP plugin (e.g. LSP4IJ) is required for live
diagnostics and whole-file formatting (**Reformat Code**). These are
the LSP features currently integrated into the IntelliJ plugin.

The server also supports hover, completion, document symbols and
semantic tokens, but the built-in IntelliJ client does not integrate
them. Syntax highlighting comes from the plugin's native lexer, not
LSP semantic tokens.

Diagnostics are tied to each project's current open client, version and
document session. Cached annotations and direct editor highlights remain
independent: stale publications or closing one project cannot clear another
owner's errors.

Each project owns its LSP client and document subscriptions. A `.ktav`
document open in two projects is synchronized to both independently;
closing it in one project does not stop updates in the other. Restored
editor tabs use the same open path, and reopening sends the current text
as a new document session. Project disposal removes its subscriptions
and closes its client, including a server process that starts late.
Closing the transport fails outstanding requests and rejects new ones
immediately instead of leaving them waiting for a response timeout.

>>>>> lang=ru
### LSP-фичи

Плагин общается с [`ktav-lsp`](../../lsp) через собственный встроенный
LSP-клиент — для live-диагностики и форматирования всего файла
(**Reformat Code**) отдельный LSP-плагин (например, LSP4IJ) не требуется.
Это LSP-функции, сейчас интегрированные в плагин IntelliJ.

Сервер также поддерживает hover, автокомплит, document symbols и
semantic tokens, но встроенный клиент IntelliJ их не интегрирует.
Подсветку синтаксиса обеспечивает нативный лексер плагина, а не
семантические токены LSP.

Диагностика привязана к текущему открытому клиенту, версии и сессии документа
каждого проекта. Кэш аннотаций и прямая подсветка редакторов независимы:
старые публикации или закрытие одного проекта не удаляют ошибки другого
владельца.

Каждый проект владеет своим LSP-клиентом и подписками документов. Документ
`.ktav`, открытый в двух проектах, синхронизируется с каждым независимо;
закрытие в одном проекте не останавливает обновления в другом.
Восстановленные вкладки используют тот же путь открытия, а повторное
открытие отправляет текущий текст как новую сессию документа. Закрытие
проекта удаляет его подписки и закрывает клиент, включая процесс сервера,
запустившийся с задержкой.
Закрытие транспорта завершает ожидающие запросы с ошибкой и сразу
отклоняет новые, не оставляя их ждать таймаута ответа.

>>>>> lang=zh
### LSP 功能

插件通过自带的内置 LSP 客户端与 [`ktav-lsp`](../../lsp) 通信 ——
实时诊断和整文件格式化(**Reformat Code**)无需安装单独的 LSP
插件(例如 LSP4IJ)。这是 IntelliJ 插件目前集成的 LSP 功能。

服务器还支持悬停、补全、文档符号和语义令牌,但内置 IntelliJ
客户端未集成这些功能。语法高亮由插件的原生词法分析器提供,
而非 LSP 语义令牌。

诊断绑定每个项目当前打开的客户端、版本及文档会话。缓存注解和编辑器
直接高亮保持独立：过期发布或关闭一个项目不会清除另一个拥有者的错误。

每个项目拥有自己的 LSP 客户端和文档订阅。同一 `.ktav` 文档在两个项目中
打开时,会分别同步到两个客户端;在一个项目中关闭它不会停止另一个项目的
更新。恢复的编辑器标签页使用相同的打开流程,重新打开时会将当前文本作为
新的文档会话发送。项目关闭会移除其订阅并关闭客户端,包括延迟启动的
服务器进程。
传输关闭时,等待中的请求会失败,新请求也会立即被拒绝,而不会继续等待
响应超时。

