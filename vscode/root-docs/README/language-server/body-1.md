>>>>> lang=en
## Language server

The extension talks to the `ktav-lsp` binary over stdio. Prebuilt binaries for
Linux, macOS and Windows are attached to every
[GitHub release](https://github.com/ktav-lang/editor/releases) — download the one
for your platform and point `ktav.server.path` at it, or drop it on your `PATH`.
To build from source instead, run `cargo build --release` in the
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) directory.

>>>>> lang=ru
## Языковой сервер

Расширение обращается к бинарю `ktav-lsp` через stdio. Готовые бинарники для
Linux, macOS и Windows прикреплены к каждому
[релизу GitHub](https://github.com/ktav-lang/editor/releases) — скачайте
вариант для своей платформы и укажите путь к нему в `ktav.server.path`
или положите его в `PATH`. Чтобы вместо этого собрать сервер из исходников,
выполните `cargo build --release` в каталоге
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp).

>>>>> lang=zh
## 语言服务器

扩展通过 stdio 与 `ktav-lsp` 二进制通信。适用于 Linux、macOS 和 Windows 的
预编译二进制随每个 [GitHub 发布](https://github.com/ktav-lang/editor/releases)
一同提供 —— 下载对应平台的版本,并通过 `ktav.server.path` 指向它,
或直接将其放入 `PATH`。若要从源代码构建,请在
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) 目录中运行
`cargo build --release`。

