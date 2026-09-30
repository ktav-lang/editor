# ktav-lsp

> Реализация Language Server Protocol для конфигурационного формата
> [Ktav](https://github.com/ktav-lang/spec). Один Rust-бинарь; тонкая
> обёртка над парсерным crate `ktav`.

**Languages:** [English](../README.md) · **Русский** · [简体中文](README.zh.md)

---

## Что это

`ktav-lsp` — это LSP-сервер. Редакторы общаются с ним по JSON-RPC через
stdin/stdout и получают диагностики, hover, автодополнение, символы
документа и semantic tokens для файлов `.ktav`. Это тонкая обёртка над
crate [`ktav`](https://crates.io/crates/ktav) — тем же парсером, который
используют все остальные биндинги Ktav (PHP / JS / Python / Go / Java /
C#), — поэтому сообщения об ошибках и поведение совпадают точно.

## Установка

Для сборки исходников с текущим закреплённым графом зависимостей нужен
Rust 1.88+. CI собирает все targets на объявленном минимуме и сохраняет
проверки stable-toolchain. Установка без `--locked` может потребовать
более новый Rust при повышении требований транзитивных зависимостей.

```bash
cargo install ktav-lsp
```

Это положит бинарь `ktav-lsp` в `~/.cargo/bin/`. Конфигурационный файл
не требуется.

При обычной установке могут разрешиться более новые транзитивные зависимости.
После публикации 0.8.0 воспроизвести граф зависимостей этого выпуска можно так:

```bash
cargo install ktav-lsp --version 0.8.0 --locked
```

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

Это включает диагностику и hover от LSP, но не подсветку в стандартном
Helix. Для подсветки нужны отдельные tree-sitter-грамматика Ktav и
запросы, которых в репозитории нет; TextMate JSON их не заменяет.

### Neovim (with `nvim-lspconfig`)

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

Также понадобится сниппет `ftdetect`:

```vim
au BufRead,BufNewFile *.ktav set filetype=ktav
```

### VS Code

Используйте [расширение Ktav для VS Code](../../vscode) — оно содержит
конфигурацию языка, а для шести платформ, под которые CI собирает
бинарники, ещё и сам `ktav-lsp`, так что отдельная установка не
нужна. Устанавливайте
[`ktav-lsp`](https://crates.io/crates/ktav-lsp) сами только на
неподдерживаемой платформе или при сборке расширения из исходников.

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
  документ и выводит структурированный `ErrorKind` из `ktav`, привязанный
  к точному байтовому диапазону на нужной строке(ах).
- **Hover** — наведение на строку `key:` показывает выведенный тип и
  значение.
- **Автодополнение** — контекстно после разделителя `:`: предлагает
  `null`, `true`, `false`, открывающие скобки (`{`, `[`, `(`, `((`),
  пустые литералы (`{}`, `[]`, `()`) и маркеры значений (`:`, `::`).
- **Document symbols** — outline отражает дерево распарсенного объекта/массива;
  скаляры становятся Property/Number/String, объекты — Module, массивы —
  Array. Диапазоны объявлений включают значения, дочерние символы и закрывающие
  скобки (в том числе у многострочных строк), но не внешние пробелы.
  Навигация выделяет только исходный ключ или начало элемента. Повторно открытые
  точечные префиксы охватывают все определения, сохраняя первую точку навигации.
- **Semantic tokens** — типы токенов `comment`, `keyword`, `number`,
  `string`, `property`, `operator`, `null`. Редакторы могут использовать их
  вместо (или поверх) TextMate-грамматик для более точной подсветки,
  особенно вокруг точечных ключей и сырых значений `::`.
- **Форматирование** — `textDocument/formatting` канонически
  переотступает вложенность объектов/массивов/скобочных групп (4
  пробела на уровень), сохраняя пустые строки, комментарии и содержимое
  многострочных строковых блоков без изменений.

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

MIT OR Apache-2.0 — см. [LICENSE-MIT](../LICENSE-MIT) и
[LICENSE-APACHE](../LICENSE-APACHE).
