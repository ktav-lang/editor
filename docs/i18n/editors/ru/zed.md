# Zed

**Languages:** [English](../../../zed.md) · **Русский** · [简体中文](../zh/zed.md)

> **Статус:** TBD. Полноценное расширение Zed в планах, но ещё не
> опубликовано. Ниже — заметки о том, как мог бы выглядеть каркас
> расширения; вклад приветствуется.

## Ручная настройка (сегодня)

Zed читает языковые настройки уровня workspace из
`.zed/settings.json`:

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

Установите сервер:

```sh
cargo install ktav-lsp
```

Пока опубликованное расширение не зарегистрирует `Ktav` как известный
язык, Zed будет считать `.ktav` обычным текстом — LSP всё равно
подключится, если файл открыт, но подсветки не будет.

## Планируемая структура расширения

```
zed-ktav/
  extension.toml          # id, name, languages.ktav
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # tree-sitter highlight queries (TBD)
  grammars/
    ktav.toml             # tree-sitter grammar source pin
```

Отслеживающий issue: <https://github.com/ktav-lang/editor/issues>
