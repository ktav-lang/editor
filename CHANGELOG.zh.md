# 变更日志

**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 Ktav 编辑器支持(VS Code 扩展、IntelliJ 插件、LSP 服务器、
共享 TextMate 语法)的所有重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);版本号遵循
[Semantic Versioning](https://semver.org/),并采用 pre-1.0 惯例:
MINOR 递进视为破坏性变更。

一个 tag(`v0.X.Y`)同时发布全部四个子项目。子项目相关的变更按
版本标题分组。

本日志记录**编辑器/IDE 支持的发布**,而不是 Ktav 格式本身的变更
—— 后者请见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec/blob/main/CHANGELOG.zh.md)。

## Unreleased

同步 `ktav` crate 0.7.0 与 `ktav-lang/spec` 0.7.0(此前为
`0.6.0-4-gc9593e8`,甚至落后于 `0.6.4`,因此这次一并纳入了 0.6.x
的补丁级规范变更)。规范新增**带引号的键片段**(§ 5.3.3:键片段可以用
`"`、`'` 或反引号包裹;引号内内容不做修剪,`. : , { } [ ]`
等结构字节在其中均为普通内容)以及 **`\uXXXX`** 转义(§ 3.7.1;
仅在键和内联复合值中识别,整行标量值和多行正文中不识别)。

- LSP:`ktav = "0.7"`(原为 `"0.6"`);`rust-version` 提升至 `1.71`
  (ktav 0.7 自身的 MSRV)。
- LSP:自有的按行分类器(`tokens.rs`——语义高亮、悬浮提示、自动补全)
  与文档结构扫描器(`symbols.rs`)现在都能正确处理带引号键片段内的
  `:` / `.`,不再将其误判为真正的分隔符或路径分割点。
- LSP:修复了在本次升级审计中发现的 `textDocument/formatting`
  编码错误——整档替换编辑的结束位置此前用 `chars().count()`
  (Unicode 标量计数)计算,而不是与文件其余部分一致的
  按字节/UTF-16 长度计算,导致最后一行含非 ASCII 内容时位置偏小。
- LSP:移除 `reindent.rs` 中关于已在 spec 0.5.0 删除的 `:i`/`:f`
  标记的失效逻辑;基于这些标记编写的测试已重写。
- TextMate 语法(`grammars/` 及同步的 `vscode/syntaxes/`):新增带引号
  键片段与 `\uXXXX` 转义的高亮支持。
- spec 子模块锁定到 `04f867f`(`v0.7.0`)。

## [0.6.1] — 2026-06-05

- 文档：将所有 README 示例改写为 spec 0.6 语法（裸数字替代已移除的 `:i`/`:f` 标记；`##` 注释替代 `#`）。
- LSP：从补全项中移除 `:i`/`:f`（类型标记已在 spec 0.5 中移除）。
