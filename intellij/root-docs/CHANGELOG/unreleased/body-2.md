>>>>> lang=en
- Lexer: multi-line `(` / `((` block bodies are opaque and CRLF no longer
  breaks value typing; a corpus-wide lexer test covers every valid
  fixture.
- LSP document subscriptions are owned per project, including shared
  documents, restored tabs and close/reopen. Disposal removes only that
  project's listeners; initialization publishes only a ready client
  and safely stops a server process created during disposal.
  Transport closure fails pending requests and immediately rejects
  requests racing with disposal.
- Formatting checks the synchronized document/version/session and client
  again when applying the result, so intervening edits, close/reopen,
  disposal, cancellation and expired responses cannot overwrite current
  text. Application is atomic with the final check and has a separate
  Undo step; the applied text is sent to every subscribed project.
- Keys are green with a soft underline in both colour schemes; quoted
  key content is bold between magenta quotes, and the paired quote is
  highlighted like a matching brace when the caret touches either quote.

>>>>> lang=ru
- Лексер: тела многострочных блоков `(` / `((` непрозрачны, а CRLF больше
  не ломает типизацию значений; тест лексера по всему корпусу покрывает
  каждую valid-фикстуру.
- LSP-подписки документов принадлежат конкретному проекту, включая общие
  документы, восстановленные вкладки и закрытие/повторное открытие.
  Закрытие удаляет только слушателей этого проекта; инициализация
  публикует только готовый клиент и безопасно останавливает процесс
  сервера, созданный во время закрытия проекта.
  Закрытие транспорта завершает ожидающие запросы с ошибкой и сразу
  отклоняет запросы, конкурирующие с закрытием.
- Форматирование повторно проверяет синхронизированный документ, версию,
  сессию и клиент при применении результата: правки, закрытие/повторное
  открытие, закрытие проекта/клиента, отмена и просроченные ответы не
  могут затереть текущий текст. Применение атомарно с итоговой проверкой,
  имеет отдельный шаг Undo, а новый текст отправляется каждому проекту
  с активной подпиской.
- Ключи зелёные с мягким подчёркиванием в обеих цветовых схемах;
  содержимое ключа в кавычках жирное, кавычки пурпурные, а парная кавычка
  подсвечивается как парная скобка, когда курсор стоит у любой из них.

>>>>> lang=zh
- 词法分析器:多行 `(` / `((` 块的正文是不透明的,CRLF 不再破坏值的
  类型判断;覆盖整个语料库的词法分析器测试涵盖每个 valid 样例。
- LSP 文档订阅按项目管理,涵盖共享文档、恢复的标签页和关闭/重新打开。
  项目关闭只移除本项目的监听器;初始化只发布就绪的客户端,并安全终止
  在项目关闭期间创建的服务器进程。
  传输关闭时会使等待中的请求失败,并立即拒绝与关闭操作竞争的新请求。
- 格式化在应用结果时再次检查已同步的文档、版本、会话和客户端,避免
  中间发生的编辑、关闭/重新打开、项目/客户端关闭、取消或过期响应
  覆盖当前文本。应用与最终检查是原子的,并有独立的 Undo 步骤;新文本
  会发送到每个订阅该文档的项目。
- 两种配色方案中键均为绿色并带柔和下划线;带引号键的内容加粗、引号为品红色,
  光标位于任一引号旁时配对引号会像匹配括号一样高亮。

