>>>>> lang=en
## License

Ktav's own code is MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE). Bundled third-party code retains its own licenses and notices; the first-party license does not replace them.

### Native binaries, notices and source

The license-checked release pipeline delivers each LSP executable together with `*.distribution.json`, readable `*.THIRD-PARTY-NOTICES.txt`, `*.NOTICE.json`, its compressed copy, the original Rust `*.COPYRIGHT-library.html`, `*.Cargo.lock`, and both Ktav license files. Platform-specific runtime notices are delivered alongside them. Keep the complete platform payload when redistributing; copying only the executable drops required materials.

The GNU Linux ARM64 payload also includes `ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz` and `*.source-binding.json`. Its retained startup code is subject to LGPL-2.1-or-later without the compiled-linking exception present in the reviewed x64 startup. The matching source archive supplies corresponding source and relinking materials; use its rebuild script to rebuild with modified startup source. This does not change the license of Ktav's own code.

Universal plugins carry the same complete payloads: `bin/<platform>/` in the VS Code extension and `lib/bin/<platform>/` in the IntelliJ plugin. The required ARM64 source archive adds approximately 94 MB to each universal plugin. CI verifies the actual VSIX/IntelliJ ZIP contents, executable notice resources and source bindings, not only staging directories. These checks do not retroactively replace historical published assets.

>>>>> lang=ru
## Лицензия

Собственный код Ktav распространяется под MIT OR Apache-2.0. См. [LICENSE-MIT](../../LICENSE-MIT) и [LICENSE-APACHE](../../LICENSE-APACHE). Сторонний код сохраняет собственные лицензии и уведомления; лицензия Ktav их не заменяет.

### Нативные бинарники, уведомления и исходники

Лицензионно проверяемый release pipeline поставляет каждый LSP executable вместе с `*.distribution.json`, читаемым `*.THIRD-PARTY-NOTICES.txt`, `*.NOTICE.json`, его сжатой копией, оригинальным отчётом Rust `*.COPYRIGHT-library.html`, `*.Cargo.lock` и обеими лицензиями Ktav. Рядом поставляются уведомления о runtime конкретной платформы. При дальнейшем распространении сохраняйте весь platform payload: копирование одного executable теряет обязательные материалы.

GNU Linux ARM64 payload также содержит `ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz` и `*.source-binding.json`. Используемый startup code подпадает под LGPL-2.1-or-later без compiled-linking exception, имеющегося у проверенного x64 startup. Matching source archive содержит соответствующие исходники и материалы для повторной линковки; приложенный rebuild script позволяет пересобрать executable с изменённым startup source. Лицензия собственного кода Ktav от этого не меняется.

Universal plugins содержат те же полные payloads: `bin/<platform>/` в VS Code extension и `lib/bin/<platform>/` в IntelliJ plugin. Обязательный ARM64 source archive добавляет около 94 МБ к каждому universal plugin. CI проверяет содержимое настоящих VSIX/IntelliJ ZIP, встроенные notice resources и source bindings, а не только staging directories. Эти проверки не заменяют задним числом исторические опубликованные assets.

>>>>> lang=zh
## 许可证

Ktav 自有代码采用 MIT OR Apache-2.0。详见 [LICENSE-MIT](../../LICENSE-MIT) 和 [LICENSE-APACHE](../../LICENSE-APACHE)。随附第三方代码保留各自的许可证和声明；Ktav 的许可证不能替代这些条款。

### 原生二进制文件、声明和源代码

经过许可证检查的发布流水线将每个 LSP 可执行文件与 `*.distribution.json`、可读的 `*.THIRD-PARTY-NOTICES.txt`、`*.NOTICE.json` 及其压缩副本、原始 Rust `*.COPYRIGHT-library.html`、`*.Cargo.lock` 和两份 Ktav 许可证一同提供。平台运行时声明也随附其中。再次分发时必须保留完整的平台文件集；仅复制可执行文件会丢失必需材料。

GNU Linux ARM64 文件集还包含 `ktav-lsp-source-aarch64-unknown-linux-gnu.tar.gz` 和 `*.source-binding.json`。其中保留的启动代码受 LGPL-2.1-or-later 约束，不具有已审查 x64 启动代码中的编译链接例外。匹配的源代码归档提供对应源代码和重新链接所需材料；可使用归档中的重建脚本，以修改后的启动源代码重新构建。这不会改变 Ktav 自有代码的许可证。

通用插件包含同样完整的文件集：VS Code 扩展中的 `bin/<platform>/`，以及 IntelliJ 插件中的 `lib/bin/<platform>/`。必需的 ARM64 源代码归档使每个通用插件增加约 94 MB。CI 检查实际 VSIX/IntelliJ ZIP 的内容、可执行文件内嵌声明及源代码绑定，而不只是暂存目录。这些检查不会追溯替换历史已发布文件。
