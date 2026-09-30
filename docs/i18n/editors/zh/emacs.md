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
cargo install ktav-lsp --version 0.8.0 --locked
```

打开 `.ktav` 文件后用 `M-x eglot` 验证 —— modeline 应显示
`[eglot:ktav]`。诊断信息通过 `flymake` 呈现;悬停用 `M-x eldoc`。

## 高亮

上面的 `ktav-mode` 有意保持极简(没有 font-lock 关键字)。
服务器提供语义令牌,但显示它们取决于已安装的 Emacs LSP 客户端的支持
和配置;接入 eglot 不会为此 mode 添加 font-lock 规则。
离线高亮需要另行实现 Ktav font-lock 规则,或 Ktav tree-sitter 语法、
高亮查询和使用它们的 major mode。本仓库不提供这些高亮集成。
共享的 TextMate JSON 不是 tree-sitter 语法;
tree-sitter 和 polymode 都不能将其作为此类语法加载。
