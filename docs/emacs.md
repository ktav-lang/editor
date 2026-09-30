# Emacs

**Languages:** **English** · [Русский](i18n/editors/ru/emacs.md) · [简体中文](i18n/editors/zh/emacs.md)

With [`eglot`](https://joaotavora.github.io/eglot/) (built-in since
Emacs 29).

## Major mode stub

Drop this into your `init.el` (or a file on `load-path`):

```elisp
(define-derived-mode ktav-mode prog-mode "Ktav"
  "Major mode for editing Ktav configuration files."
  (setq-local comment-start "## ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "##+\\s-*"))

(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
```

## eglot wiring

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))

(add-hook 'ktav-mode-hook #'eglot-ensure)
```

Install the server:

```sh
cargo install ktav-lsp --version 0.8.0 --locked
```

Verify with `M-x eglot` after opening a `.ktav` file — the
modeline should show `[eglot:ktav]`. Diagnostics appear via
`flymake`; hover with `M-x eldoc`.

## Highlighting

`ktav-mode` above is intentionally minimal (no font-lock keywords).
The server offers semantic tokens, but displaying them depends on the
installed Emacs LSP client's support and configuration; attaching eglot
does not add font-lock rules to this mode.
Offline highlighting needs separately implemented Ktav font-lock rules,
or a Ktav tree-sitter grammar, highlight queries and a major mode that
uses them. This repository supplies none of those highlighting
integrations. The shared TextMate JSON is not a tree-sitter grammar;
neither tree-sitter nor polymode loads it as one.
