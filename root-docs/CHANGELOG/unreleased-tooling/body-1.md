>>>>> lang=en
### Repository and release tooling

- IntelliJ CI builds the locked `ktav-lsp` for live formatting smoke; local-file
  fixtures apply the platform-specific WSL registry override only on Windows.
- The IntelliJ rebuild helper uses the current checkout and explicitly
  selected IDE/platform paths. It never kills processes or clears IDE caches,
  logs, unrelated plugins or archives. Python 3 stages ZIP installation and
  preserves Unix executable modes.
- Helix, Zed and Sublime guides now state the actual client prerequisites:
  tree-sitter highlighting for stock Helix, a registered language/server
  adapter for Zed, and XML TextMate syntax matching Sublime's `source.ktav`.
- Shared-grammar, Emacs and Neovim guidance now distinguishes TextMate JSON
  hosts from separate tree-sitter/font-lock integrations. Collisions with both
  canonical multiline closer forms cause `BothFormsRequired`, not preservation
  by string splitting; literal key dots require escaping or quoting.

>>>>> lang=ru
### Репозиторий и релизная оснастка

- IntelliJ CI собирает закреплённый `ktav-lsp` для живого smoke форматирования;
  fixtures локальных файлов применяют WSL registry override только в Windows.
- IntelliJ rebuild-helper использует текущий checkout и явно выбранные пути
  IDE/платформы. Он не завершает процессы и не удаляет кэши, логи, чужие
  плагины или архивы. Python 3 устанавливает ZIP через staging с сохранением
  Unix-прав исполнения.
- Инструкции Helix, Zed и Sublime описывают реальные требования клиентов:
  tree-sitter для подсветки стандартного Helix, зарегистрированный язык и
  адаптер сервера для Zed, XML TextMate со scope `source.ktav` для Sublime.
- Инструкции общей грамматики, Emacs и Neovim различают хосты TextMate JSON
  и отдельные tree-sitter/font-lock интеграции. Коллизии обоих closer у
  канонической multiline-формы дают `BothFormsRequired`, а не разделение
  строки; буквальные точки ключа требуют экранирования или кавычек.

>>>>> lang=zh
### 仓库与发布工具

- IntelliJ CI 构建锁定的 `ktav-lsp` 以运行真实格式化 smoke；
  本地文件 fixture 仅在 Windows 应用平台专用 WSL registry 覆盖。
- IntelliJ 重建 helper 使用当前 checkout 及显式选择的 IDE/平台路径，
  不终止进程或清理 IDE 缓存、日志、其他插件及归档。Python 3 通过 staging
  安装 ZIP 并保留 Unix 可执行权限。
- Helix、Zed 和 Sublime 指南明确实际前提：标准 Helix 的 tree-sitter 高亮、
  Zed 已注册的语言/服务器适配器，以及 Sublime 匹配 `source.ktav` 的 XML
  TextMate 语法。
- 共享语法、Emacs 和 Neovim 指南区分 TextMate JSON 宿主与独立的
  tree-sitter/font-lock 集成。规范多行形式的双闭合符冲突产生
  `BothFormsRequired`，而非拆分字符串；键中字面点需转义或引号。

