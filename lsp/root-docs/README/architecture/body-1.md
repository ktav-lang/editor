>>>>> lang=en
## Architecture

Single Rust crate, single binary. Stack:

- [`tower-lsp`](https://crates.io/crates/tower-lsp) for the JSON-RPC /
  capability plumbing.
- [`tokio`](https://tokio.rs/) runtime, reading from stdin and writing
  to stdout.
- [`ktav`](https://crates.io/crates/ktav) — the same parser crate every
  Ktav binding uses. Diagnostics, hover, and document symbols all go
  through `ktav::parse`.
- [`dashmap`](https://crates.io/crates/dashmap) — thread-safe per-`Url`
  document store. `TextDocumentSyncKind::FULL` keeps the loop simple:
  small config files re-parse fast enough that incremental sync would
  add code without saving wall time.

Logs go to stderr (stdout is reserved for LSP traffic). Set
`KTAV_LSP_LOG=debug` to crank verbosity.

>>>>> lang=ru
## Архитектура

Один Rust-crate, один бинарь. Стек:

- [`tower-lsp`](https://crates.io/crates/tower-lsp) — JSON-RPC и
  обработка возможностей сервера.
- [`tokio`](https://tokio.rs/) — runtime, читает из stdin, пишет в stdout.
- [`ktav`](https://crates.io/crates/ktav) — тот же парсерный crate, что
  использует каждый биндинг Ktav. Диагностики, hover, document symbols —
  всё через `ktav::parse`.
- [`dashmap`](https://crates.io/crates/dashmap) — потокобезопасное
  хранилище документов по `Url`. `TextDocumentSyncKind::FULL` упрощает
  цикл: небольшие конфиг-файлы перепарсятся настолько быстро, что
  инкрементальная синхронизация прибавит кода, не сэкономив время.

Логи идут в stderr (stdout зарезервирован за LSP-трафиком). Уровень
логирования: `KTAV_LSP_LOG=debug`.

>>>>> lang=zh
## 架构

单一 Rust crate,单一二进制。技术栈:

- [`tower-lsp`](https://crates.io/crates/tower-lsp):JSON-RPC 与服务器
  能力管线。
- [`tokio`](https://tokio.rs/):运行时,从 stdin 读、向 stdout 写。
- [`ktav`](https://crates.io/crates/ktav):每个 Ktav 绑定都使用的同一个
  解析器 crate。诊断、hover、文档符号都走 `ktav::parse`。
- [`dashmap`](https://crates.io/crates/dashmap):按 `Url` 键的线程安全
  文档存储。`TextDocumentSyncKind::FULL` 让循环保持简单:小型配置文件
  重新解析足够快,增量同步只会徒增代码而不省时间。

日志写到 stderr(stdout 留给 LSP 流量)。设置 `KTAV_LSP_LOG=debug`
提高日志级别。

