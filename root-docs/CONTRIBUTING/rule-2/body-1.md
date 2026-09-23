>>>>> lang=en
### 2. Don't reinvent the format in the editor layer

Editor extensions and the LSP are thin consumers of the `ktav` parser
crate. Format behaviour belongs in the Rust crate
([`ktav-lang/rust`](https://github.com/ktav-lang/rust)) — changing it
there updates every consumer at once. Only **editor-specific
ergonomics** (TextMate scopes, LSP feature wiring, IDE-specific
integrations) belong in this repo.

If your change requires a format change, start a discussion in
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) first.

>>>>> lang=ru
### 2. Не изобретайте формат на уровне редактора

Расширения редакторов и LSP — тонкие потребители крейта-парсера
`ktav`. Поведение формата принадлежит Rust-крейту
([`ktav-lang/rust`](https://github.com/ktav-lang/rust)) — правка там
обновляет всех потребителей сразу. В этом репозитории живёт только
**эргономика, специфичная для редакторов** (TextMate-скоупы, обвязка
LSP-фич, интеграции с конкретными IDE).

Если ваше изменение требует правки формата — начните с обсуждения в
[`ktav-lang/spec`](https://github.com/ktav-lang/spec).

>>>>> lang=zh
### 2. 不要在编辑器层重新发明格式

编辑器扩展和 LSP 只是 `ktav` 解析器 crate 的薄消费者。格式行为属于
Rust crate
([`ktav-lang/rust`](https://github.com/ktav-lang/rust)) —— 改那里
等于一次更新所有消费者。本仓库只放**编辑器特定的人体工学**
(TextMate scope、LSP 功能接线、IDE 特定集成)。

如果改动需要格式变更,请先去
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) 讨论。

