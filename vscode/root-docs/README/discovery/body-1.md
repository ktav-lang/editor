>>>>> lang=en
### Discovery order

When activating, the extension looks for the server in this order:

1. **Explicit setting** — `ktav.server.path` (absolute path).
2. **Bundled binary** — `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]` (when the build ships one).
3. **PATH** — falls back to spawning `ktav-lsp` and letting the OS resolve it.

If none of the above succeed, an error toast is shown and a `Ktav Language Server` output channel records the failure.

>>>>> lang=ru
### Порядок поиска

При активации расширение ищет сервер в следующем порядке:

1. **Явная настройка** — `ktav.server.path` (абсолютный путь).
2. **Встроенный бинарник** — `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]` (если он входит в сборку).
3. **PATH** — запускается `ktav-lsp`, и его поиск остаётся за операционной системой.

Если ни один из вариантов не сработал, показывается всплывающее уведомление об ошибке, а канал вывода `Ktav Language Server` фиксирует сбой.

>>>>> lang=zh
### 查找顺序

激活时,扩展按以下顺序查找服务器:

1. **显式设置** —— `ktav.server.path`(绝对路径)。
2. **内置二进制** —— `<extension>/bin/<platform>-<arch>/ktav-lsp[.exe]`(当构建随附它时)。
3. **PATH** —— 回退为启动 `ktav-lsp`,交由操作系统解析。

如果以上均告失败,将弹出错误提示,并由 `Ktav Language Server` 输出通道记录此次失败。

