>>>>> lang=en
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

>>>>> lang=ru
## Требования для настройки

Стандартному Zed нужно установленное расширение (опубликованное или
development), регистрирующее язык `Ktav`, расширение `.ktav` и адаптер
language server `ktav-lsp`. Этот репозиторий такое расширение не
поставляет. Одних settings недостаточно для регистрации языка или
подключения сервера к Plain Text. После установки такого расширения
зарегистрированные им имена можно настроить в `.zed/settings.json`:

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

Установите сервер (для locked-сборки 0.8.0 нужен Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Используйте имена языка и адаптера, зарегистрированные расширением.
Для подсветки дополнительно нужны tree-sitter-грамматика Ktav и
запросы; общая TextMate JSON-грамматика не является tree-sitter.
См. [языковые расширения Zed](https://zed.dev/docs/extensions/languages)
и [настройку language server](https://zed.dev/docs/configuring-languages#configuring-language-servers).

>>>>> lang=zh
## 配置前提

标准 Zed 需要安装扩展(已发布或开发扩展),注册 `Ktav` 语言、
`.ktav` 后缀和 `ktav-lsp` language-server 适配器。本仓库不提供该
扩展。仅靠 settings 无法注册语言或将此服务器连接到 Plain Text。
安装这样的扩展后,可在 `.zed/settings.json` 中配置它注册的名称:

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

安装服务器(锁定的 0.8.0 构建需要 Rust 1.88+):

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

请使用扩展注册的语言和适配器名称。语法高亮还需要 Ktav
tree-sitter 语法和查询;共享的 TextMate JSON 不是 tree-sitter 语法。
参见 [Zed 语言扩展](https://zed.dev/docs/extensions/languages)与
[语言服务器配置](https://zed.dev/docs/configuring-languages#configuring-language-servers)。

