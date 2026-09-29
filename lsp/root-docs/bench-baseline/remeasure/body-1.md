>>>>> lang=en
## Post-0.2.x — re-measure (2026-05-08)

Same Win10 host, IDE / language servers running concurrently
(ambient noise NOT suppressed — see methodology caveat). `cargo bench
-- --quick` against ktav-lsp 0.2.0 + locally-pinned ktav 0.2.x
(stripped multiline default + `:f` accepts integer + duplicate-key
span fix + paren-scalar parser strictness).

### parse_for_diagnostics

| size       | 0.1.x baseline | 0.2.x   | Δ        |
|------------|----------------|---------|----------|
| small_1k   | ~15.6 µs       | 15.0 µs | −4 %     |
| medium_50k | ~1.05 ms       | 644 µs  | **−39 %**|
| large_500k | ~7.8 ms        | 8.85 ms | +13 %    |

The `medium_50k` improvement is the biggest delta — the paren-scalar
classifier is faster than the previous "any scalar" fallthrough on
the synthesised fixture (which is heavy on plain pairs). `large_500k`
+13 % is within the host's noise envelope; the win and loss roughly
cancel across the workload sizes.

### semantic_tokens

| size       | 0.1.x baseline | 0.2.x   | Δ        |
|------------|----------------|---------|----------|
| small_1k   | ~6.3 µs        | 6.36 µs | ≈        |
| medium_50k | ~244 µs        | 251 µs  | +3 %     |
| large_500k | ~3.7 ms        | 3.90 ms | +5 %     |

>>>>> lang=ru
## Повторный замер после 0.2.x (2026-05-08)

Та же машина Win10; IDE и языковые серверы работали одновременно
(фоновый шум НЕ подавлялся, см. оговорку о методике). Команда `cargo bench
-- --quick` запущена для ktav-lsp 0.2.0 и локально закреплённого ktav 0.2.x
(новое поведение многострочного значения по умолчанию, `:f` принимает целое,
исправлены диапазон дублирующегося ключа и строгость скалярного парсера в скобках).

### parse_for_diagnostics

| размер     | база 0.1.x | 0.2.x   | Δ        |
|------------|------------|---------|----------|
| small_1k   | ~15.6 µs   | 15.0 µs | −4 %     |
| medium_50k | ~1.05 ms   | 644 µs  | **−39 %**|
| large_500k | ~7.8 ms    | 8.85 ms | +13 %    |

Наибольшее изменение у `medium_50k`: классификатор скаляров в скобках
быстрее прежнего запасного варианта «любой скаляр» на синтетическом
наборе с большим числом обычных пар. +13 % для `large_500k` укладываются
в шум машины; выигрыш и проигрыш примерно уравновешиваются между размерами.

### semantic_tokens

| размер     | база 0.1.x | 0.2.x   | Δ    |
|------------|------------|---------|------|
| small_1k   | ~6.3 µs    | 6.36 µs | ≈    |
| medium_50k | ~244 µs    | 251 µs  | +3 % |
| large_500k | ~3.7 ms    | 3.90 ms | +5 % |

>>>>> lang=zh
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

