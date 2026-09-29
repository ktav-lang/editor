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
cargo install ktav-lsp
```

Проверьте `M-x eglot` после открытия файла `.ktav` — в modeline
должно появиться `[eglot:ktav]`. Диагностика приходит через
`flymake`; hover — `M-x eldoc`.

## Подсветка

`ktav-mode` выше намеренно минимален (без font-lock keywords).
Semantic-tokens ответ LSP даёт большую часть подсветки после
подключения eglot. Для более богатой офлайн-подсветки можно
пропустить общую TextMate-грамматику из `editor/grammars/` через
`tree-sitter` или `polymode`, но это выходит за рамки этой заглушки.
