>>>>> lang=en
## Unreleased

Tracks `ktav` Rust crate `0.8.0` and `ktav-lang/spec` `0.8.0`. The
universal breaking change is that **leading-zero decimal integers remain
strings** (§ 5.2, so `01234` keeps its leading zero). Separately,
§ 8.1 requires a strict parsing entry point that rejects lossy scalar
forms with `LossyScalar`. Quoted key segments
and the `\uXXXX` escape were introduced in 0.7.0.

- All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
  plugin) align on **0.8.0** with the `ktav` crate and the
  specification: `lsp/Cargo.toml` now depends on `ktav = "0.8"`, the
  spec submodule is re-pinned to `v0.8.0`, and the LSP conformance test
  now walks the 0.8 corpus (it previously walked the long-gone 0.6
  corpus, and its category check was silently disabled by an oracle-key
  mismatch; categories are now read from `ktav::ErrorEnvelope`).

### LSP server (`ktav-lsp`)

>>>>> lang=ru
## Не выпущено

Синхронизация с крейтом `ktav` и `ktav-lang/spec` версии `0.8.0`.
Единственное универсальное ломающее изменение спецификации:
**десятичные целые с ведущими нулями остаются строками** (§ 5.2:
`01234` сохраняет начальный ноль). Отдельно § 8.1 требует строгую
точку входа разбора, отклоняющую теряющие данные скалярные формы с
`LossyScalar`. Квотированные сегменты ключей и экранирование
`\uXXXX` появились в версии 0.7.0.

- Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
  синхронизированы на версии **0.8.0** с крейтом `ktav` и спецификацией:
  `lsp/Cargo.toml` теперь зависит от `ktav = "0.8"`, подмодуль spec
  перезакреплён на `v0.8.0`, а тест соответствия LSP теперь проходит
  по корпусу 0.8 (раньше по давно исчезнувшему корпусу 0.6, причём его
  проверка категорий была молча отключена из-за несовпадения ключей
  оракула; теперь категории читаются из `ktav::ErrorEnvelope`).

### LSP server (`ktav-lsp`)

>>>>> lang=zh
## 未发布

同步至 `ktav` crate 与 `ktav-lang/spec` `0.8.0`。唯一普遍适用的破坏性
变更是**带前导零的十进制整数保留为字符串**(§ 5.2，`01234` 保留前导零)。
另有 § 8.1 要求提供严格解析入口，遇到有损标量形式时以 `LossyScalar`
拒绝。带引号的键片段
和 `\uXXXX` 转义已在 0.7.0 中引入。

- 三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)同步升至
  **0.8.0**,与 `ktav` crate 和规范保持一致:`lsp/Cargo.toml` 现在
  依赖 `ktav = "0.8"`,spec 子模块重新锁定到 `v0.8.0`,LSP 一致性
  测试现在遍历 0.8 语料库(此前遍历的是早已消失的 0.6 语料库,而且
  它的类别检查因 oracle 键不匹配而被静默禁用;现在类别从
  `ktav::ErrorEnvelope` 读取)。

### LSP server (`ktav-lsp`)

