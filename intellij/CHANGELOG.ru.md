# Журнал изменений

**Languages:** [English](CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)

Все значимые изменения плагина Ktav для IntelliJ Platform
документируются здесь. Формат:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); версии
следуют [Semantic Versioning](https://semver.org/) с pre-1.0
конвенцией: MINOR-бамп считается ломающим.

Этот файл также отображается в IDE в pane'е плагинов — только первые
двадцать строк форвардятся через build (см. `changeNotes` mapping в
`build.gradle.kts`), поэтому держи свежие релизы наверху и используй
короткие булиты.

## 0.5.1

- Диапазон совместимости поднят до IntelliJ 2023.1+ (`since-build 231`).
  Верификатор Marketplace выдавал жёсткую несовместимость на 2022.1–2022.3;
  все сборки 231+ проходят как Compatible, теперь диапазон соответствует
  реальности.
- Чистка устаревших API (без изменения поведения): `CodeInsightColors.INFO_ATTRIBUTES`
  → `WEAK_WARNING_ATTRIBUTES`; `TextFieldWithBrowseButton.addBrowseFolderListener(title, …)`
  → ручной `FileChooser.chooseFile` с заголовком на дескрипторе;
  `TextAttributesKey.createTextAttributesKey(String, TextAttributes)` →
  `enforcedTextAttributes`. Плагин верифицируется без предупреждений на 2023.1–2024.3.
- Бандлит тот же `ktav-lsp 0.5.0`.

## 0.1.0

- Первый релиз плагина Ktav для IntelliJ Platform.
- Регистрирует тип файлов `Ktav` для `*.ktav`.
- Переключение комментария (`# `) подключено через `lang.commenter`.
- Бандлит общую TextMate-грамматику из `editor/grammars/`.
- Целевая платформа: IntelliJ Platform 2024.3 (build `243`) — 2025.1 (`251.*`).
