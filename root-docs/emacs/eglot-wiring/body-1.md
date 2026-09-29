>>>>> lang=en
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

>>>>> lang=ru
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

>>>>> lang=zh
## 接入 eglot

```elisp
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs
               '(ktav-mode . ("ktav-lsp"))))

(add-hook 'ktav-mode-hook #'eglot-ensure)
```

安装服务器:

```sh
cargo install ktav-lsp
```

打开 `.ktav` 文件后用 `M-x eglot` 验证 —— modeline 应显示
`[eglot:ktav]`。诊断信息通过 `flymake` 呈现;悬停用 `M-x eldoc`。

