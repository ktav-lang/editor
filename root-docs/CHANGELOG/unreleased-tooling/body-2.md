>>>>> lang=en
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
- Release workflow dry run: a manual `workflow_dispatch` with `publish: false`
  (now the default) builds and verifies every artifact, checks that the
  CHANGELOG section for the version is non-empty and creates no GitHub
  Release. `tag` may be left empty to build a branch; `publish: true` still
  requires a `v*` tag. Release notes are cut by comparing the heading's
  version literally instead of through a regex built from the tag.
- Layout: the LSP end-to-end suites share one `tests/e2e` target, the
  position/key scanners live in `lsp/src/tokens/scan/`, the VSIX smoke and
  archive checks in `vscode/src/test/vsix/`, the IntelliJ LSP tests mirror
  the main packages (`client`, `settings`, `formatting`) and the lexer
  tests are split by topic. No behaviour change; the test counts are the same.
- IntelliJ: the unused `MARKER_INT`/`MARKER_FLOAT` token types (typed
  markers were removed in 0.5.0) are gone.
- Documentation: the VS Code and LSP READMEs now agree that the
  Marketplace and Open VSX extension bundles `ktav-lsp` for six
  platforms (a separate install is only needed elsewhere);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` and
  `lsp/docs/bench-baseline.md` are marked as historical.
- IntelliJ `sinceBuild = 231` is now backed by the Plugin Verifier: CI
  verifies 2023.1, 2024.1, 2024.3, 2025.1 and 2025.2 (231, 233, 241, 243,
  251 and 252 were all Compatible).
- `scripts/dev-install.mjs` builds the local plugins and installs them into
  explicitly named IDEs: dry run by default, never deletes (a previous
  `ktav-intellij` is moved to a backup folder), refuses running JetBrains
  IDEs and symlinks, and updates VS Code only through its CLI.

>>>>> lang=ru
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
- Пробный прогон релизного workflow: ручной `workflow_dispatch` с
  `publish: false` (теперь по умолчанию) собирает и проверяет все
  артефакты, требует непустую секцию версии в CHANGELOG и не создаёт
  GitHub Release. `tag` можно не указывать — тогда собирается ветка;
  `publish: true` по-прежнему требует тег `v*`. Заметки релиза вырезаются
  сравнением версии в заголовке как строки, а не регулярным выражением из
  тега.
- Структура: end-to-end наборы LSP собраны в одну цель `tests/e2e`,
  сканеры позиций и ключей лежат в `lsp/src/tokens/scan/`, VSIX-smoke и
  проверка архива — в `vscode/src/test/vsix/`, LSP-тесты IntelliJ повторяют
  пакеты main (`client`, `settings`, `formatting`), а тесты лексера
  разбиты по темам. Поведение не менялось; число тестов прежнее.
- IntelliJ: неиспользуемые типы токенов `MARKER_INT`/`MARKER_FLOAT`
  (типизированные маркеры удалены в 0.5.0) убраны.
- Документация: README VS Code и LSP теперь согласованы в том, что
  расширение из Marketplace и Open VSX содержит `ktav-lsp` для шести
  платформ (отдельная установка нужна только в других случаях);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` и
  `lsp/docs/bench-baseline.md` помечены как исторические.
- IntelliJ `sinceBuild = 231` теперь подтверждён Plugin Verifier: CI
  проверяет 2023.1, 2024.1, 2024.3, 2025.1 и 2025.2 (231, 233, 241, 243,
  251 и 252 — Compatible).
- `scripts/dev-install.mjs` собирает локальные плагины и ставит их в явно
  указанные IDE: по умолчанию пробный прогон, ничего не удаляет (прежний
  `ktav-intellij` переносится в папку бэкапа), отклоняет запущенные
  JetBrains IDE и symlink-и, VS Code обновляет только через его CLI.

>>>>> lang=zh
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
- 发布 workflow 试运行:手动 `workflow_dispatch` 且 `publish: false`(现为默认值)会构建并验证所有产物,检查该版本的 CHANGELOG 小节非空,且不创建 GitHub Release。`tag` 可留空以构建分支;`publish: true` 仍需要 `v*` 标签。发布说明通过按字面比较标题中的版本来截取,而不再使用由标签拼出的正则表达式。
- 结构:LSP 端到端测试合并为一个 `tests/e2e` 目标,位置/键扫描器位于 `lsp/src/tokens/scan/`,VSIX 冒烟与归档检查位于 `vscode/src/test/vsix/`,IntelliJ 的 LSP 测试与 main 包对应(`client`、`settings`、`formatting`),词法分析器测试按主题拆分。行为不变;测试数量相同。
- IntelliJ:移除未使用的 `MARKER_INT`/`MARKER_FLOAT` 词法类型(带类型的标记已在 0.5.0 中移除)。
- 文档:VS Code 与 LSP 的 README 现在一致说明,Marketplace 与 Open VSX
  上的扩展已为六个平台捆绑 `ktav-lsp`(仅其他情况才需要单独安装);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` 与
  `lsp/docs/bench-baseline.md` 已标注为历史文档。
- IntelliJ 的 `sinceBuild = 231` 现由 Plugin Verifier 验证:CI 检查 2023.1、2024.1、2024.3、2025.1 和 2025.2(231、233、241、243、251 和 252 均为 Compatible)。
- `scripts/dev-install.mjs` 构建本地插件并安装到显式指定的 IDE:默认试运行,从不删除(原有 `ktav-intellij` 移入备份目录),拒绝正在运行的 JetBrains IDE 和符号链接,VS Code 只通过其 CLI 更新。

