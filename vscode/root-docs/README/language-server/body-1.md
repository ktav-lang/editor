>>>>> lang=en
## Language server

The extension talks to the `ktav-lsp` binary over stdio. The VSIX
published to the Marketplace and Open VSX already bundles a `ktav-lsp`
binary for each of the six platforms CI builds — Linux, macOS and
Windows on x64 and arm64 — so it works out of the box; see
[Discovery order](#discovery-order) below for how the extension finds it.
On an unsupported platform, download a binary from a
[GitHub release](https://github.com/ktav-lang/editor/releases) and point
`ktav.server.path` at it, or drop it on your `PATH`. To build from source
instead, run `cargo build --release` in the
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) directory.

>>>>> lang=ru
## Языковой сервер

Расширение обращается к бинарю `ktav-lsp` через stdio. VSIX,
публикуемый в Marketplace и Open VSX, уже содержит `ktav-lsp` для
каждой из шести платформ, которые собирает CI, — Linux, macOS и
Windows на x64 и arm64, — так что всё работает из коробки; порядок
поиска бинарника описан ниже в разделе «Порядок поиска». На
неподдерживаемой платформе скачайте бинарник с
[релиза GitHub](https://github.com/ktav-lang/editor/releases) и
укажите путь к нему в `ktav.server.path` или положите его в `PATH`.
Чтобы вместо этого собрать сервер из исходников, выполните
`cargo build --release` в каталоге
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp).

>>>>> lang=zh
## 语言服务器

扩展通过 stdio 与 `ktav-lsp` 二进制通信。发布到 Marketplace 和 Open
VSX 的 VSIX 已经为 CI 构建的六个平台 —— Linux、macOS 和 Windows 的
x64 与 arm64 —— 各自捆绑了一份 `ktav-lsp`,因此开箱即用;具体查找
顺序见下方“查找顺序”一节。在不受支持的平台上,请从
[GitHub 发布](https://github.com/ktav-lang/editor/releases)下载对应
版本,并通过 `ktav.server.path` 指向它,或将其放入 `PATH`。若要从
源代码构建,请在
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) 目录中运行
`cargo build --release`。

