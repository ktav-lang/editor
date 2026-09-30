>>>>> lang=en
### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Then `cargo install ktav-lsp --version 0.8.0 --locked`.

>>>>> lang=ru
### Helix

Добавьте в `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

Затем `cargo install ktav-lsp --version 0.8.0 --locked`.

>>>>> lang=zh
### Helix

在 `~/.config/helix/languages.toml` 中添加:

```toml
[[language]]
name = "ktav"
file-types = ["ktav"]
language-servers = ["ktav-lsp"]

[language-server.ktav-lsp]
command = "ktav-lsp"
```

然后执行 `cargo install ktav-lsp --version 0.8.0 --locked`。

