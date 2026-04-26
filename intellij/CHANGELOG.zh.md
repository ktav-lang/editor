# 变更日志

**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 Ktav IntelliJ Platform 插件的所有重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);版本号遵循
[Semantic Versioning](https://semver.org/),并采用 pre-1.0 惯例:
MINOR 递进视为破坏性变更。

本文件也会显示在 IDE 的插件面板中 —— 构建时只转发前 20 行
(见 `build.gradle.kts` 中的 `changeNotes` 映射),所以请把最新版本
放在最上面,并使用简短的项目符号。

## 0.1.0

- Ktav IntelliJ Platform 插件首发。
- 为 `*.ktav` 文件注册了 `Ktav` 文件类型。
- 通过 `lang.commenter` 接入注释切换(`# `)。
- 捆绑来自 `editor/grammars/` 的共享 TextMate 语法。
- 目标平台:IntelliJ Platform 2024.3(build `243`)— 2025.1(`251.*`)。
