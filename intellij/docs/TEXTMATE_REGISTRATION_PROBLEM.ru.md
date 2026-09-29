# Проблема автоматической регистрации TextMate-бандла в плагинах IntelliJ

> **Исторический документ.** Описывает отвергнутый подход с попыткой
> автоматически регистрировать встроенную TextMate-грамматику во время
> выполнения. Начиная с 0.2.0 (2026-05-07) плагин вместо этого
> использует нативный `KtavLexer` / `KtavSyntaxHighlighterFactory`
> (подключены через `KtavParserDefinition`) для подсветки синтаксиса:
> нет TextMate-бандла, `KtavTextMateLoader` и шага ручной регистрации.
> Документ сохранён как объяснение отказа от прежнего пути; упоминания
> «текущей реализации» ниже относятся к попытке до 0.2.0, а не к
> выпущенному плагину.

## Краткое содержание

При разработке плагина Ktav для IntelliJ столкнулись с проблемой автоматической регистрации встроенной TextMate-грамматики. TextMate API IntelliJ имеет серьёзные ограничения, которые делают программную регистрацию невозможной или нестабильной.

## Что мы пытались сделать

Целью было обеспечить автоматическое включение подсветки синтаксиса для `.ktav` файлов при установке плагина **без** ручной регистрации в настройках IDE.

## Архитектура решения

### Реализация отвергнутой попытки (KtavTextMateLoader.kt, до 0.2.0)

```
1. Хук appFrameCreated (запуск IDE)
   ├─ Проверить бандл в файловой системе (режим разработки)
   ├─ Если не найден, извлечь из JAR плагина
   └─ Попытаться зарегистрировать в TextMate

2. Хук projectOpened (открытие проекта)
   └─ Повторить регистрацию (для динамической загрузки плагина)

3. Извлечение бандла
   ├─ Найти ktav-intellij-*.jar в lib/ (исключая searchableOptions)
   ├─ Извлечь grammars/ktav/ во временную директорию
   └─ Создать директорию .tmbundle с правильной структурой

4. Попытка регистрации
   ├─ Попытка 1: обновить textmate.xml (TextMateUserBundlesSettings)
   ├─ Попытка 2: вызвать TextMateService.readBundle()
   └─ Попытка 3: вызвать reloadEnabledBundles()
```

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

Попытка 1: JSON как простая карта ключей и значений:
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**Результат:** `XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`.

Попытка 2: JSON как массив объектов:
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**Результат:** десериализация не сработала, ошибки не записаны в журнал.

### 3. textmate.xml недоступен во время регистрации

- Плагин инициализируется при **appFrameCreated**, слишком рано.
- IDE создаёт `textmate.xml` позже.
- Найти файл не удаётся.
- Даже переписанный файл IDE может не перечитать.

### 4. TextMate API очень ограничен

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

Их реализация вызывает:
```java
TextMateService.getInstance().registerEnabledBundles(false);
```

**Проблема:** `registerEnabledBundles()` лишь перезагружает существующие
бандлы, а не регистрирует новые. Как они добавляют бандл в список
включённых, неясно.

### Другие плагины

Большинство используют TextMate-бандлы одним из способов:
1. Через bundled_plugins.txt (официальные плагины JetBrains)
2. Явным копированием в известные директории
3. Требуют ручного добавления и не предлагают автоматическую регистрацию

## Структура TextMate-бандла

Извлечённый бандл имеет структуру:
```
Ktav.tmbundle/
├── language-configuration.json
└── Syntaxes/
    └── ktav.tmLanguage.json
```

Она соответствует обычному формату VS Code TextMate, но IDE может
ожидать дополнительные файлы или иную структуру.

## Структура TextMateUserBundlesSettings

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

## Возможные решения

### 1. Ручная регистрация и понятный интерфейс ✓ (РЕКОМЕНДОВАЛОСЬ)

**Преимущества**:
- Работает надёжно
- Не зависит от внутреннего API
- Совместимо со всеми версиями IDE

**Реализация**:
- Бандл автоматически извлекается во временную директорию
- Плагин предлагает пользователю скопировать путь
- Или: действие меню «Register Ktav TextMate Bundle» открывает Settings → TextMate Bundles
- Либо: подробная пошаговая документация

### 2. Записывать textmate.xml до загрузки IDE

**Сложность**: IDE инициализирует эту настройку очень рано, до загрузки плагина.

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

## Рекомендация

**Использовать решение № 1 (ручная регистрация и понятный интерфейс).**
Это была историческая рекомендация до перехода на нативный лексер.

Это означало:
1. ✓ Сохранить тогдашнее извлечение бандла, которое работало
2. ✓ Добавить журналирование пути к извлечённому бандлу
3. ✓ Создать действие IDE или уведомление для пользователя
4. ✓ Написать подробную документацию
5. ✓ В будущем перейти к автоматической регистрации, если JetBrains откроет публичный API

## Код отвергнутой попытки (историческое состояние, до 0.2.0)

**Файлы** (в текущем плагине отсутствуют, заменены нативным лексером):
- `src/main/kotlin/lang/ktav/KtavTextMateLoader.kt` — основная логика
- `src/main/kotlin/lang/ktav/KtavProjectActivity.kt` — хук жизненного цикла проекта
- `src/main/resources/META-INF/plugin.xml` — конфигурация плагина

**Статус на момент отказа от подхода**:
- Извлечение бандла: ✓ Работало
- Обновление настроек: ⚠️ Выполнялось, но не применялось
- Автоматическая регистрация: ✗ Невозможно надёжно реализовать

С 0.2.0 подсветку обеспечивает `KtavParserDefinition` вместе с
`KtavSyntaxHighlighterFactory` (нативный `KtavLexer`), см.
`intellij/src/main/resources/META-INF/plugin.xml`.

## Источники

- [Исходный код IntelliJ TextMate Plugin](https://github.com/JetBrains/intellij-community/tree/master/plugins/textmate)
- [Формат TextMate-бандла](https://macromates.com/manual/en/bundles)
- [WDL IDE Plugin](https://github.com/broadinstitute/wdl-ide)
- IntelliJ API: `org.jetbrains.plugins.textmate.TextMateService`
