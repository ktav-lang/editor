# 为 ktav-lang/editor 做贡献

**语言:** [English](CONTRIBUTING.md) · [Русский](CONTRIBUTING.ru.md) · **简体中文**

## 核心规则

### 1. 每个 bug 修复都伴随一个回归测试

发现 bug 时,**在修复之前** 先写一个复现它的测试 —— 测试在
`main` 分支上 **必须失败**,修复之后才通过。两者放在同一个 PR。

### 2. 不要在编辑器层重新发明格式

编辑器扩展和 LSP 是 `ktav` 解析器 crate 的薄消费者。格式行为属于
Rust crate
([`ktav-lang/rust`](https://github.com/ktav-lang/rust)) —— 改那里
等于同步更新所有消费者。这里仅收 **编辑器特定的人体工学**
(TextMate scope、LSP 功能接线、IDE 特定集成)。

如果改动需要格式变更,先去
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) 讨论。

### 3. 一个概念一次提交

提交要保持原子:bug 修复与其测试一起、新功能与其测试一起、
重命名单独、重构单独。`git log --oneline` 应当读起来像 changelog。
不要使用 `feat:` / `fix:` 前缀。

## 开发环境

每个子项目都有自己的工具链。详见各子项目下的 `README.md`:

- `grammars/` —— 纯 JSON,无需构建
- `vscode/` —— Node + `vsce`
- `intellij/` —— JDK 17 + Gradle
- `lsp/` —— Rust 1.70+

## 语言政策

本仓库参与组织级三语政策(EN / RU / ZH)。每份 prose 文档都有三种
并行版本 —— 命名约定和"三份一并更新"规则见
[`ktav-lang/.github/AGENTS.md`](https://github.com/ktav-lang/.github/blob/main/AGENTS.md)。

### 贡献的许可

除非您另有明确声明,否则您有意提交以纳入本项目的任何贡献(按
Apache-2.0 许可证中的定义),均按 **MIT OR Apache-2.0** 双重许可,
不附加任何额外条款或条件。
