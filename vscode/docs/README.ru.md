# Ktav для Visual Studio Code

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=ktav-lang.ktav)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Registry-c160ef)](https://open-vsx.org/extension/ktav-lang/ktav)

Подсветка синтаксиса и языковая поддержка конфигурационного формата [Ktav](https://github.com/ktav-lang/spec) внутри Visual Studio Code.

**Languages:** [English](../README.md) · **Русский** · [简体中文](README.zh.md)

## Возможности

- Подсветка синтаксиса для файлов `.ktav` — ключи, скаляры, строки-литералы `::`, многострочные блоки, инлайн- и блочные составные конструкции, комментарии
- Сопоставление скобок и автозакрытие для `{}`, `[]`, `()`
- Переключение комментария (`##`)
- Автоотступ внутри объектов, массивов и скобочных групп

С языковым сервером [`ktav-lsp`](https://github.com/ktav-lang/editor/tree/main/lsp):

- Диагностика ошибок разбора
- Семантическая подсветка и структура документа (ключи и вложенность)
- Подсказки при наведении (hover) для типов скаляров

## Языковой сервер

Расширение обращается к бинарю `ktav-lsp` через stdio. VSIX,
публикуемый в Marketplace и Open VSX, уже содержит `ktav-lsp` для
каждой из шести платформ, которые собирает CI, — Linux, macOS и
Windows на x64 и arm64, — так что всё работает из коробки; порядок
поиска бинарника описан ниже в разделе «Порядок поиска». На
неподдерживаемой платформе скачайте бинарник с
[релиза GitHub](https://github.com/ktav-lang/editor/releases) и
укажите путь к нему в `ktav.server.path` или положите его в `PATH`.
Чтобы вместо этого собрать сервер из исходников, выполните
`cargo build --release` в каталоге
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp).

### Порядок поиска

При активации расширение ищет сервер в следующем порядке:

1. **Явная настройка** — `ktav.server.path` (абсолютный путь).
2. **Встроенный бинарник** — `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]` (если он входит в сборку).
3. **PATH** — запускается `ktav-lsp`, и его поиск остаётся за операционной системой.

Если ни один из вариантов не сработал, показывается всплывающее уведомление об ошибке, а канал вывода `Ktav Language Server` фиксирует сбой.

### Настройки

| Параметр             | Тип                                 | По умолчанию | Описание                                          |
|----------------------|-------------------------------------|---------|--------------------------------------------------|
| `ktav.server.path`   | string                              | `""`    | Абсолютный путь к `ktav-lsp`. Пустое значение означает автопоиск (см. выше). |
| `ktav.trace.server`  | `"off"` \| `"messages"` \| `"verbose"` | `"off"` | Выводит трафик JSON-RPC в канал вывода.                           |

## Установка

Из Visual Studio Code Marketplace:

```
ext install ktav-lang.ktav
```

Или открой панель расширений (`Ctrl+Shift+X` / `Cmd+Shift+X`) и найди **Ktav**.

Расширение также публикуется в [Open VSX](https://open-vsx.org/) для VSCodium и других совместимых редакторов.

## Пример

```ktav
## Ktav — no quotes, no commas, no indentation traps.
service: socks5-rotator
## bare scalars are auto-typed: int / float / bool / null
port: 20082
debug: true

## Dotted keys are a flat alternative to nesting.
node.host: a.example
node.port: 1080

## '::' forces a literal string — keeps "8080" a string, not a number.
node.token:: 8080

## Need a literal '.' or ':' inside a key? Escape it (new in 0.6.0).
metric.http\.requests: 42

## Comma-free arrays and inline objects.
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

## Multi-line strings.
motd: (
    Welcome to the node.
    Please behave.
)
```

## Экосистема — официальные биндинги

Ktav — это одно Rust-ядро, обёрнутое тонкими биндингами на каждом языке:
идентичное поведение, нативная скорость и WebAssembly в браузере:

| Язык            | Установка                           | Репозиторий |
|-----------------|-------------------------------------|-------------|
| Rust            | `cargo add ktav`                    | [ktav-lang/rust](https://github.com/ktav-lang/rust) |
| JavaScript / TS | `npm i @ktav-lang/ktav`             | [ktav-lang/js](https://github.com/ktav-lang/js) |
| Python          | `pip install ktav`                  | [ktav-lang/python](https://github.com/ktav-lang/python) |
| Go              | `go get github.com/ktav-lang/golang` | [ktav-lang/golang](https://github.com/ktav-lang/golang) |
| PHP             | `composer require ktav-lang/ktav`   | [ktav-lang/php](https://github.com/ktav-lang/php) |
| Java / JVM      | `io.github.ktav-lang:ktav:0.8.0`    | [ktav-lang/java](https://github.com/ktav-lang/java) |
| C# / .NET       | `dotnet add package Ktav`           | [ktav-lang/csharp](https://github.com/ktav-lang/csharp) |

## Ресурсы

- [Спецификация Ktav](https://github.com/ktav-lang/spec)
- [Конвертер в браузере](https://ktav-lang.github.io/) — JSON / YAML / TOML / INI ⇄ Ktav
- [Грамматика tree-sitter](https://github.com/ktav-lang/tree-sitter-ktav)
- [Все репозитории](https://github.com/ktav-lang)
- [Трекер проблем](https://github.com/ktav-lang/editor/issues)

## Лицензия

MIT OR Apache-2.0 — см. [LICENSE-MIT](./LICENSE-MIT) и [LICENSE-APACHE](./LICENSE-APACHE).
