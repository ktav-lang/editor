>>>>> lang=en
### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

This enables LSP diagnostics and hover, not syntax highlighting in stock
Helix. Highlighting requires a separate Ktav tree-sitter grammar and
queries, which this repo does not supply; TextMate JSON is not a substitute.

>>>>> lang=ru
### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

Это включает диагностику и hover от LSP, но не подсветку в стандартном
Helix. Для подсветки нужны отдельные tree-sitter-грамматика Ktav и
запросы, которых в репозитории нет; TextMate JSON их не заменяет.

>>>>> lang=zh
### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

此配置启用 LSP 诊断和悬停,不会在标准 Helix 中启用语法高亮。
高亮需要独立的 Ktav tree-sitter 语法和查询,本仓库不提供它们;
TextMate JSON 不能代替它们。

