>>>>> lang=en
## Where it is used

These two JSON files are the canonical TextMate and VS Code configuration
artifacts:

- `vscode/` — bundles them via `package.json` `contributes.languages`
  and `contributes.grammars`.
- `intellij/` — uses its own `KtavLexer` and
  `KtavSyntaxHighlighterFactory` for native highlighting; it does not load
  this TextMate grammar.

Grammar changes reach the VS Code extension on its next build / packaging
step. Update the IntelliJ lexer separately when the same behavior is needed.

>>>>> lang=ru
## Где используется

Эти два JSON-файла — канонические артефакты TextMate и конфигурации VS Code:

- `vscode/` — включает их через `contributes.languages` и
  `contributes.grammars` в `package.json`.
- `intellij/` — использует собственные `KtavLexer` и
  `KtavSyntaxHighlighterFactory` для нативной подсветки; TextMate-грамматика
  здесь не загружается.

Изменения грамматики попадают в расширение VS Code при следующей сборке /
упаковке. При необходимости такое же поведение в лексере IntelliJ нужно
обновить отдельно.

>>>>> lang=zh
## 使用位置

这两个 JSON 文件是 TextMate 语法和 VS Code 语言配置的规范产物:

- `vscode/` — 通过 `package.json` 的 `contributes.languages` 与
  `contributes.grammars` 打包它们。
- `intellij/` — 使用自己的 `KtavLexer` 和 `KtavSyntaxHighlighterFactory`
  进行原生语法高亮,不加载此 TextMate 语法。

此处的语法修改会在下次构建 / 打包时进入 VS Code 扩展。若 IntelliJ
词法分析器也需要相同行为,必须单独更新。

