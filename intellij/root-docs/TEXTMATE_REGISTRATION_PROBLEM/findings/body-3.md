>>>>> lang=en
Available `TextMateService` methods in WebStorm 2025.3:
```
- readBundle(Path)              → Bundle object or null
- reloadEnabledBundles()        → void (reloads already enabled bundles)
- getFileNameMatcherToScopeNameMapping()
- getLanguageDescriptorByExtension(String)
- getLanguageDescriptorByFileName(String)
- getShellVariableRegistry()
- getSnippetRegistry()
- getPreferenceRegistry()
```

**Missing**:
- `registerEnabledBundle()` - does not exist in 2025.3
- `enableBundle()` - does not exist
- `registerBundle()` - does not exist
- A public way to add a bundle to the enabled list

### 5. The TextMateBundleProvider Extension Point Does Not Exist (or Is Internal)

Attempt to use the extension point in plugin.xml:
```xml
<textmate.bundleProvider
  implementation="lang.ktav.KtavTextMateBundleProvider" />
```

**Result**: `Unresolved reference 'TextMateBundleProvider'` - the class is not exported through the public API

## Investigation of Other Plugins

### WDL IDE Plugin (Broad Institute)

>>>>> lang=ru
Доступные методы `TextMateService` в WebStorm 2025.3:
```
- readBundle(Path)              → объект Bundle или null
- reloadEnabledBundles()        → void (перезагружает уже включённые)
- getFileNameMatcherToScopeNameMapping()
- getLanguageDescriptorByExtension(String)
- getLanguageDescriptorByFileName(String)
- getShellVariableRegistry()
- getSnippetRegistry()
- getPreferenceRegistry()
```

**Отсутствуют**:
- `registerEnabledBundle()` — нет в 2025.3
- `enableBundle()` — не существует
- `registerBundle()` — не существует
- Публичный способ добавить бандл в список включённых

### 5. Extension point TextMateBundleProvider не существует (или внутренний)

Попытка указать точку расширения в plugin.xml:
```xml
<textmate.bundleProvider
  implementation="lang.ktav.KtavTextMateBundleProvider" />
```

**Результат:** `Unresolved reference 'TextMateBundleProvider'`; класс
не экспортирован в публичный API.

## Исследование других плагинов

### WDL IDE Plugin (Broad Institute)

>>>>> lang=zh
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

