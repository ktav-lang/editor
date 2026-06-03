# Ktav for Visual Studio Code

[![VS Code Marketplace](https://img.shields.io/badge/VS%20Code-Marketplace-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=ktav-lang.ktav)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Registry-c160ef)](https://open-vsx.org/extension/ktav-lang/ktav)

Syntax highlighting and language support for the [Ktav](https://github.com/ktav-lang/spec) configuration format inside Visual Studio Code.

## Features

- Syntax highlighting for `.ktav` files — keys, scalars, `::` literal strings, multi-line blocks, inline/block compounds, comments
- Bracket matching and auto-closing for `{}`, `[]`, `()`
- Comment toggle with `#`
- Auto-indent inside object / array / parenthesised compounds

With the [`ktav-lsp`](https://github.com/ktav-lang/editor/tree/main/lsp) language server:

- Diagnostics for parse errors and type mismatches
- Semantic highlighting and a document outline (keys & nesting)
- Hover info for scalar types

## Language server

The extension talks to the `ktav-lsp` binary over stdio. Prebuilt binaries for
Linux, macOS and Windows are attached to every
[GitHub release](https://github.com/ktav-lang/editor/releases) — download the one
for your platform and point `ktav.server.path` at it, or drop it on your `PATH`.
To build from source instead, run `cargo build --release` in the
[`lsp/`](https://github.com/ktav-lang/editor/tree/main/lsp) directory.

### Discovery order

When activating, the extension looks for the server in this order:

1. **Explicit setting** — `ktav.server.path` (absolute path).
2. **Bundled binary** — `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]` (when the build ships one).
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
# Ktav — no quotes, no commas, no indentation traps.
service: socks5-rotator
port: 20082          # bare scalars are auto-typed: int / float / bool / null
debug: true

# Dotted keys are a flat alternative to nesting.
node.host: a.example
node.port: 1080

# '::' forces a literal string — keeps "8080" a string, not a number.
node.token:: 8080

# Need a literal '.' or ':' inside a key? Escape it (new in 0.6.0).
metric.http\.requests: 42

# Comma-free arrays and inline objects.
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

# Multi-line strings.
motd: (
    Welcome to the node.
    Please behave.
)
```

## Ecosystem — official bindings

Ktav is one Rust core wrapped by thin bindings in every language — identical
behaviour, native speed, and WebAssembly in the browser:

| Language        | Install                              | Repository |
|-----------------|--------------------------------------|------------|
| Rust            | `cargo add ktav`                     | [ktav-lang/rust](https://github.com/ktav-lang/rust) |
| JavaScript / TS | `npm i @ktav-lang/ktav`              | [ktav-lang/js](https://github.com/ktav-lang/js) |
| Python          | `pip install ktav`                   | [ktav-lang/python](https://github.com/ktav-lang/python) |
| Go              | `go get github.com/ktav-lang/golang` | [ktav-lang/golang](https://github.com/ktav-lang/golang) |
| PHP             | `composer require ktav-lang/ktav`    | [ktav-lang/php](https://github.com/ktav-lang/php) |
| Java / JVM      | `io.github.ktav-lang:ktav:0.6.0`     | [ktav-lang/java](https://github.com/ktav-lang/java) |
| C# / .NET       | `dotnet add package Ktav`            | [ktav-lang/csharp](https://github.com/ktav-lang/csharp) |

## Resources

- [Ktav specification](https://github.com/ktav-lang/spec)
- [In-browser converter](https://ktav-lang.github.io/) — JSON / YAML / TOML / INI ⇄ Ktav
- [tree-sitter grammar](https://github.com/ktav-lang/tree-sitter-ktav)
- [All repositories](https://github.com/ktav-lang)
- [Issue tracker](https://github.com/ktav-lang/editor/issues)

## License

MIT OR Apache-2.0. See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE).
