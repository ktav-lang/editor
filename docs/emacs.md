# Emacs

With [`eglot`](https://joaotavora.github.io/eglot/) (built-in since
Emacs 29).

## Major mode stub

Drop this into your `init.el` (or a file on `load-path`):

```elisp
(define-derived-mode ktav-mode prog-mode "Ktav"
  "Major mode for editing Ktav configuration files."
  (setq-local comment-start "# ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "#+\\s-*"))

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
cargo install ktav-lsp
```

Verify with `M-x eglot` after opening a `.ktav` file — the
modeline should show `[eglot:ktav]`. Diagnostics appear via
`flymake`; hover with `M-x eldoc`.

## Highlighting

`ktav-mode` above is intentionally minimal (no font-lock keywords).
The LSP's semantic-tokens response gives most of the colouring once
eglot is attached. For richer offline highlighting you can run the
shared TextMate grammar from `editor/grammars/` through `tree-sitter`
or `polymode`, but that's beyond the scope of this stub.
