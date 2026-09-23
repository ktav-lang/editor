>>>>> lang=en
## Where it is used

These two JSON files are the canonical artifacts. Downstream packages
consume them by reference (copy or symlink) — no logic is duplicated:

- `vscode/` — bundles them via `package.json` `contributes.languages`
  and `contributes.grammars`.
- `intellij/` — loads `ktav.tmLanguage.json` through the IntelliJ
  TextMate bundle API.

When you change the grammar here, both downstream packages pick up
the new behavior on their next build / packaging step. Do not fork
copies in the downstream subprojects; fix the bug here.

>>>>> lang=ru
## Где используется

Эти два JSON-файла — канонические артефакты. Downstream-пакеты
подключают их по ссылке (копия или symlink) — логика не дублируется:

- `vscode/` — включает их через `contributes.languages` и
  `contributes.grammars` в `package.json`.
- `intellij/` — загружает `ktav.tmLanguage.json` через API TextMate-бандлов
  IntelliJ.

Когда вы меняете грамматику здесь, оба downstream-пакета получают новое
поведение при следующей сборке / упаковке. Не заводите копии в
downstream-подпроектах; исправляйте ошибку здесь.

>>>>> lang=zh
## 使用位置

这两个 JSON 文件是规范产物。下游包通过引用(复制或符号链接)使用它们
——不重复任何逻辑:

- `vscode/` — 通过 `package.json` 的 `contributes.languages` 与
  `contributes.grammars` 打包它们。
- `intellij/` — 通过 IntelliJ TextMate 包 API 加载 `ktav.tmLanguage.json`。

在此修改语法后,两个下游包会在下一次构建 / 打包时获得新行为。不要在
下游子项目中分叉副本;请在这里修复问题。

