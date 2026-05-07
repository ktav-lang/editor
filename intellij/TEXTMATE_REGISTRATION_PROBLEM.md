# TextMate Bundle Auto-Registration Problem in IntelliJ Plugins

## Summary

При разработке плагина Ktav для IntelliJ столкнулись с проблемой автоматической регистрации bundled TextMate грамматики. TextMate API IntelliJ имеет серьёзные ограничения, которые делают программную регистрацию невозможной или нестабильной.

## Что мы пытались сделать

Целью было обеспечить автоматическое включение синтаксического выделения для `.ktav` файлов при установке плагина **без** необходимости ручной регистрации через IDE Settings.

## Архитектура решения

### Текущая реализация (KtavTextMateLoader.kt)

```
1. appFrameCreated hook (IDE startup)
   ├─ Проверить bundle в file-system (dev mode)
   ├─ Если не найден, извлечь из plugin JAR
   └─ Попытаться зарегистрировать в TextMate

2. projectOpened hook (Project load)
   └─ Повторить регистрацию (для dynamic plugin loading)

3. Bundle extraction
   ├─ Найти ktav-intellij-*.jar в lib/ (исключая searchableOptions)
   ├─ Извлечь содержимое grammars/ktav/ в temp directory
   └─ Создать .tmbundle директорию с правильной структурой

4. Registration attempt
   ├─ Попытка 1: Обновить textmate.xml (TextMateUserBundlesSettings)
   ├─ Попытка 2: Вызвать TextMateService.readBundle()
   └─ Попытка 3: Вызвать reloadEnabledBundles()
```

## Что работает ✓

1. **Bundle extraction из JAR** - успешно извлекается в формат `.tmbundle`
2. **JAR filter** - правильно исключает searchableOptions.jar
3. **Lifecycle hooks** - оба хука срабатывают правильно
4. **Reflection-based API calls** - успешно вызываются методы TextMateService
5. **reloadEnabledBundles()** - выполняется без ошибок

## Что НЕ работает ✗

### 1. TextMateService.readBundle() возвращает null

```kotlin
val readBundleMethod = serviceCls.getMethod("readBundle", Path::class.java)
val bundle = readBundleMethod.invoke(service, bundlePath)
// → Результат: null ❌
```

**Причина**: Bundle формат не соответствует ожиданиям TextMate API. Возможные причины:
- Отсутствуют обязательные файлы (например, `info.plist`, `menu.plist`)
- Неверная структура директорий
- API ожидает другой формат bundle'а

### 2. Обновление textmate.xml работает, но не применяется

Попытка 1: JSON как простой key-value map
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**Результат**: `XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`

Попытка 2: JSON как array объектов
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**Результат**: Десериализация не сработала, ошибок не логируется

### 3. textmate.xml недоступен на время регистрации

- Plugin инициализируется в момент **appFrameCreated** (очень рано)
- Файл `textmate.xml` создается IDE позже
- Попытка найти файл завершается неудачей
- Даже если переписать файл, IDE может не перечитать его

### 4. TextMate API очень ограничен

Доступные методы на `TextMateService` в WebStorm 2025.3:
```
- readBundle(Path)              → Bundle object или null
- reloadEnabledBundles()        → void (перезагружает уже включённые)
- getFileNameMatcherToScopeNameMapping()
- getLanguageDescriptorByExtension(String)
- getLanguageDescriptorByFileName(String)
- getShellVariableRegistry()
- getSnippetRegistry()
- getPreferenceRegistry()
```

**Отсутствуют**:
- `registerEnabledBundle()` - не существует в 2025.3
- `enableBundle()` - не существует
- `registerBundle()` - не существует
- Публичный способ добавить bundle в "включённые"

### 5. Extension point TextMateBundleProvider не существует (или internal)

Попытка использовать extension point в plugin.xml:
```xml
<textmate.bundleProvider
  implementation="lang.ktav.KtavTextMateBundleProvider" />
```

**Результат**: `Unresolved reference 'TextMateBundleProvider'` - класс не экспортирован в публичный API

## Исследование других плагинов

### WDL IDE Plugin (Broad Institute)

Их реализация просто вызывает:
```java
TextMateService.getInstance().registerEnabledBundles(false);
```

**Проблема**: Метод `registerEnabledBundles()` не регистрирует новые bundle'ы, а только перезагружает уже существующие. Не ясно, как они добавляют bundle в "включённые".

### Другие плагины

Большинство плагинов используют TextMate bundle'ы либо:
1. Через bundled_plugins.txt (для официальных плагинов JetBrains)
2. Через явное копирование в известные директории
3. Не предоставляют автоматическую регистрацию, требуя ручного добавления

## Структура TextMate Bundle

Наш extracted bundle имеет правильную структуру:
```
Ktav.tmbundle/
├── language-configuration.json
└── Syntaxes/
    └── ktav.tmLanguage.json
```

Это соответствует стандартному формату VS Code TextMate bundle'а. Однако IDE может ожидать дополнительные файлы или другую структуру.

## TextMateUserBundlesSettings структура

В WebStorm 2025.3 конфиг хранится в `~\AppData\Roaming\JetBrains\WebStorm2025.3\options\textmate.xml`:

```xml
<application>
  <component name="TextMateUserBundlesSettings">
    <![CDATA[{}]]>
  </component>
</application>
```

**Что здесь происходит**:
- `TextMateUserBundlesSettings` - это AppState component
- Содержимое - это JSON, обёрнутый в CDATA
- При ручной регистрации bundle'а JSON обновляется и переписывается
- IDE читает JSON при загрузке и вызывает `TextMateUserBundlesSettings.deserialize()`

**Проблема**: Мы не знаем точный JSON schema, который ожидает IDE версии 2025.3

## Почему это сложно

1. **TextMate API internal** - используемые методы не являются публичной частью IntelliJ SDK
2. **API меняется между версиями** - методы и сигнатуры отличаются между 2024.x и 2025.x
3. **Нет документации** - JetBrains не документирует внутреннюю работу TextMate плагина
4. **Timing проблемы** - требуется точная синхронизация инициализации плагина и IDE state
5. **Нет extension point'а** - нельзя declaratively определить bundle'ы в plugin.xml
6. **Версионность** - разные IDE версии имеют разные internal API

## Возможные решения

### 1. Manual Registration + Good UX ✓ (РЕКОМЕНДУЕТСЯ)

**Преимущества**:
- Работает надёжно
- Не зависит от internal API
- Совместимо со всеми версиями IDE

**Реализация**:
- Bundle автоматически извлекается в temp directory
- Plugin предлагает пользователю скопировать path
- Или: Action в меню → "Register Ktav TextMate Bundle" → открыть Settings → TextMate Bundles
- Либо: Подробная документация с пошаговыми инструкциями

### 2. Write to textmate.xml перед IDE load

**Сложность**: IDE инициализирует эту настройку очень рано, до того как плагин загружается.

**Возможное решение**:
- Использовать `AppLifecycleListener.appStarting()` вместо `appFrameCreated()`
- Писать в textmate.xml ДО того как IDE его прочитает
- Требует точного timing и может быть нестабильным

### 3. Использовать другой хранилище конфигурации

Вместо textmate.xml, писать в:
- `.idea/` проекта (но это не совместимо с глобальным TextMate)
- Custom конфиг плагина (но IDE не будет его читать)

### 4. Bundled TextMate bundles (официальный способ)

**Требует**:
- Зарегистрировать bundle в bundled_plugins.txt
- Возможно только для официальных плагинов JetBrains
- Недоступно для сторонних разработчиков

### 5. Ожидать улучшения API в будущих версиях IDE

**Состояние**: JetBrains может выпустить публичный extension point в 2026.x или позже

## Рекомендация

**Использовать решение #1 (Manual Registration + Good UX)**

Это означает:
1. ✓ Keep текущую реализацию (bundle extraction работает отлично)
2. ✓ Добавить логирование пути к extracted bundle'у
3. ✓ Создать IDE Action или Notification для пользователя
4. ✓ Написать подробную документацию
5. ✓ На будущее: когда JetBrains выпустит публичный API, переключиться на auto-registration

## Текущий код

**Файлы**:
- `src/main/kotlin/lang/ktav/KtavTextMateLoader.kt` - основная логика
- `src/main/kotlin/lang/ktav/KtavProjectActivity.kt` - project lifecycle hook
- `src/main/resources/META-INF/plugin.xml` - конфигурация плагина

**Статус**:
- Bundle extraction: ✓ Работает
- Settings update: ⚠️ Работает, но не применяется
- Auto-registration: ✗ Невозможно надёжно реализовать

## References

- [IntelliJ TextMate Plugin Source](https://github.com/JetBrains/intellij-community/tree/master/plugins/textmate)
- [TextMate Bundle Format](https://macromates.com/manual/en/bundles)
- [WDL IDE Plugin](https://github.com/broadinstitute/wdl-ide)
- IntelliJ API: `org.jetbrains.plugins.textmate.TextMateService`
