>>>>> lang=en
## Unreleased

Tracks `ktav` Rust crate `0.7.0` and `ktav-lang/spec` `0.7.0` (up from
`0.6.0-4-gc9593e8` — behind even `0.6.4` — so 0.6.x patch-level spec
changes are folded into this jump too). The headline spec change is
**quoted key segments** (§ 5.3.3: a key segment may be wrapped in `"`,
`'` or a backtick; the delimiter is chosen per segment, content is
never trimmed, and structural bytes — `.` `:` `,` `{` `}` `[` `]` —
inside the quotes are opaque) and the **`\uXXXX` escape** (§ 3.7.1,
recognised in keys and inline-compound values only, never in
whole-line scalar values or multi-line bodies), plus two new error
kinds: `UnterminatedQuotedKey` (§ 6.16) and a top-level `InvalidUtf8`
(§ 6.15).

- All three components (`ktav-lsp`, the VS Code extension, the IntelliJ
  plugin) move to **0.8.0** in step with the `ktav` crate and the
  specification: `lsp/Cargo.toml` now depends on `ktav = "0.8"`, the
  spec submodule is re-pinned to `v0.8.0`, and the LSP conformance test
  now walks the 0.8 corpus (it previously walked the long-gone 0.6
  corpus, and its category check was silently disabled by an oracle-key
  mismatch; categories are now read from `ktav::ErrorEnvelope`).

### LSP server (`ktav-lsp`)

>>>>> lang=ru
## Не выпущено

Синхронизация с крейтом `ktav` `0.7.0` и `ktav-lang/spec` `0.7.0` (было
`0.6.0-4-gc9593e8` — отстаёт даже от `0.6.4`, так что попутно
подтянуты и патч-изменения спецификации 0.6.x). Главное изменение
спецификации — **квотированные сегменты ключа** (§ 5.3.3: сегмент
ключа можно обернуть в `"`, `'` или обратную кавычку; разделитель
выбирается для каждого сегмента, содержимое никогда не обрезается, а
структурные байты — `.` `:` `,` `{` `}` `[` `]` — внутри кавычек
непрозрачны) и экранирование **`\uXXXX`** (§ 3.7.1, распознаётся
только в ключах и инлайн-составных значениях, никогда в построчных
скалярах и многострочных телах), плюс два новых вида ошибок:
`UnterminatedQuotedKey` (§ 6.16) и `InvalidUtf8` на уровне документа
(§ 6.15).

- Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
  переходят на **0.8.0** вслед за крейтом `ktav` и спецификацией:
  `lsp/Cargo.toml` теперь зависит от `ktav = "0.8"`, подмодуль spec
  перезакреплён на `v0.8.0`, а тест соответствия LSP теперь проходит
  по корпусу 0.8 (раньше по давно исчезнувшему корпусу 0.6, причём его
  проверка категорий была молча отключена из-за несовпадения ключей
  оракула; теперь категории читаются из `ktav::ErrorEnvelope`).

### LSP server (`ktav-lsp`)

>>>>> lang=zh
## 未发布

同步 `ktav` crate `0.7.0` 与 `ktav-lang/spec` `0.7.0`(此前为
`0.6.0-4-gc9593e8`,甚至落后于 `0.6.4`,因此这次一并纳入了 0.6.x 的
补丁级规范变更)。规范的头号改动是**带引号的键片段**(§ 5.3.3:键
片段可以用 `"`、`'` 或反引号包裹;每个片段各自选择定界符,内容永不做
修剪,结构字节 `.` `:` `,` `{` `}` `[` `]` 在引号内均为普通
内容)以及 **`\uXXXX`** 转义(§ 3.7.1,仅在键和内联复合值中识别,
整行标量值和多行正文中均不识别),另有两个新的错误类型:
`UnterminatedQuotedKey`(§ 6.16)和顶层 `InvalidUtf8`(§ 6.15)。

- 三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)同步升至
  **0.8.0**,与 `ktav` crate 和规范保持一致:`lsp/Cargo.toml` 现在
  依赖 `ktav = "0.8"`,spec 子模块重新锁定到 `v0.8.0`,LSP 一致性
  测试现在遍历 0.8 语料库(此前遍历的是早已消失的 0.6 语料库,而且
  它的类别检查因 oracle 键不匹配而被静默禁用;现在类别从
  `ktav::ErrorEnvelope` 读取)。

### LSP server (`ktav-lsp`)

