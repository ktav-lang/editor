# ktav-lang/editor

[![Release](https://img.shields.io/github/v/release/ktav-lang/editor?style=flat-square&sort=semver&label=release)](https://github.com/ktav-lang/editor/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/editor/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/editor/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

**Languages:** [English](../../README.md) · **Русский** · [简体中文](README.zh.md)

**Песочница:** конвертация JSON / YAML / TOML / INI ⇄ Ktav прямо в браузере — **[ktav-lang.github.io](https://ktav-lang.github.io/)**.

> Поддержка редакторов для конфигурационного формата
> **[Ktav](https://github.com/ktav-lang/spec)** — подсветка синтаксиса,
> плагины для IDE, Language Server. Один репозиторий, четыре
> подпроекта, одна общая TextMate-грамматика.

## Что внутри

| Подпроект               | Что это                                                            | Куда публикуется                                      |
|-------------------------|--------------------------------------------------------------------|-------------------------------------------------------|
| [`grammars/`](../../grammars/) | Общая TextMate-грамматика + VS Code language configuration | Используется `vscode/` и `intellij/`                  |
| [`vscode/`](../../vscode/)    | Расширение для Visual Studio Code                            | VS Code Marketplace + Open VSX                        |
| [`intellij/`](../../intellij/)| Плагин для IntelliJ Platform (IntelliJ IDEA, RustRover, GoLand, …) | JetBrains Marketplace                                 |
| [`lsp/`](../../lsp/)          | Реализация Language Server Protocol (Rust, `tower-lsp`)        | crates.io как `ktav-lsp`                              |
| [`docs/`](../../docs/)        | Сниппеты настройки редакторов (Helix, Neovim, Emacs, Sublime, Zed) | —                                                  |

## Что получает пользователь Ktav

LSP-функции зависят от клиента редактора. Плагин IntelliJ интегрирует
только live-диагностику и форматирование всего файла; hover,
автокомплит, document symbols и semantic tokens не интегрированы.
Подсветку синтаксиса обеспечивает нативный лексер.

- **Подсветка синтаксиса** — ключи, скаляры, raw-string-маркер (`::`),
  многострочные строки, комментарии
- **Парные скобки + автозакрытие** — `{}` `[]` `()`
- **Переключение комментария** — `Ctrl/Cmd+/` → `## comment`
- **Live-диагностика** (с LSP) — каждый `MissingSeparatorSpace`,
  дубликат ключа, конфликт dotted-префикса всплывает красной
  подчёркивающей линией на проблемной строке, с тем же сообщением,
  что выдаёт парсер
- **Hover-подсказки** (с LSP) — dotted-путь ключа под курсором,
  выведенный тип значения
- **Автокомплит** (с LSP) — ключевые слова (`null` / `true` / `false`),
  raw-string-маркер (`::`), открытие compound'ов
- **Document symbols** (с LSP) — outline отражает структуру
  `Value::Object`

## Архитектура

```
                       ┌─────────────────────────────────────┐
                       │ ktav-lsp  (Rust binary, this repo)  │
                       │ • parses via the `ktav` crate       │
                       │ • diagnostics, hover, completion,   │
                       │   semantic tokens, document symbols │
                       └─────────────────────────────────────┘
                                          ▲
                                          │ LSP (JSON-RPC over stdio)
                 ┌────────────────────────┼───────────────────────┐
                 │                        │                       │
       ┌─────────┴────────┐   ┌───────────┴──────────┐   ┌────────┴───────┐
       │ VS Code          │   │ IntelliJ Platform    │   │ Helix / Neovim │
       │ ext (`vscode/`)  │   │ plugin (`intellij/`) │   │ + LSP config   │
       │                  │   │                      │   │                │
       │ TextMate grammar │   │ Native lexer         │   │ Highlighting:  │
       │ from grammars/   │   │ (no TextMate)        │   │ client-specific│
       │                  │   │                      │   │ (see below)    │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

TextMate-грамматика (VS Code) и нативный лексер IntelliJ одинаково
дают мгновенную косметическую подсветку (языковой сервер не нужен).
LSP-сервер предлагает *интеллектуальные* функции, но их интеграция
зависит от клиента редактора. Встроенный клиент IntelliJ предоставляет
только live-диагностику и форматирование всего файла; hover,
автокомплит, document symbols и semantic tokens не интегрированы.
Установка `ktav-lsp` в PATH не добавляет недостающие интеграции клиента.

Стандартный Helix использует tree-sitter для подсветки, а не semantic
tokens LSP; Ktav tree-sitter-грамматики в этом репозитории нет.
TextMate JSON не может её заменить. Semantic-подсветка Neovim зависит
от настройки LSP-клиента. Zed дополнительно нужно расширение,
регистрирующее язык Ktav и адаптер сервера; settings сами по себе
не подключают LSP к Plain Text.

## Установка для пользователя

### VS Code

```
ext install ktav-lang.ktav
```

Расширение включает LSP-сервер в комплект для основных платформ —
дополнительных шагов не нужно.

### IntelliJ IDEA / RustRover / GoLand / etc.

Плагины → Marketplace → найти **Ktav** → Установить.

### Helix

Добавьте в `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Затем `cargo install ktav-lsp --version 0.8.1 --locked`.

### Neovim

С [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig):

```lua
vim.filetype.add({ extension = { ktav = "ktav" } })
require("lspconfig.configs").ktav = {
  default_config = {
    cmd = { "ktav-lsp" },
    filetypes = { "ktav" },
    root_dir = require("lspconfig.util").root_pattern(".git"),
    settings = {},
  },
}
require("lspconfig").ktav.setup({})
```

Затем `cargo install ktav-lsp --version 0.8.1 --locked`.

### Другие редакторы

См. [`docs/`](../../docs/) для Emacs (eglot), Sublime и Zed.

## Разработка в этом репозитории

Каждый подпроект имеет свой toolchain. См. его `README.md`:

- `grammars/` — чистый JSON; без сборки
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.88+ (минимум для закреплённого графа зависимостей)

Один тег запускает релиз всех четырёх (см. [`.github/workflows/release.yml`](../../.github/workflows/release.yml)).

## Версионирование

Единый semver на весь monorepo: все четыре
подпроекта публикуются под одним тегом одновременно. CHANGELOG
перечисляет изменения по подпроектам внутри каждого раздела версии.

## Лицензия

Собственный код Ktav распространяется под MIT OR Apache-2.0. См. [LICENSE-MIT](../../LICENSE-MIT) и [LICENSE-APACHE](../../LICENSE-APACHE). Сторонний код сохраняет собственные лицензии и уведомления; лицензия Ktav их не заменяет.

### Нативные бинарники, уведомления и исходники

Лицензионно проверяемый release pipeline поставляет каждый LSP executable вместе с `*.distribution.json`, читаемым `*.THIRD-PARTY-NOTICES.txt`, `*.NOTICE.json`, его сжатой копией, оригинальным отчётом Rust `*.COPYRIGHT-library.html`, `*.Cargo.lock` и обеими лицензиями Ktav. Рядом поставляются уведомления о runtime конкретной платформы. При дальнейшем распространении сохраняйте весь platform payload: копирование одного executable теряет обязательные материалы.

GNU Linux ARM64 payload также содержит `ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz` и `*.source-binding.json`. Используемый startup code подпадает под LGPL-2.1-or-later без compiled-linking exception, имеющегося у проверенного x64 startup. Matching source archive содержит соответствующие исходники и материалы для повторной линковки; приложенный rebuild script позволяет пересобрать executable с изменённым startup source. Лицензия собственного кода Ktav от этого не меняется.

Universal plugins содержат те же полные payloads: `bin/<platform>/` в VS Code extension и `lib/bin/<platform>/` в IntelliJ plugin. Обязательный ARM64 source archive добавляет около 94 МБ к каждому universal plugin. CI проверяет содержимое настоящих VSIX/IntelliJ ZIP, встроенные notice resources и source bindings, а не только staging directories. Эти проверки не заменяют задним числом исторические опубликованные assets.

## Другие реализации Ktav

- [`spec`](https://github.com/ktav-lang/spec) — спецификация + conformance-набор
- [`rust`](https://github.com/ktav-lang/rust) — эталонный Rust-крейт (`cargo add ktav`)
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` на Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
