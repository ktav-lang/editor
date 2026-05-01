# Ktav — плагин для IntelliJ Platform

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

> Поддержка редактора для конфигурационного формата
> [Ktav](https://github.com/ktav-lang/spec) внутри IDE от JetBrains.

Это подпроект `intellij/` монорепозитория
[`ktav-lang/editor`](https://github.com/ktav-lang/editor). Использует
ту же TextMate-грамматику, что и расширение для VS Code, обёрнутую
как плагин для IntelliJ Platform.

## Поддерживаемые IDE

Любая IDE на IntelliJ Platform **2024.3** (build `243`) или новее:

- IntelliJ IDEA Community / Ultimate
- RustRover
- GoLand
- WebStorm
- PyCharm Community / Professional
- PhpStorm
- RubyMine
- CLion
- DataGrip
- Android Studio (Iguana / Jellyfish или новее, когда их платформа
  догонит)
- Aqua, Rider, Fleet (на совместимых билдах)

`untilBuild` сейчас `251.*` (покрывает 2024.3 → 2025.1). Каждый
релиз плагина проходит проверку на новых версиях IDE.

## Установка

### Из JetBrains Marketplace (рекомендуется)

1. Открой **Settings → Plugins → Marketplace** в любой поддерживаемой
   IDE.
2. Найди **Ktav**.
3. Нажми **Install** и перезапусти IDE.

### Из локального zip (для тестирования)

Собери плагин (см. ниже), потом в **Settings → Plugins** нажми на
шестерёнку → **Install Plugin from Disk…** и выбери
`build/distributions/ktav-intellij-<version>.zip`.

## Возможности

- Подсветка синтаксиса для `.ktav` файлов через общую TextMate-грамматику.
- Переключение комментария (`Ctrl/Cmd+/`) добавляет `# ` согласно
  спецификации Ktav.
- Парные скобки и автозакрытие для `{}` `[]` `()`.
- Иконка файла и File → New → Ktav file (иконка TODO; пока используется
  дефолтная иконка text-файла платформы).

### LSP-фичи (опционально)

Если рядом с Ktav установлен плагин
[LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij), вы
получаете live-диагностику, hover, автокомплит, document symbols и
semantic tokens из [`ktav-lsp`](../lsp). Без LSP4IJ плагин продолжает
работать в режиме TextMate-only — поставьте LSP4IJ из Marketplace,
когда понадобятся расширенные возможности.

Бинарник сервера ищется в таком порядке:

1. Явный путь, настроенный в **Settings → Tools → Ktav**.
2. Бинарник, поставляемый внутри дистрибутива плагина по пути
   `bin/<platform>-<arch>/ktav-lsp` (в текущем релизе не поставляется).
3. `ktav-lsp`, найденный через `PATH` — установите его командой
   `cargo install ktav-lsp` (совпадает с порядком поиска в VS Code).

## Сборка локально

Требования:

- JDK 17 или новее (build пиннит Kotlin toolchain на 17).
- Gradle **не нужен** — используется wrapper.

```sh
./gradlew syncGrammars   # синхронизировать ../grammars/ в resources/
./gradlew buildPlugin    # собрать build/distributions/*.zip
./gradlew runIde         # запустить sandbox-IDE с подгруженным плагином
./gradlew verifyPlugin   # прогнать JetBrains plugin verifier
./gradlew test           # JUnit 5 smoke-тесты
```

Таски `processResources` и `compileKotlin` зависят от `syncGrammars`,
поэтому простой `./gradlew buildPlugin` уже подтягивает свежую
грамматику из `../grammars/`. Если забыл и файл устарел — просто
перезапусти `syncGrammars`.

## Публикация

CI запускает `./gradlew publishPlugin` с marketplace-PAT в переменной
окружения `INTELLIJ_PUBLISH_TOKEN`. Токен генерируется на
<https://plugins.jetbrains.com/author/me/tokens> и должен принадлежать
maintainer'у плагина с id `lang.ktav` на marketplace. Локальная
публикация умышленно не поддерживается — релиз только через тегированные
CI-прогоны.

## Ссылки

- Спецификация формата и эталонный парсер:
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
- Эталонная Rust-реализация:
  [`ktav-lang/rust`](https://github.com/ktav-lang/rust)
- Остальные биндинги, LSP-сервер и расширение для VS Code — в
  [монорепо editor](https://github.com/ktav-lang/editor).
