>>>>> lang=en
### Repository and release tooling

- The IntelliJ rebuild helper uses the current checkout and explicitly
  selected IDE/platform paths. It never kills processes or clears IDE caches,
  logs, unrelated plugins or archives. Python 3 stages ZIP installation and
  preserves Unix executable modes.
- Helix, Zed and Sublime guides now state the actual client prerequisites:
  tree-sitter highlighting for stock Helix, a registered language/server
  adapter for Zed, and XML TextMate syntax matching Sublime's `source.ktav`.
- Shared-grammar, Emacs and Neovim guidance now distinguishes TextMate JSON
  hosts from separate tree-sitter/font-lock integrations. Collisions with both
  canonical multiline closer forms cause `BothFormsRequired`, not preservation
  by string splitting; literal key dots require escaping or quoting.

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
- Documentation: the VS Code and LSP READMEs now agree that the
  Marketplace and Open VSX extension bundles `ktav-lsp` for six
  platforms (a separate install is only needed elsewhere);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` and
  `lsp/docs/bench-baseline.md` are marked as historical.

>>>>> lang=ru
### Репозиторий и релизная оснастка

- IntelliJ rebuild-helper использует текущий checkout и явно выбранные пути
  IDE/платформы. Он не завершает процессы и не удаляет кэши, логи, чужие
  плагины или архивы. Python 3 устанавливает ZIP через staging с сохранением
  Unix-прав исполнения.
- Инструкции Helix, Zed и Sublime описывают реальные требования клиентов:
  tree-sitter для подсветки стандартного Helix, зарегистрированный язык и
  адаптер сервера для Zed, XML TextMate со scope `source.ktav` для Sublime.
- Инструкции общей грамматики, Emacs и Neovim различают хосты TextMate JSON
  и отдельные tree-sitter/font-lock интеграции. Коллизии обоих closer у
  канонической multiline-формы дают `BothFormsRequired`, а не разделение
  строки; буквальные точки ключа требуют экранирования или кавычек.

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
- Документация: README VS Code и LSP теперь согласованы в том, что
  расширение из Marketplace и Open VSX содержит `ktav-lsp` для шести
  платформ (отдельная установка нужна только в других случаях);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` и
  `lsp/docs/bench-baseline.md` помечены как исторические.

>>>>> lang=zh
### 仓库与发布工具

- IntelliJ 重建 helper 使用当前 checkout 及显式选择的 IDE/平台路径，
  不终止进程或清理 IDE 缓存、日志、其他插件及归档。Python 3 通过 staging
  安装 ZIP 并保留 Unix 可执行权限。
- Helix、Zed 和 Sublime 指南明确实际前提：标准 Helix 的 tree-sitter 高亮、
  Zed 已注册的语言/服务器适配器，以及 Sublime 匹配 `source.ktav` 的 XML
  TextMate 语法。
- 共享语法、Emacs 和 Neovim 指南区分 TextMate JSON 宿主与独立的
  tree-sitter/font-lock 集成。规范多行形式的双闭合符冲突产生
  `BothFormsRequired`，而非拆分字符串；键中字面点需转义或引号。

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
- 文档:VS Code 与 LSP 的 README 现在一致说明,Marketplace 与 Open VSX
  上的扩展已为六个平台捆绑 `ktav-lsp`(仅其他情况才需要单独安装);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` 与
  `lsp/docs/bench-baseline.md` 已标注为历史文档。

