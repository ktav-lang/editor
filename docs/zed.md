# Zed

**Languages:** **English** · [Русский](i18n/editors/ru/zed.md) · [简体中文](i18n/editors/zh/zed.md)

> **Status:** TBD. A first-party Zed extension is on the roadmap but
> not yet published. The notes below outline what an extension stub
> would look like — contributions welcome.

## Configuration prerequisites

Stock Zed requires an installed extension (published or development)
that registers the `Ktav` language, `.ktav` suffix, and `ktav-lsp`
language-server adapter. This repository does not ship that extension.
Settings alone cannot register a language or attach this server to
Plain Text. Once such an extension is installed, its registered names
can be configured in `.zed/settings.json`, for example:

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

Install the server (Rust 1.88+ for the locked 0.8.0 build):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Use the language and adapter names registered by your extension.
Syntax highlighting additionally requires Ktav tree-sitter grammar
and queries; the shared TextMate JSON is not a tree-sitter grammar.
See [Zed language extensions](https://zed.dev/docs/extensions/languages)
and [language-server configuration](https://zed.dev/docs/configuring-languages#configuring-language-servers).

## Planned extension layout

```
zed-ktav/
  extension.toml          # extension metadata, grammar pin, language_servers adapter
  languages/ktav/
    config.toml           # name, path_suffixes = ["ktav"], comment chars
    highlights.scm        # queries for a separately implemented tree-sitter grammar
  src/lib.rs              # Zed extension API: ktav-lsp command adapter
```

Tracking issue: <https://github.com/ktav-lang/editor/issues>
