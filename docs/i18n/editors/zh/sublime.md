# Sublime Text

**Languages:** [English](../../../sublime.md) · [Русский](../ru/sublime.md) · **简体中文**

Sublime Text 4 搭配 [`LSP`](https://packagecontrol.io/packages/LSP) 包。

## 安装

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp`

## 配置

Preferences → Package Settings → LSP → Settings:

```jsonc
{
  "clients": {
    "ktav-lsp": {
      "enabled": true,
      "command": ["ktav-lsp"],
      "selector": "source.ktav"
    }
  }
}
```

## 文件类型关联

保存一个 `.ktav` 文件,然后 **View → Syntax → Open all with current
extension as…**,选择 *Plain Text*(或任意基础语法 —— 一旦配置好,
LSP 会通过上面的 `selector` 按文件扩展名接入)。

要获得真正的高亮,把共享 TextMate 语法
`editor/grammars/ktav.tmLanguage.json` 放到:

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

Sublime 会自动加载 `Packages/User/` 中的 `.tmLanguage.json` 文件。

## 验证

打开一个 `.ktav` 文件 → `Tools → LSP → Show diagnostics` —— `ktav-lsp`
客户端应显示为已连接。
