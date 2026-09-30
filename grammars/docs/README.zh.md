# Ktav 编辑器语法

[Ktav](../../spec/) 纯文本配置格式(`.ktav`)的规范 TextMate 语法与
VS Code 语言配置。

**Languages:** [English](../README.md) · [Русский](README.ru.md) · **简体中文**

## 文件

- `ktav.tmLanguage.json` — TextMate 语法。scope 名为 `source.ktav`,
  文件扩展名为 `.ktav`。直接加载 JSON 需要接受此格式的宿主,例如
  VS Code。Sublime Text 需要按[安装指南](../../docs/i18n/editors/zh/sublime.md)
  导出 XML。这不是 tree-sitter 语法。
- `language-configuration.json` — VS Code 语言配置:注释、括号、自动
  闭合对、缩进规则、单词模式。

## 使用位置

这两个 JSON 文件是 TextMate 语法和 VS Code 语言配置的规范产物:

- `vscode/` — 通过 `package.json` 的 `contributes.languages` 与
  `contributes.grammars` 打包它们。
- `intellij/` — 使用自己的 `KtavLexer` 和 `KtavSyntaxHighlighterFactory`
  进行原生语法高亮,不加载此 TextMate 语法。

此处的语法修改会在下次构建 / 打包时进入 VS Code 扩展。若 IntelliJ
词法分析器也需要相同行为,必须单独更新。

## 本地测试

### VS Code

1. 在 VS Code 窗口中打开命令面板,运行
   `Developer: Inspect Editor Tokens and Scopes`。
2. 打开 `spec/versions/0.8/tests/valid/**/*.ktav` 中的任意示例。
3. 点击某个 token;面板会显示解析出的 scope 链。下文"Token 类别"一节
   列出的每个 scope 都应出现在对应的 token 上。

如需端到端检查,可通过 `code --install-extension` 安装 `vscode/` 扩展
(或在 `vscode/` 工作区中按 `F5`),并目视确认注释、键、分隔符、标记、
标量、关键字和括号在你的配色主题下各自显示不同。

### 其他编辑器

直接使用 `ktav.tmLanguage.json` 需要编辑器接受 JSON 格式的 TextMate
语法,例如 VS Code;仅支持 TextMate 并不能保证这一点。请按宿主的语法
注册说明操作,并将 `*.ktav` 关联到 scope `source.ktav`。
Sublime Text 需要 XML:请使用现有的
`grammars/scripts/export-tmlanguage.js` 导出器,并遵循
[Sublime 安装步骤](../../docs/i18n/editors/zh/sublime.md#文件类型关联)。
Tree-sitter 需要独立的 Ktav 语法和高亮查询;
共享的 TextMate JSON 不能作为 tree-sitter 语法安装。

## Token 类别

语法用以下 scope 名称标记文本片段。为这些 scope 设置样式的主题会一致地
渲染 Ktav。

| Scope                                              | 匹配内容                                   |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `## …` 行注释                              |
| `entity.name.tag.ktav`                             | 裸键段（`:` 左侧）                         |
| `string.quoted.double.key.ktav`                    | 双引号键段                                 |
| `string.quoted.single.key.ktav`                    | 单引号键段                                 |
| `string.quoted.backtick.key.ktav`                  | 反引号键段                                 |
| `punctuation.accessor.dot.ktav`                    | 分隔点号键段的 `.`                         |
| `punctuation.separator.key-value.ktav`             | 普通键值对的 `:`                           |
| `keyword.operator.marker.raw.ktav`                 | `::`(原始字符串标记)                     |
| `constant.language.boolean.ktav`                   | `true`、`false` 标量                       |
| `constant.language.null.ktav`                      | `null` 标量                                |
| `constant.numeric.integer.ktav`                    | 整数字面量(§ 3.6;无冗余前导零)         |
| `constant.numeric.float.ktav`                      | 浮点字面量(§ 3.6;含 `.` / 指数)        |
| `constant.numeric.ktav`                            | 整行 Array 元素的数字字面量                |
| `invalid.illegal.escape.unicode.ktav`              | 孤立代理项或格式错误的 `\uXXXX`           |
| `string.unquoted.ktav`                             | 普通字符串标量                             |
| `string.unquoted.raw.ktav`                         | `::` 之后的主体                            |
| `string.quoted.multiline.stripped.ktav`            | `( … )` 内的内容                           |
| `string.quoted.multiline.verbatim.ktav`            | `(( … ))` 内的内容                         |
| `punctuation.section.braces.begin.ktav`            | `{`                                        |
| `punctuation.section.braces.end.ktav`              | `}`                                        |
| `punctuation.section.brackets.begin.ktav`          | `[`                                        |
| `punctuation.section.brackets.end.ktav`            | `]`                                        |
| `punctuation.section.parens.begin.ktav`            | `(`, `((`                                  |
| `punctuation.section.parens.end.ktav`              | `)`, `))`                                  |

## 实现说明

- 标记消歧。在 `pair` 内部,备选项按以下顺序排列:`pair-raw`(`::`)→
  空/开放的复合与多行形式 → `pair-value`(`:` 兜底)。在普通 `:` 之后,
  裸主体按词法形式分类(§ 3.6 / § 5.2):整数字面量 →
  `constant.numeric.integer`,浮点字面量 → `constant.numeric.float`,
  其他一切(包括 `01234` 这类带冗余前导零的十进制数)→
  `string.unquoted`(与规范中分隔符后必须有空格的规则一致,
  § 5.3 / § 6.10)。
- 分类位于直接扫描的模式中,而不是经由 `captures` 引入的规则:那里的
  `^` / `$` 锚定的是行而不是捕获,因此捕获级检查在非第 0 列时会静默失效。
- 复合值的闭合符锚定在独立的行上:`^\s*\)\s*$`、`^\s*\)\)\s*$`、
  `^\s*\}\s*$`、`^\s*\]\s*$`。形如 `) x` 或 `))suffix` 的行不会闭合块
  ——它是内容(在多行字符串中)或语法错误(在 Object/Array 上下文中,
  语法对此不做高亮)。
- 数组上下文通过专门的 `array-*` 仓库规则跟踪,这些规则只在 `[ … ]`
  区域内引入,因此元素形式的行(`:: foo`、裸标量)只在那里高亮。
- 多行字符串内容以单一字符串 scope 高亮——不做内部分类,与规范的
  "原始内容"语义一致(§ 5.6)。

## 已知限制

- 仅提供静态语法高亮。语法不强制语义规则(重复名称检测、路径冲突、
  超出 regex 形式的数字有效性校验、点号键展开、空键检查)。这些属于
  解析器/linter 的职责。
- 按规范,行内 `#` *不是*注释,语法也遵循这一点。值主体中的 `#` 作为
  字符串的一部分高亮。
- 多行字符串内容使用一个字符串 scope;形如 `key: value` 的行不是
  键值对。修剪后恰为 `)` 的行关闭 stripped 形式,恰为 `))` 的行关闭
  verbatim 形式(§ 5.6.1)。要将一种冲突的闭合符作为内容,需切换形式;
  其他规范可表示性检查仍然适用。若同一 String 含两种冲突行,
  规范多行 writer 必须以 `BothFormsRequired` 拒绝它(§ 5.9.7)。
  相邻块不会拼接成一个 String。这是规范输出的限制,并非禁止所有
  可解析写法:inline 文档 `{s: ))\n)}` 产生含两种行的一个 String,
  但不存在规范多行表示。
- 键中的点遵循 § 4 和 § 5.3.2–5.3.3:引号片段外未转义的 `.`
  是路径分隔符。裸片段中的字面点需要 `\.`;引号片段中的点则是普通
  内容。`example.com: 1` 是点分路径;`example\.com: 1` 和
  `"example.com": 1` 都使用一个字面键。`x.y\.z: v` 的路径片段为
  `x` 和 `y.z`。Regex 高亮不会验证这些路径的语义效果。
