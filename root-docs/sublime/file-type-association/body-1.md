>>>>> lang=en
## File-type association

Sublime Text loads XML `.tmLanguage` files (or `.sublime-syntax` files),
not the shared `.tmLanguage.json` directly. With Node.js installed, run
this dependency-free conversion from the editor repository root:

```sh
node grammars/scripts/export-tmlanguage.js grammars/ktav.tmLanguage
```

The command reads the canonical `grammars/ktav.tmLanguage.json` and writes
`grammars/ktav.tmLanguage`, an XML TextMate syntax named **Ktav** with base
scope `source.ktav` and the `ktav` file extension. Copy that generated
`.tmLanguage` file into your Sublime **Packages/User** directory (use
**Preferences → Browse Packages…**):

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Open a `.ktav` file, then **View → Syntax → Open all with current
extension as…** and pick **Ktav**. The LSP `selector` above matches the
view's base scope `source.ktav`, not its file extension. **Plain Text**
uses `text.plain` and does not match that selector. Re-export and replace
the XML syntax when updating the canonical grammar.

>>>>> lang=ru
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

>>>>> lang=zh
## 文件类型关联

Sublime Text 加载 XML `.tmLanguage` 文件（或 `.sublime-syntax` 文件），
不能直接加载共享的 `.tmLanguage.json`。安装 Node.js 后，在 editor
仓库根目录执行以下无需额外依赖的转换命令：

```sh
node grammars/scripts/export-tmlanguage.js grammars/ktav.tmLanguage
```

该命令读取规范源文件 `grammars/ktav.tmLanguage.json`，生成
`grammars/ktav.tmLanguage`：名称为 **Ktav**、基础 scope 为 `source.ktav`、
扩展名为 `ktav` 的 XML TextMate 语法。将生成的 `.tmLanguage` 文件复制到
Sublime 的 **Packages/User** 目录（通过 **Preferences → Browse Packages…**
打开）：

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

打开 `.ktav` 文件，然后在 **View → Syntax → Open all with current
extension as…** 中选择 **Ktav**。上面的 LSP `selector` 匹配编辑视图的
基础 scope `source.ktav`，而不是文件扩展名。**Plain Text** 使用
`text.plain`，不匹配此 selector。更新规范源语法后，请重新导出并替换
XML 语法文件。

