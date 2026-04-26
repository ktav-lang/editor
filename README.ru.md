# ktav-lang/editor

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

> Поддержка редакторов для конфигурационного формата
> **[Ktav](https://github.com/ktav-lang/spec)** — подсветка синтаксиса,
> плагины для IDE, Language Server. Один репозиторий, четыре подпроекта,
> одна общая TextMate-грамматика.

## Что внутри

| Подпроект               | Что это                                                          | Куда публикуется                                        |
|-------------------------|------------------------------------------------------------------|---------------------------------------------------------|
| [`grammars/`](grammars/)| Общая TextMate-грамматика + VS Code language configuration       | Используется `vscode/` и `intellij/`                    |
| [`vscode/`](vscode/)    | Расширение для Visual Studio Code                                | VS Code Marketplace + Open VSX                          |
| [`intellij/`](intellij/)| Плагин для IntelliJ Platform (IDEA, RustRover, GoLand, …)        | JetBrains Marketplace                                   |
| [`lsp/`](lsp/)          | Реализация Language Server Protocol (Rust, `tower-lsp`)          | crates.io как `ktav-lsp`                                |
| [`docs/`](docs/)        | Сниппеты для Helix / Neovim / Emacs                              | —                                                       |

## Что получает пользователь Ktav

- **Подсветка синтаксиса** — ключи, скаляры, тела типизированных
  маркеров (`:i` / `:f`), raw-string-маркер (`::`), многострочные
  строки, комментарии
- **Парные скобки + автозакрытие** — `{}` `[]` `()`
- **Переключение комментария** — `Ctrl/Cmd+/` → `# comment`
- **Live-диагностика** (с LSP) — каждый `MissingSeparatorSpace`,
  дубликат ключа, конфликт dotted-префикса всплывает красной
  подчёркивающей линией с тем же сообщением, что выдаёт парсер
- **Hover-подсказки** (с LSP) — dotted-путь ключа под курсором,
  выведенный тип значения
- **Автокомплит** (с LSP) — ключевые слова (`null` / `true` / `false`),
  маркеры (`:i` / `:f` / `::`), открытие compound'ов
- **Document symbols** (с LSP) — outline отражает структуру `Value::Object`

## Архитектура

```
                    ┌─────────────────────────────────────────┐
                    │  ktav-lsp  (Rust binary, этот репо)    │
                    │  • парсит через crate `ktav`           │
                    │  • diagnostics, hover, completion,     │
                    │    semantic tokens, document symbols   │
                    └─────────────────────────────────────────┘
                                       ▲
                                       │ LSP (JSON-RPC через stdio)
                ┌──────────────────────┼──────────────────────┐
                │                      │                      │
       ┌────────┴────────┐   ┌─────────┴─────────┐   ┌────────┴────────┐
       │ VS Code         │   │ IntelliJ Platform │   │ Helix / Neovim  │
       │ ext (`vscode/`) │   │ plugin (`intellij/`)│  │ + LSP config    │
       │                 │   │                     │  │                 │
       │ TextMate-grammar│   │ TextMate-grammar    │  │ (LSP semantic   │
       │ из grammars/    │   │ из grammars/        │  │  tokens для     │
       │                 │   │                     │  │  подсветки)     │
       └─────────────────┘   └─────────────────────┘  └─────────────────┘
```

TextMate-грамматика даёт мгновенную косметическую подсветку (без
LSP). LSP-слой добавляет *интеллект* — диагностику, hover,
автокомплит. Слои наслаиваются: ставишь только extension — есть
подсветка; добавляешь `ktav-lsp` в PATH — получаешь всё остальное.

## Установка для пользователя

### VS Code

```
ext install ktav-lang.ktav
```

Расширение само бандлит LSP-сервер для основных платформ — никаких
дополнительных шагов.

### IntelliJ IDEA / RustRover / GoLand / etc.

Plugins → Marketplace → search **Ktav** → Install.

### Helix

В `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Потом `cargo install ktav-lsp`.

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

Потом `cargo install ktav-lsp`.

### Другие редакторы

См. [`docs/`](docs/) для Emacs (eglot), Sublime, Zed.

## Разработка

Каждый подпроект — свой toolchain. См. README в каждом:

- `grammars/` — чистый JSON; без build'а
- `vscode/` — Node + `vsce`
- `intellij/` — JDK 17 + Gradle
- `lsp/` — Rust 1.70+

Один тег запускает релиз всех четырёх (см. [`.github/workflows/release.yml`](.github/workflows/release.yml)).

## Версионирование

Один semver на весь monorepo: `0.1.0`, `0.1.1`, `0.2.0`. Все четыре
подпроекта публикуются под одним тегом одновременно. CHANGELOG
перечисляет изменения по подпроектам внутри каждой версии.

## Лицензия

MIT. См. [LICENSE](LICENSE).

## Другие реализации Ktav

- [`spec`](https://github.com/ktav-lang/spec) — спецификация + conformance-тесты
- [`rust`](https://github.com/ktav-lang/rust) — эталонный Rust crate (`cargo add ktav`)
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` на Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
