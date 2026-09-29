# LSP 基准测试历史基线

> **不适用于当前 0.8.0。** 以下所有测量均于 2026-05-08 针对
> `ktav-lsp` 0.2.0 在临时使用的 Windows 10 开发主机上完成。
> 此后未重新运行，不能代表当前性能。本文仅供参考历史测量方法，
> 不应当作今天的性能数据。

在 Windows 10 开发主机上以 `cargo bench -- --quick` 采集
(硬件未注明，未抑制前台干扰)。数据仅供参考，个位数百分比的回归
处于噪声范围内。精确的本地比较应重新运行不带 `--quick` 的
`cargo bench`，并与自己新采集的基线比较。

## 方法

- 生成器 `benches/fixtures.rs` 确定性地混合普通键值对、点分键、
  原始标记 (`::`)、嵌套对象、数组、多行原始块和注释。
- 文件大小：`small_1k` ≈ 1 KiB、`medium_50k` ≈ 50 KiB、`large_500k` ≈ 500 KiB。
- Criterion 的 `--quick` 配置缩短测量窗口；以下结果只能用于判断数量级。

## parse_for_diagnostics

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

## 编码热点路径

| 测试                                   | 中位时间 |
|----------------------------------------|----------|
| `byte_to_utf16` ascii_end              | ~78 ns   |
| `byte_to_utf16` ascii_mid              | ~42 ns   |
| `byte_to_utf16` cyrillic_end           | ~86 ns   |
| `byte_to_utf16` emoji_end              | ~70 ns   |
| `prefix_by_encoding` utf16_ascii_end   | ~96 ns   |
| `prefix_by_encoding` utf16_cyrillic_end| ~137 ns  |
| `prefix_by_encoding` utf16_emoji_mid   | ~48 ns   |
| `prefix_by_encoding` utf8_ascii_end    | ~3.6 ns  |
| `prefix_by_encoding` utf8_cyrillic_end | ~3.6 ns  |

同一输入下 UTF-8 协商比 UTF-16 快约 25–40 倍：UTF-8 路径只需截断
并走到字符边界，UTF-16 路径则逐字符统计代码单元。

## CI

CI 不运行基准测试；它们是开发者工具。应手动重跑 `cargo bench`，
并与本文件或新采集的本地基线比较，以发现性能回归。

## 0.2.x 之后的复测 (2026-05-08)

仍使用同一台 Win10 主机，IDE 和语言服务器同时运行
(环境噪声未抑制，参见方法说明)。对 ktav-lsp 0.2.0 和本地固定的
ktav 0.2.x 运行 `cargo bench -- --quick`；该版本包含多行默认值调整、
`:f` 接受整数、重复键范围修复、括号标量解析器严格性调整。

### parse_for_diagnostics

| 大小       | 0.1.x 基线 | 0.2.x   | Δ        |
|------------|------------|---------|----------|
| small_1k   | ~15.6 µs   | 15.0 µs | −4 %     |
| medium_50k | ~1.05 ms   | 644 µs  | **−39 %**|
| large_500k | ~7.8 ms    | 8.85 ms | +13 %    |

`medium_50k` 改善最大：在普通键值对占比较高的合成样本中，
括号标量分类器比以前的“任意标量”后备路径更快。`large_500k`
的 +13 % 处于主机噪声范围内，各规模上的得失大致抵消。

### semantic_tokens

| 大小       | 0.1.x 基线 | 0.2.x   | Δ    |
|------------|------------|---------|------|
| small_1k   | ~6.3 µs    | 6.36 µs | ≈    |
| medium_50k | ~244 µs    | 251 µs  | +3 % |
| large_500k | ~3.7 ms    | 3.90 ms | +5 % |

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

### `semantic_tokens`

| 大小       | 中位时间 | 吞吐量      |
|------------|----------|-------------|
| small_1k   | 6.81 µs  | ~152 MiB/s  |
| medium_50k | 288 µs   | ~169 MiB/s  |
| large_500k | 4.27 ms  | ~115 MiB/s  |

### `build_symbols` — O(N²)→O(N) 重写之后

| 大小       | 中位时间 | 吞吐量     | 相比优化前 |
|------------|----------|------------|------------|
| small_1k   | 23.9 µs  | ~43 MiB/s  | 快 3.05×   |
| medium_50k | 1.14 ms  | ~43 MiB/s  | 快 94.7×   |
| large_500k | 13.2 ms  | ~37 MiB/s  | **快 871×**|

`large_500k` 的实际耗时从 **11.5 s 降至 13.2 ms**，是这次优化的
决定性成果。吞吐量现在与 `parse_for_diagnostics` 相差不到一个
数量级 (解析步骤与 `build_symbols` 的文本扫描都受相同 I/O 限制)，
不再慢三个数量级。

### `encoding_hot_paths`

这些纳秒级测量用于 LSP 位置编码协商。在多次运行间基本稳定，
因此列作性能下限参考。

| 测试                                   | 中位时间 |
|----------------------------------------|----------|
| `byte_to_utf16/ascii_end`              | 62.5 ns  |
| `byte_to_utf16/ascii_mid`              | 40.0 ns  |
| `byte_to_utf16/cyrillic_end`           | 93.1 ns  |
| `byte_to_utf16/emoji_end`              | 70.8 ns  |
| `prefix_by_encoding/utf16_ascii_end`   | 109 ns   |
| `prefix_by_encoding/utf16_cyrillic_end`| 166 ns   |
| `prefix_by_encoding/utf16_emoji_mid`   | 54.6 ns  |
| `prefix_by_encoding/utf8_ascii_end`    | 3.96 ns  |
| `prefix_by_encoding/utf8_cyrillic_end` | 4.34 ns  |

UTF-8 协商仍比 UTF-16 快约 25–40 倍，与之前运行观察到的不对称性一致。
