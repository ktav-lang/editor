# Sublime Text

**Languages:** [English](../../../sublime.md) · **Русский** · [简体中文](../zh/sublime.md)

Sublime Text 4 с пакетом [`LSP`](https://packagecontrol.io/packages/LSP).

## Установка

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp --version 0.8.0 --locked`

## Настройка

Preferences → Package Settings → LSP → Settings:

```jsonc
{
  "clients": {
    "ktav-lsp": {
      "enabled": true,
      "command": ["ktav-lsp"],
      "selector": "source.ktav"
    }
  }
}
```

## Привязка типа файла

Sublime Text загружает XML-файлы `.tmLanguage` (или `.sublime-syntax`),
но не общую `.tmLanguage.json` напрямую. Установив Node.js, выполните
конвертацию без дополнительных зависимостей из корня репозитория editor:

```sh
node grammars/scripts/export-tmlanguage.js grammars/ktav.tmLanguage
```

Команда читает каноническую `grammars/ktav.tmLanguage.json` и создаёт
`grammars/ktav.tmLanguage` — XML-синтаксис TextMate с именем **Ktav**,
базовым scope `source.ktav` и расширением `ktav`. Скопируйте созданный
файл `.tmLanguage` в каталог Sublime **Packages/User** (откройте его через
**Preferences → Browse Packages…**):

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Откройте файл `.ktav`, затем **View → Syntax → Open all with current
extension as…** и выберите **Ktav**. LSP `selector` выше сопоставляется
с базовым scope редактора `source.ktav`, а не с расширением файла.
**Plain Text** использует `text.plain` и не соответствует этому selector.
При обновлении канонической грамматики повторите экспорт и замените
XML-синтаксис.

## Проверка

Откройте файл `.ktav` с синтаксисом **Ktav**. **Tools → Developer → Show
Scope Name** должен показать базовый scope `source.ktav`. При запуске
сервера в строке состояния появляется `ktav-lsp`. Если сервер не
запускается, перейдите в этот файл и выполните **LSP: Troubleshoot server**
из Command Palette, затем выберите `ktav-lsp`. См. [руководство по диагностике LSP](https://lsp.sublimetext.io/troubleshooting/).
