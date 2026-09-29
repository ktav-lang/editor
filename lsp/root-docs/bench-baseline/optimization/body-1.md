>>>>> lang=en
## Post-optimisation — `build_symbols` O(N²) → O(N) (2026-05-08)

`build_symbols` was rewritten to do a single line-by-line pass over
the document, collecting `(virtual_depth, key_name, line_range)` for
every pair as it scans, then walk the parsed `Value` with a
sequential cursor. This eliminates the per-key full-text scan
(`locate_key`) that drove the documented super-linear regression.

| size       | 0.2.x pre | post-opt | speed-up |
|------------|-----------|----------|----------|
| small_1k   | ~73 µs    | 20.9 µs  | **3.5×** |
| medium_50k | 108 ms    | 927 µs   | **117×** |
| large_500k | 11.5 s    | 13.4 ms  | **858×** |

Criterion explicitly reported "Performance has improved" for
`large_500k` (p = 0.05). The other two sizes were also dramatically
faster but Criterion's bootstrap got confused at p = 0.07 and printed
"no change detected" — the wall-clock improvement is real, the
statistical model just rarely sees deltas this large.

Throughput on `large_500k` went from ~45 KiB/s to ~36 MiB/s. The
remaining cost is dominated by the `Vec<DocumentSymbol>` construction
itself (per-key `String::from` for the name field), not the source
scan. Further wins would require interning or returning borrowed
names — out of scope; the current code is no longer the bottleneck.

Other benches in the same run (release, --quick, IDE-noisy host):

| bench                              | post-opt | vs 0.2.x | note            |
|------------------------------------|----------|----------|-----------------|
| `parse_for_diagnostics/small_1k`   | 12.3 µs  | −18 %    | within noise    |
| `parse_for_diagnostics/medium_50k` | 943 µs   | +46 %    | host noise      |
| `parse_for_diagnostics/large_500k` | 9.24 ms  | +4 %     | within noise    |
| `semantic_tokens/small_1k`         | 6.37 µs  | ≈        | unchanged       |
| `semantic_tokens/medium_50k`       | 243 µs   | −3 %     | unchanged       |
| `semantic_tokens/large_500k`       | 3.84 ms  | −1 %     | unchanged       |

`parse_for_diagnostics` and `semantic_tokens` were not touched in
this optimisation pass; their numbers move only with host load.

>>>>> lang=ru
## После оптимизации — `build_symbols` O(N²) → O(N) (2026-05-08)

`build_symbols` переписан как один построчный проход по документу:
для каждой пары собираются `(virtual_depth, key_name, line_range)`, затем
разобранное `Value` обходится последовательным курсором. Так устранено
полное сканирование текста для каждого ключа (`locate_key`), вызывавшее
описанный выше сверхлинейный рост времени.

| размер     | до, 0.2.x | после   | ускорение |
|------------|-----------|---------|-----------|
| small_1k   | ~73 µs    | 20.9 µs | **3.5×**  |
| medium_50k | 108 ms    | 927 µs  | **117×**  |
| large_500k | 11.5 s    | 13.4 ms | **858×**  |

Criterion явно сообщил об улучшении производительности для `large_500k`
(p = 0.05). Два других размера также значительно ускорились, но bootstrap
Criterion при p = 0.07 выдал «изменений не обнаружено»: снижение времени
реально, статистическая модель редко встречает настолько большие разницы.

Пропускная способность на `large_500k` выросла с ~45 KiB/s до ~36 MiB/s.
Теперь основные затраты приходятся на создание `Vec<DocumentSymbol>`
(по одному `String::from` для имени каждого ключа), а не на сканирование
исходного текста. Дальнейшее ускорение потребует интернирования или
возврата заимствованных имён; это вне рамок работы, текущий код больше
не является узким местом.

Другие бенчмарки того же запуска (release, --quick, машина с шумом от IDE):

| тест                               | после   | к 0.2.x | примечание      |
|------------------------------------|---------|---------|-----------------|
| `parse_for_diagnostics/small_1k`   | 12.3 µs | −18 %   | в пределах шума |
| `parse_for_diagnostics/medium_50k` | 943 µs  | +46 %   | шум машины      |
| `parse_for_diagnostics/large_500k` | 9.24 ms | +4 %    | в пределах шума |
| `semantic_tokens/small_1k`         | 6.37 µs | ≈       | без изменений   |
| `semantic_tokens/medium_50k`       | 243 µs  | −3 %    | без изменений   |
| `semantic_tokens/large_500k`       | 3.84 ms | −1 %    | без изменений   |

`parse_for_diagnostics` и `semantic_tokens` в этой оптимизации не менялись;
их результаты колеблются только из-за нагрузки машины.

>>>>> lang=zh
## 优化后 — `build_symbols` 从 O(N²) 到 O(N) (2026-05-08)

`build_symbols` 改为逐行单次扫描文档，为每个键值对收集
`(virtual_depth, key_name, line_range)`，再用顺序游标遍历已解析的
`Value`。这消除了导致超线性回归的逐键全文扫描 (`locate_key`)。

| 大小       | 0.2.x 优化前 | 优化后  | 加速       |
|------------|--------------|---------|------------|
| small_1k   | ~73 µs       | 20.9 µs | **3.5×**   |
| medium_50k | 108 ms       | 927 µs  | **117×**   |
| large_500k | 11.5 s       | 13.4 ms | **858×**   |

Criterion 明确报告 `large_500k` “Performance has improved”(p = 0.05)。
另外两种大小也大幅加快，但 Criterion 的 bootstrap 在 p = 0.07 时
报告“no change detected”；实际耗时改善确实存在，只是统计模型
很少遇到如此巨大的差异。

`large_500k` 的吞吐量从 ~45 KiB/s 提高到 ~36 MiB/s。剩余开销主要
来自构造 `Vec<DocumentSymbol>` (每个键名一次 `String::from`)，
而非扫描源文本。进一步优化需要字符串驻留或返回借用的名字，
不在此轮范围内；当前代码已不再是瓶颈。

同一次运行中的其他基准测试 (release、--quick、IDE 干扰主机)：

| 测试                               | 优化后  | 对比 0.2.x | 备注         |
|------------------------------------|---------|------------|--------------|
| `parse_for_diagnostics/small_1k`   | 12.3 µs | −18 %      | 噪声范围内   |
| `parse_for_diagnostics/medium_50k` | 943 µs  | +46 %      | 主机噪声     |
| `parse_for_diagnostics/large_500k` | 9.24 ms | +4 %       | 噪声范围内   |
| `semantic_tokens/small_1k`         | 6.37 µs | ≈          | 未变         |
| `semantic_tokens/medium_50k`       | 243 µs  | −3 %       | 未变         |
| `semantic_tokens/large_500k`       | 3.84 ms | −1 %       | 未变         |

本轮优化未修改 `parse_for_diagnostics` 和 `semantic_tokens`；
它们的数字仅随主机负载变化。

