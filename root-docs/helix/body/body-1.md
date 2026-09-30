>>>>> lang=en
Add the following to `~/.config/helix/languages.toml`:

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

Install the language server (Rust 1.88+ for the locked 0.8.0 build):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Verify with `:lsp-restart` after opening a `.ktav` file. Diagnostics
appear inline; hover with `K` (default keymap).

Stock Helix does not support LSP semantic-token highlighting. This
configuration enables LSP features such as diagnostics and hover, not
syntax colouring. Highlighting requires a separately implemented Ktav
tree-sitter grammar and highlight queries; this repository does not
provide them. `grammars/ktav.tmLanguage.json` is a TextMate grammar,
not a tree-sitter grammar, and cannot be installed as one.
See [Helix language configuration](https://docs.helix-editor.com/languages.html#tree-sitter-grammar-configuration).
>>>>> lang=ru
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
>>>>> lang=zh
添加以下内容到 `~/.config/helix/languages.toml`:

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

安装 language server(锁定的 0.8.0 构建需要 Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

打开 `.ktav` 文件后用 `:lsp-restart` 验证。诊断信息内联显示;
悬停(默认按键 `K`)可查看提示。

标准 Helix 不支持 LSP 语义令牌高亮。此配置启用诊断、悬停等
LSP 功能,不会启用语法高亮。高亮需要另行实现 Ktav tree-sitter
语法和高亮查询;本仓库不提供这些文件。
`grammars/ktav.tmLanguage.json` 是 TextMate 语法,不是
tree-sitter 语法,不能按后者安装。
参见 [Helix 语言配置](https://docs.helix-editor.com/languages.html#tree-sitter-grammar-configuration)。
