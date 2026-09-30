>>>>> lang=en
## Highlighting

`ktav-mode` above is intentionally minimal (no font-lock keywords).
The server offers semantic tokens, but displaying them depends on the
installed Emacs LSP client's support and configuration; attaching eglot
does not add font-lock rules to this mode.
Offline highlighting needs separately implemented Ktav font-lock rules,
or a Ktav tree-sitter grammar, highlight queries and a major mode that
uses them. This repository supplies none of those highlighting
integrations. The shared TextMate JSON is not a tree-sitter grammar;
neither tree-sitter nor polymode loads it as one.
>>>>> lang=ru
## Подсветка

`ktav-mode` выше намеренно минимален (без font-lock keywords).
Сервер предоставляет semantic tokens, но их отображение зависит от
поддержки и настройки установленного LSP-клиента Emacs; подключение
eglot не добавляет правила font-lock в этот mode.
Для офлайн-подсветки нужны отдельно реализованные правила font-lock для
Ktav либо tree-sitter-грамматика Ktav, запросы подсветки и major mode,
который их использует. В репозитории этих интеграций подсветки нет.
Общая TextMate JSON не является tree-sitter-грамматикой;
ни tree-sitter, ни polymode не загружают её как такую грамматику.
>>>>> lang=zh
## 高亮

上面的 `ktav-mode` 有意保持极简(没有 font-lock 关键字)。
服务器提供语义令牌,但显示它们取决于已安装的 Emacs LSP 客户端的支持
和配置;接入 eglot 不会为此 mode 添加 font-lock 规则。
离线高亮需要另行实现 Ktav font-lock 规则,或 Ktav tree-sitter 语法、
高亮查询和使用它们的 major mode。本仓库不提供这些高亮集成。
共享的 TextMate JSON 不是 tree-sitter 语法;
tree-sitter 和 polymode 都不能将其作为此类语法加载。
