>>>>> lang=en
## File-type association

Save a `.ktav` file, then **View → Syntax → Open all with current
extension as…** and pick *Plain Text* (or any base syntax — the LSP
attaches via the `selector` above by file extension once you wire one
up).

For real highlighting, drop the shared TextMate grammar from
`editor/grammars/ktav.tmLanguage.json` into:

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Sublime auto-loads `.tmLanguage.json` files from `Packages/User/`.

>>>>> lang=ru
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

>>>>> lang=zh
## 文件类型关联

保存一个 `.ktav` 文件,然后 **View → Syntax → Open all with current
extension as…**,选择 *Plain Text*(或任意基础语法 —— 一旦配置好,
LSP 会通过上面的 `selector` 按文件扩展名接入)。

要获得真正的高亮,把共享 TextMate 语法
`editor/grammars/ktav.tmLanguage.json` 放到:

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Sublime 会自动加载 `Packages/User/` 中的 `.tmLanguage.json` 文件。

