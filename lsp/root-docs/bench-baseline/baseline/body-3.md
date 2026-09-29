>>>>> lang=en
## Encoding hot paths

| bench                                  | median time |
|----------------------------------------|-------------|
| `byte_to_utf16` ascii_end              | ~78 ns      |
| `byte_to_utf16` ascii_mid              | ~42 ns      |
| `byte_to_utf16` cyrillic_end           | ~86 ns      |
| `byte_to_utf16` emoji_end              | ~70 ns      |
| `prefix_by_encoding` utf16_ascii_end   | ~96 ns      |
| `prefix_by_encoding` utf16_cyrillic_end| ~137 ns     |
| `prefix_by_encoding` utf16_emoji_mid   | ~48 ns      |
| `prefix_by_encoding` utf8_ascii_end    | ~3.6 ns     |
| `prefix_by_encoding` utf8_cyrillic_end | ~3.6 ns     |

UTF-8 negotiation is ~25–40× cheaper than UTF-16 on the same input —
expected, since the UTF-8 path is just a clamp + boundary-walk and the
UTF-16 path counts code units char-by-char.

## CI

Benches are NOT run in CI — they're a developer tool. Regressions
should be caught manually by re-running `cargo bench` and diffing
against this file (or a freshly captured local baseline).

>>>>> lang=ru
## Горячие пути кодирования

| тест                                   | медиана |
|----------------------------------------|---------|
| `byte_to_utf16` ascii_end              | ~78 ns  |
| `byte_to_utf16` ascii_mid              | ~42 ns  |
| `byte_to_utf16` cyrillic_end           | ~86 ns  |
| `byte_to_utf16` emoji_end              | ~70 ns  |
| `prefix_by_encoding` utf16_ascii_end   | ~96 ns  |
| `prefix_by_encoding` utf16_cyrillic_end| ~137 ns |
| `prefix_by_encoding` utf16_emoji_mid   | ~48 ns  |
| `prefix_by_encoding` utf8_ascii_end    | ~3.6 ns |
| `prefix_by_encoding` utf8_cyrillic_end | ~3.6 ns |

Согласование UTF-8 дешевле UTF-16 примерно в 25–40 раз на том же вводе:
путь UTF-8 только ограничивает позицию и проходит до границы символа,
а путь UTF-16 посимвольно считает кодовые единицы.

## CI

Бенчмарки НЕ запускаются в CI: это инструмент разработчика. Регрессии
нужно выявлять вручную, повторно запуская `cargo bench` и сравнивая с этим
файлом либо со свежим локальным базовым замером.

>>>>> lang=zh
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

