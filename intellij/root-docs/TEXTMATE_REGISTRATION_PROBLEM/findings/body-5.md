>>>>> lang=en
In WebStorm 2025.3, the configuration is stored at `~\AppData\Roaming\JetBrains\WebStorm2025.3\options\textmate.xml`:

```xml
<application>
  <component name="TextMateUserBundlesSettings">
    <![CDATA[{}]]>
  </component>
</application>
```

**How it works**:
- `TextMateUserBundlesSettings` is an AppState component
- Its contents are JSON wrapped in CDATA
- Manual bundle registration updates and rewrites the JSON
- The IDE reads the JSON at startup and calls `TextMateUserBundlesSettings.deserialize()`

**Problem**: The exact JSON schema expected by IDE version 2025.3 is unknown

## Why This Is Difficult

1. **Internal TextMate API** - the methods used are not part of the public IntelliJ SDK
2. **API changes between versions** - methods and signatures differ between 2024.x and 2025.x
3. **No documentation** - JetBrains does not document the TextMate plugin's internals
4. **Timing problems** - plugin initialization and IDE state require precise synchronization
5. **No extension point** - bundles cannot be declared in plugin.xml
6. **Version differences** - different IDE versions have different internal APIs

>>>>> lang=ru
В WebStorm 2025.3 конфигурация хранится в
`~\AppData\Roaming\JetBrains\WebStorm2025.3\options\textmate.xml`:

```xml
<application>
  <component name="TextMateUserBundlesSettings">
    <![CDATA[{}]]>
  </component>
</application>
```

**Как это работает**:
- `TextMateUserBundlesSettings` — компонент AppState
- Содержимое — JSON внутри CDATA
- При ручной регистрации JSON изменяется и записывается заново
- IDE читает JSON при загрузке и вызывает `TextMateUserBundlesSettings.deserialize()`

**Проблема:** точная схема JSON для IDE версии 2025.3 неизвестна.

## Почему это сложно

1. **Внутренний TextMate API** — методы не входят в публичный IntelliJ SDK.
2. **Изменение API** — сигнатуры и методы различаются между 2024.x и 2025.x.
3. **Нет документации** — JetBrains не описывает внутреннее устройство TextMate-плагина.
4. **Проблема времени** — нужна точная синхронизация инициализации плагина и состояния IDE.
5. **Нет точки расширения** — нельзя декларативно определить бандлы в plugin.xml.
6. **Различия версий** — внутренние API разных версий IDE отличаются.

>>>>> lang=zh
WebStorm 2025.3 的配置位于
`~\AppData\Roaming\JetBrains\WebStorm2025.3\options\textmate.xml`：

```xml
<application>
  <component name="TextMateUserBundlesSettings">
    <![CDATA[{}]]>
  </component>
</application>
```

**工作方式**：
- `TextMateUserBundlesSettings` 是 AppState 组件
- 内容是 CDATA 包裹的 JSON
- 手动注册包后，JSON 会被更新并重写
- IDE 在加载时读取 JSON 并调用 `TextMateUserBundlesSettings.deserialize()`

**问题：**我们不知道 2025.3 版 IDE 所要求的确切 JSON 模式。

## 为何如此困难

1. **TextMate API 属于内部接口**：这些方法不是公开 IntelliJ SDK 的一部分。
2. **API 随版本变化**：2024.x 与 2025.x 的方法和签名不同。
3. **没有文档**：JetBrains 未记录 TextMate 插件的内部工作机制。
4. **时序问题**：插件初始化必须与 IDE 状态精确同步。
5. **没有扩展点**：无法在 plugin.xml 中声明式定义包。
6. **版本差异**：不同 IDE 版本具有不同内部 API。

