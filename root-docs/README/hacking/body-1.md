>>>>> lang=en
## Hacking on this repo

Each subproject has its own toolchain. See its `README.md`:

- `grammars/` — pure JSON; no build
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.88+ (minimum required by the locked dependency graph)

A single tag triggers a release of all four (see [`.github/workflows/release.yml`](.github/workflows/release.yml)).

>>>>> lang=ru
## Разработка в этом репозитории

Каждый подпроект имеет свой toolchain. См. его `README.md`:

- `grammars/` — чистый JSON; без сборки
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.88+ (минимум для закреплённого графа зависимостей)

Один тег запускает релиз всех четырёх (см. [`.github/workflows/release.yml`](../../.github/workflows/release.yml)).

>>>>> lang=zh
## 开发本仓库

每个子项目都有自己的工具链。详见各子项目自己的 `README.md`:

- `grammars/` —— 纯 JSON,无需构建
- `vscode/` —— Node + `vsce`
- `intellij/` —— JDK 17 + Gradle
- `lsp/` —— Rust 1.88+(锁定依赖图要求的最低版本)

一个 tag 同时触发全部四个子项目的发布(参见 [`.github/workflows/release.yml`](../../.github/workflows/release.yml))。

