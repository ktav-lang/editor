# Sublime Text

**Languages:** [English](../../../sublime.md) · [Русский](../ru/sublime.md) · **简体中文**

Sublime Text 4 搭配 [`LSP`](https://packagecontrol.io/packages/LSP) 包。

## 安装

1. Package Control → Install Package → **LSP**
2. `cargo install ktav-lsp --version 0.8.0 --locked`

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

Sublime Text 加载 XML `.tmLanguage` 文件（或 `.sublime-syntax` 文件），
不能直接加载共享的 `.tmLanguage.json`。安装 Node.js 后，在 editor
仓库根目录执行以下无需额外依赖的转换命令：

```sh
node grammars/scripts/export-tmlanguage.js grammars/ktav.tmLanguage
```

该命令读取规范源文件 `grammars/ktav.tmLanguage.json`，生成
`grammars/ktav.tmLanguage`：名称为 **Ktav**、基础 scope 为 `source.ktav`、
扩展名为 `ktav` 的 XML TextMate 语法。将生成的 `.tmLanguage` 文件复制到
Sublime 的 **Packages/User** 目录（通过 **Preferences → Browse Packages…**
打开）：

- macOS: `~/Library/Application Support/Sublime Text/Packages/User/`
- Linux: `~/.config/sublime-text/Packages/User/`
- Windows: `%APPDATA%\Sublime Text\Packages\User\`

打开 `.ktav` 文件，然后在 **View → Syntax → Open all with current
extension as…** 中选择 **Ktav**。上面的 LSP `selector` 匹配编辑视图的
基础 scope `source.ktav`，而不是文件扩展名。**Plain Text** 使用
`text.plain`，不匹配此 selector。更新规范源语法后，请重新导出并替换
XML 语法文件。

## 验证

使用 **Ktav** 语法打开 `.ktav` 文件。**Tools → Developer → Show Scope Name**
应显示基础 scope `source.ktav`。语言服务器启动时，状态栏会显示 `ktav-lsp`。
若没有启动，请聚焦该文件，在 Command Palette 中运行 **LSP: Troubleshoot
server**，然后选择 `ktav-lsp`。参见 [LSP 排障指南](https://lsp.sublimetext.io/troubleshooting/)。
