# Журнал изменений

**Languages:** [English](../CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)

Все значимые изменения плагина Ktav для IntelliJ Platform
документируются здесь. Формат:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); версии
следуют [Semantic Versioning](https://semver.org/) с соглашением
pre-1.0: MINOR-бамп считается ломающим.

Этот файл также отображается в панели плагинов IDE — сборка
пересылает только первые двадцать строк (см. маппинг `changeNotes`
в `build.gradle.kts`), поэтому держи свежие релизы наверху и
предпочитай короткие пункты.

## Не выпущено

Все три компонента (`ktav-lsp`, расширение VS Code, плагин IntelliJ)
переходят на 0.8.0 синхронно с crate `ktav` и спецификацией — см.
корневой [`CHANGELOG.md`](../../CHANGELOG.md) (раздел «Не выпущено»)
для деталей.

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

## 0.5.0

- Бандлит `ktav-lsp 0.5.0` (синхронизация с ktav 0.5.0 и spec 0.5.0).
- TextMate-грамматика: паттерн комментариев изменён на `##`, убраны
  паттерны типизированных маркеров `:i`/`:f`, добавлены паттерны
  инлайн-составных значений и числовых литералов.
- Лицензия: двойная `MIT OR Apache-2.0`.

## 0.3.1

- Бандлит `ktav-lsp 0.3.1` (синхронизация с ktav 0.3.1 и spec 0.1.1).
- Дерево document-symbols теперь для элементов Array верхнего
  уровня выводит записи `[0]`, `[1]`, …

## 0.1.0

- Первый релиз плагина Ktav для IntelliJ Platform.
- Регистрирует тип файлов `Ktav` для файлов `*.ktav`.
- Переключение комментария (`# `) подключено через `lang.commenter`.
- Бандлит общую TextMate-грамматику из `editor/grammars/`.
- Целевые платформы: IntelliJ Platform 2024.3 (build `243`) —
  2025.1 (`251.*`).
