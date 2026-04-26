# ktav-lsp

> Реализация Language Server Protocol для конфигурационного формата
> [Ktav](https://github.com/ktav-lang/spec). Один Rust-бинарь; тонкая
> обёртка над парсером из crate `ktav`.

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

---

## Что это

`ktav-lsp` — это LSP-сервер. Редакторы общаются с ним по JSON-RPC через
stdin/stdout и получают диагностики, hover, автодополнение, символы
документа и semantic tokens для файлов `.ktav`. Это тонкая обёртка над
crate [`ktav`](https://crates.io/crates/ktav) — тем же парсером, который
используют все остальные биндинги Ktav (PHP / JS / Python / Go / Java /
C#), — поэтому сообщения об ошибках и поведение совпадают точно.

## Установка

```bash
cargo install ktav-lsp
```

Это положит бинарь `ktav-lsp` в `~/.cargo/bin/`. Конфигурационный файл
не требуется.

## Настройка редактора

### Helix (`languages.toml`)

```toml
[language-server.ktav-lsp]
command = "ktav-lsp"

[[language]]
name = "ktav"
scope = "source.ktav"
file-types = ["ktav"]
roots = []
language-servers = ["ktav-lsp"]
```

### Neovim (с `nvim-lspconfig`)

`ktav-lsp` (пока) нет в реестре `lspconfig`, поэтому регистрируем
вручную:

```lua
local lspconfig = require("lspconfig")
local configs = require("lspconfig.configs")

if not configs.ktav_lsp then
  configs.ktav_lsp = {
    default_config = {
      cmd = { "ktav-lsp" },
      filetypes = { "ktav" },
      root_dir = lspconfig.util.find_git_ancestor,
      settings = {},
    },
  }
end

lspconfig.ktav_lsp.setup({})
```

И сниппет для определения файлового типа:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

### VS Code

Используйте [расширение Ktav для VS Code](../vscode) — оно содержит
конфигурацию языка и мост к `ktav-lsp`. Сам `ktav-lsp` устанавливается
отдельно через `cargo install`.

### Emacs (`eglot`)

```elisp
(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
(define-derived-mode ktav-mode prog-mode "Ktav")

(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))
```

## Возможности

- **Диагностики** — каждый `did_open` / `did_change` перепарсивает
  документ и выводит сообщения `ktav::Error::Syntax` на нужной строке.
- **Hover** — наведение на строку `key:` показывает выведенный тип и
  значение.
- **Автодополнение** — контекстно после разделителя `:`: предлагает
  `null`, `true`, `false`, открывающие скобки (`{`, `[`, `(`, `((`),
  пустые литералы (`{}`, `[]`, `()`) и типизированные маркеры (`:`,
  `:i`, `:f`).
- **Document symbols** — outline отражает дерево распарсенного объекта;
  скаляры становятся Property/Number/String, объекты — Module, массивы —
  Array.
- **Semantic tokens** — типы токенов `comment`, `keyword`, `number`,
  `string`, `property`, `operator`. Редакторы могут использовать их
  вместо (или поверх) TextMate-грамматик для более точной подсветки,
  особенно вокруг точечных ключей и типизированных маркеров.

## Архитектура

Один Rust-crate, один бинарь. Стек:

- [`tower-lsp`](https://crates.io/crates/tower-lsp) — JSON-RPC и
  обработка возможностей сервера.
- [`tokio`](https://tokio.rs/) — runtime, читает из stdin, пишет в stdout.
- [`ktav`](https://crates.io/crates/ktav) — тот же парсерный crate, что
  использует каждый биндинг Ktav. Диагностики, hover, document symbols —
  всё через `ktav::parse`.
- [`dashmap`](https://crates.io/crates/dashmap) — потокобезопасное
  хранилище документов по `Url`. `TextDocumentSyncKind::FULL` упрощает
  цикл: небольшие конфиг-файлы перепарсятся настолько быстро, что
  инкрементальная синхронизация прибавит кода, не сэкономив время.

Логи идут в stderr (stdout зарезервирован за LSP-трафиком). Уровень
логирования: `KTAV_LSP_LOG=debug`.

## Лицензия

MIT — см. [LICENSE](LICENSE).
