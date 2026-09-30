# Zed

**Languages:** [English](../../../zed.md) · **Русский** · [简体中文](../zh/zed.md)

> **Статус:** TBD. Полноценное расширение Zed в планах, но ещё не
> опубликовано. Ниже — заметки о том, как мог бы выглядеть каркас
> расширения; вклад приветствуется.

## Требования для настройки

Стандартному Zed нужно установленное расширение (опубликованное или
development), регистрирующее язык `Ktav`, расширение `.ktav` и адаптер
language server `ktav-lsp`. Этот репозиторий такое расширение не
поставляет. Одних settings недостаточно для регистрации языка или
подключения сервера к Plain Text. После установки такого расширения
зарегистрированные им имена можно настроить в `.zed/settings.json`:

```jsonc
{
  "languages": {
    "Ktav": {
      "tab_size": 2,
      "language_servers": ["ktav-lsp"]
    }
  },
  "lsp": {
    "ktav-lsp": {
      "binary": { "path": "ktav-lsp" }
    }
  }
}
```

Установите сервер (для locked-сборки 0.8.0 нужен Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Используйте имена языка и адаптера, зарегистрированные расширением.
Для подсветки дополнительно нужны tree-sitter-грамматика Ktav и
запросы; общая TextMate JSON-грамматика не является tree-sitter.
См. [языковые расширения Zed](https://zed.dev/docs/extensions/languages)
и [настройку language server](https://zed.dev/docs/configuring-languages#configuring-language-servers).

## Планируемая структура расширения

```
zed-ktav/
  extension.toml          # extension metadata, grammar pin, language_servers adapter
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # queries for a separately implemented tree-sitter grammar
  src/lib.rs              # Zed extension API: ktav-lsp command adapter
```

Отслеживающий issue: <https://github.com/ktav-lang/editor/issues>
