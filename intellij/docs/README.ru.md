# Ktav — плагин для IntelliJ Platform

**Languages:** [English](../README.md) · **Русский** · [简体中文](README.zh.md)

> Поддержка редактора для конфигурационного формата
> [Ktav](https://github.com/ktav-lang/spec) внутри IDE от JetBrains.

Это подпроект `intellij/` монорепозитория
[`ktav-lang/editor`](https://github.com/ktav-lang/editor). Он поставляет
нативный лексер и подсветку IntelliJ Platform (а не общую
TextMate-грамматику, которую использует расширение для VS Code),
упакованные как плагин для IntelliJ Platform.

## Поддерживаемые IDE

Любая IDE на IntelliJ Platform **2023.1** (build `231`) или новее —
верхней границы нет (`untilBuild` не задан):

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio (когда её платформа достигнет 2023.1)
- Aqua, Rider, Fleet (на совместимых билдах)

## Установка

### Из JetBrains Marketplace (рекомендуется)

1. Открой **Settings → Plugins → Marketplace** в любой поддерживаемой
   IDE.
2. Найди **Ktav**.
3. Нажми **Install** и перезапусти IDE.

### Из локального zip (для тестирования)

Собери плагин (см. ниже), потом в **Settings → Plugins** открой
меню шестерёнки → **Install Plugin from Disk…** и выбери
`build/distributions/ktav-intellij-<version>.zip`.

## Возможности

- Нативная подсветка синтаксиса для `.ktav` файлов (собственный лексер,
  без TextMate).
- Переключение комментария (`Ctrl/Cmd+/`) добавляет `## ` согласно
  спецификации Ktav.
- Парные скобки и автозакрытие для `{}` `[]` `()`.
- Иконка файла и File → New → Ktav file (иконка TODO; пока используется
  стандартная иконка текстового файла платформы).

### LSP-фичи

Плагин общается с [`ktav-lsp`](../../lsp) через собственный встроенный
LSP-клиент — отдельный LSP-плагин (например, LSP4IJ) не требуется. Из
коробки доступны live-диагностика, hover, автокомплит, document
symbols, semantic tokens и форматирование.

Бинарник сервера ищется в таком порядке:

1. Явный путь, заданный в **Settings → Tools → Ktav**.
2. Бинарник, вложенный в дистрибутив плагина по пути
   `lib/bin/<platform>-<arch>/ktav-lsp` — по одному на каждую
   поддерживаемую платформу.
3. `ktav-lsp`, найденный через `PATH` вашей оболочки — установите
   его командой `cargo install ktav-lsp` (совпадает с порядком
   поиска в расширении VS Code).

## Сборка локально

Требования:

- JDK 17 или новее (сборка пиннит Kotlin toolchain на 17).
- Gradle **не нужен** — используйте wrapper.

```sh
./gradlew syncGrammars   # mirror ../grammars/ into resources/
./gradlew buildPlugin    # produces build/distributions/*.zip
./gradlew runIde         # boot a sandbox IDE with the plugin loaded
./gradlew verifyPlugin   # run the JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke tests
```
Задачи `processResources` и `compileKotlin` зависят от
`syncGrammars`, поэтому простой `./gradlew buildPlugin` уже
подтягивает свежую грамматику из `../grammars/`. Если вы забыли и
файл устарел, просто перезапустите `syncGrammars`.

## Публикация

CI запускает `./gradlew publishPlugin` с marketplace-PAT в переменной
окружения `INTELLIJ_PUBLISH_TOKEN`. Токен генерируется на
<https://plugins.jetbrains.com/author/me/tokens> и должен принадлежать
одному из мейнтейнеров плагина с id `lang.ktav` на marketplace.
Локальная публикация умышленно не поддерживается — релиз только
через тегированные CI-прогоны.

## Ссылки

- Спецификация формата и эталонный парсер:
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
- Эталонная реализация на Rust:
  [`ktav-lang/rust`](https://github.com/ktav-lang/rust)
- Прочие биндинги, LSP-сервер и расширение для VS Code — в
  [монорепозитории editor](https://github.com/ktav-lang/editor).
