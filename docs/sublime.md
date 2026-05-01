# Sublime Text

Sublime Text 4 with the [`LSP`](https://packagecontrol.io/packages/LSP)
package.

## Install

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp`

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

## Verify

Open a `.ktav` file → `Tools → LSP → Show diagnostics` — the
`ktav-lsp` client should appear as connected.
