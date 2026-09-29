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
server offers *intelligent* features, but each editor's client decides
which are integrated. IntelliJ's built-in client provides live
diagnostics and whole-file formatting only; it does not integrate
hover, completion, document symbols or semantic tokens. Installing
`ktav-lsp` on PATH does not enable those missing client integrations.

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
LSP-сервер предлагает *интеллектуальные* функции, но их интеграция
зависит от клиента редактора. Встроенный клиент IntelliJ предоставляет
только live-диагностику и форматирование всего файла; hover,
автокомплит, document symbols и semantic tokens не интегрированы.
Установка `ktav-lsp` в PATH не добавляет недостающие интеграции клиента.

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
表层高亮(无需语言服务器)。LSP 服务器提供*智能*功能,但具体集成
取决于编辑器客户端。IntelliJ 内置客户端仅提供实时诊断和整文件
格式化,未集成悬停、补全、文档符号或语义令牌。将 `ktav-lsp`
加入 PATH 不会补上这些缺失的客户端集成。

