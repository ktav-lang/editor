>>>>> lang=en
### Repository and release tooling

- The prebuilt `ktav-lsp` binaries are no longer committed: the
  tracked copies had drifted (five platforms embedded `ktav` 0.5.0,
  win32-x64 embedded 0.7.1, and a stray copy under
  `intellij/src/main/resources/bin/` embedded 0.1.5). `intellij/bin/`,
  `vscode/bin/` and `intellij/src/main/resources/bin/` are git-ignored;
  the release workflow builds all six platforms from source, and
  `scripts/build-binaries.sh` does the same locally.
- The TextMate tokenizer test adds pinned dev-only dependencies
  (`vscode-textmate`, `vscode-oniguruma`) to the VS Code project — not
  shipped in the VSIX — and `tsconfig.json` gains the `DOM` lib for
  their WebAssembly typings.
- Release workflow: `vsce package` / `vsce publish` no longer pass
  `--no-dependencies`. The extension needs `vscode-languageclient` at
  runtime and the 0.6.1 VSIX shipped without `node_modules`, so the
  packed extension could not load its language client; production
  dependencies are now packed (devDependencies are not).
- CI: the docs job uses `actions/setup-node@v6`.

>>>>> lang=ru
### Репозиторий и релизная оснастка

- Готовые бинарники `ktav-lsp` больше не хранятся в репозитории:
  отслеживаемые копии устарели (пять платформ содержали `ktav` 0.5.0,
  win32-x64 — 0.7.1, а лишняя копия в
  `intellij/src/main/resources/bin/` — 0.1.5). `intellij/bin/`,
  `vscode/bin/` и `intellij/src/main/resources/bin/` добавлены в
  `.gitignore`; релизный workflow собирает все шесть платформ из
  исходников, а `scripts/build-binaries.sh` делает то же локально.
- Тест токенизатора TextMate добавляет в проект VS Code закреплённые
  dev-зависимости (`vscode-textmate`, `vscode-oniguruma`) — в VSIX они
  не входят, — а `tsconfig.json` получает библиотеку `DOM` для их
  типов WebAssembly.
- Релизный workflow: `vsce package` / `vsce publish` больше не
  передают `--no-dependencies`. Расширению нужен `vscode-languageclient`
  во время работы, а VSIX 0.6.1 поставлялся без `node_modules`, поэтому
  упакованное расширение не могло загрузить языковой клиент; теперь
  упаковываются production-зависимости (devDependencies — нет).
- CI: job docs использует `actions/setup-node@v6`.

>>>>> lang=zh
### 仓库与发布工具

- 预构建的 `ktav-lsp` 二进制不再提交到仓库:已跟踪的副本早已过时
  (五个平台内嵌 `ktav` 0.5.0,win32-x64 内嵌 0.7.1,
  `intellij/src/main/resources/bin/` 下多余的一份内嵌 0.1.5)。
  `intellij/bin/`、`vscode/bin/` 和 `intellij/src/main/resources/bin/`
  已加入 `.gitignore`;发布 workflow 从源码构建全部六个平台,
  `scripts/build-binaries.sh` 在本地做同样的事。
- TextMate 分词器测试为 VS Code 项目增加了固定版本的开发依赖
  (`vscode-textmate`、`vscode-oniguruma`)——不会打进 VSIX,同时
  `tsconfig.json` 为其 WebAssembly 类型加入 `DOM` 库。
- 发布 workflow:`vsce package` / `vsce publish` 不再传 `--no-dependencies`。
  扩展运行时需要 `vscode-languageclient`,而 0.6.1 的 VSIX 未带
  `node_modules`,因此打包后的扩展无法加载语言客户端;现在会打包生产
  依赖(不含 devDependencies)。
- CI:docs job 使用 `actions/setup-node@v6`。

