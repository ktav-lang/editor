# Zed

**Languages:** [English](../../../zed.md) · [Русский](../ru/zed.md) · **简体中文**

> **状态:** 待定。官方 Zed 扩展已列入路线图,但尚未发布。下面的说明
> 概述了扩展骨架大致会是什么样子 —— 欢迎贡献。

## 配置前提

标准 Zed 需要安装扩展(已发布或开发扩展),注册 `Ktav` 语言、
`.ktav` 后缀和 `ktav-lsp` language-server 适配器。本仓库不提供该
扩展。仅靠 settings 无法注册语言或将此服务器连接到 Plain Text。
安装这样的扩展后,可在 `.zed/settings.json` 中配置它注册的名称:

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

安装服务器(锁定的 0.8.0 构建需要 Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

请使用扩展注册的语言和适配器名称。语法高亮还需要 Ktav
tree-sitter 语法和查询;共享的 TextMate JSON 不是 tree-sitter 语法。
参见 [Zed 语言扩展](https://zed.dev/docs/extensions/languages)与
[语言服务器配置](https://zed.dev/docs/configuring-languages#configuring-language-servers)。

## 计划中的扩展结构

```
zed-ktav/
  extension.toml          # extension metadata, grammar pin, language_servers adapter
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # queries for a separately implemented tree-sitter grammar
  src/lib.rs              # Zed extension API: ktav-lsp command adapter
```

跟踪 issue:<https://github.com/ktav-lang/editor/issues>
