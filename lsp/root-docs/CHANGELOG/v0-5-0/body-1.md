>>>>> lang=en
## [0.5.0] — 2026-05-27

Sync to `ktav 0.5.0` and spec `0.5.0`.

### Changed

- `Cargo.toml`: `ktav = "0.5.0"` (was `"0.3.1"`).
- License: dual `MIT OR Apache-2.0` (was MIT-only).
- `tokens`: comments now require `##`; single `#` is an ordinary character.
- `tokens`: typed markers `:i` and `:f` removed (spec 0.5.0 drop); type is
  inferred from the lexical form of the scalar.
- `tokens`: `looks_numeric` extended with hex (`0x`), octal (`0o`), binary
  (`0b`) prefixes and underscore separators (spec 0.5.0 number literals).
- `diagnostics`: removed legacy `InvalidTypedScalar` range heuristic.

>>>>> lang=ru
## [0.5.0] — 2026-05-27

Синхронизация с `ktav 0.5.0` и spec `0.5.0`.

### Изменено

- `Cargo.toml`: `ktav = "0.5.0"` (было `"0.3.1"`).
- Лицензия: двойная `MIT OR Apache-2.0` (была только MIT).
- `tokens`: комментарии теперь требуют `##`; одиночный `#` — обычный символ.
- `tokens`: типизированные маркеры `:i` и `:f` удалены (отказ в spec
  0.5.0); тип выводится из лексической формы скаляра.
- `tokens`: `looks_numeric` расширен hex- (`0x`), octal- (`0o`) и
  binary- (`0b`) префиксами, а также разделителями подчёркивания
  (числовые литералы spec 0.5.0).
- `diagnostics`: убрана legacy-эвристика диапазона `InvalidTypedScalar`.

>>>>> lang=zh
## [0.5.0] — 2026-05-27

同步到 `ktav 0.5.0` 与 spec `0.5.0`。

### 变更

- `Cargo.toml`:`ktav = "0.5.0"`(原为 `"0.3.1"`)。
- 许可证:双许可 `MIT OR Apache-2.0`(原为仅 MIT)。
- `tokens`:注释现在要求 `##`;单个 `#` 为普通字符。
- `tokens`:移除 `:i` 与 `:f` 类型标记(spec 0.5.0 已移除);类型由标量的
  词法形式推断。
- `tokens`:`looks_numeric` 扩展支持十六进制(`0x`)、八进制(`0o`)、二进制
  (`0b`)前缀与下划线分隔符(spec 0.5.0 数字字面量)。
- `diagnostics`:移除遗留的 `InvalidTypedScalar` 范围启发式。

