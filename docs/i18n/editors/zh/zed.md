# Zed

**Languages:** [English](../../../zed.md) · [Русский](../ru/zed.md) · **简体中文**

> **状态:** 待定。官方 Zed 扩展已列入路线图,但尚未发布。下面的说明
> 概述了扩展骨架大致会是什么样子 —— 欢迎贡献。

## 手动配置(目前)

Zed 从 `.zed/settings.json` 读取工作区级别的语言设置:

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

安装服务器:

```sh
cargo install ktav-lsp
```

在已发布的扩展把 `Ktav` 注册为已知语言之前,Zed 会把 `.ktav` 当作
纯文本 —— 如果文件已打开,LSP 仍会接入,但不会有高亮。

## 计划中的扩展结构

```
zed-ktav/
  extension.toml          # id, name, languages.ktav
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # tree-sitter highlight queries (TBD)
  grammars/
    ktav.toml             # tree-sitter grammar source pin
```

跟踪 issue:<https://github.com/ktav-lang/editor/issues>
