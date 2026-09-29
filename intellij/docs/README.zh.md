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

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 打包在插件分发包中的二进制
   `lib/bin/<platform>-<arch>/ktav-lsp`,每个受支持平台各一份。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp` 安装(与 VS Code 扩展的查找顺序一致)。

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
