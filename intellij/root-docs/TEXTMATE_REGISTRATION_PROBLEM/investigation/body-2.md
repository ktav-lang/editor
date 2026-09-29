>>>>> lang=en
### Implementation of the Rejected Attempt (KtavTextMateLoader.kt, before 0.2.0)

```
1. appFrameCreated hook (IDE startup)
   ├─ Check for the bundle in the file system (development mode)
   ├─ If not found, extract it from the plugin JAR
   └─ Attempt to register it with TextMate

2. projectOpened hook (Project load)
   └─ Retry registration (for dynamic plugin loading)

3. Bundle extraction
   ├─ Find ktav-intellij-*.jar in lib/ (excluding searchableOptions)
   ├─ Extract grammars/ktav/ into a temporary directory
   └─ Create a .tmbundle directory with the correct structure

4. Registration attempt
   ├─ Attempt 1: Update textmate.xml (TextMateUserBundlesSettings)
   ├─ Attempt 2: Call TextMateService.readBundle()
   └─ Attempt 3: Call reloadEnabledBundles()
```

>>>>> lang=ru
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

>>>>> lang=zh
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

