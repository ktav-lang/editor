>>>>> lang=en
## Install

Source builds require Rust 1.88+ for the current locked dependency graph.
CI builds all targets on the declared minimum as well as running the
existing stable-toolchain checks. Unlocked installs may need newer Rust
if transitive dependencies raise their requirements.

```bash
cargo install ktav-lsp
```

This drops a `ktav-lsp` binary into `~/.cargo/bin/`. No configuration file
required.

An ordinary install may resolve newer transitive dependencies. Once 0.8.0 is
published, reproduce that release's dependency graph with:

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

>>>>> lang=ru
## Установка

Для сборки исходников с текущим закреплённым графом зависимостей нужен
Rust 1.88+. CI собирает все targets на объявленном минимуме и сохраняет
проверки stable-toolchain. Установка без `--locked` может потребовать
более новый Rust при повышении требований транзитивных зависимостей.

```bash
cargo install ktav-lsp
```

Это положит бинарь `ktav-lsp` в `~/.cargo/bin/`. Конфигурационный файл
не требуется.

При обычной установке могут разрешиться более новые транзитивные зависимости.
После публикации 0.8.0 воспроизвести граф зависимостей этого выпуска можно так:

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

>>>>> lang=zh
## 安装

当前锁定依赖图的源码构建需要 Rust 1.88+。CI 在声明的最低版本上
构建所有 targets,同时保留原有 stable 工具链检查。若传递依赖提高
要求,不带 `--locked` 的安装可能需要更新的 Rust。

```bash
cargo install ktav-lsp
```

这会将 `ktav-lsp` 二进制安装到 `~/.cargo/bin/`。无需配置文件。

普通安装可能解析到更新的传递依赖。0.8.0 发布后，可用以下命令复现该版本的依赖图：

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

