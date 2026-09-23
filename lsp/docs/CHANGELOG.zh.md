# 变更日志 —— `ktav-lsp`

本文件记录 `ktav-lsp` crate 的全部重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);crate
遵循 [Semantic Versioning](https://semver.org/),并采用 Cargo 惯例:
在 1.0 之前,MINOR 递进视为破坏性变更。

**Languages:** [English](../CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

## 未发布

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 详情见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(未发布章节)。

## [0.5.0] — 2026-05-27

同步到 `ktav 0.5.0` 与 spec `0.5.0`。

### 变更

- `Cargo.toml`:`ktav = "0.5.0"`(原为 `"0.3.1"`)。
- 许可证:双许可 `MIT OR Apache-2.0`(原为仅 MIT)。
- `tokens`:注释现在要求 `##`;单个 `#` 为普通字符。
- `tokens`:移除 `:i` 与 `:f` 类型标记(spec 0.5.0 已移除);类型由标量的
  词法形式推断。
- `tokens`:`looks_numeric` 扩展支持十六进制(`0x`)、八进制(`0o`)、二进制
  (`0b`)前缀与下划线分隔符(spec 0.5.0 数字字面量)。
- `diagnostics`:移除遗留的 `InvalidTypedScalar` 范围启发式。

## [0.3.1] — 2026-05-10

同步到 `ktav 0.3.1` 与 spec `0.1.1`。文档符号大纲新增顶层 Array 支持。

### 变更

- `build_symbols` 现在识别顶层 Array 根(spec § 5.0.1),并将其各项渲染为
  大纲条目 `[0]`、`[1]`、…。对象根不变。
- `Cargo.toml`:`ktav = "0.3.1"`(原为 `"0.3.0"`)。

### 测试

- 新增:`document_symbols_built_from_top_level_array_of_scalars`、
  `document_symbols_top_level_array_of_objects_have_children`(位于
  `tests/integration.rs`)。
- 新增:`top_level_array_of_scalars_preserved`、
  `top_level_array_of_objects_preserved`(位于
  `tests/format_pipeline.rs`)。
- 更新:既有 `MissingSeparatorSpace` / `EmptyKey` / 西里尔字节列固件改用
  前置锚定对,确保 malformed-pair 分支在 spec 0.1.1 下仍被覆盖。

## [0.1.5] — 2026-05-01

诊断管线的内部重构。公共 API 无变化。

### 变更

- **诊断** 现在消费 `ktav 0.1.5+` 的 `ktav::Error::Structured(ErrorKind)`:
  `Diagnostic.range` 直接由变体的字节偏移 `Span` 经 `Span::line_col`
  构建,而不再通过正则匹配格式化的错误消息、并将出错行重新过一遍共享
  行分类器来推导。范围精度现在由解析器自身对失败点的认知驱动,对每个
  类别都得到严格相等或更窄的范围 —— 例如 `key:value\n` 的
  `MissingSeparatorSpace` 现在高亮字节 4..9(粘连的主体 `value`),
  而不再是 3..9(冒号 + 主体)。
- 现有的 `tests/error_format_pinning.rs` 同时接受 `Error::Syntax(_)`
  与 `Error::Structured(_)`,并断言渲染后的 Display 字符串 —— 契约是
  消息文本,而非枚举变体。
- `tests/integration.rs` 的范围预期收紧到新的 structured spans
  (`MissingSeparatorSpace` 4..9、`InvalidTypedScalar` 6..10 等)。
  西里尔字节列测试重新固定到主体 span 的起点(`имя:значение\n` 的
  字节 7)。
- `tests/spec_conformance.rs` 接受新的 `MissingSeparator` /
  `UnbalancedBracket` Display 字符串,作为规范类别 `OrphanLine` /
  `MismatchedBracket` 的别名。

### 新增

- `tests/structured_diagnostics.rs` —— 逐变体断言,覆盖规范定义的全部
  10 个 `ErrorKind` 变体,外加一个西里尔字节列测试,在诊断范围转换
  路径上演练 `tokens::byte_to_utf16`。

### 内部

- 正则抽取代码(`extract_line_number`、`extract_quoted_key`、各类别的
  `range_for_*` 辅助函数)保留为 `compute_range_legacy`,仅通过
  `Error::Syntax(_)` 可达。`ktav 0.1.5+` 的解析器不再构造该变体,但
  `ktav::Error` 标注了 `#[non_exhaustive]`,下游包装器仍可能暴露该
  变体 —— legacy 路径作为纵深防御保留。
- `ktav` 依赖从 `0.1.4` 升至 `0.1.5`(registry pin),以引入结构化错误
  API。需要针对未发布 ktav 进行本地 sibling-checkout 开发时,可通过
  `.cargo/config.toml` patch 恢复(per-developer,不纳入 git)。

### 破坏性(内部)

- crate 版本提升到 **0.1.5**,因此 `ktav-lsp` 0.1.4 形态的调用方
  (无外部 —— 这是叶子二进制)可干净地针对 `ktav 0.1.5+` 重新编译。

## [0.1.0] — 2026-04-26

首个版本。

### 新增

- 通过 stdin/stdout 提供的单二进制 LSP 服务器(`ktav-lsp`),基于
  `tower-lsp` 0.20 与 `tokio`。
- **诊断**:在 `did_open` / `did_change` / `did_save` 时重新解析,并在
  出错行上发布 `ktav::Error::Syntax` 消息。行号通过两条正则从消息中恢复,
  覆盖 `ktav` 0.1.4 的全部形态(`Line N: …`、`Invalid key at line N: …`、
  `Empty key at line N`)。
- **Hover**:对 `key: …` 行,在已解析的树中按点状路径查找,并报告推断的
  类型与值。
- **补全**:在 `:` 分隔符之后上下文感知补全:提供 `null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)、raw 标记
  (`:`)以及类型标记(`i`、`f`)。
- **文档符号**:由 `Value::Object` 构建的大纲树;标量映射为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens (full)**:六种 token 类型 —— `comment`、`keyword`、
  `number`、`string`、`property`、`operator`。索引顺序为公开 legend
  的一部分。
- 通过 `tracing` 将日志写到 stderr;级别由 `KTAV_LSP_LOG` 环境变量控制。
- 覆盖诊断正则、semantic tokens legend 与文档符号树结构的集成测试与
  单元测试。
