>>>>> lang=en
## What Worked ✓

1. **Bundle extraction from the JAR** - successfully produces a `.tmbundle`
2. **JAR filter** - correctly excludes searchableOptions.jar
3. **Lifecycle hooks** - both hooks fire as expected
4. **Reflection-based API calls** - successfully invoke TextMateService methods
5. **reloadEnabledBundles()** - runs without errors

## What Did Not Work ✗

### 1. TextMateService.readBundle() Returns null

```kotlin
val readBundleMethod = serviceCls.getMethod("readBundle", Path::class.java)
val bundle = readBundleMethod.invoke(service, bundlePath)
// → Result: null ❌
```

**Cause**: The bundle format does not meet the TextMate API's expectations. Possible reasons:
- Required files are missing (for example, `info.plist` or `menu.plist`)
- The directory structure is incorrect
- The API expects a different bundle format

### 2. Updating textmate.xml Works but Does Not Take Effect

>>>>> lang=ru
## Что работает ✓

1. **Извлечение бандла из JAR** — успешно получается формат `.tmbundle`
2. **Фильтр JAR** — правильно исключает searchableOptions.jar
3. **Хуки жизненного цикла** — оба срабатывают
4. **Рефлексивные вызовы API** — методы TextMateService вызываются
5. **reloadEnabledBundles()** — выполняется без ошибок

## Что НЕ работает ✗

### 1. TextMateService.readBundle() возвращает null

```kotlin
val readBundleMethod = serviceCls.getMethod("readBundle", Path::class.java)
val bundle = readBundleMethod.invoke(service, bundlePath)
// → Результат: null ❌
```

**Причина**: формат бандла не соответствует ожиданиям TextMate API. Возможные причины:
- Отсутствуют обязательные файлы, например `info.plist` или `menu.plist`
- Неверная структура директорий
- API ожидает другой формат бандла

### 2. Обновление textmate.xml работает, но не применяется

>>>>> lang=zh
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

