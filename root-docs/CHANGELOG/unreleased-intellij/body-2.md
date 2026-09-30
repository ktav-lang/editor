>>>>> lang=en
- Incremental lexer states retain exact root/container context without
  packed-depth limits or hash collisions. Pair-shaped Array items and lone
  `)`/`))` remain Strings; positional quotes in multiword bare keys stay literal.
- Folding consumes lexer tokens, leaving raw scalars and multiline bodies
  opaque instead of opening scopes from bracket-shaped text.
- Diagnostics belong to each project and exact open client/version/session.
  Stale, closed or replaced publications cannot overwrite current results;
  every Editor's highlighters are removed through their actual owning model.
  Closing one owner preserves the other project's diagnostics and highlights.

>>>>> lang=ru
- Инкрементальные состояния лексера сохраняют точный контекст корня и
  контейнеров без ограничения глубины packed-state и коллизий хеша.
  Похожие на пары элементы Array и отдельные `)`/`))` остаются строками;
  позиционные кавычки в многословных некавыченных ключах остаются буквальными.
- Folding использует токены лексера: raw-значения и многострочное содержимое
  непрозрачны и не открывают контейнеры из похожего на скобки текста.
- Диагностика принадлежит проекту и точной открытой сессии/версии/клиенту.
  Старые, закрытые или заменённые публикации не затирают текущий результат;
  highlighter каждого Editor удаляется через его реальный MarkupModel.
  Закрытие одного владельца сохраняет диагностику и подсветку другого.

>>>>> lang=zh
- 增量词法状态保留精确的根/容器上下文，无 packed-depth 上限或哈希碰撞。
  形似键值对的 Array 元素及独立 `)`/`))` 保持为字符串；多词裸键中的
  位置性引号仍是普通字符。
- Folding 使用词法标记；raw 标量与多行正文保持不透明，不从类似括号的
  文本错误地打开容器。
- 诊断按项目及精确的打开客户端/版本/会话管理。过期、关闭或已替换的
  发布不会覆盖当前结果；每个 Editor 的 highlighter 都通过实际所属
  MarkupModel 清除。关闭一个拥有者不会移除另一个项目的诊断及高亮。

