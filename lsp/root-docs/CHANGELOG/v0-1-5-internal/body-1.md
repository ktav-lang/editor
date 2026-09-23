>>>>> lang=en
### Internal

- Regex-extraction code (`extract_line_number`, `extract_quoted_key`,
  per-category `range_for_*` helpers) is preserved as
  `compute_range_legacy`, reachable only via `Error::Syntax(_)`. The
  parser in `ktav 0.1.5+` no longer constructs that variant, but
  `ktav::Error` is `#[non_exhaustive]` and downstream wrappers may
  still surface it — the legacy path is kept as defence-in-depth.
- `ktav` dependency bumped from `0.1.4` to `0.1.5` (registry pin),
  picking up the structured-error API. Local sibling-checkout
  development can be restored via `.cargo/config.toml` patch (per-
  developer, not tracked) when iterating against an unpublished
  ktav.

>>>>> lang=ru
### Внутреннее

- Извлечение через regex (`extract_line_number`, `extract_quoted_key`,
  per-category хелперы `range_for_*`) сохранено как
  `compute_range_legacy` и достижимо только через `Error::Syntax(_)`.
  Парсер в `ktav 0.1.5+` больше не создаёт этот вариант, но `ktav::Error`
  помечен `#[non_exhaustive]`, и зависимые обёртки могут по-прежнему
  возвращать его — legacy-путь оставлен как defence-in-depth.
- Зависимость `ktav` поднята с `0.1.4` до `0.1.5` (registry pin), что
  подтягивает structured-error API. Разработку против локального
  sibling-checkout можно восстановить через patch в `.cargo/config.toml`
  (per-developer, не трекается), когда итерируешься против
  неопубликованного ktav.

>>>>> lang=zh
### 内部

- 正则抽取代码(`extract_line_number`、`extract_quoted_key`、各类别的
  `range_for_*` 辅助函数)保留为 `compute_range_legacy`,仅通过
  `Error::Syntax(_)` 可达。`ktav 0.1.5+` 的解析器不再构造该变体,但
  `ktav::Error` 标注了 `#[non_exhaustive]`,下游包装器仍可能暴露该
  变体 —— legacy 路径作为纵深防御保留。
- `ktav` 依赖从 `0.1.4` 升至 `0.1.5`(registry pin),以引入结构化错误
  API。需要针对未发布 ktav 进行本地 sibling-checkout 开发时,可通过
  `.cargo/config.toml` patch 恢复(per-developer,不纳入 git)。

