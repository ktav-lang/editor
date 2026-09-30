>>>>> lang=en
## Installing a local build into your IDEs

`scripts/dev-install.mjs` (Node 20+) builds `ktav-lsp` for this machine,
the VSIX and the IntelliJ plugin ZIP, and installs them into IDEs you
name explicitly:

```sh
node scripts/dev-install.mjs list
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium --apply
```

Without `--apply` it only validates the targets and prints the plan. It
never deletes anything: an existing `ktav-intellij` folder must be the
`lang.ktav` plugin and is moved to
`<ide-config>/ktav-dev-backups/<stamp>/previous/`. Symlinks, other
plugins, settings and caches are left alone, and a running JetBrains IDE
is refused. VS Code-family editors are updated only through their own
`--install-extension` CLI.

>>>>> lang=ru
## Установка локальной сборки в IDE

`scripts/dev-install.mjs` (Node 20+) собирает `ktav-lsp` для текущей
машины, VSIX и ZIP плагина IntelliJ и устанавливает их в явно указанные
IDE:

```sh
node scripts/dev-install.mjs list
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium --apply
```

Без `--apply` скрипт только проверяет цели и печатает план. Он ничего не
удаляет: существующая папка `ktav-intellij` должна быть плагином
`lang.ktav` и переносится в
`<ide-config>/ktav-dev-backups/<stamp>/previous/`. Symlink-и, другие
плагины, настройки и кэши не трогаются, запущенная JetBrains IDE
отклоняется. Редакторы семейства VS Code обновляются только через их
собственный CLI `--install-extension`.

>>>>> lang=zh
## 将本地构建安装到 IDE

`scripts/dev-install.mjs`(Node 20+)为本机构建 `ktav-lsp`、VSIX 和
IntelliJ 插件 ZIP,并安装到显式指定的 IDE:

```sh
node scripts/dev-install.mjs list
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium
node scripts/dev-install.mjs install --jetbrains WebStorm2025.3 --vscode codium --apply
```

不带 `--apply` 时只校验目标并打印计划。它从不删除任何内容:已有的
`ktav-intellij` 目录必须是 `lang.ktav` 插件,并会被移动到
`<ide-config>/ktav-dev-backups/<stamp>/previous/`。符号链接、其他插件、
设置和缓存都不会被改动,正在运行的 JetBrains IDE 会被拒绝。VS Code 系列
编辑器只通过其自身的 `--install-extension` CLI 更新。

