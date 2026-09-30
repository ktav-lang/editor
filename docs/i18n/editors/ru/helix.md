# Helix

**Languages:** [English](../../../helix.md) · **Русский** · [简体中文](../zh/helix.md)

Добавьте в `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = [".git"]
comment-token = "##"
indent = { tab-width = 2, unit = "  " }
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Установите language server (для locked-сборки 0.8.0 нужен Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Проверьте `:lsp-restart` после открытия файла `.ktav`. Диагностика
отображается инлайново; hover — `K` (клавиша по умолчанию).

Стандартный Helix не поддерживает подсветку semantic tokens от LSP.
Эта настройка включает LSP-функции, например диагностику и hover,
но не подсветку синтаксиса. Для неё нужны отдельно реализованные
tree-sitter-грамматика Ktav и запросы подсветки; в этом репозитории их
нет. `grammars/ktav.tmLanguage.json` — грамматика TextMate, а не
tree-sitter, и установить её как tree-sitter-грамматику нельзя.
См. [настройку языков Helix](https://docs.helix-editor.com/languages.html#tree-sitter-grammar-configuration).
