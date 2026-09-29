>>>>> lang=en
# LSP benchmark baseline

> **Not current for 0.8.0.** All measurements below were captured
> against `ktav-lsp` 0.2.0 (2026-05-08) on ad hoc Windows 10 dev
> hosts; they have not been re-run since and are not a claim about
> current performance. Use them only as historical methodology
> reference, not as today's numbers.

Captured with `cargo bench -- --quick` on a Windows 10 dev host
(unspecified hardware, foreground noise NOT suppressed). Numbers are
indicative — single-digit-percent regressions here are within noise.
For accurate local comparisons re-run `cargo bench` (without `--quick`)
and compare against your own freshly captured baseline.

## Methodology

- Generator: `benches/fixtures.rs` — deterministic synthesizer mixing
  plain pairs, dotted keys, raw markers (`::`), nested objects, arrays,
  multi-line raw blocks, and comments.
- Sizes: `small_1k` ≈ 1 KiB, `medium_50k` ≈ 50 KiB, `large_500k` ≈ 500 KiB.
- Criterion `--quick` profile: shorter measurement windows. Treat the
  numbers below as order-of-magnitude only.

## parse_for_diagnostics

>>>>> lang=ru
# Исторические результаты бенчмарков LSP

> **Неактуально для 0.8.0.** Все приведённые ниже измерения сделаны
> для `ktav-lsp` 0.2.0 (2026-05-08) на рабочих машинах с Windows 10.
> С тех пор тесты не запускались повторно; эти цифры не характеризуют
> текущую производительность. Используйте документ только как описание
> исторической методики, а не как сегодняшние результаты.

Замеры выполнены командой `cargo bench -- --quick` на машине с Windows 10
(оборудование не указано, фоновая активность НЕ подавлялась). Цифры
ориентировочные: отклонения в несколько процентов укладываются в шум.
Для точного локального сравнения запустите `cargo bench` без `--quick`
и сравните с собственным свежим базовым замером.

## Методика

- Генератор `benches/fixtures.rs` детерминированно смешивает обычные пары,
  составные ключи, сырые маркеры (`::`), вложенные объекты, массивы,
  многострочные сырые блоки и комментарии.
- Размеры: `small_1k` ≈ 1 KiB, `medium_50k` ≈ 50 KiB, `large_500k` ≈ 500 KiB.
- Профиль Criterion `--quick` сокращает время измерения. Цифры ниже
  годятся лишь для оценки порядка величины.

## parse_for_diagnostics

>>>>> lang=zh
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

