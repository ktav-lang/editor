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

## 0.1.0

- Первый релиз плагина Ktav для IntelliJ Platform.
- Регистрирует тип файлов `Ktav` для `*.ktav`.
- Переключение комментария (`# `) подключено через `lang.commenter`.
- Бандлит общую TextMate-грамматику из `editor/grammars/`.
- Целевая платформа: IntelliJ Platform 2024.3 (build `243`) — 2025.1 (`251.*`).
