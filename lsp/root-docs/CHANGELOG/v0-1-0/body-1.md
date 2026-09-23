>>>>> lang=en
## [0.1.0] — 2026-04-26

Initial release.

### Added

- Single-binary LSP server (`ktav-lsp`) over stdin/stdout, built on
  `tower-lsp` 0.20 and `tokio`.
- **Diagnostics** — re-parses on `did_open` / `did_change` / `did_save`,
  publishes `ktav::Error::Syntax` messages on the offending line. Line
  numbers are recovered from the message via two regexes that cover all
  shapes emitted by `ktav` 0.1.4 (`Line N: …`, `Invalid key at line N: …`,
  `Empty key at line N`).
- **Hover** — for `key: …` lines, looks the dotted path up in the parsed
  tree and reports the inferred type and value.
- **Completion** — context-aware after a `:` separator: offers `null`,
  `true`, `false`, openers (`{`, `[`, `(`, `((`), empty literals (`{}`,
  `[]`, `()`), the raw marker (`:`), and typed-scalar markers (`i`, `f`).
- **Document symbols** — outline tree from `Value::Object`; scalars map
  to Property/Number/String, objects to Module, arrays to Array.
- **Semantic tokens (full)** — six token types: `comment`, `keyword`,
  `number`, `string`, `property`, `operator`. Index ordering is part of
  the public legend.
- Logging via `tracing` to stderr; `KTAV_LSP_LOG` env var controls level.
- Integration and unit tests covering the diagnostic regexes, semantic
  token legend, and document-symbol tree shape.
>>>>> lang=ru
## [0.1.0] — 2026-04-26

Первый релиз.

### Добавлено

- LSP-сервер одним бинарём (`ktav-lsp`) через stdin/stdout, на базе
  `tower-lsp` 0.20 и `tokio`.
- **Диагностики** — перепарсивание на `did_open` / `did_change` /
  `did_save`, публикация сообщений `ktav::Error::Syntax` на нужной
  строке. Номер строки восстанавливается из сообщения двумя regex',
  покрывающими все формы из `ktav` 0.1.4 (`Line N: …`, `Invalid key at
  line N: …`, `Empty key at line N`).
- **Hover** — для строк `key: …` ищет точечный путь в распарсенном
  дереве и сообщает выведенный тип и значение.
- **Автодополнение** — контекстно после разделителя `:`: предлагает
  `null`, `true`, `false`, открывающие скобки (`{`, `[`, `(`, `((`),
  пустые литералы (`{}`, `[]`, `()`), raw-маркер (`:`) и типизированные
  маркеры (`i`, `f`).
- **Document symbols** — outline-дерево из `Value::Object`; скаляры
  соответствуют Property/Number/String, объекты — Module, массивы —
  Array.
- **Semantic tokens (full)** — шесть типов токенов: `comment`, `keyword`,
  `number`, `string`, `property`, `operator`. Порядок индексов — часть
  публичной легенды.
- Логирование через `tracing` в stderr; уровень задаёт переменная
  окружения `KTAV_LSP_LOG`.
- Интеграционные и модульные тесты: regex'ы диагностик, легенда
  semantic tokens и форма дерева document-symbols.
>>>>> lang=zh
## [0.1.0] — 2026-04-26

首个版本。

### 新增

- 通过 stdin/stdout 提供的单二进制 LSP 服务器(`ktav-lsp`),基于
  `tower-lsp` 0.20 与 `tokio`。
- **诊断**:在 `did_open` / `did_change` / `did_save` 时重新解析,并在
  出错行上发布 `ktav::Error::Syntax` 消息。行号通过两条正则从消息中恢复,
  覆盖 `ktav` 0.1.4 的全部形态(`Line N: …`、`Invalid key at line N: …`、
  `Empty key at line N`)。
- **Hover**:对 `key: …` 行,在已解析的树中按点状路径查找,并报告推断的
  类型与值。
- **补全**:在 `:` 分隔符之后上下文感知补全:提供 `null`、`true`、`false`、
  开括号(`{`、`[`、`(`、`((`)、空字面量(`{}`、`[]`、`()`)、raw 标记
  (`:`)以及类型标记(`i`、`f`)。
- **文档符号**:由 `Value::Object` 构建的大纲树;标量映射为
  Property/Number/String,对象为 Module,数组为 Array。
- **Semantic tokens (full)**:六种 token 类型 —— `comment`、`keyword`、
  `number`、`string`、`property`、`operator`。索引顺序为公开 legend
  的一部分。
- 通过 `tracing` 将日志写到 stderr;级别由 `KTAV_LSP_LOG` 环境变量控制。
- 覆盖诊断正则、semantic tokens legend 与文档符号树结构的集成测试与
  单元测试。
