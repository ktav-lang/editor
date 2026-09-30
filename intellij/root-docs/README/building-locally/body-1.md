>>>>> lang=en
## Building locally

Prerequisites:

- JDK 17 or newer (the build pins the Kotlin toolchain to 17).
- Gradle is **not** required — use the wrapper.
- Rust 1.88+ when rebuilding the current locked LSP.

```sh
./gradlew syncGrammars   # mirror ../grammars/ into resources/
./gradlew buildPlugin    # produces build/distributions/*.zip
./gradlew runIde         # boot a sandbox IDE with the plugin loaded
./gradlew verifyPlugin   # run the JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke tests
```

The `processResources` and `compileKotlin` tasks depend on
`syncGrammars` so a plain `./gradlew buildPlugin` already pulls in the
latest grammar from `../grammars/`. If you forget and the file is stale,
just rerun `syncGrammars`.

For an installed IDE, close that IDE manually, then run the portable helper
from any working directory:

```sh
./dev-rebuild.sh --plugins-dir /path/to/selected-ide/plugins --platform linux-x64 --no-restart
```

Use `--plugin` (Python 3 required for ZIP installation) to rebuild/install
the whole Ktav plugin.
If multiple output ZIPs exist, select the exact built archive with `--plugin-zip PATH`.
`--ide-executable PATH` optionally launches only the selected IDE afterward;
it never terminates an existing IDE. `--cache-dir PATH` only reports the log
location, without deleting caches or logs. Equivalent environment variables:
`KTAV_PLUGINS_DIR`, `KTAV_PLATFORM`, `KTAV_PLUGIN_ZIP`,
`KTAV_IDE_EXECUTABLE`, `KTAV_IDE_CACHE_DIR`. No machine-specific defaults.
Only `plugins/ktav-intellij` is replaced; unrelated plugins and ZIPs stay put.
See `./dev-rebuild.sh --help` for all supported platform names.

>>>>> lang=ru
## Сборка локально

Требования:

- JDK 17 или новее (сборка пиннит Kotlin toolchain на 17).
- Gradle **не нужен** — используйте wrapper.
- Rust 1.88+ при пересборке текущего locked LSP.

```sh
./gradlew syncGrammars   # mirror ../grammars/ into resources/
./gradlew buildPlugin    # produces build/distributions/*.zip
./gradlew runIde         # boot a sandbox IDE with the plugin loaded
./gradlew verifyPlugin   # run the JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke tests
```
Задачи `processResources` и `compileKotlin` зависят от
`syncGrammars`, поэтому простой `./gradlew buildPlugin` уже
подтягивает свежую грамматику из `../grammars/`. Если вы забыли и
файл устарел, просто перезапустите `syncGrammars`.

Для установленного IDE закройте выбранный IDE вручную, затем запустите
переносимый helper из любого рабочего каталога:

```sh
./dev-rebuild.sh --plugins-dir /path/to/selected-ide/plugins --platform linux-x64 --no-restart
```

`--plugin` пересобирает и устанавливает весь Ktav-плагин (для ZIP нужен
Python 3).
Если ZIP-файлов несколько, выберите конкретный собранный архив через `--plugin-zip PATH`.
`--ide-executable PATH` опционально запускает только выбранный IDE после
установки; работающие IDE не завершаются. `--cache-dir PATH` лишь сообщает
путь лога, не удаляя кэши и логи. Эквивалентные переменные окружения:
`KTAV_PLUGINS_DIR`, `KTAV_PLATFORM`, `KTAV_PLUGIN_ZIP`,
`KTAV_IDE_EXECUTABLE`, `KTAV_IDE_CACHE_DIR`. Машинных defaults нет.
Заменяется только `plugins/ktav-intellij`; чужие плагины и ZIP сохраняются.
Все поддерживаемые платформы перечислены в `./dev-rebuild.sh --help`.

>>>>> lang=zh
## 本地构建

前提条件:

- JDK 17 或更新版(构建将 Kotlin toolchain 锁定在 17)。
- 不需要 Gradle —— 使用 wrapper。
- 重建当前锁定的 LSP 时需要 Rust 1.88+。

```sh
./gradlew syncGrammars   # mirror ../grammars/ into resources/
./gradlew buildPlugin    # produces build/distributions/*.zip
./gradlew runIde         # boot a sandbox IDE with the plugin loaded
./gradlew verifyPlugin   # run the JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke tests
```
`processResources` 和 `compileKotlin` 任务依赖于 `syncGrammars`,
因此一条 `./gradlew buildPlugin` 就已经会拉取最新的
`../grammars/` 语法。如果忘了导致文件陈旧,重新运行
`syncGrammars` 即可。

更新已安装的 IDE 时,先手动关闭选定的 IDE,然后从任意工作目录
运行可移植 helper:

```sh
./dev-rebuild.sh --plugins-dir /path/to/selected-ide/plugins --platform linux-x64 --no-restart
```

`--plugin` 重建并安装整个 Ktav 插件(ZIP 安装需要 Python 3)。
存在多个输出 ZIP 时,通过
`--plugin-zip PATH` 选择实际构建的具体归档。
`--ide-executable PATH` 可在安装后仅启动选定 IDE,不会终止已运行的
IDE。`--cache-dir PATH` 只报告日志位置,不会删除缓存或日志。
对应环境变量为 `KTAV_PLUGINS_DIR`、`KTAV_PLATFORM`、`KTAV_PLUGIN_ZIP`、
`KTAV_IDE_EXECUTABLE`、`KTAV_IDE_CACHE_DIR`,没有机器私有的默认值。
仅替换 `plugins/ktav-intellij`,其他插件和 ZIP 保持不变。
所有支持的平台名称参见 `./dev-rebuild.sh --help`。

