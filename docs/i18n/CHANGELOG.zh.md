# 变更日志

**Languages:** [English](../../CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 Ktav 编辑器支持(VS Code 扩展、IntelliJ 插件、LSP 服务器、
共享 TextMate 语法)的所有重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);版本号遵循
[Semantic Versioning](https://semver.org/),并采用 pre-1.0 惯例:
MINOR 递进视为破坏性变更。

一个 tag(`v0.X.Y`)同时发布全部四个子项目。子项目相关的变更按版本
标题分组。

本日志记录**编辑器/IDE 支持的发布**,而不是 Ktav 格式本身的变更 ——
后者请见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec/blob/main/CHANGELOG.md)。

## 未发布

同步 `ktav` crate `0.7.0` 与 `ktav-lang/spec` `0.7.0`(此前为
`0.6.0-4-gc9593e8`,甚至落后于 `0.6.4`,因此这次一并纳入了 0.6.x 的
补丁级规范变更)。规范的头号改动是**带引号的键片段**(§ 5.3.3:键
片段可以用 `"`、`'` 或反引号包裹;每个片段各自选择定界符,内容永不做
修剪,结构字节 `.` `:` `,` `{` `}` `[` `]` 在引号内均为普通
内容)以及 **`\uXXXX`** 转义(§ 3.7.1,仅在键和内联复合值中识别,
整行标量值和多行正文中均不识别),另有两个新的错误类型:
`UnterminatedQuotedKey`(§ 6.16)和顶层 `InvalidUtf8`(§ 6.15)。

- 三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)同步升至
  **0.8.0**,与 `ktav` crate 和规范保持一致:`lsp/Cargo.toml` 现在
  依赖 `ktav = "0.8"`,spec 子模块重新锁定到 `v0.8.0`,LSP 一致性
  测试现在遍历 0.8 语料库(此前遍历的是早已消失的 0.6 语料库,而且
  它的类别检查因 oracle 键不匹配而被静默禁用;现在类别从
  `ktav::ErrorEnvelope` 读取)。

### LSP server (`ktav-lsp`)

- `Cargo.toml`:`ktav = "0.7"`(原为 `"0.6"`);`rust-version` 提升至
  `1.71`(ktav 0.7 自身的 MSRV,原为 `1.70`)。
- **带引号的键现在在 `ktav` 解析器之外也能被正确理解。** 基于
  `ktav::parse` 的诊断/符号无需任何代码改动即可正确处理 0.7 语法
  (见下文),但 LSP 自有的按行分类器(`tokens::classify_line`、
  `split_dotted`,用于语义高亮、悬停提示和自动补全)与文档结构扫描器
  (`symbols::collect_key_hits`)各自手工扫描原始文本中的冒号和点,
  不知道 `"..."` / `'...'` / `` `...` `` 内的 `:` 或 `.` 只是普通
  内容。含结构字节的带引号键 —— 例如 `"a:b": 1` 或 `a."b.c".d: 1`
  —— 其冒号/点会被误读,进而破坏语义高亮、补全上下文检测以及文档
  符号大纲的键边界。修复方式是新增感知引号的冒号扫描
  (`tokens::find_key_separator`,取代 `tokens::find_unescaped`),并让
  `tokens::split_dotted` 对引号不透明;`symbols.rs` 中独立的重复
  扫描器已删除,改为从 `tokens` 导入同样的两个函数,它们现在终于
  配得上自己「单一事实来源」的文档注释。不在片段首位的引号
  (`don't: 1`)不受影响,符合 § 5.3.3 的位置规则。
- 修复了 `textDocument/formatting` 中一个既有的跨度编码 bug,是在
  为本次升级审计字节偏移 `Span` 契约时发现的:整档替换编辑的结束
  `Position.character` 此前用 `str::chars().count()`(Unicode 标量
  计数)计算,而不是 `server.rs` 其他处理函数都在用的、同样考虑编码
  的转换。只要最后一行含非 ASCII 内容,两种协商编码下都会少算
  (UTF-8:任何多字节字符都会少算字节长度;UTF-16:任何星界平面/代理
  对字符都会少算),可能导致格式编辑后最后一行尾部字节未被替换。
  相关逻辑已提取到 `end_of_document()`,并有新测试覆盖。
- 新的 `ErrorKind` 变体无需改动代码:
  `diagnostics::parse_for_diagnostics` 已经通过 `ErrorKind::span()` /
  `::line()` / `Display` 通用地驱动一切,因此 `UnterminatedQuotedKey`
  免费获得正确而精确的诊断。`Error::InvalidUtf8` 不会经由本 crate 的
  `&str` 入口出现(Rust 的 `&str` 按构造即为合法 UTF-8 —— 该变体
  只为字节级入口 `ktav::from_file` 触发,而本 LSP 从不调用它);
  已验证无现存代码路径需要修复。
- `reindent::canonicalise_paren_scalar`:删除了关于 `:i` / `:f` 类型
  标记的失效推理,这些标记早在 0.5.0 就已从规范中移除。该特殊分支
  本就不可达(任何 `:i` / `:f` 序列已经会因为另一个原因 —— 「分隔符
  后必须跟空格」检查 —— 失败),删除它不改变任何行为。基于已删除
  标记编写的测试在 `reindent.rs` 和 `tests/format_pipeline.rs` 中已
  改名/重写(包括 `complex_document_canonicalised` 中的示例行
  `port:i 8080` / `timeout:f 5.0`,现在只是普通键值对)。`::` 原始
  标记未动 —— 它仍是现行语法。

### TextMate grammar (VS Code + shared `grammars/`)

- 带引号的键片段:所有 `pair-*` / `inline-pair` 规则共享的点分键
  模式,现在在每个片段边界处都接受 `"..."`、`'...'` 或 `` `...` ``
  作为裸片段的替代形式,并遵守位置规则(`don't: 1` 不受影响)。
  `key-name` 为每种带引号形式用各自的 scope 子高亮
  (`string.quoted.{double,single,backtick}.key.ktav`)。
- `key-name` 和 `inline-scalar-body` 现在识别 `\uXXXX` 转义
  (`constant.character.escape.unicode.ktav`);`inline-scalar-body` 的
  命名转义字符类也补入了 `\.` `\:` `\"` `\'` `` \` `` —— 规范自
  0.6.0/0.7.0 起就有它们,但本语法的转义高亮列表一直缺失。
- `grammars/ktav.tmLanguage.json` 是唯一事实来源;
  `vscode/syntaxes/ktav.tmLanguage.json` 是由
  `vscode/scripts/sync-grammars.js` 同步生成的镜像(经 `npm run
  sync-grammars` / `compile` / `vscode:prepublish` 运行,并在发布
  workflow 打包前显式执行)。两个文件此处均通过该脚本更新,以保持
  字节级一致。

### Spec submodule

- 锁定到 `04f867f`(`v0.7.0`),此前为 `c9593e8`
  (`v0.6.0-4-gc9593e8`)。

## [0.6.1] — 2026-06-05

- 文档:所有 README 示例改写为 spec 0.6 语法(以裸数字取代已移除的 `:i`/`:f` 标记;以 `##` 注释取代 `#`)。
- LSP:从补全项中移除 `:i`/`:f`(类型标记已在 spec 0.5 中移除)。

## [0.6.0] — 2026-06-01

同步 `ktav` crate `0.6.0` 与 `ktav-lang/spec` `0.6.0`。版本号重新对齐,
与格式/核心保持同步步调(此前的编辑器版本为 `0.3.1`)。规范改动是
**键转义**:键现在会处理 § 3.7 转义集,并新增 `\.`(字面点,*不*分割
点分路径)和 `\:`(字面冒号,*不*作为键值分隔符);键中的字面反斜杠
现在写作 `\\`。编辑器支持已端到端更新,使高亮、令牌和诊断都能正确
处理转义后的键字节。

### LSP server (`ktav-lsp`)

- 语义令牌、文档符号和诊断现在**支持转义**:键值分隔符是第一个
  *未转义*的 `:` / `::`,点分路径只在*未转义*的 `.` 处切分。因此
  `a\.b: v` 是单个键 `a.b`,而 `a\:: v` 是键 `a:`、值为 `v`。
- 依赖 `ktav` 0.6.0。

### IntelliJ plugin

- `KtavLexer` 现在能切分转义的键片段
  (`\\ \. \: \, \} \] \{ \[ \n \r`),含转义点或转义冒号的键会作为
  单个键高亮。

### TextMate grammar (VS Code + shared `grammars/`)

- 键片段模式现在接受 `\` 转义,与规范的键转义集一致。


## [0.3.1] — 2026-05-10

同步 `ktav` crate `0.3.1` 与 `ktav-lang/spec` `0.1.1`。规范改动新增了
顶层数组(Array)作为一种受支持的根类型(属于增量变更,现有 Object
文档解析结果完全相同);编辑器在 LSP 大纲中体现它,并锁定了格式化器
的行为。

### LSP server (`ktav-lsp`)

- **顶层数组**(spec § 5.0.1)现在在 `build_symbols` 中是一等公民:
  当解析根节点为数组时,各元素在大纲中渲染为 `[0]`、`[1]`、…… 条目
  (与嵌套数组既有的渲染方式一致)。对象元素的符号范围指向该元素第一
  个键所在行;裸标量元素的范围则覆盖该元素自身所在行。
- 已验证 `reindent` 能保留裸顶层数组形态 —— 格式化器不会在文档根部
  凭空合成 `[ ... ]` 方括号。由 `tests/format_pipeline.rs` 中的新测试
  用例锁定。
- 0.3.0 中 `name: (value)` → `name:: (value)` 的自动消歧在
  `reindent` 中得以保留。
- 现有诊断/锁定测试已更新为使用前置锚定对(`anchor: 1\n…`)—— 针对
  那些此前在文档开头呈裸键值形态的输入:在 spec 0.1.1 下,这些裸行
  现在会解析为顶层数组的字符串条目,因此夹具被显式锚定,以保持
  malformed-pair 分支仍然被覆盖。

### Sync

- `lsp/Cargo.toml`:`ktav = "0.3.1"`(0.3.0 → 0.3.1 本地开发期间曾为
  path-dep)。
- `editor/spec` 子模块锁定在 `7256816`(spec 0.1.1)。


## [0.3.0] — 2026-05-08

同步 `ktav` crate `0.3.0`。涵盖解析器严格性变化(内联 `(value)` /
`((value))` 现在报错)、重复键跨度修复以及热路径微优化。另有一项
用户可见的 LSP 改进:

### LSP server (`ktav-lsp`)

- **`build_symbols` 重写为 O(N) 单遍扫描器** —— 此前,IDE 的文档大纲
  会让语言服务器在 500 KiB 的文档上卡住约 11.5 秒,因为每个顶层键都
  会触发一次全文扫描。新扫描器只遍历文本一次,为每一对记录
  `(virtual_depth, key, line_range)`,而对解析后 `Value` 的 DFS 则用
  一个顺序游标推进这些命中。实测耗时:11.5 秒 → 13.2 毫秒(快 871×)。
  支持大纲的编辑器(JetBrains、VSCode)不再在大配置文件上卡死。

- **格式化流水线:基于行的 reindent 无条件运行**,而不是以解析成功为
  前提。`ktav` 0.3.0 解析器严格拒绝内联 `(value)`,此前「仅在解析
  成功时格式化」的开关,恰恰在格式化本可以修复问题的时候把用户挡在
  门外。`canonicalise_paren_scalar` 会在保存时把 `key: (value)` 改写
  为 `key:: (value)`。`tests/format_pipeline.rs` 中新增了 22 个测试的
  集成套件,覆盖缩进规范化、空行保留、注释保留、多行
  stripped/verbatim 形式(字节级一致)、内联圆括号标量的自动修复,
  以及边界情况(CRLF、无结尾换行、空文档)。

### IntelliJ plugin

- 跟进 `ktav-lsp` 0.3.0 二进制(含符号大纲加速与「先格式化后解析」
  的改动)。本版本没有独立的 IntelliJ 改动;带构建时间戳的版本号
  `0.3.0+YYYYMMDD-HHMM` 使 IDE 中显示的版本在反复重新构建之间仍可
  区分。

### VS Code extension

- 跟进 `ktav-lsp` 0.3.0 二进制。本版本没有独立的 VS Code 改动。


## [0.2.0] — 2026-05-07

四个子项目(LSP、VS Code、IntelliJ、共享语法)的首个同步发布。同步
`ktav` crate `0.2.0`。

### LSP server (`ktav-lsp`)

- **textDocument/formatting** 能力 + 处理函数:经 `ktav` crate 解析
  → 渲染。内容已 canonical 或输入无法解析时给出空编辑。
- 从 crates.io 锁定 `ktav = "0.2.0"`(开发期间曾为 path-dep)。
- 不再为 `:f 42` 发出诊断(已在 `ktav` 语义层处理 —— 整数字面量会
  强制转换为浮点)。
- 多行字符串默认以 stripped `( ... )` 形式渲染(verbatim
  `(( ... ))` 仍作为后备,用于带前导空白或仅含 `)` 的行的内容)。

### VS Code extension

- 显式注册 `DocumentFormattingEditProvider`(使
  `editor.defaultFormatter = ktav-lang.ktav` 能正确解析,VS Code 也
  不再提示安装其他格式化工具)。
- 为 `[ktav]` 设置 `configurationDefaults`:tabSize 4、insertSpaces,
  默认格式化器固定为本扩展。
- VSIX 打包时包含 `vscode-languageclient` 运行时目录 —— 修复了此前
  `--no-dependencies` 打包引入的
  `Cannot find module 'vscode-languageclient/node'` 激活失败。

### IntelliJ plugin

- 用自研的 JSON-RPC LSP 客户端取代 LSP4IJ 依赖(无需外部插件)。
- 基于状态机词法分析器的原生语法高亮:
  KEY / KEY_DOT / MARKER_INT / MARKER_FLOAT / DOUBLE_COLON / COLON /
  STRING_VALUE / INT_VALUE / FLOAT_VALUE / BOOLEAN / NULL /
  MULTILINE_OPEN/CLOSE / BRACES / BRACKETS / COMMENT。
- `KtavParserDefinition`(极简扁平解析器)—— 提供 PSI 树,使
  `ExternalAnnotator`(LSP 诊断 → 工具提示 + Problems View)得以工作。
- `KtavFoldingBuilder` —— 折叠 `{}`、`[]`、`()`、`(())`。
- `KtavBraceMatcher` —— 配对括号高亮 + 输入时自动闭合。
- `KtavUnicodeAnnotator` —— 对键中的非 ASCII 字符加红色方框高亮
  (类似于 VS Code 的 `editor.unicodeHighlight`)。
- `KtavFormattingService`(AsyncDocumentFormattingService)把
  Ctrl+Alt+L 挂到 LSP `textDocument/formatting`。
- `KtavStartupActivity` 在插件动态加载时同步已打开的 `.ktav` 文件。
- 捆绑跨平台 `ktav-lsp` 二进制文件(Windows x64)。
- 插件分发 ZIP 通过 `_repackageWithBinaries` 任务正确打包二进制文件。
- `pluginVerification` 锁定在 `IC-2024.3 / 2025.1 / 2025.2`(避免
  `recommended()` 在仅有元数据的未来版本上失败)。
- `untilBuild = provider { null }` —— plugin.xml 中不设 IDE 版本上限
  (此前会发出 `until-build=""`,被验证器拒绝)。
- 插件版本 0.1.5 → 0.2.0。

### Shared grammar (`grammars/`)

- `pair-float` 与 `array-item-float` 正则现在接受 `:f` 后的整数字面量
  (小数点现在是可选的)。已同步进 VS Code `syntaxes/` 和 IntelliJ
  `resources/grammars/`。

### Spec submodule

- 夹具 `typed_float_without_decimal` 从 `invalid/` 移至
  `valid/typed_float_integer_body`(与新的 `:f 42` 语义一致)。
