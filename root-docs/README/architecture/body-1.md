>>>>> lang=en
## Architecture

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
       │ TextMate grammar │   │ Native lexer         │   │ (LSP semantic  │
       │ from grammars/   │   │ (no TextMate)        │   │  tokens for    │
       │                  │   │                      │   │  highlighting) │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

The TextMate grammar (VS Code) and the native IntelliJ lexer both give
instant cosmetic highlighting (no language server needed). The LSP
layer adds the *intelligent* features — diagnostics, hover, completion.
They stack: install the extension/plugin alone for highlighting, add
`ktav-lsp` to your PATH for everything else.

>>>>> lang=ru
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
       │ TextMate grammar │   │ Native lexer         │   │ (LSP semantic  │
       │ from grammars/   │   │ (no TextMate)        │   │  tokens for    │
       │                  │   │                      │   │  highlighting) │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

TextMate-грамматика (VS Code) и нативный лексер IntelliJ одинаково
дают мгновенную косметическую подсветку (языковой сервер не нужен).
LSP-слой добавляет *интеллектуальные* функции — диагностику, hover,
автокомплит. Слои складываются: поставьте только расширение/плагин —
получите подсветку; добавьте `ktav-lsp` в PATH — и всё остальное.

>>>>> lang=zh
## 架构

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
       │ TextMate grammar │   │ Native lexer         │   │ (LSP semantic  │
       │ from grammars/   │   │ (no TextMate)        │   │  tokens for    │
       │                  │   │                      │   │  highlighting) │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

TextMate 语法(VS Code)与 IntelliJ 的原生词法分析器都能即时提供
表层高亮(无需语言服务器)。LSP 层增加*智能*功能 —— 诊断、悬停、
补全。两层可叠加:只安装扩展/插件即有高亮;将 `ktav-lsp` 加入
PATH 即可获得其余功能。

