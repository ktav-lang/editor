>>>>> lang=en
**Possible solution**:
- Use `AppLifecycleListener.appStarting()` instead of `appFrameCreated()`
- Write to textmate.xml BEFORE the IDE reads it
- This requires precise timing and may be unstable

### 3. Use a Different Configuration Store

Instead of textmate.xml, write to:
- The project's `.idea/` directory (not compatible with global TextMate settings)
- A custom plugin configuration file (which the IDE will not read)

### 4. Bundled TextMate Bundles (Official Method)

**Requirements**:
- Register the bundle in bundled_plugins.txt
- This is possible only for official JetBrains plugins
- This method is unavailable to third-party developers

### 5. Wait for API Improvements in Future IDE Versions

**Status**: JetBrains might release a public extension point in 2026.x or later

>>>>> lang=ru
**Возможное решение**:
- Использовать `AppLifecycleListener.appStarting()` вместо `appFrameCreated()`
- Записывать textmate.xml ДО чтения IDE
- Нужна точная синхронизация; решение может быть нестабильным

### 3. Использовать другое хранилище конфигурации

Вместо textmate.xml писать в:
- `.idea/` проекта (несовместимо с глобальным TextMate)
- Собственную конфигурацию плагина (IDE её не прочитает)

### 4. Встроенные TextMate-бандлы (официальный способ)

**Требуется**:
- Зарегистрировать бандл в bundled_plugins.txt
- Это возможно только для официальных плагинов JetBrains
- Сторонним разработчикам способ недоступен

### 5. Дождаться улучшения API в будущих версиях IDE

**Состояние:** JetBrains может выпустить публичную точку расширения
в 2026.x или позже.

>>>>> lang=zh
**可能的解决办法**：
- 使用 `AppLifecycleListener.appStarting()`，而不是 `appFrameCreated()`
- 在 IDE 读取前写入 textmate.xml
- 需要精确控制时序，可能不稳定

### 3. 使用其他配置存储位置

替代 textmate.xml 的位置：
- 项目的 `.idea/` (不兼容全局 TextMate)
- 插件自有配置文件 (IDE 不会读取)

### 4. 内置 TextMate 包 (官方方式)

**要求**：
- 在 bundled_plugins.txt 中注册包
- 只有 JetBrains 官方插件可以这样做
- 第三方开发者无法使用

### 5. 等待未来 IDE 版本改善 API

**现状：**JetBrains 可能在 2026.x 或更晚版本发布公开扩展点。

