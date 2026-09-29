>>>>> lang=en
## Historical post-optimisation baseline (2026-05-08, full)

Captured against `ktav-lsp` 0.2.0 with **full** Criterion (no `--quick`,
100 samples, 5s warm-up) on a quieter Win10 host with IDE / language
servers closed during the run.

The fixture generator and LSP have changed
since then. These measurements are historical, not a regression reference
for 0.8.0; capture a fresh 0.8.0 baseline before comparing performance.
To run the current benchmarks from the `editor/lsp/` crate root:

```sh
cargo bench --bench parse_for_diagnostics
cargo bench --bench semantic_tokens
cargo bench --bench build_symbols
cargo bench --bench encoding_hot_paths
```

Unlike earlier `--quick` measurements, this section used a full Criterion
run, but its numbers are valid only for that historical build and fixture.

### `parse_for_diagnostics`

>>>>> lang=ru
## Исторические замеры после оптимизации (2026-05-08, полный запуск)

Измерения для `ktav-lsp` 0.2.0 выполнены полным Criterion (без `--quick`,
100 выборок, 5 секунд прогрева) на более спокойной Win10-машине; IDE и
языковые серверы были закрыты. С тех пор генератор образцов и LSP
изменились. Эти цифры исторические, а не эталон для поиска регрессий в
0.8.0; перед сравнением производительности нужен новый базовый замер
0.8.0. Для запуска текущих бенчмарков из корня crate `editor/lsp/`:

```sh
cargo bench --bench parse_for_diagnostics
cargo bench --bench semantic_tokens
cargo bench --bench build_symbols
cargo bench --bench encoding_hot_paths
```

В отличие от предыдущих замеров с `--quick`, здесь использован полный
запуск Criterion, но цифры относятся только к тому историческому коду
и генератору образцов.

### `parse_for_diagnostics`

| размер     | медиана | пропускная способность |
|------------|---------|------------------------|
| small_1k   | 12.2 µs | ~80 MiB/s              |
| medium_50k | 837 µs  | ~58 MiB/s              |
| large_500k | 9.13 ms | ~54 MiB/s              |

>>>>> lang=zh
## 优化后的历史基线 (2026-05-08，完整测量)

针对 `ktav-lsp` 0.2.0 使用**完整** Criterion 测量 (未使用 `--quick`，
100 个样本，预热 5 秒)。在较安静的 Win10 主机上运行，期间关闭 IDE
和语言服务器。此后测试数据生成器和 LSP 均已变更。这些数据仅是历史
测量结果，不能作为 0.8.0 的回归检测基线；比较性能前须重新采集
0.8.0 基线。从 `editor/lsp/` crate 根目录运行当前基准测试：

```sh
cargo bench --bench parse_for_diagnostics
cargo bench --bench semantic_tokens
cargo bench --bench build_symbols
cargo bench --bench encoding_hot_paths
```

与前面使用 `--quick` 的章节不同，本节使用完整 Criterion 测量，
但数据仅适用于当时的代码和测试数据生成器。

### `parse_for_diagnostics`

| 大小       | 中位时间 | 吞吐量     |
|------------|----------|------------|
| small_1k   | 12.2 µs  | ~80 MiB/s  |
| medium_50k | 837 µs   | ~58 MiB/s  |
| large_500k | 9.13 ms  | ~54 MiB/s  |

