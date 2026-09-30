# Helix

**Languages:** [English](../../../helix.md) · [Русский](../ru/helix.md) · **简体中文**

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
