# Zed

> **Status:** TBD. A first-party Zed extension is on the roadmap but
> not yet published. The notes below outline what an extension stub
> would look like — contributions welcome.

## Manual configuration (today)

Zed reads workspace-level language settings from
`.zed/settings.json`:

```jsonc
{
  "languages": {
    "Ktav": {
      "tab_size": 2,
      "language_servers": ["ktav-lsp"]
    }
  },
  "lsp": {
    "ktav-lsp": {
      "binary": { "path": "ktav-lsp" }
    }
  }
}
```

Install the server:

```sh
cargo install ktav-lsp
```

Until a published extension registers `Ktav` as a known language,
Zed will treat `.ktav` as plain text — the LSP will still attach if
the file has been opened, but highlighting will be off.

## Planned extension layout

```
zed-ktav/
  extension.toml          # id, name, languages.ktav
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # tree-sitter highlight queries (TBD)
  grammars/
    ktav.toml             # tree-sitter grammar source pin
```

Tracking issue: <https://github.com/ktav-lang/editor/issues>
