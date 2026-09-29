>>>>> lang=en
Within noise. The semantic tokenizer doesn't touch the changed code
paths, so the small drift is host load.

### build_symbols

| size       | 0.1.x baseline | 0.2.x    | Δ      |
|------------|----------------|----------|--------|
| small_1k   | ~71 µs         | ~73 µs   | +3 %   |
| medium_50k | ~101 ms        | 108 ms   | +7 %   |
| large_500k | ~11.0 s        | 11.5 s   | +4 %   |

The super-linear scaling on `large_500k` (already flagged in the
baseline above) is unchanged — `locate_key` full-text scan per
top-level key remains the bottleneck. Optimisation target for a
future patch (out of scope for 0.2.x).

### Notes

- Criterion reported "no change in performance detected" (p > 0.05)
  for **every** row. Treat all deltas above as noise unless reproduced
  on a quiet host.
- The major architectural change in 0.2.x — switching the formatter
  from `parse → render(value)` to `reindent(text)` — does not appear
  in these benches; reindent has its own micro-tests under
  `tests/format_pipeline.rs` (22 cases) but no Criterion benchmark
  yet. Worth adding when the formatter sees production usage profiles.

>>>>> lang=ru
В пределах шума. Изменённые пути кода не затрагивают семантический
токенизатор, поэтому небольшое смещение объясняется загрузкой машины.

### build_symbols

| размер     | база 0.1.x | 0.2.x  | Δ    |
|------------|------------|--------|------|
| small_1k   | ~71 µs     | ~73 µs | +3 % |
| medium_50k | ~101 ms    | 108 ms | +7 % |
| large_500k | ~11.0 s    | 11.5 s | +4 % |

Сверхлинейный рост времени на `large_500k`, отмеченный выше, не изменился:
узким местом остаётся полный проход `locate_key` по тексту для каждого
ключа верхнего уровня. Оптимизация оставлена для будущего изменения
(вне рамок 0.2.x).

### Примечания

- Criterion сообщил «изменения производительности не обнаружены» (p > 0.05)
  **для каждой** строки. Считайте все отклонения шумом, пока они не
  воспроизведены на спокойной машине.
- Крупная архитектурная перемена 0.2.x, переход форматтера с
  `parse → render(value)` на `reindent(text)`, этими бенчмарками не измеряется.
  Для reindent есть 22 микро-теста в `tests/format_pipeline.rs`, но пока нет
  бенчмарка Criterion. Его стоит добавить при появлении профилей реального использования.

>>>>> lang=zh
属于噪声。语义标记器不经过被修改的代码路径，微小漂移来自主机负载。

### build_symbols

| 大小       | 0.1.x 基线 | 0.2.x  | Δ    |
|------------|------------|--------|------|
| small_1k   | ~71 µs     | ~73 µs | +3 % |
| medium_50k | ~101 ms    | 108 ms | +7 % |
| large_500k | ~11.0 s    | 11.5 s | +4 % |

上文指出的 `large_500k` 超线性增长没有变化：`locate_key` 对每个
顶层键全文扫描仍是瓶颈。优化留待后续补丁，不属于 0.2.x 的范围。

### 注释

- Criterion 对**每一行**都报告“未检测到性能变化”(p > 0.05)。
  除非在安静的主机上复现，否则所有差异均应视作噪声。
- 0.2.x 的重大架构变更，即格式化器从 `parse → render(value)`
  切换到 `reindent(text)`，未包含在这些基准测试中。reindent 在
  `tests/format_pipeline.rs` 中有 22 个微型测试，但还没有 Criterion
  基准测试；有生产使用特征后值得补充。

