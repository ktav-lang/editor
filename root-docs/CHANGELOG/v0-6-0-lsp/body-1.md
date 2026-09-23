>>>>> lang=en
### LSP server (`ktav-lsp`)

- Semantic tokens, document symbols and diagnostics are now
  **escape-aware**: the pair separator is the first *unescaped* `:` / `::`
  and dotted-path splitting happens only on *unescaped* `.`. So
  `a\.b: v` is the single key `a.b`, and `a\:: v` is the key `a:` with
  value `v`.
- Depends on `ktav` 0.6.0.

>>>>> lang=ru
### LSP server (`ktav-lsp`)

- Семантические токены, символы документа и диагностика теперь
  **учитывают escape'и**: разделитель пары — первый *неэкранированный*
  `:` / `::`, а дробление точечного пути происходит только по
  *неэкранированной* `.`. Так что `a\.b: v` — единый ключ `a.b`, а
  `a\:: v` — ключ `a:` со значением `v`.
- Зависит от `ktav` 0.6.0.

>>>>> lang=zh
### LSP server (`ktav-lsp`)

- 语义令牌、文档符号和诊断现在**支持转义**:键值分隔符是第一个
  *未转义*的 `:` / `::`,点分路径只在*未转义*的 `.` 处切分。因此
  `a\.b: v` 是单个键 `a.b`,而 `a\:: v` 是键 `a:`、值为 `v`。
- 依赖 `ktav` 0.6.0。

