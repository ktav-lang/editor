# Журнал изменений — `ktav-lsp`

Все значимые изменения в crate `ktav-lsp` документируются здесь. Формат
основан на [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
crate следует [Semantic Versioning](https://semver.org/) с
Cargo-конвенцией: до 1.0 bump MINOR считается ломающим.

**Languages:** [English](CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)

## [0.1.5] — 2026-05-01

Внутренний рефакторинг конвейера диагностик. Публичный API не менялся.

### Изменено

- **Диагностики** теперь потребляют `ktav::Error::Structured(ErrorKind)`
  из `ktav 0.1.5+`: `Diagnostic.range` строится напрямую из byte-offset
  `Span` варианта (через `Span::line_col`) вместо повторного выведения
  через regex по форматированному сообщению и повторного прогона
  ошибочной строки через общий line-классификатор. Точность диапазона
  теперь определяется собственным знанием парсера о месте ошибки, что
  даёт строго равные или более узкие диапазоны для каждой категории —
  например, `MissingSeparatorSpace` для `key:value\n` теперь подсвечивает
  байты 4..9 (склеенное тело `value`) вместо 3..9 (двоеточие + тело).
- Существующий `tests/error_format_pinning.rs` принимает и
  `Error::Syntax(_)`, и `Error::Structured(_)` и проверяет рендерящуюся
  Display-строку — контракт это текст сообщения, а не вариант enum.
- Ожидания диапазонов в `tests/integration.rs` подтянуты к новым
  structured-spans (`MissingSeparatorSpace` 4..9, `InvalidTypedScalar`
  6..10 и т.д.). Тест на кириллическую byte-column перепинен на старт
  span тела (байт 7 для `имя:значение\n`).
- `tests/spec_conformance.rs` принимает новые Display-строки
  `MissingSeparator` / `UnbalancedBracket` как алиасы спек-категорий
  `OrphanLine` / `MismatchedBracket`.

### Добавлено

- `tests/structured_diagnostics.rs` — пер-вариантные проверки,
  покрывающие все 10 спек-определённых вариантов `ErrorKind` плюс
  тест на кириллическую byte-column, прогоняющий `tokens::byte_to_utf16`
  на пути конвертации диагностик.

### Внутреннее

- Regex-извлечение (`extract_line_number`, `extract_quoted_key`,
  per-category `range_for_*` хелперы) сохранено как
  `compute_range_legacy`, достижимо только через `Error::Syntax(_)`.
  Парсер в `ktav 0.1.5+` больше не конструирует этот вариант, но
  `ktav::Error` помечен `#[non_exhaustive]`, и downstream-обёртки могут
  по-прежнему его поверхностить — legacy-путь оставлен как
  defence-in-depth.
- Зависимость `ktav` поднята с `0.1.4` до `0.1.5` (registry pin),
  чтобы подтянуть structured-error API. Локальная разработка против
  sibling-checkout-а возможна через `.cargo/config.toml` patch
  (per-developer, не трекается) при итерации против неопубликованного
  ktav.

### Ломающее (внутреннее)

- Версия crate-а поднята до **0.1.5**, так что вызывающие
  `ktav-lsp` 0.1.4-формы (внешних нет — это leaf-бинарь)
  пересобираются чисто против `ktav 0.1.5+`.

## [0.1.0] — 2026-04-26

Первый релиз.

### Добавлено

- LSP-сервер одним бинарём (`ktav-lsp`) через stdin/stdout, на базе
  `tower-lsp` 0.20 и `tokio`.
- **Диагностики** — перепарсивание на `did_open` / `did_change` /
  `did_save`, публикация сообщений `ktav::Error::Syntax` на нужной
  строке. Номер строки восстанавливается из сообщения двумя regex'ами,
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
- **Semantic tokens (full)** — шесть типов: `comment`, `keyword`,
  `number`, `string`, `property`, `operator`. Порядок индексов — часть
  публичного legend.
- Логирование через `tracing` в stderr; уровень — переменная окружения
  `KTAV_LSP_LOG`.
- Интеграционные и unit-тесты regex'ов диагностик, legend semantic
  tokens и формы дерева document-symbols.
