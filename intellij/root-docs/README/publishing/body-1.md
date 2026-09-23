>>>>> lang=en
## Publishing

CI invokes `./gradlew publishPlugin` with the marketplace PAT in the
`INTELLIJ_PUBLISH_TOKEN` environment variable. The token is generated at
<https://plugins.jetbrains.com/author/me/tokens> and must belong to a
maintainer of the `lang.ktav` plugin id on the marketplace. Local
publishing is intentionally not supported — release via tagged CI runs
only.

>>>>> lang=ru
## Публикация

CI запускает `./gradlew publishPlugin` с marketplace-PAT в переменной
окружения `INTELLIJ_PUBLISH_TOKEN`. Токен генерируется на
<https://plugins.jetbrains.com/author/me/tokens> и должен принадлежать
одному из мейнтейнеров плагина с id `lang.ktav` на marketplace.
Локальная публикация умышленно не поддерживается — релиз только
через тегированные CI-прогоны.

>>>>> lang=zh
## 发布

CI 通过环境变量 `INTELLIJ_PUBLISH_TOKEN` 中的 marketplace PAT 调用
`./gradlew publishPlugin`。Token 在
<https://plugins.jetbrains.com/author/me/tokens> 生成,且必须属于
marketplace 上 `lang.ktav` 插件 id 的 maintainer。有意不支持本地
发布 —— 仅通过 tagged CI 运行发布。

