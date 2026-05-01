# Ktav for Visual Studio Code

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=ktav-lang.ktav)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Registry-c160ef)](https://open-vsx.org/extension/ktav-lang/ktav)

Syntax highlighting and language support for the [Ktav](https://github.com/ktav-lang/spec) configuration format inside Visual Studio Code.

## Features

- Syntax highlighting for `.ktav` files (keys, scalars, tagged scalars, multiline blocks, compounds, comments)
- Bracket matching and auto-closing for `{}`, `[]`, `()`
- Comment toggle with `#`
- Auto-indent inside object / array / parenthesised compounds

Powered by [`ktav-lsp`](https://crates.io/crates/ktav-lsp) when installed:

- Diagnostics for parse errors and tag/type mismatches
- Hover info for tags and scalar types
- Completion for tag names and known keys

## Language server

The extension talks to the `ktav-lsp` binary over stdio. Install it once with:

```
cargo install ktav-lsp
```

### Discovery order

When activating, the extension looks for the server in this order:

1. **Explicit setting** — `ktav.server.path` (absolute path).
2. **Bundled binary** — `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]` (reserved for future prebuilt releases; not bundled today).
3. **PATH** — falls back to spawning `ktav-lsp` and letting the OS resolve it.

If none of the above succeed, an error toast is shown and a `Ktav Language Server` output channel records the failure.

### Settings

| Setting              | Type                                | Default | Description                                                              |
|----------------------|-------------------------------------|---------|--------------------------------------------------------------------------|
| `ktav.server.path`   | string                              | `""`    | Absolute path to `ktav-lsp`. Empty means auto-discovery (see above).     |
| `ktav.trace.server`  | `"off"` \| `"messages"` \| `"verbose"` | `"off"` | Traces JSON-RPC traffic to the output channel.                           |

## Installation

From the Visual Studio Code Marketplace:

```
ext install ktav-lang.ktav
```

Or open the Extensions sidebar (`Ctrl+Shift+X` / `Cmd+Shift+X`) and search for **Ktav**.

The extension is also published to [Open VSX](https://open-vsx.org/) for VSCodium and other compatible editors.

## Example

```ktav
# A small Ktav config — gets fully highlighted in VS Code
name: my-service
port: $i 8080
debug: $b true
tags: [ web, api, $s "rest+json" ]
endpoints: {
  health: /healthz
  ready:  /readyz
}
```

## Resources

- [Ktav specification](https://github.com/ktav-lang/spec)
- [Other implementations and editor integrations](https://github.com/ktav-lang)
- [Issue tracker](https://github.com/ktav-lang/editor/issues)

## License

[MIT](./LICENSE)
