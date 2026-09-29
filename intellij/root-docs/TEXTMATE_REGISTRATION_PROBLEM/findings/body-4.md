>>>>> lang=en
Their implementation simply calls:
```java
TextMateService.getInstance().registerEnabledBundles(false);
```

**Problem**: `registerEnabledBundles()` does not register new bundles; it only reloads existing ones. It is unclear how they add a bundle to the enabled list.

### Other Plugins

Most plugins use TextMate bundles in one of these ways:
1. Through bundled_plugins.txt (for official JetBrains plugins)
2. By explicitly copying them into known directories
3. Without automatic registration, requiring users to add them manually

## TextMate Bundle Structure

The extracted bundle has the correct structure:
```
Ktav.tmbundle/
├── language-configuration.json
└── Syntaxes/
    └── ktav.tmLanguage.json
```

This matches the standard VS Code TextMate bundle format. However, the IDE might expect additional files or a different structure.

## TextMateUserBundlesSettings Structure

>>>>> lang=ru
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

>>>>> lang=zh
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

