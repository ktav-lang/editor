# Журнал изменений — `ktav-lsp`

Все значимые изменения в crate `ktav-lsp` документируются здесь. Формат
основан на [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
crate следует [Semantic Versioning](https://semver.org/) с
Cargo-конвенцией: до 1.0 bump MINOR считается ломающим.

**Languages:** [English](CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)

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
