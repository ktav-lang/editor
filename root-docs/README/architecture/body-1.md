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
       │ TextMate grammar │   │ Native lexer         │   │ Highlighting:  │
       │ from grammars/   │   │ (no TextMate)        │   │ client-specific│
       │                  │   │                      │   │ (see below)    │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

The TextMate grammar (VS Code) and the native IntelliJ lexer both give
instant cosmetic highlighting (no language server needed). The LSP
server offers *intelligent* features, but each editor's client decides
which are integrated. IntelliJ's built-in client provides live
diagnostics and whole-file formatting only; it does not integrate
hover, completion, document symbols or semantic tokens. Installing
`ktav-lsp` on PATH does not enable those missing client integrations.

Stock Helix uses tree-sitter for highlighting, not LSP semantic tokens;
this repo provides no Ktav tree-sitter grammar. The TextMate JSON cannot
serve as one. Neovim semantic highlighting depends on its LSP client
configuration. Zed additionally needs an extension registering the
Ktav language and server adapter; settings alone do not attach LSP
to Plain Text.

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
       │ TextMate grammar │   │ Native lexer         │   │ Highlighting:  │
       │ from grammars/   │   │ (no TextMate)        │   │ client-specific│
       │                  │   │                      │   │ (see below)    │
       └──────────────────┘   └──────────────────────┘   └────────────────┘
```

TextMate 语法(VS Code)与 IntelliJ 的原生词法分析器都能即时提供
表层高亮(无需语言服务器)。LSP 服务器提供*智能*功能,但具体集成
取决于编辑器客户端。IntelliJ 内置客户端仅提供实时诊断和整文件
格式化,未集成悬停、补全、文档符号或语义令牌。将 `ktav-lsp`
加入 PATH 不会补上这些缺失的客户端集成。

标准 Helix 使用 tree-sitter 高亮,不使用 LSP 语义令牌;本仓库不提供
Ktav tree-sitter 语法,TextMate JSON 也不能代替它。Neovim 语义高亮
取决于其 LSP 客户端配置。Zed 还需要扩展来注册 Ktav 语言和服务器
适配器;仅靠 settings 无法将 LSP 连接到 Plain Text。

