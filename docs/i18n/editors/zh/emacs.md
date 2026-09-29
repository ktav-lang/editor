# Emacs

**Languages:** [English](../../../emacs.md) · [Русский](../ru/emacs.md) · **简体中文**

搭配 [`eglot`](https://joaotavora.github.io/eglot/)(Emacs 29 起内置)。

## Major mode 骨架

将以下内容放入 `init.el`(或 `load-path` 上的某个文件):

```elisp
(define-derived-mode ktav-mode prog-mode "Ktav"
  "Major mode for editing Ktav configuration files."
  (setq-local comment-start "## ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "##+\\s-*"))

(add-to-list 'auto-mode-alist '("\\.ktav\\'" . ktav-mode))
```

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

## 高亮

上面的 `ktav-mode` 有意保持极简(没有 font-lock 关键字)。接入
eglot 后,LSP 的 semantic-tokens 响应即可提供大部分高亮。若想要更
丰富的离线高亮,可以将 `editor/grammars/` 中的共享 TextMate 语法通过
`tree-sitter` 或 `polymode` 处理,但这超出了本骨架的范围。
