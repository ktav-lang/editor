# Sublime Text

**Languages:** **English** · [Русский](i18n/editors/ru/sublime.md) · [简体中文](i18n/editors/zh/sublime.md)

Sublime Text 4 with the [`LSP`](https://packagecontrol.io/packages/LSP)
package.

## Install

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp --version 0.8.0 --locked`

## Configure

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

## Verify

Open a `.ktav` file using the **Ktav** syntax. **Tools → Developer → Show
Scope Name** should show `source.ktav` as the base scope. When the language
server starts, `ktav-lsp` appears in the status bar. If it does not start,
focus that file and run **LSP: Troubleshoot server** from the Command
Palette, then select `ktav-lsp`. See the [LSP troubleshooting guide](https://lsp.sublimetext.io/troubleshooting/).
