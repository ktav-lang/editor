>>>>> lang=en
## What it is

`ktav-lsp` is an LSP server. Editors talk JSON-RPC to it over stdin/stdout
and get back diagnostics, hover, completion, document symbols, and
semantic tokens for `.ktav` files. It is a thin wrapper over the
[`ktav`](https://crates.io/crates/ktav) crate — the same parser every
other Ktav binding (PHP / JS / Python / Go / Java / C#) walks through —
so error messages and behaviour match exactly.

>>>>> lang=ru
## Что это

`ktav-lsp` — это LSP-сервер. Редакторы общаются с ним по JSON-RPC через
stdin/stdout и получают диагностики, hover, автодополнение, символы
документа и semantic tokens для файлов `.ktav`. Это тонкая обёртка над
crate [`ktav`](https://crates.io/crates/ktav) — тем же парсером, который
используют все остальные биндинги Ktav (PHP / JS / Python / Go / Java /
C#), — поэтому сообщения об ошибках и поведение совпадают точно.

>>>>> lang=zh
## 这是什么

`ktav-lsp` 是一个 LSP 服务器。编辑器通过 stdin/stdout 与它进行 JSON-RPC
通信,获取 `.ktav` 文件的诊断、hover、补全、文档符号和 semantic tokens。
它是 [`ktav`](https://crates.io/crates/ktav) crate 的薄封装 ——
所有其他 Ktav 绑定(PHP / JS / Python / Go / Java / C#)使用的同一个
解析器 —— 因此错误消息和行为完全一致。

