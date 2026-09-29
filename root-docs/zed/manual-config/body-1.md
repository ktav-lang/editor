>>>>> lang=en
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

>>>>> lang=ru
## Ручная настройка (сегодня)

Zed читает языковые настройки уровня workspace из
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

Установите сервер:

```sh
cargo install ktav-lsp
```

Пока опубликованное расширение не зарегистрирует `Ktav` как известный
язык, Zed будет считать `.ktav` обычным текстом — LSP всё равно
подключится, если файл открыт, но подсветки не будет.

>>>>> lang=zh
## 手动配置(目前)

Zed 从 `.zed/settings.json` 读取工作区级别的语言设置:

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

安装服务器:

```sh
cargo install ktav-lsp
```

在已发布的扩展把 `Ktav` 注册为已知语言之前,Zed 会把 `.ktav` 当作
纯文本 —— 如果文件已打开,LSP 仍会接入,但不会有高亮。

