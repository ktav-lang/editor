# Ktav —— IntelliJ Platform 插件

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

> 在 JetBrains IDE 中为 [Ktav](https://github.com/ktav-lang/spec)
> 朴素配置格式提供编辑器支持。

这是 [`ktav-lang/editor`](https://github.com/ktav-lang/editor) 单一
仓库下的 `intellij/` 子项目。复用与 VS Code 扩展同一份 TextMate
语法,以 IntelliJ Platform 插件的形式封装。

## 支持的 IDE

任何基于 IntelliJ Platform **2024.3**(build `243`)及以上的 IDE:

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio(Iguana / Jellyfish 或更新版,待其平台基线跟进)
- Aqua、Rider、Fleet(在兼容版本上)

`untilBuild` 当前为 `251.*`(覆盖 2024.3 → 2025.1)。每次插件发布
前都会针对新 IDE 版本进行验证。

## 安装

### 从 JetBrains Marketplace(推荐)

1. 在任意支持的 IDE 中打开 **Settings → Plugins → Marketplace**。
2. 搜索 **Ktav**。
3. 点击 **Install** 并重启 IDE。

### 从本地 zip 安装(用于测试)

按下文构建插件,然后在 **Settings → Plugins** 点击齿轮 →
**Install Plugin from Disk…**,选择
`build/distributions/ktav-intellij-<version>.zip`。

## 功能

- 通过共享 TextMate 语法为 `.ktav` 文件提供语法高亮。
- 注释切换(`Ctrl/Cmd+/`)按 Ktav 规范前置 `# `。
- `{}` `[]` `()` 的括号匹配与自动闭合。
- 文件图标与 File → New → Ktav file(图标 TODO;暂时使用平台
  默认的文本文件图标)。

### LSP 功能(可选)

当与 Ktav 一起安装了
[LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) 插件时,
即可获得由 [`ktav-lsp`](../lsp) 提供的实时诊断、悬停、补全、
document symbols 和 semantic tokens。未安装 LSP4IJ 时,本插件仍以
TextMate-only 模式正常工作 —— 需要更丰富的功能时再从 Marketplace
安装 LSP4IJ 即可。

服务器二进制按以下顺序查找:

1. **Settings → Tools → Ktav** 中显式配置的路径。
2. 插件分发包中随附的二进制
   `bin/<platform>-<arch>/ktav-lsp`(当前版本未随附)。
3. 通过 shell `PATH` 解析的 `ktav-lsp` —— 用
   `cargo install ktav-lsp` 安装(与 VS Code 扩展的查找顺序一致)。

## 本地构建

依赖:

- JDK 17 或更新版(构建将 Kotlin toolchain 锁定在 17)。
- 不需要 Gradle —— 使用 wrapper。

```sh
./gradlew syncGrammars   # 把 ../grammars/ 镜像到 resources/
./gradlew buildPlugin    # 生成 build/distributions/*.zip
./gradlew runIde         # 启动一个加载本插件的沙盒 IDE
./gradlew verifyPlugin   # 运行 JetBrains plugin verifier
./gradlew test           # JUnit 5 冒烟测试
```

`processResources` 和 `compileKotlin` 任务依赖于 `syncGrammars`,
因此一条 `./gradlew buildPlugin` 就已经会拉取最新的
`../grammars/` 语法。如果忘了导致文件陈旧,重新运行
`syncGrammars` 即可。

## 发布

CI 通过环境变量 `INTELLIJ_PUBLISH_TOKEN` 中的 marketplace PAT 调用
`./gradlew publishPlugin`。Token 在
<https://plugins.jetbrains.com/author/me/tokens> 生成,并必须属于
marketplace 上 `lang.ktav` 插件 id 的 maintainer。有意不支持本地
发布 —— 仅通过 tagged CI 运行发布。

## 参考

- 格式规范与参考解析器:
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
- 参考 Rust 实现:
  [`ktav-lang/rust`](https://github.com/ktav-lang/rust)
- 其他绑定、LSP 服务器以及 VS Code 扩展位于
  [editor 单一仓库](https://github.com/ktav-lang/editor) 中。
