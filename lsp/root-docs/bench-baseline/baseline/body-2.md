>>>>> lang=en
| size       | median time | throughput |
|------------|-------------|------------|
| small_1k   | ~15.6 µs    | ~65 MiB/s  |
| medium_50k | ~1.05 ms    | ~47 MiB/s  |
| large_500k | ~7.8 ms     | ~62 MiB/s  |

## semantic_tokens

| size       | median time | throughput |
|------------|-------------|------------|
| small_1k   | ~6.3 µs     | ~163 MiB/s |
| medium_50k | ~244 µs     | ~200 MiB/s |
| large_500k | ~3.7 ms     | ~132 MiB/s |

## build_symbols (post-parse, time excludes parse)

| size       | median time | throughput  |
|------------|-------------|-------------|
| small_1k   | ~71 µs      | ~14 MiB/s   |
| medium_50k | ~101 ms     | ~494 KiB/s  |
| large_500k | ~11.0 s     | ~45 KiB/s   |

The cliff at medium/large is the `locate_key` full-text scan per top-
level key (super-linear with key count × text length). Optimization
target — out of scope for this baseline pass.

>>>>> lang=ru
| размер     | медиана     | пропускная способность |
|------------|-------------|------------------------|
| small_1k   | ~15.6 µs    | ~65 MiB/s              |
| medium_50k | ~1.05 ms    | ~47 MiB/s              |
| large_500k | ~7.8 ms     | ~62 MiB/s              |

## semantic_tokens

| размер     | медиана  | пропускная способность |
|------------|----------|------------------------|
| small_1k   | ~6.3 µs  | ~163 MiB/s             |
| medium_50k | ~244 µs  | ~200 MiB/s             |
| large_500k | ~3.7 ms  | ~132 MiB/s             |

## build_symbols (после разбора, без учёта времени разбора)

| размер     | медиана | пропускная способность |
|------------|---------|------------------------|
| small_1k   | ~71 µs  | ~14 MiB/s              |
| medium_50k | ~101 ms | ~494 KiB/s             |
| large_500k | ~11.0 s | ~45 KiB/s              |

Резкое ухудшение на средних и больших файлах вызвано полным сканированием
текста в `locate_key` для каждого ключа верхнего уровня (рост быстрее
линейного по числу ключей × длине текста). Оптимизация не входила в этот
базовый проход.

>>>>> lang=zh
| 大小       | 中位时间  | 吞吐量     |
|------------|-----------|------------|
| small_1k   | ~15.6 µs  | ~65 MiB/s  |
| medium_50k | ~1.05 ms  | ~47 MiB/s  |
| large_500k | ~7.8 ms   | ~62 MiB/s  |

## semantic_tokens

| 大小       | 中位时间 | 吞吐量     |
|------------|----------|------------|
| small_1k   | ~6.3 µs  | ~163 MiB/s |
| medium_50k | ~244 µs  | ~200 MiB/s |
| large_500k | ~3.7 ms  | ~132 MiB/s |

## build_symbols (解析后计时，不含解析)

| 大小       | 中位时间 | 吞吐量     |
|------------|----------|------------|
| small_1k   | ~71 µs   | ~14 MiB/s  |
| medium_50k | ~101 ms  | ~494 KiB/s |
| large_500k | ~11.0 s  | ~45 KiB/s  |

中、大文件的性能断崖源于 `locate_key` 为每个顶层键全文扫描
(复杂度随键数 × 文本长度超线性增长)。优化不在这次基线工作范围内。

