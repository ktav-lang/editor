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

Установите language server:

```sh
cargo install ktav-lsp
```

Проверьте `:lsp-restart` после открытия файла `.ktav`. Диагностика
отображается инлайново; hover — `K` (клавиша по умолчанию).

Подсветка обеспечивается semantic-tokens ответом LSP — грамматика
TextMate / tree-sitter в Helix для косметической подсветки не нужна.
Если нужна более богатая подсветка без запущенного LSP, можно
подключить общую грамматику из `editor/grammars/` в отдельную
конфигурацию tree-sitter, но это уже забота самого Helix.
