>>>>> lang=en
## [0.8.0] — 2026-09-28

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
- Number, keyword and escape highlighting now follows the spec exactly
  in all three components: redundant-leading-zero decimals (`01234`,
  `0_7`, § 5.2) and malformed literals (`1_`, `1__0`, `0X1A`,
  `2026-09-28`, § 3.6) are Strings; values after `::` are never typed,
  including inside inline compounds; quoted key segments are opaque
  inside inline objects (§ 5.3.3). The prebuilt `ktav-lsp` binaries are
  no longer committed (see *Repository and release tooling*).

### LSP server (`ktav-lsp`)

>>>>> lang=ru
## [0.8.0] — 2026-09-28

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
- Подсветка чисел, ключевых слов и экранирования во всех трёх
  компонентах теперь в точности следует спецификации: десятичные с
  избыточным ведущим нулём (`01234`, `0_7`, § 5.2) и неверно
  записанные литералы (`1_`, `1__0`, `0X1A`, `2026-09-28`, § 3.6) —
  строки; значения после `::` никогда не типизируются, в том числе
  внутри inline-структур; сегменты ключей в кавычках непрозрачны
  внутри inline-объектов (§ 5.3.3). Готовые бинарники `ktav-lsp`
  больше не хранятся в репозитории (см. *Репозиторий и релизная
  оснастка*).

### LSP server (`ktav-lsp`)

>>>>> lang=zh
## [0.8.0] — 2026-09-28

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
- 三个组件中数字、关键字和转义的高亮现在严格遵循规范:带冗余前导零的
  十进制数(`01234`、`0_7`,§ 5.2)和格式错误的字面量(`1_`、`1__0`、
  `0X1A`、`2026-09-28`,§ 3.6)为字符串;`::` 之后的值绝不做类型区分,
  内联复合值中同样如此;带引号的键片段在内联对象中保持不透明
  (§ 5.3.3)。预构建的 `ktav-lsp` 二进制不再提交到仓库(见
  *仓库与发布工具*)。

### LSP server (`ktav-lsp`)

