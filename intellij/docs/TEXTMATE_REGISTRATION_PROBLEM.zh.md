# IntelliJ 插件中 TextMate 包自动注册的问题

> **历史文档。** 本文记录一种已被否决的方案：运行时自动注册内置的
> TextMate 语法。从 0.2.0 (2026-05-07) 起，插件改用原生
> `KtavLexer` / `KtavSyntaxHighlighterFactory`，通过
> `KtavParserDefinition` 接入语法高亮；不再使用 TextMate 包、
> `KtavTextMateLoader`，也不需要手动注册。保留本文是为了解释
> 为何放弃原方案。下文所谓“当前实现”指 0.2.0 之前的尝试，
> 而非现已发布的插件。

## 概述

开发 Ktav IntelliJ 插件时，遇到了自动注册内置 TextMate 语法的问题。
IntelliJ 的 TextMate API 有重大限制，使程序化注册无法实现或不稳定。

## 我们试图实现什么

目标是在安装插件后自动为 `.ktav` 文件启用语法高亮，**无需**
用户通过 IDE 设置手动注册。

## 方案架构

### 已否决尝试的实现 (KtavTextMateLoader.kt，0.2.0 之前)

```
1. appFrameCreated 钩子 (IDE 启动)
   ├─ 检查文件系统中的包 (开发模式)
   ├─ 若未找到，从插件 JAR 中解压
   └─ 尝试向 TextMate 注册

2. projectOpened 钩子 (项目加载)
   └─ 再次尝试注册 (支持动态加载插件)

3. 解压包
   ├─ 在 lib/ 找到 ktav-intellij-*.jar (排除 searchableOptions)
   ├─ 将 grammars/ktav/ 解压到临时目录
   └─ 建立结构正确的 .tmbundle 目录

4. 注册尝试
   ├─ 尝试 1：更新 textmate.xml (TextMateUserBundlesSettings)
   ├─ 尝试 2：调用 TextMateService.readBundle()
   └─ 尝试 3：调用 reloadEnabledBundles()
```

## 已证实可工作的部分 ✓

1. **从 JAR 解压包**：成功生成 `.tmbundle` 格式。
2. **JAR 过滤**：正确排除 searchableOptions.jar。
3. **生命周期钩子**：两个钩子均被正确触发。
4. **反射调用 API**：成功调用 TextMateService 方法。
5. **reloadEnabledBundles()**：执行时没有报错。

## 未能工作的部分 ✗

### 1. TextMateService.readBundle() 返回 null

```kotlin
val readBundleMethod = serviceCls.getMethod("readBundle", Path::class.java)
val bundle = readBundleMethod.invoke(service, bundlePath)
// → 结果：null ❌
```

**原因**：包的格式不符合 TextMate API 的预期。可能的原因：
- 缺少必需的文件，例如 `info.plist` 或 `menu.plist`
- 目录结构不正确
- API 期望另一种包格式

### 2. 更新 textmate.xml 成功，但设置没有生效

尝试 1：用简单键值映射表示 JSON：
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**结果：**`XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`。

尝试 2：用对象数组表示 JSON：
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**结果：**反序列化未成功，也没有记录错误。

### 3. 注册时 textmate.xml 尚不可用

- 插件在 **appFrameCreated** 时初始化，时间很早。
- IDE 稍后才创建 `textmate.xml`。
- 因此查找该文件失败。
- 即使重写文件，IDE 也可能不会重新读取。

### 4. TextMate API 极其有限

WebStorm 2025.3 的 `TextMateService` 可用方法：
```
- readBundle(Path)              → Bundle 对象或 null
- reloadEnabledBundles()        → void (重载已启用的包)
- getFileNameMatcherToScopeNameMapping()
- getLanguageDescriptorByExtension(String)
- getLanguageDescriptorByFileName(String)
- getShellVariableRegistry()
- getSnippetRegistry()
- getPreferenceRegistry()
```

**缺少**：
- `registerEnabledBundle()` — 2025.3 中不存在
- `enableBundle()` — 不存在
- `registerBundle()` — 不存在
- 将包加入“已启用”列表的公开接口

### 5. TextMateBundleProvider 扩展点不存在 (或是内部接口)

在 plugin.xml 中尝试使用扩展点：
```xml
<textmate.bundleProvider
  implementation="lang.ktav.KtavTextMateBundleProvider" />
```

**结果：**`Unresolved reference 'TextMateBundleProvider'`；
该类未通过公开 API 导出。

## 对其他插件的调查

### WDL IDE Plugin (Broad Institute)

其实现仅调用：
```java
TextMateService.getInstance().registerEnabledBundles(false);
```

**问题：**`registerEnabledBundles()` 只会重载已有包，并不会注册
新包。尚不清楚它们如何把包加入“已启用”列表。

### 其他插件

大多数插件使用 TextMate 包的方式包括：
1. 使用 bundled_plugins.txt (JetBrains 官方插件)
2. 显式复制到已知目录
3. 不提供自动注册，要求用户手动添加

## TextMate 包结构

解压后的包结构如下：
```
Ktav.tmbundle/
├── language-configuration.json
└── Syntaxes/
    └── ktav.tmLanguage.json
```

这符合标准 VS Code TextMate 包格式，但 IDE 可能要求额外文件
或不同的结构。

## TextMateUserBundlesSettings 结构

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

## 可选解决方案

### 1. 手动注册并提供良好体验 ✓ (当时推荐)

**优点**：
- 可靠工作
- 不依赖内部 API
- 兼容所有 IDE 版本

**实现**：
- 自动把包解压到临时目录
- 插件提示用户复制路径
- 或者：菜单操作“Register Ktav TextMate Bundle”打开 Settings → TextMate Bundles
- 也可以提供详细的分步说明

### 2. 在 IDE 加载前写入 textmate.xml

**难点**：IDE 在插件加载前很早就初始化该设置。

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

## 建议

**采用方案 1 (手动注册并提供良好体验)。**这是改用原生词法分析器
之前的历史建议，并非当前插件的使用说明。

当时意味着：
1. ✓ 保留当时正常工作的包解压实现
2. ✓ 记录解压后包的路径
3. ✓ 为用户创建 IDE 操作或通知
4. ✓ 编写详细文档
5. ✓ 若 JetBrains 将来发布公开 API，再改用自动注册

## 已否决方案的代码 (0.2.0 之前的历史状态)

**文件** (当前插件已无这些文件，改由原生词法分析器实现)：
- `src/main/kotlin/lang/ktav/KtavTextMateLoader.kt`：核心逻辑
- `src/main/kotlin/lang/ktav/KtavProjectActivity.kt`：项目生命周期钩子
- `src/main/resources/META-INF/plugin.xml`：插件配置

**放弃方案时的状态**：
- 包解压：✓ 正常工作
- 设置更新：⚠️ 已执行但未生效
- 自动注册：✗ 无法可靠实现

从 0.2.0 起，`KtavParserDefinition` 与
`KtavSyntaxHighlighterFactory` (原生 `KtavLexer`) 提供语法高亮，
参见 `intellij/src/main/resources/META-INF/plugin.xml`。

## 参考资料

- [IntelliJ TextMate 插件源代码](https://github.com/JetBrains/intellij-community/tree/master/plugins/textmate)
- [TextMate 包格式](https://macromates.com/manual/en/bundles)
- [WDL IDE 插件](https://github.com/broadinstitute/wdl-ide)
- IntelliJ API：`org.jetbrains.plugins.textmate.TextMateService`
