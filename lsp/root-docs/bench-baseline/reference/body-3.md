>>>>> lang=en
The `large_500k` wall-clock went from **11.5 s → 13.2 ms** — the
defining win of this optimisation pass. Throughput is now within an
order of magnitude of `parse_for_diagnostics` (the parse step,
which is bounded by the same I/O as `build_symbols`'s text scan)
instead of three orders of magnitude slower.

### `encoding_hot_paths`

These are nanosecond-scale measurements driving the LSP position-
encoding negotiation. Mostly stable across runs — included as a
floor reference.

| Bench                                  | median   |
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

UTF-8 negotiation remains ~25–40× cheaper than UTF-16 — exactly the
asymmetry observed in earlier runs.
>>>>> lang=ru
### `encoding_hot_paths`

Эти наносекундные измерения относятся к согласованию кодировки позиций LSP.
Результаты в основном стабильны между запусками и приведены как нижний ориентир.

| тест                                   | медиана  |
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

Согласование UTF-8 остаётся примерно в 25–40 раз дешевле UTF-16 —
та же асимметрия наблюдалась в предыдущих запусках.
>>>>> lang=zh
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
