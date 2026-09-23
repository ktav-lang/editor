>>>>> lang=en
## 0.5.1

- Compatibility range raised to IntelliJ 2023.1+ (`since-build 231`).
  The Marketplace verifier reported a hard incompatibility on 2022.1–2022.3;
  every build 231+ verifies as Compatible, so the range now matches reality.
- API-deprecation / internal-API cleanup (no behaviour change): `INFO_ATTRIBUTES`
  → `WEAK_WARNING_ATTRIBUTES`; `addBrowseFolderListener(title, …)` → manual
  `FileChooser.chooseFile`; `createTextAttributesKey(String, TextAttributes)` →
  `enforcedTextAttributes`; `FileChooserDescriptorFactory.createSingleFileDescriptor()`
  → the `FileChooserDescriptor` constructor; `Document.addDocumentListener(l)` → the
  `Disposable` overload (also fixes a listener leak); `DaemonCodeAnalyzer.restart()` →
  per-file `restart(PsiFile)`; internal `PluginManagerCore.getPlugin(id)` → the
  plugin's own class-loader descriptor. Verifies warning-free on 2023.1–2024.3.
- Bundles the same `ktav-lsp 0.5.0`.

>>>>> lang=ru
## 0.5.1

- Диапазон совместимости поднят до IntelliJ 2023.1+ (`since-build 231`).
  Верификатор Marketplace выдавал жёсткую несовместимость на
  2022.1–2022.3; все сборки 231+ проходят как Compatible, поэтому
  теперь диапазон соответствует реальности.
- Чистка устаревших / внутренних API (без изменения поведения):
  `INFO_ATTRIBUTES` → `WEAK_WARNING_ATTRIBUTES`;
  `addBrowseFolderListener(title, …)` → ручной `FileChooser.chooseFile`;
  `createTextAttributesKey(String, TextAttributes)` →
  `enforcedTextAttributes`;
  `FileChooserDescriptorFactory.createSingleFileDescriptor()` →
  конструктор `FileChooserDescriptor`;
  `Document.addDocumentListener(l)` → перегрузка с `Disposable`
  (заодно исправляет утечку слушателя);
  `DaemonCodeAnalyzer.restart()` → пофайловый `restart(PsiFile)`;
  внутренний `PluginManagerCore.getPlugin(id)` → дескриптор
  собственного class-loader плагина. Проверяется без предупреждений
  на 2023.1–2024.3.
- Бандлит тот же `ktav-lsp 0.5.0`.

>>>>> lang=zh
## 0.5.1

- 兼容范围提升至 IntelliJ 2023.1+(`since-build 231`)。
  Marketplace 验证器在 2022.1–2022.3 上报告了硬性不兼容;所有 231+ 构建
  均验证为 Compatible,因此范围现已与实际情况一致。
- 清理弃用 / 内部 API(行为不变):`INFO_ATTRIBUTES` → `WEAK_WARNING_ATTRIBUTES`;
  `addBrowseFolderListener(title, …)` → 手动 `FileChooser.chooseFile`;
  `createTextAttributesKey(String, TextAttributes)` → `enforcedTextAttributes`;
  `FileChooserDescriptorFactory.createSingleFileDescriptor()` →
  `FileChooserDescriptor` 构造函数;
  `Document.addDocumentListener(l)` → 带 `Disposable` 的重载(顺带修复监听器泄漏);
  `DaemonCodeAnalyzer.restart()` → 按文件的 `restart(PsiFile)`;内部
  `PluginManagerCore.getPlugin(id)` → 插件自身 class-loader 的描述符。
  在 2023.1–2024.3 上验证零警告。
- 仍捆绑相同的 `ktav-lsp 0.5.0`。

