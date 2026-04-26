# Ktav for Visual Studio Code

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=ktav-lang.ktav)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Registry-c160ef)](https://open-vsx.org/extension/ktav-lang/ktav)

Syntax highlighting and language support for the [Ktav](https://github.com/ktav-lang/spec) configuration format inside Visual Studio Code.

## Features

- Syntax highlighting for `.ktav` files (keys, scalars, tagged scalars, multiline blocks, compounds, comments)
- Bracket matching and auto-closing for `{}`, `[]`, `()`
- Comment toggle with `#`
- Auto-indent inside object / array / parenthesised compounds

Coming soon (via the upcoming `ktav-lsp` integration):

- Diagnostics for parse errors and tag/type mismatches
- Hover info for tags and scalar types
- Completion for tag names and known keys

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
