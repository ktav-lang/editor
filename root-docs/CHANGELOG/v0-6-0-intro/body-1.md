>>>>> lang=en
## [0.6.0] — 2026-06-01

Tracks `ktav` Rust crate `0.6.0` and `ktav-lang/spec` `0.6.0`. The
version is realigned to move in lockstep with the format/core (the
previous editor release was `0.3.1`). The spec change is **key
escaping**: keys now process the § 3.7 escape set and add `\.` (a
literal dot that does *not* split a dotted path) and `\:` (a literal
colon that is *not* a pair separator); a literal backslash in a key is
now written `\\`. Editor support is updated end-to-end so highlighting,
tokens and diagnostics treat escaped key bytes correctly.

>>>>> lang=ru
## [0.6.0] — 2026-06-01

Синхронизация с крейтом `ktav` `0.6.0` и `ktav-lang/spec` `0.6.0`. Версия
выровнена, чтобы двигаться в ногу с форматом и ядром (предыдущий
редакторский релиз был `0.3.1`). Изменение спецификации —
**экранирование ключей**: ключи теперь обрабатывают набор escape'ов
§ 3.7 и добавляют `\.` (литеральная точка, которая *не* разделяет
точечный путь) и `\:` (литеральное двоеточие, которое *не* является
разделителем пары); литеральный обратный слеш в ключе теперь
пишется `\\`. Поддержка редакторов обновлена end-to-end, чтобы
подсветка, токены и диагностика корректно обрабатывали
экранированные байты ключей.

>>>>> lang=zh
## [0.6.0] — 2026-06-01

同步 `ktav` crate `0.6.0` 与 `ktav-lang/spec` `0.6.0`。版本号重新对齐,
与格式/核心保持同步步调(此前的编辑器版本为 `0.3.1`)。规范改动是
**键转义**:键现在会处理 § 3.7 转义集,并新增 `\.`(字面点,*不*分割
点分路径)和 `\:`(字面冒号,*不*作为键值分隔符);键中的字面反斜杠
现在写作 `\\`。编辑器支持已端到端更新,使高亮、令牌和诊断都能正确
处理转义后的键字节。

