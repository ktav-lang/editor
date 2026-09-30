# Ktav —— IntelliJ Platform 插件

**Languages:** [English](../README.md) · [Русский](README.ru.md) · **简体中文**

> 在 JetBrains IDE 中为 [Ktav](https://github.com/ktav-lang/spec)
> 朴素配置格式提供编辑器支持。

这是 [`ktav-lang/editor`](https://github.com/ktav-lang/editor) 单一
仓库下的 `intellij/` 子项目,提供原生的 IntelliJ Platform 词法分析
与高亮(而非 VS Code 扩展所用的共享 TextMate 语法),以 IntelliJ
Platform 插件的形式封装。

## 支持的 IDE

任何基于 IntelliJ Platform **2023.1**(build `231`)及以上的 IDE ——
没有上限(未设置 `untilBuild`):

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio(待其平台基线达到 2023.1)
- Aqua、Rider、Fleet(在兼容版本上)

## 安装

### 从 JetBrains Marketplace(推荐)

1. 在任意支持的 IDE 中打开 **Settings → Plugins → Marketplace**。
2. 搜索 **Ktav**。
3. 点击 **Install** 并重启 IDE。

### 从本地 zip 安装(用于测试)

按下文构建插件,然后在 **Settings → Plugins** 中点击齿轮 →
**Install Plugin from Disk…**,选择
`build/distributions/ktav-intellij-<version>.zip`。

## 功能

- 原生语法高亮支持 `.ktav` 文件(自有词法分析器,不依赖 TextMate)。
  增量高亮保留首个内容行确定的根类型和嵌套 Object/Array 上下文；
  多词裸键内部的引号仍视为普通字符。
- 多行容器和字符串的折叠使用相同的词法标记，不解析 raw 标量或多行字符串正文。
- 注释切换(`Ctrl/Cmd+/`)按 Ktav 规范在行首添加 `## `。
- `{}` `[]` `()` 的括号匹配与自动闭合。
- 文件图标与 File → New → Ktav file(图标 TODO;暂时使用平台
  默认的文本文件图标)。

### LSP 功能

插件通过自带的内置 LSP 客户端与 [`ktav-lsp`](../../lsp) 通信 ——
实时诊断和整文件格式化(**Reformat Code**)无需安装单独的 LSP
插件(例如 LSP4IJ)。这是 IntelliJ 插件目前集成的 LSP 功能。

服务器还支持悬停、补全、文档符号和语义令牌,但内置 IntelliJ
客户端未集成这些功能。语法高亮由插件的原生词法分析器提供,
而非 LSP 语义令牌。

诊断绑定每个项目当前打开的客户端、版本及文档会话。缓存注解和编辑器
直接高亮保持独立：过期发布或关闭一个项目不会清除另一个拥有者的错误。

每个项目拥有自己的 LSP 客户端和文档订阅。同一 `.ktav` 文档在两个项目中
打开时,会分别同步到两个客户端;在一个项目中关闭它不会停止另一个项目的
更新。恢复的编辑器标签页使用相同的打开流程,重新打开时会将当前文本作为
新的文档会话发送。项目关闭会移除其订阅并关闭客户端,包括延迟启动的
服务器进程。
传输关闭时,等待中的请求会失败,新请求也会立即被拒绝,而不会继续等待
响应超时。

格式化只应用于发起请求时已同步的文档会话。编辑(即使通过 Undo 恢复为
原始文本)、关闭/重新打开、项目/客户端关闭、取消或结果过期,都会阻止
旧响应替换文档。成功的结果会在编辑器线程中一并检查和应用,同步回每个
拥有该文档的项目,并具有独立的 **Undo** 步骤。

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 打包在插件分发包中的二进制
   `lib/bin/<platform>-<arch>/ktav-lsp`,每个受支持平台各一份。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp --version 0.8.0 --locked` 安装(与 VS Code 扩展的查找顺序一致)。

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

## 发布

CI 通过环境变量 `INTELLIJ_PUBLISH_TOKEN` 中的 marketplace PAT 调用
`./gradlew publishPlugin`。Token 在
<https://plugins.jetbrains.com/author/me/tokens> 生成,且必须属于
marketplace 上 `lang.ktav` 插件 id 的 maintainer。有意不支持本地
发布 —— 仅通过 tagged CI 运行发布。

## 参考

- 格式规范与参考解析器:
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
- 参考 Rust 实现:
  [`ktav-lang/rust`](https://github.com/ktav-lang/rust)
- 其他绑定、LSP 服务器以及 VS Code 扩展位于
  [editor 单一仓库](https://github.com/ktav-lang/editor) 中。
