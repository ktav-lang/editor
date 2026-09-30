# Emacs

**Languages:** [English](../../../emacs.md) · **Русский** · [简体中文](../zh/emacs.md)

С [`eglot`](https://joaotavora.github.io/eglot/) (встроен начиная с
Emacs 29).

## Заглушка major mode

Поместите это в `init.el` (или файл на `load-path`):

```elisp
(define-derived-mode ktav-mode prog-mode "Ktav"
  "Major mode for editing Ktav configuration files."
  (setq-local comment-start "## ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "##+\\s-*"))

(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
```

## Подключение eglot

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))

(add-hook 'ktav-mode-hook #'eglot-ensure)
```

Установите сервер:

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Проверьте `M-x eglot` после открытия файла `.ktav` — в modeline
должно появиться `[eglot:ktav]`. Диагностика приходит через
`flymake`; hover — `M-x eldoc`.

## Подсветка

`ktav-mode` выше намеренно минимален (без font-lock keywords).
Сервер предоставляет semantic tokens, но их отображение зависит от
поддержки и настройки установленного LSP-клиента Emacs; подключение
eglot не добавляет правила font-lock в этот mode.
Для офлайн-подсветки нужны отдельно реализованные правила font-lock для
Ktav либо tree-sitter-грамматика Ktav, запросы подсветки и major mode,
который их использует. В репозитории этих интеграций подсветки нет.
Общая TextMate JSON не является tree-sitter-грамматикой;
ни tree-sitter, ни polymode не загружают её как такую грамматику.
