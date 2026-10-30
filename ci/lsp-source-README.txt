Ktav LSP source/relink materials — audit prototype

ENGLISH

The Linux ARM64 executable uses GNU C Library startup code, including
sysdeps/aarch64/start-2.33.S from the exact Zig source revision recorded in
provenance.json. That file is licensed under GNU LGPL 2.1 or, at your option,
any later version. Its original copyright and license notices are preserved
in upstream/zig-source.tar.gz. The full LGPL 2.1 text is in legal/LGPL-2.1.txt.
Other upstream files may carry additional permissions or separate terms;
those original notices remain applicable. Ktav's own source remains under
its existing MIT OR Apache-2.0 license; both texts accompany project/.

You may modify your copy of the work for your own use and reverse engineer
it to debug such modifications, subject to the applicable original licenses.
Ktav adds no contractual prohibition on those activities. The source package
is intended to support rebuilding/relinking with a modified startup library,
not merely to provide links to somebody else's source repository.

Contents:
- project/: actual LSP source with notice-only overlay, exact Cargo.lock and
  vendored Cargo dependencies; the lock is not refreshed during rebuilding.
- upstream/zig-source.tar.gz: complete original source of the recorded Zig
  revision, including its C library sources, assembly and SDK headers.
- toolchain/: the actual target Rust standard-library link inputs used by
  the build; original standard-library notices accompany legal/.
- legal/: original GNU license, source-bound notices, provenance and the
  exact compressed notice resource needed by the provided build hook.
- MANIFEST.json: checksums of the original supplied files. It is evidence
  of the original materials, not a restriction on modifying your copy.

Rebuild on Linux with Python 3.12+, Rust 1.99.1, Zig 0.14.0 and
cargo-zigbuild 0.24.0 installed. The Rust compiler's ordinary host tools are
prerequisites; Cargo dependencies and target standard-library inputs are
included. From this directory run:

    bash rebuild.sh aarch64-unknown-linux-gnu

Use x86_64-unknown-linux-gnu only for a source package whose provenance.json
records that target. The script extracts the verified original Zig source
once, then uses its lib/ directory through ZIG_LIB_DIR. To modify the startup
library, edit the extracted source and run the same command again. Existing
extracted source is never silently overwritten. A rebuilt executable appears
under target-relinked/<target>/release/ktav-lsp. Cargo dependency resolution
is offline and locked. Source changes are allowed; binary identity is not
expected after a modification.

This is an audit prototype. Successful relinking alone does not prove that
all notices are included in the final release or that historical published
artifacts have been repaired. Production distribution must provide the
matching source materials with equivalent access from the same distribution
location, and retain the applicable original notices with each executable.

РУССКИЙ

Linux ARM64 executable использует startup code GNU C Library, в том числе
sysdeps/aarch64/start-2.33.S из точной ревизии Zig, записанной в provenance.json.
Этот файл распространяется по GNU LGPL 2.1 либо, по вашему выбору, более
поздней версии. Исходные copyright и условия сохранены в архиве исходников;
полный текст LGPL 2.1 находится в legal/LGPL-2.1.txt. Собственный код Ktav
сохраняет MIT OR Apache-2.0; оба текста находятся в project/. Условия и
дополнительные разрешения других upstream-файлов также сохраняют силу.

Вы можете изменять свою копию для собственного использования и выполнять
обратную разработку для отладки таких изменений с соблюдением исходных
лицензий. Ktav не вводит договорного запрета на эти действия.

В комплект входят фактический LSP source с notice-only overlay, точный
Cargo.lock, vendored Cargo dependencies, полный исходный архив Zig, исходные
SDK headers, target Rust standard-library link inputs и их уведомления.
MANIFEST.json фиксирует исходный комплект, но не запрещает его изменение.

Для пересборки на Linux нужны Python 3.12+, Rust 1.99.1, Zig 0.14.0 и
cargo-zigbuild 0.24.0. Выполните указанную выше команду с target из
provenance.json. Скрипт распакует проверенный Zig source один раз и будет
использовать его lib/ через ZIG_LIB_DIR. Измените распакованный startup source
и повторите команду: ваши изменения не перезаписываются. Cargo работает
с включённым offline и существующим lockfile. Результат находится в
 target-relinked/<target>/release/ktav-lsp.

Это audit prototype, а не завершённое исправление опубликованных пакетов.
В production matching source должен предоставляться наравне с executable
из того же места распространения; исходные условия и уведомления должны
сохраняться. Одной успешной пересборки для этого недостаточно.

中文

Linux ARM64 可执行文件使用 GNU C Library 的启动代码，包括 provenance.json
所记录的准确 Zig 修订中的 sysdeps/aarch64/start-2.33.S。该文件采用 GNU
LGPL 2.1，或由您选择的更新版本。原始版权声明和许可条件保留在源码归档中，
完整 LGPL 2.1 文本位于 legal/LGPL-2.1.txt。Ktav 自有代码仍采用 MIT OR
Apache-2.0，两个许可文本均随 project/ 提供。其他上游文件的原始条件和
附加许可仍然适用。

在遵守原始许可证的前提下，您可以为自己使用而修改作品，并为调试这些修改
进行逆向工程。Ktav 不额外通过合同禁止这些活动。

此包包含实际 LSP 源码和仅用于通知的 overlay、准确的 Cargo.lock、vendored
Cargo 依赖、完整的 Zig 源码归档及 SDK headers、实际目标 Rust 标准库链接
输入和原始许可通知。MANIFEST.json 用于校验最初提供的文件，不限制修改。

在 Linux 上安装 Python 3.12+、Rust 1.99.1、Zig 0.14.0 和 cargo-zigbuild
0.24.0，然后使用 provenance.json 中的 target 执行上面的命令。脚本仅首次
解压已校验的 Zig 源码，随后通过 ZIG_LIB_DIR 使用其 lib/ 目录。修改已解压
的启动源码后可再次执行命令，已有源码不会被静默覆盖。Cargo 使用 offline
模式和现有 lockfile，生成文件位于 target-relinked/<target>/release/ktav-lsp。

这是审计原型，不表示历史发布包已被修复。正式分发仍须在同一分发位置提供
可同等获取的匹配源码，并为每份可执行文件保留适用的原始通知和许可条件。
仅成功重新链接并不足以完成这些分发义务。
