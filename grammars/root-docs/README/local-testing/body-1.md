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

Any editor with TextMate-grammar support can consume
`ktav.tmLanguage.json` directly. Drop it into a TextMate bundle (or
the editor's grammar directory) and associate `*.ktav` with scope
`source.ktav`.

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

Любой редактор с поддержкой TextMate-грамматик может использовать
`ktav.tmLanguage.json` напрямую. Положите его в TextMate-бандл (или в
каталог грамматик редактора) и свяжите `*.ktav` со scope `source.ktav`.

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

任何支持 TextMate 语法的编辑器都可以直接使用 `ktav.tmLanguage.json`。
把它放进 TextMate 包(或编辑器的语法目录),并将 `*.ktav` 关联到
scope `source.ktav`。

