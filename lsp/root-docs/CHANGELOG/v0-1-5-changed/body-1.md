>>>>> lang=en
### Changed

- **Diagnostics** now consume `ktav::Error::Structured(ErrorKind)` from
  `ktav 0.1.5+`: the `Diagnostic.range` is built directly from the
  variant's byte-offset `Span` (via `Span::line_col`) instead of being
  re-derived by regex-matching the formatted error message and re-running
  the offending line through the shared line classifier. Range tightness
  is now driven by the parser's own knowledge of the failure point,
  yielding strictly equal-or-narrower ranges for every category — e.g.
  `MissingSeparatorSpace` for `key:value\n` now highlights bytes 4..9
  (the glued body `value`) rather than 3..9 (colon + body).
- Existing `tests/error_format_pinning.rs` accepts both `Error::Syntax(_)`
  and `Error::Structured(_)` and asserts on the rendered Display string —
  the contract is the message text, not the enum variant.
- `tests/integration.rs` range expectations tightened to the new
  structured spans (`MissingSeparatorSpace` 4..9, `InvalidTypedScalar`
  6..10 etc.). Cyrillic byte-column test re-pinned to the body span
  start (byte 7 for `имя:значение\n`).
- `tests/spec_conformance.rs` accepts the new `MissingSeparator` /
  `UnbalancedBracket` Display strings as aliases for the spec
  categories `OrphanLine` / `MismatchedBracket`.

>>>>> lang=ru
### Changed

- **Диагностики** теперь потребляют `ktav::Error::Structured(ErrorKind)`
  из `ktav 0.1.5+`: `Diagnostic.range` строится напрямую из byte-offset
  `Span` варианта (через `Span::line_col`) вместо повторного выведения
  через regex-разбор форматированного сообщения и повторного прогона
  ошибочной строки через общий line-классификатор. Точность диапазона
  теперь определяется собственным знанием парсера о месте сбоя, что даёт
  строго равные или более узкие диапазоны для каждой категории — например,
  `MissingSeparatorSpace` для `key:value\n` теперь подсвечивает байты 4..9
  (склеенное тело `value`) вместо 3..9 (двоеточие + тело).
- Существующий `tests/error_format_pinning.rs` принимает и
  `Error::Syntax(_)`, и `Error::Structured(_)` и проверяет
  отрендеренную Display-строку — контракт это текст сообщения, а не
  вариант enum.
- Ожидания диапазонов в `tests/integration.rs` подтянуты к новым
  structured spans (`MissingSeparatorSpace` 4..9, `InvalidTypedScalar`
  6..10 и т.д.). Тест на кириллическую byte-column перепинен на старт
  span тела (байт 7 для `имя:значение\n`).
- `tests/spec_conformance.rs` принимает новые Display-строки
  `MissingSeparator` / `UnbalancedBracket` как алиасы категорий
  спецификации `OrphanLine` / `MismatchedBracket`.

>>>>> lang=zh
### 变更

- **诊断** 现在消费 `ktav 0.1.5+` 的 `ktav::Error::Structured(ErrorKind)`:
  `Diagnostic.range` 直接由变体的字节偏移 `Span` 经 `Span::line_col`
  构建,而不再通过正则匹配格式化的错误消息、并将出错行重新过一遍共享
  行分类器来推导。范围精度现在由解析器自身对失败点的认知驱动,对每个
  类别都得到严格相等或更窄的范围 —— 例如 `key:value\n` 的
  `MissingSeparatorSpace` 现在高亮字节 4..9(粘连的主体 `value`),
  而不再是 3..9(冒号 + 主体)。
- 现有的 `tests/error_format_pinning.rs` 同时接受 `Error::Syntax(_)`
  与 `Error::Structured(_)`,并断言渲染后的 Display 字符串 —— 契约是
  消息文本,而非枚举变体。
- `tests/integration.rs` 的范围预期收紧到新的 structured spans
  (`MissingSeparatorSpace` 4..9、`InvalidTypedScalar` 6..10 等)。
  西里尔字节列测试重新固定到主体 span 的起点(`имя:значение\n` 的
  字节 7)。
- `tests/spec_conformance.rs` 接受新的 `MissingSeparator` /
  `UnbalancedBracket` Display 字符串,作为规范类别 `OrphanLine` /
  `MismatchedBracket` 的别名。

