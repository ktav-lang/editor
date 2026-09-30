# 变更日志

**Languages:** [English](../CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 Ktav IntelliJ Platform 插件的所有重要变更。格式遵循
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);版本号遵循
[Semantic Versioning](https://semver.org/),并采用 pre-1.0 惯例:
MINOR 递进视为破坏性变更。

本文件也会显示在 IDE 的插件面板中 —— 构建时只转发前 20 行
(见 `build.gradle.kts` 中的 `changeNotes` 映射),所以请将最新版本
放在最前面,并使用简短的项目符号。

## [0.8.0] — 2026-09-30

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- 增量高亮保留根/容器上下文及完整 Array 字符串，包括独立的右圆括号和
  含引号的多词裸键。
- Folding 使用词法标记，raw 值及多行正文保持不透明。
- 诊断绑定项目/客户端/版本/会话，延迟发布及其他项目关闭不会覆盖当前
  结果；每个 split-editor 的 highlighter 通过实际所属 MarkupModel 清除。
- 高亮词法分析器:精确的 § 3.6 / § 5.2 数字(仅 ASCII 数字)、§ 3.3 空白、
  带引号的键片段、内联复合值中的原始 `::` 值。
- 移除 `src/main/resources/bin/` 下过时的内置 `ktav-lsp.exe`(由 `ktav`
  0.1.5 构建);插件从未使用它。
- Marketplace/Settings 文案不再将注释切换写成 `#`(应为 `##`),也不再
  声称需要 LSP4IJ 或未内置二进制文件 —— 插件拥有自己内置的 LSP 客户端,
  并为每个平台内置了 `ktav-lsp` 二进制文件。

- 词法分析器:多行 `(` / `((` 块的正文是不透明的,CRLF 不再破坏值的
  类型判断;覆盖整个语料库的词法分析器测试涵盖每个 valid 样例。
- LSP 文档订阅按项目管理,涵盖共享文档、恢复的标签页和关闭/重新打开。
  项目关闭只移除本项目的监听器;初始化只发布就绪的客户端,并安全终止
  在项目关闭期间创建的服务器进程。
  传输关闭时会使等待中的请求失败,并立即拒绝与关闭操作竞争的新请求。
- 格式化在应用结果时再次检查已同步的文档、版本、会话和客户端,避免
  中间发生的编辑、关闭/重新打开、项目/客户端关闭、取消或过期响应
  覆盖当前文本。应用与最终检查是原子的,并有独立的 Undo 步骤;新文本
  会发送到每个订阅该文档的项目。
- 两种配色方案中键均为绿色并带柔和下划线;带引号键的内容加粗、引号为品红色,
  光标位于任一引号旁时配对引号会像匹配括号一样高亮。

## 0.5.1

- 兼容范围提升至 IntelliJ 2023.1+(`since-build 231`)。
  Marketplace 验证器在 2022.1–2022.3 上报告了硬性不兼容;所有 231+ 构建
  均验证为 Compatible,因此范围现已与实际情况一致。
- 清理弃用 / 内部 API(行为不变):`INFO_ATTRIBUTES` → `WEAK_WARNING_ATTRIBUTES`;
  `addBrowseFolderListener(title, …)` → 手动 `FileChooser.chooseFile`;
  `createTextAttributesKey(String, TextAttributes)` → `enforcedTextAttributes`;
  `FileChooserDescriptorFactory.createSingleFileDescriptor()` →
  `FileChooserDescriptor` 构造函数;
  `Document.addDocumentListener(l)` → 带 `Disposable` 的重载(顺带修复监听器泄漏);
  `DaemonCodeAnalyzer.restart()` → 按文件的 `restart(PsiFile)`;内部
  `PluginManagerCore.getPlugin(id)` → 插件自身 class-loader 的描述符。
  在 2023.1–2024.3 上验证零警告。
- 仍捆绑相同的 `ktav-lsp 0.5.0`。

## 0.5.0

- 捆绑 `ktav-lsp 0.5.0`(同步至 ktav 0.5.0 与 spec 0.5.0)。
- TextMate 语法:注释模式更新为 `##`,移除了 `:i`/`:f` 类型标记
  模式,并新增了内联复合值与数字字面量模式。
- 许可证:双授权 `MIT OR Apache-2.0`。

## 0.3.1

- 捆绑 `ktav-lsp 0.3.1`(同步至 ktav 0.3.1 与 spec 0.1.1)。
- 捆绑服务器的 document-symbols 响应现在将顶层 Array 项列为
  `[0]`、`[1]`、… 等条目。内置 IntelliJ 客户端不显示文档符号大纲。

## 0.1.0

- Ktav IntelliJ Platform 插件首发。
- 为 `*.ktav` 文件注册 `Ktav` 文件类型。
- 通过 `lang.commenter` 接入注释切换(`# `)。
- 捆绑来自 `editor/grammars/` 的共享 TextMate 语法。
- 目标平台:IntelliJ Platform 2024.3(build `243`)至 2025.1(`251.*`)。
