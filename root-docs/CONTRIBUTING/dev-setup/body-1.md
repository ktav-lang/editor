>>>>> lang=en
## Dev setup

Each subproject has its own toolchain. See the README in each:

- `grammars/` — pure JSON; no build
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.71+

The prebuilt `ktav-lsp` binaries (`vscode/bin/`, `intellij/bin/`) are not
committed. Build them with `scripts/build-binaries.sh` (it uses `cross`
for the Linux and macOS targets); the release workflow builds every
platform from source. The VS Code project also has a grammar tokenizer
test: `npm run test:unit` in `vscode/`.

>>>>> lang=ru
## Dev-окружение

Каждый подпроект имеет свой toolchain. См. README в каждом:

- `grammars/` — чистый JSON; без сборки
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.71+

Готовые бинарники `ktav-lsp` (`vscode/bin/`, `intellij/bin/`) в
репозитории не хранятся. Соберите их через `scripts/build-binaries.sh`
(для целей Linux и macOS он использует `cross`); релизный workflow
собирает все платформы из исходников. У проекта VS Code есть также тест
токенизатора грамматики: `npm run test:unit` в `vscode/`.

>>>>> lang=zh
## 开发环境

每个子项目都有自己的工具链。详见各子项目中的 README:

- `grammars/` —— 纯 JSON,无需构建
- `vscode/` —— Node + `vsce`
- `intellij/` —— JDK 17 + Gradle
- `lsp/` —— Rust 1.71+

预构建的 `ktav-lsp` 二进制(`vscode/bin/`、`intellij/bin/`)不提交到仓库。
请用 `scripts/build-binaries.sh` 构建(Linux 和 macOS 目标使用 `cross`);
发布 workflow 从源码构建所有平台。VS Code 项目还有语法分词器测试:在
`vscode/` 中运行 `npm run test:unit`。

