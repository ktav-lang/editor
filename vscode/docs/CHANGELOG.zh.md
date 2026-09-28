# 变更日志

**Languages:** [English](../CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

## [0.8.0] — 2026-09-28

全部三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)随 `ktav` crate 与
规范同步升至 0.8.0 —— 完整列表见根目录的
[`CHANGELOG.md`](../../CHANGELOG.md)(0.8.0 章节)。

- TextMate 语法:精确的 § 3.6 / § 5.2 数字 scope、识别代理对的
  `\uXXXX`、内联对象中的原始 `::` 值和带引号的键,并修复了在非第 0 列时
  失效的分类。
- 新增真实的分词器测试(`npm run test:unit`),用这些向量运行语法。
- 打包后的扩展现在包含其生产依赖(`vscode-languageclient`);0.6.1 的
  VSIX 构建时缺少它们。

## 0.5.0

- 附带 `ktav-lsp 0.5.0`(同步至 ktav 0.5.0 + spec 0.5.0)。
- TextMate 语法:注释模式更新为 `##`(spec 0.5.0)。
- TextMate 语法:移除 `:i`/`:f` 类型标记模式。
- TextMate 语法:新增行内复合结构模式(`{key: val, …}`、`[v1, v2, …]`)、
  转义序列以及十六进制/八进制/二进制数字字面量。
- 许可证:双许可 `MIT OR Apache-2.0`。

## 0.3.1

- 附带 `ktav-lsp 0.3.1`(同步至 ktav 0.3.1 + spec 0.1.1)。
- 文档符号大纲现在会将顶层 Array 元素列为 `[0]`、`[1]`…… 条目。

## 0.1.0

首个版本。

- 通过共享 TextMate 语法为 `.ktav` 文件提供语法高亮
- `{}` `[]` `()` 的括号匹配
- 注释切换(`#`)
- 复合结构内的自动缩进
