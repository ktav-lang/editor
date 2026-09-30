>>>>> lang=en
Formatting applies only to the synchronized document session that
requested it. Edits (even undoing back to the original text), close/reopen,
project/client disposal, cancellation or an expired result prevent a
stale response from replacing the document. A successful result is
applied and checked together on the editor thread, is synchronized back
to each owner, and has its own **Undo** step.

The server binary is discovered in this order:

1. The explicit path configured under **Settings → Tools → Ktav**.
2. The binary bundled inside the plugin distribution at
   `lib/bin/<platform>-<arch>/ktav-lsp`, one per supported platform.
3. `ktav-lsp` resolved via your shell `PATH` — install it with
   `cargo install ktav-lsp --version 0.8.0 --locked` (matches the VS Code extension's
   discovery order).

>>>>> lang=ru
Форматирование применяется только к синхронизированной сессии документа,
из которой поступил запрос. Правки (даже возврат к исходному тексту через
Undo), закрытие/повторное открытие, закрытие проекта/клиента, отмена или
истечение срока результата не позволяют устаревшему ответу заменить
документ. Успешный результат проверяется и применяется вместе в потоке
редактора, синхронизируется с каждым владельцем и имеет отдельный шаг
**Undo**.

Бинарник сервера ищется в таком порядке:

1. Явный путь, заданный в **Settings → Tools → Ktav**.
2. Бинарник, вложенный в дистрибутив плагина по пути
   `lib/bin/<platform>-<arch>/ktav-lsp` — по одному на каждую
   поддерживаемую платформу.
3. `ktav-lsp`, найденный через `PATH` вашей оболочки — установите
   его командой `cargo install ktav-lsp --version 0.8.0 --locked` (совпадает с порядком
   поиска в расширении VS Code).

>>>>> lang=zh
格式化只应用于发起请求时已同步的文档会话。编辑(即使通过 Undo 恢复为
原始文本)、关闭/重新打开、项目/客户端关闭、取消或结果过期,都会阻止
旧响应替换文档。成功的结果会在编辑器线程中一并检查和应用,同步回每个
拥有该文档的项目,并具有独立的 **Undo** 步骤。

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 打包在插件分发包中的二进制
   `lib/bin/<platform>-<arch>/ktav-lsp`,每个受支持平台各一份。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp --version 0.8.0 --locked` 安装(与 VS Code 扩展的查找顺序一致)。

