>>>>> lang=en
### LSP server (`ktav-lsp`)

- **textDocument/formatting** capability + handler:
  parse → render through `ktav` crate. Empty edits when content is
  already canonical or input fails to parse.
- Snap to `ktav = "0.2.0"` from crates.io (was a path-dep during
  development).
- Diagnostics for `:f 42` no longer emitted (handled at `ktav`
  semantics layer — integer literals coerce to float).
- Multi-line strings render in stripped `( ... )` form by default
  (verbatim `(( ... ))` remains as fallback for content with leading
  whitespace or sole-`)` lines).

>>>>> lang=ru
### LSP server (`ktav-lsp`)

- Возможность **textDocument/formatting** + обработчик: парсинг →
  рендер через крейт `ktav`. Пустые правки, когда содержимое уже
  канонично или вход не парсится.
- Переход на `ktav = "0.2.0"` из crates.io (раньше был path-dep во время
  разработки).
- Диагностика для `:f 42` больше не выдаётся (обрабатывается на уровне
  семантики `ktav` — целочисленные литералы приводятся к float).
- Многострочные строки рендерятся в stripped-форме `( ... )` по
  умолчанию (verbatim `(( ... ))` остаётся запасным вариантом для
  содержимого с ведущими пробелами или строками из одной `)`).

>>>>> lang=zh
### LSP server (`ktav-lsp`)

- **textDocument/formatting** 能力 + 处理函数:经 `ktav` crate 解析
  → 渲染。内容已 canonical 或输入无法解析时给出空编辑。
- 从 crates.io 锁定 `ktav = "0.2.0"`(开发期间曾为 path-dep)。
- 不再为 `:f 42` 发出诊断(已在 `ktav` 语义层处理 —— 整数字面量会
  强制转换为浮点)。
- 多行字符串默认以 stripped `( ... )` 形式渲染(verbatim
  `(( ... ))` 仍作为后备,用于带前导空白或仅含 `)` 的行的内容)。

