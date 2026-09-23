>>>>> lang=en
### 3. One concept per commit

Commits should be atomic: a bug fix and its test together, a feature
and its tests together, a rename on its own, a refactor on its own.
`git log --oneline` should read like a changelog. Don't prefix commit
messages with `feat:` / `fix:` — no conventional commits here.

>>>>> lang=ru
### 3. Один концепт — один коммит

Коммиты должны быть атомарными: багфикс и его тест вместе, фича и её
тесты вместе, переименование — отдельно, рефакторинг — отдельно.
`git log --oneline` должен читаться как журнал изменений. Не
префиксируйте сообщения коммитов `feat:` / `fix:` — здесь нет
conventional commits.

>>>>> lang=zh
### 3. 一个概念一次提交

提交应当保持原子性:bug 修复与其测试一起、功能与其测试一起、重命名
单独、重构单独。`git log --oneline` 应当读起来像变更日志。不要给
提交信息加 `feat:` / `fix:` 前缀 —— 这里不用 conventional commits。

