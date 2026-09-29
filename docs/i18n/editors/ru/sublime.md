# Sublime Text

**Languages:** [English](../../../sublime.md) · **Русский** · [简体中文](../zh/sublime.md)

Sublime Text 4 с пакетом [`LSP`](https://packagecontrol.io/packages/LSP).

## Установка

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp`

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

Сохраните файл `.ktav`, затем **View → Syntax → Open all with current
extension as…** и выберите *Plain Text* (или любой базовый синтаксис
— LSP подключается через `selector` выше по расширению файла, как
только вы это настроите).

Для настоящей подсветки поместите общую TextMate-грамматику из
`editor/grammars/ktav.tmLanguage.json` в:

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Sublime автоматически загружает файлы `.tmLanguage.json` из
`Packages/User/`.

## Проверка

Откройте файл `.ktav` → `Tools → LSP → Show diagnostics` — клиент
`ktav-lsp` должен отображаться как подключённый.
