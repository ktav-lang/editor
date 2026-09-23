>>>>> lang=en
### VS Code extension

- Explicit `DocumentFormattingEditProvider` registration (so
  `editor.defaultFormatter = ktav-lang.ktav` resolves correctly and
  VS Code does not prompt to install another formatter).
- `configurationDefaults` for `[ktav]`: tabSize 4, insertSpaces,
  defaultFormatter pinned to our extension.
- VSIX packaged with `vscode-languageclient` runtime tree included —
  fixes `Cannot find module 'vscode-languageclient/node'` activation
  failure introduced by an earlier `--no-dependencies` packaging.

>>>>> lang=ru
### VS Code extension

- Явная регистрация `DocumentFormattingEditProvider` (чтобы
  `editor.defaultFormatter = ktav-lang.ktav` резолвился корректно и
  VS Code не предлагал поставить другой форматтер).
- `configurationDefaults` для `[ktav]`: tabSize 4, insertSpaces,
  defaultFormatter зафиксирован на нашем расширении.
- VSIX упакован вместе с рантайм-деревом `vscode-languageclient` —
  исправляет сбой активации
  `Cannot find module 'vscode-languageclient/node'`, внесённый
  предыдущей упаковкой с `--no-dependencies`.

>>>>> lang=zh
### VS Code extension

- 显式注册 `DocumentFormattingEditProvider`(使
  `editor.defaultFormatter = ktav-lang.ktav` 能正确解析,VS Code 也
  不再提示安装其他格式化工具)。
- 为 `[ktav]` 设置 `configurationDefaults`:tabSize 4、insertSpaces,
  默认格式化器固定为本扩展。
- VSIX 打包时包含 `vscode-languageclient` 运行时目录 —— 修复了此前
  `--no-dependencies` 打包引入的
  `Cannot find module 'vscode-languageclient/node'` 激活失败。

