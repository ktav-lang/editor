# 变更日志 —— `ktav-lsp`

本文件记录 `ktav-lsp` crate 的全部重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);crate
遵循 [Semantic Versioning](https://semver.org/),并采用 Cargo 惯例:
在 1.0 之前,MINOR 递进视为破坏性变更。

**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

## [0.1.0] —— 2026-04-26

首个版本。

### 新增

- 通过 stdin/stdout 提供的单二进制 LSP 服务器(`ktav-lsp`),基于
  `tower-lsp` 0.20 与 `tokio`。
- **诊断**:在 `did_open` / `did_change` / `did_save` 时重新解析,
  在出错行上发布 `ktav::Error::Syntax` 消息。行号通过两条正则从消息中
  恢复,覆盖 `ktav` 0.1.4 的全部形态(`Line N: …`、`Invalid key at
  line N: …`、`Empty key at line N`)。
- **Hover**:对 `key: …` 行,在已解析的树中按点状路径查找并报告推断
  的类型与值。
- **补全**:在 `:` 分隔符之后上下文感知:`null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)、
  raw 标记(`:`)以及类型标记(`i`、`f`)。
- **文档符号**:由 `Value::Object` 构建的大纲树;标量映射为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens (full)**:六种 token 类型 —— `comment`、
  `keyword`、`number`、`string`、`property`、`operator`。索引顺序
  是公开 legend 的一部分。
- 通过 `tracing` 将日志写到 stderr;级别由 `KTAV_LSP_LOG` 环境变量
  控制。
- 覆盖诊断正则、semantic tokens legend 与文档符号树结构的集成与
  单元测试。
