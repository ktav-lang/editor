>>>>> lang=en
## Recommendation

**Use solution 1 (manual registration with good UX).** This was the historical recommendation before the switch to the native lexer.

That meant:
1. ✓ Keep the then-current bundle extraction implementation, which worked well
2. ✓ Log the path to the extracted bundle
3. ✓ Create an IDE action or notification for the user
4. ✓ Write detailed documentation
5. ✓ Switch to automatic registration if JetBrains releases a public API in the future

## Code from the Rejected Attempt (Historical State, Before 0.2.0)

**Files** (absent from the current plugin, replaced by the native lexer):
- `src/main/kotlin/lang/ktav/KtavTextMateLoader.kt` - main logic
- `src/main/kotlin/lang/ktav/KtavProjectActivity.kt` - project lifecycle hook
- `src/main/resources/META-INF/plugin.xml` - plugin configuration

**Status when the approach was abandoned**:
- Bundle extraction: ✓ Worked
- Settings update: ⚠️ Ran but did not take effect
- Auto-registration: ✗ Could not be implemented reliably

Since 0.2.0, syntax highlighting has been provided by `KtavParserDefinition` and
`KtavSyntaxHighlighterFactory` (using the native `KtavLexer`); see
`intellij/src/main/resources/META-INF/plugin.xml`.

## References

- [IntelliJ TextMate Plugin Source](https://github.com/JetBrains/intellij-community/tree/master/plugins/textmate)
- [TextMate Bundle Format](https://macromates.com/manual/en/bundles)
- [WDL IDE Plugin](https://github.com/broadinstitute/wdl-ide)
- IntelliJ API: `org.jetbrains.plugins.textmate.TextMateService`
>>>>> lang=ru
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
>>>>> lang=zh
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
