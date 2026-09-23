>>>>> lang=en
### Added

- `tests/structured_diagnostics.rs` — per-variant assertions covering
  all 10 spec-defined `ErrorKind` variants plus a Cyrillic byte-column
  test that exercises `tokens::byte_to_utf16` on the diagnostic-range
  conversion path.

>>>>> lang=ru
### Добавлено

- `tests/structured_diagnostics.rs` — проверки по каждому варианту
  `ErrorKind`: покрыты все 10 вариантов, определённых спецификацией, плюс
  тест на кириллическую byte-column, выполняющий `tokens::byte_to_utf16`
  на пути конвертации диагностического диапазона.

>>>>> lang=zh
### 新增

- `tests/structured_diagnostics.rs` —— 逐变体断言,覆盖规范定义的全部
  10 个 `ErrorKind` 变体,外加一个西里尔字节列测试,在诊断范围转换
  路径上演练 `tokens::byte_to_utf16`。

