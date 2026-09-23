# 适用于 Visual Studio Code 的 Ktav

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=ktav-lang.ktav)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Registry-c160ef)](https://open-vsx.org/extension/ktav-lang/ktav)

为 [Ktav](https://github.com/ktav-lang/spec) 配置格式提供语法高亮与语言支持,内置于 Visual Studio Code。

**Languages:** [English](../README.md) · [Русский](README.ru.md) · **简体中文**

## 功能

- `.ktav` 文件的语法高亮 —— 键、标量、`::` 字符串字面量、多行块、行内/块状复合结构、注释
- `{}`、`[]`、`()` 的括号匹配与自动闭合
- 使用 `#` 切换注释
- 对象 / 数组 / 括号复合结构内的自动缩进

配合 [`ktav-lsp`](https://github.com/ktav-lang/editor/tree/main/lsp) 语言服务器:

- 解析错误与类型不匹配的诊断
- 语义高亮与文档大纲(键与嵌套)
- 标量类型的悬停信息

## 语言服务器

扩展通过 stdio 与 `ktav-lsp` 二进制通信。适用于 Linux、macOS 和 Windows 的
预编译二进制随每个 [GitHub 发布](https://github.com/ktav-lang/editor/releases)
一同提供 —— 下载对应平台的版本,并通过 `ktav.server.path` 指向它,
或直接将其放入 `PATH`。若要从源代码构建,请在
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) 目录中运行
`cargo build --release`。

### 查找顺序

激活时,扩展按以下顺序查找服务器:

1. **显式设置** —— `ktav.server.path`(绝对路径)。
2. **内置二进制** —— `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]`(当构建随附它时)。
3. **PATH** —— 回退为启动 `ktav-lsp`,交由操作系统解析。

如果以上均告失败,将弹出错误提示,并由 `Ktav Language Server` 输出通道记录此次失败。

### 设置

| 设置                  | 类型                                | 默认值     | 说明                                    |
|----------------------|-------------------------------------|---------|--------------------------------------------------------------------------|
| `ktav.server.path`   | string                              | `""`    | `ktav-lsp` 的绝对路径。留空表示自动查找(见上文)。 |
| `ktav.trace.server`  | `"off"` \| `"messages"` \| `"verbose"` | `"off"` | 将 JSON-RPC 流量记录到输出通道。                           |

## 安装

从 Visual Studio Code Marketplace 安装:

```
ext install ktav-lang.ktav
```

或者打开扩展侧边栏(`Ctrl+Shift+X` / `Cmd+Shift+X`),搜索 **Ktav**。

该扩展同时发布在 [Open VSX](https://open-vsx.org/),适用于 VSCodium 及其他兼容编辑器。

## 示例

```ktav
## Ktav — no quotes, no commas, no indentation traps.
service: socks5-rotator
## bare scalars are auto-typed: int / float / bool / null
port: 20082
debug: true

## Dotted keys are a flat alternative to nesting.
node.host: a.example
node.port: 1080

## '::' forces a literal string — keeps "8080" a string, not a number.
node.token:: 8080

## Need a literal '.' or ':' inside a key? Escape it (new in 0.6.0).
metric.http\.requests: 42

## Comma-free arrays and inline objects.
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

## Multi-line strings.
motd: (
    Welcome to the node.
    Please behave.
)
```

## 生态系统 —— 官方绑定

Ktav 是同一个 Rust 核心,以各语言的薄绑定封装 —— 行为一致、
速度原生,并可在浏览器中通过 WebAssembly 运行:

| 语言            | 安装                                 | 仓库 |
|-----------------|--------------------------------------|------------|
| Rust            | `cargo add ktav`                     | [ktav-lang/rust](https://github.com/ktav-lang/rust) |
| JavaScript / TS | `npm i @ktav-lang/ktav`              | [ktav-lang/js](https://github.com/ktav-lang/js) |
| Python          | `pip install ktav`                   | [ktav-lang/python](https://github.com/ktav-lang/python) |
| Go              | `go get github.com/ktav-lang/golang` | [ktav-lang/golang](https://github.com/ktav-lang/golang) |
| PHP             | `composer require ktav-lang/ktav`    | [ktav-lang/php](https://github.com/ktav-lang/php) |
| Java / JVM      | `io.github.ktav-lang:ktav:0.8.0`     | [ktav-lang/java](https://github.com/ktav-lang/java) |
| C# / .NET       | `dotnet add package Ktav`            | [ktav-lang/csharp](https://github.com/ktav-lang/csharp) |

## 资源

- [Ktav 规范](https://github.com/ktav-lang/spec)
- [浏览器内转换器](https://ktav-lang.github.io/) —— JSON / YAML / TOML / INI ⇄ Ktav
- [tree-sitter 语法](https://github.com/ktav-lang/tree-sitter-ktav)
- [全部仓库](https://github.com/ktav-lang)
- [问题跟踪](https://github.com/ktav-lang/editor/issues)

## 许可证

MIT OR Apache-2.0 —— 见 [LICENSE-MIT](./LICENSE-MIT) 与 [LICENSE-APACHE](./LICENSE-APACHE)。
