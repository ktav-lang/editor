>>>>> lang=en
## Local testing

### VS Code

1. From a VS Code window, open the Command Palette and run
   `Developer: Inspect Editor Tokens and Scopes`.
2. Open any sample from `spec/versions/0.8/tests/valid/**/*.ktav`.
3. Click into a token; the panel shows the resolved scope chain. Each
   scope listed in the "Token classes" section below should appear on
   the corresponding token.

For an end-to-end check, install the `vscode/` extension via
`code --install-extension` (or `F5` from the `vscode/` workspace) and
visually verify that comments, keys, separators, markers, scalars,
keywords, and brackets all render distinctly under your color theme.

### Other editors

Direct use of `ktav.tmLanguage.json` requires an editor that accepts
TextMate grammars in JSON format, such as VS Code; TextMate support
alone does not guarantee that. Follow the host's grammar-registration
instructions and associate `*.ktav` with scope `source.ktav`.
Sublime Text requires XML: use the existing
`grammars/scripts/export-tmlanguage.js` exporter and follow the
[Sublime installation recipe](../docs/sublime.md#file-type-association).
Tree-sitter requires a separate Ktav grammar and highlight queries;
the shared TextMate JSON cannot be installed as a tree-sitter grammar.

>>>>> lang=ru
## Локальное тестирование

### VS Code

1. В окне VS Code откройте Command Palette и выполните
   `Developer: Inspect Editor Tokens and Scopes`.
2. Откройте любой пример из `spec/versions/0.8/tests/valid/**/*.ktav`.
3. Щёлкните по токену; панель покажет разрешённую цепочку scope. Каждый
   scope из раздела «Классы токенов» ниже должен появляться на
   соответствующем токене.

Для сквозной проверки установите расширение `vscode/` через
`code --install-extension` (или `F5` из workspace `vscode/`) и визуально
убедитесь, что комментарии, ключи, разделители, маркеры, скаляры,
ключевые слова и скобки отображаются различимо в вашей цветовой теме.

### Другие редакторы

Для прямого использования `ktav.tmLanguage.json` редактор должен
принимать TextMate-грамматики в формате JSON, как VS Code; одной
поддержки TextMate недостаточно. Следуйте инструкции хоста по регистрации
грамматики и свяжите `*.ktav` со scope `source.ktav`.
Sublime Text нужен XML: используйте существующий экспортёр
`grammars/scripts/export-tmlanguage.js` и
[инструкцию Sublime](../../docs/i18n/editors/ru/sublime.md#привязка-типа-файла).
Tree-sitter нужны отдельные грамматика Ktav и запросы подсветки;
общую TextMate JSON нельзя установить как tree-sitter-грамматику.

>>>>> lang=zh
## 本地测试

### VS Code

1. 在 VS Code 窗口中打开命令面板,运行
   `Developer: Inspect Editor Tokens and Scopes`。
2. 打开 `spec/versions/0.8/tests/valid/**/*.ktav` 中的任意示例。
3. 点击某个 token;面板会显示解析出的 scope 链。下文"Token 类别"一节
   列出的每个 scope 都应出现在对应的 token 上。

如需端到端检查,可通过 `code --install-extension` 安装 `vscode/` 扩展
(或在 `vscode/` 工作区中按 `F5`),并目视确认注释、键、分隔符、标记、
标量、关键字和括号在你的配色主题下各自显示不同。

### 其他编辑器

直接使用 `ktav.tmLanguage.json` 需要编辑器接受 JSON 格式的 TextMate
语法,例如 VS Code;仅支持 TextMate 并不能保证这一点。请按宿主的语法
注册说明操作,并将 `*.ktav` 关联到 scope `source.ktav`。
Sublime Text 需要 XML:请使用现有的
`grammars/scripts/export-tmlanguage.js` 导出器,并遵循
[Sublime 安装步骤](../../docs/i18n/editors/zh/sublime.md#文件类型关联)。
Tree-sitter 需要独立的 Ktav 语法和高亮查询;
共享的 TextMate JSON 不能作为 tree-sitter 语法安装。

