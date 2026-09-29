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

安装 language server:

```sh
cargo install ktav-lsp
```

打开 `.ktav` 文件后用 `:lsp-restart` 验证。诊断信息内联显示;
悬停(默认按键 `K`)可查看提示。

高亮由 LSP 的 semantic-tokens 响应提供 —— Helix 中无需 TextMate /
tree-sitter 语法即可获得基本高亮。如果想在不运行 LSP 的情况下获得
更丰富的高亮,可以将 `editor/grammars/` 中的共享语法接入自定义的
tree-sitter 配置,但这属于 Helix 上游的范畴。
