>>>>> lang=en
## Building locally

Prerequisites:

- JDK 17 or newer (the build pins the Kotlin toolchain to 17).
- Gradle is **not** required — use the wrapper.

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

>>>>> lang=ru
## Сборка локально

Требования:

- JDK 17 или новее (сборка пиннит Kotlin toolchain на 17).
- Gradle **не нужен** — используйте wrapper.

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

>>>>> lang=zh
## 本地构建

前提条件:

- JDK 17 或更新版(构建将 Kotlin toolchain 锁定在 17)。
- 不需要 Gradle —— 使用 wrapper。

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

