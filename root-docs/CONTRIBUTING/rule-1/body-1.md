>>>>> lang=en
### 1. Every bug fix ships with a regression test

When you find a bug, **before fixing it**, write a test that reproduces
it — the test **must fail on `main`** and pass after the fix. Include
both in the same PR.

>>>>> lang=ru
### 1. Каждый багфикс сопровождается регрессионным тестом

Найдя баг, **до его исправления** напишите тест, который его
воспроизводит — тест **должен падать на `main`** и проходить после
исправления. Оба — в одном PR.

>>>>> lang=zh
### 1. 每个 bug 修复都伴随一个回归测试

发现 bug 时,**在修复之前**先写一个能复现它的测试 —— 测试在 `main`
上**必须失败**,修复之后才通过。两者放在同一个 PR 中。

