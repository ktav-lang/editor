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

## [0.8.0] — 2026-09-28

同步至 `ktav` crate 与 `ktav-lang/spec` `0.8.0`。唯一普遍适用的破坏性
变更是**带前导零的十进制整数保留为字符串**(§ 5.2，`01234` 保留前导零)。
另有 § 8.1 要求提供严格解析入口，遇到有损标量形式时以 `LossyScalar`
拒绝。带引号的键片段
和 `\uXXXX` 转义已在 0.7.0 中引入。

- 三个组件(`ktav-lsp`、VS Code 扩展、IntelliJ 插件)同步升至
  **0.8.0**,与 `ktav` crate 和规范保持一致:`lsp/Cargo.toml` 现在
  依赖 `ktav = "=0.8.0"`,spec 子模块重新锁定到 `v0.8.0`,LSP 一致性
  测试现在遍历 0.8 语料库(此前遍历的是早已消失的 0.6 语料库,而且
  它的类别检查因 oracle 键不匹配而被静默禁用;现在类别从
  `ktav::ErrorEnvelope` 读取)。
- 三个组件中数字、关键字和转义的高亮现在严格遵循规范:带冗余前导零的
  十进制数(`01234`、`0_7`,§ 5.2)和格式错误的字面量(`1_`、`1__0`、
  `0X1A`、`2026-09-28`,§ 3.6)为字符串;`::` 之后的值绝不做类型区分,
  内联复合值中同样如此;带引号的键片段在内联对象中保持不透明
  (§ 5.3.3)。预构建的 `ktav-lsp` 二进制不再提交到仓库(见
  *仓库与发布工具*)。

### LSP server (`ktav-lsp`)

- `Cargo.toml`:`ktav = "=0.8.0"`(原为 `"0.6"`,即上一个已发布版本
  v0.6.1 使用的版本);`rust-version` 从 `1.70` 提升至 `1.88`,与锁定依赖图
  的要求一致。CI 在声明的最低 Rust 版本上构建所有 targets。
- 值补全保留 `:`/`::` 后必需的空白，包括光标右侧已有空白。
  raw 标记项修改分隔符本身；Array 字符串、多行正文及已存在值中不提供
  键值对补全。
- 解析器语料库检查输入接受及结构化错误类别，不固定错误/hover 文案。
  编辑器功能检查使用锁定解析器的树，包括格式化保值；这些检查不验证
  独立 JSON oracle 值或 strict body/canonical 字段。
- 文档符号区分完整声明范围和精确导航选择。Object/Array/String 范围包含
  值、子符号及匹配闭合符，但不含外围空白；重新打开的点状前缀覆盖所有
  出现位置，同时保留首次源键定位。
- **`ktav` 解析器自 0.7.0 起支持带引号的键,但 LSP 此前支持并不完整。**
  基于解析器的诊断无需改动;文档符号则需要本次发布中的
  `decode_symbol_key` 和位置修复,才能正确匹配、定位带引号的键。
  LSP 自有的按行分类器(`tokens::classify_line`、
  `split_dotted`,用于语义高亮、悬停提示和自动补全)与文档结构扫描器
  (`symbols::collect_key_hits`)各自手工扫描原始文本中的冒号和点,
  不知道 `"..."` / `'...'` / `` `...` `` 内的 `:` 或 `.` 只是普通
  内容。含结构字节的带引号键 —— 例如 `"a:b": 1` 或 `a."b.c".d: 1`
  —— 其冒号/点会被误读,进而破坏语义高亮、补全上下文检测以及文档
  符号大纲的键名和位置。修复方式是新增感知引号的冒号扫描
  (`tokens::find_key_separator`,取代 `tokens::find_unescaped`),并让
  `tokens::split_dotted` 对引号不透明;`symbols.rs` 中独立的重复
  扫描器已删除,改为从 `tokens` 导入同样的两个函数。扫描到的键片段
  还会先解码再与解析后的键匹配,文档符号的位置也已修复。
  不在片段首位的引号
  (`don't: 1`)不受影响,符合 § 5.3.3 的位置规则。
- 修复了 `textDocument/formatting` 中一个既有的跨度编码 bug,是在
  为本次升级审计字节偏移 `Span` 契约时发现的:整档替换编辑的结束
  `Position.character` 此前用 `str::chars().count()`(Unicode 标量
  计数)计算,而不是 `lsp/src/server/mod.rs` 其他处理函数都在用的、同样考虑编码
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
- 标量分类(`tokens::classify_value`)现在严格实现 § 3.6,并加入 § 5.2
  的冗余前导零例外:`01234`、`00`、`-045`、`0_7`、`01.5`、`05e3` 为
  字符串;`1_`、`1__0`、`0x_1`、`0X1A`、`1.`、`.5`、`2026-09-28` 以及
  类 IPv4 的串也是字符串,而 `0`、`0.5`、`0e0`、`0x1_A` 仍是数字。旧的
  启发式允许符号或下划线出现在任意位置。超出 i64 范围的整数仍按数字
  高亮(范围检查属于解析器)。
- 裁剪使用 § 3.3 中精确的 25 个空白码位(此前行尾只裁剪空格、制表符
  和 CR),因此行尾的 NBSP 不再让 `true` 变成字符串。
- 内联复合值:`::` 之后的值是原始字符串,绝不按数字或关键字高亮;
  带引号的键片段(`{"a,b": 1}`、`{'x:y': 2}`)对 `,`、`:` 和括号不透明;
  含转义的值为字符串(§ 3.7)。
- Hover:修复崩溃——长度超过 80 字节且截断点处是多字节字符的字符串值
  会 panic,而 release 配置在 panic 时中止进程,导致服务器退出。现在
  hover 会解析完整的键路径,因此嵌套在对象和对象数组中的键、带引号的键
  (`"a.b"`)和带转义的键(`a\.b`)都能显示其值;标签为 `integer` /
  `float`(自 0.5 起不再有类型标记)。
- `tests/spec_conformance.rs` 校验语料库 manifest(结构、类别、样例数量),
  另外运行 `parseable-unrepresentable` 和 `strict-lossy`(通过
  `ktav::parse_strict`),检查 `unrepresentable` 的 oracle,并且在缺少 spec
  子模块时失败而不是静默通过。
- Semantic tokens:多行字符串块(`(` / `((`)此前没有跨行状态,导致
  `classify_line` 对每一行内容都单独重新解析——某行内容只是碰巧像注释、
  键值对或单独的闭合符,就会被当作对应类型高亮,而 verbatim 形式的
  闭合符 `))` 也会误判为 String token 而非 Operator。现通过在
  `semantic_tokens` 中跨行携带的小型 `MultiForm` 状态修复:块内容行
  现在各自产生一个裁剪后的 String token,`(` / `((` / `)` / `))`
  标记始终为 Operator。`hover` 与 `completion` 也获得了相同的防护
  (`tokens::line_is_multiline_content`),不会再把开放块内的一行误判为
  真正的 `key:` 键值对。
- Semantic tokens:内联对象的键在逗号之后紧跟转义结构字节(例如
  `{x: 0, \[a: 1}`)会使内联扫描器失步——单独的 `\` 变成 PROPERTY
  token,被转义的 `[` 会打开一个虚假的嵌套数组,真正的值则被折叠进
  一个字符串。`emit_inline` 的键扫描器现在将 `\X` 视为一个整体的转义
  单元,与值扫描器现有的处理方式一致。
- `tests/spec_conformance.rs` 新增了覆盖整个语料库的回归测试:对每个
  `valid/**.ktav` 样例,已解析 `Value` 树中 Number/Bool/Null *叶子*
  的数量必须与 `semantic_tokens` 输出的 Number/Keyword/"null" *token*
  数量一致(五个数值溢出为 String 的样例为固定的例外——高亮是词法层面
  的,不检查 i64/f64 的取值范围)。
- 所有处理器现在都通过同一个辅助模块 `tokens::lines`,将文档按 LF、CR
  和 CRLF 一视同仁地拆分为行(§ 3.2,LSP 的位置模型也是如此计数)。
  仅含 CR 的文档现在能得到正确的位置、token、hover、补全、符号、诊断
  (范围不再取自只统计 `\n` 的 `ktav::Span::line_col`)以及格式化。
  格式化器始终输出 LF。
- 格式化器:注释只是行首的 `##`(单个 `#` 是普通内容,因此 `#child: {`
  与任何键一样产生嵌套);块与复合值的起始行按结构识别,所以字面字符串
  `key:: ((` 不再开启多行块;`name: (value)` → `name:: (value)` 的规范化
  与解析器一样查找键分隔符,并且不改动空形式 `()` / `(())`;行首 BOM 会
  被保留。
- `key:: {`、`key:: ((` 等是字符串值,不是起始行;`(())` 被归类为空复合值
  简写(§ 5.7)。
- 行首 BOM(§ 3.1)不再成为 hover、符号、token 和诊断中第一个键的一部分。
- 文档符号:内联对象与数组中的键拥有各自的范围(此前会塌缩到第 0 行),
  顶层的 `{ … }` / `[ … ]` 外壳是透明的。
- Hover 通过共享分类器跳过注释与块内容行,而不是使用自己的文本启发式。
- 语料库测试:`spec_conformance.rs` 现在要求 `.ktav` / `.canonical.ktav` /
  `.json` 三件套完整且每个类别的样例数量精确,缺少 oracle 的 `invalid/`
  样例会失败而不是被跳过;`editor_features_corpus.rs` 在全部 223 个 valid
  样例上检查文档符号、hover 和 semantic tokens,并在全部 446 个 `.ktav`
  文件上检查格式化幂等且保持值不变。

### TextMate grammar (VS Code + shared `grammars/`)

- TextMate 按首个内容行保留跨行及嵌套容器的 Object/Array 上下文；
  形似键值对的 Array 字符串不再错误地打开块。
- 裸 `#`、多词键、点分片段边缘空白与位置性引号保留精确的键/值 scope。
  主题文档列出实际的 Boolean 和 Null scope；无需额外依赖的导出器可生成
  Sublime `source.ktav` 语法支持的 XML `.tmLanguage`。

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
- 数字 scope 严格遵循 § 3.6 / § 5.2(整行键值对、数组元素和内联值):
  冗余前导零(`01234`、`0_7`、`01.5`)、位置错误的下划线(`1_`、`1__0`、
  `0x_1`)、大写进制前缀(`0X1A`)以及日期或点分串(`2026-09-28`、
  `127.0.0.1`)都是字符串,而不是数字。
- `\uXXXX`:高位加低位代理对是一个转义 token;孤立代理项或格式错误的
  `\u` 获得 `invalid.illegal.escape.unicode.ktav`(§ 3.7.1)。
- 内联对象:`::` 之后的值始终为 `string.unquoted.raw.ktav`(绝不是数字或
  关键字);带引号的键片段在任意缩进以及 `{` 或 `,` 之后都能识别。
- 修复结构性缺陷:经由 `captures` 引入的规则中的 `^` / `$` 锚定的是行而
  不是捕获,因此只要值不从第 0 列开始,数字/关键字/带引号键的分类就会
  静默失效。现在分类已并入直接扫描的模式。
- 新增分词器测试(`vscode/src/test/unit/grammar-tokens.test.ts`,基于
  `vscode-textmate` + `vscode-oniguruma`),在整行、数组元素和内联上下文
  中用真实语法跑这些向量。
- 顶层就是单行内联对象或数组(没有前导键)的文档此前完全不被高亮;
  根 patterns 现在包含 `top-level-inline-object` /
  `top-level-inline-array`。新增覆盖整个语料库的分词器测试
  (`corpus-coverage.test.ts`),对每个 valid 样例运行该语法:不出现
  `invalid.*` scope,数字、布尔和 null 的 scope 数量与样例的期望值一致。

### IntelliJ 插件

- 增量词法状态保留精确的根/容器上下文，无 packed-depth 上限或哈希碰撞。
  形似键值对的 Array 元素及独立 `)`/`))` 保持为字符串；多词裸键中的
  位置性引号仍是普通字符。
- Folding 使用词法标记；raw 标量与多行正文保持不透明，不从类似括号的
  文本错误地打开容器。
- 诊断按项目及精确的打开客户端/版本/会话管理。过期、关闭或已替换的
  发布不会覆盖当前结果；每个 Editor 的 highlighter 都通过实际所属
  MarkupModel 清除。关闭一个拥有者不会移除另一个项目的诊断及高亮。

- 高亮词法分析器遵循规范:严格的 § 3.6 数字语法并加入 § 5.2 前导零例外
  (仅 ASCII 数字——旧检查使用 `Char.isDigit()`,会接受其他文字的数字)、
  精确的关键字,以及 § 3.3 中精确的 25 个空白码位。
- 整行键与内联键都支持带引号的键片段(§ 5.3.3);未闭合的引号退化为
  普通值,并保持增量重新词法分析的状态有效。
- 内联复合值中 `::` 之后的值同样是字符串;含转义的内联标量为字符串;
  内联值中的字面 `:` 不再被当作分隔符。
- 修复了 Marketplace/Settings 中过时的文案:注释切换的描述写的是
  `#` 而非 `##`(`plugin.xml`、`build.gradle.kts`);Settings → Tools
  → Ktav 的帮助文本以及 `KtavLanguage`/`KtavConfigurable` 的 KDoc
  声称 LSP 功能需要单独安装 LSP4IJ 插件、且未内置二进制文件 ——
  这两点只在插件拥有自己内置的 LSP 客户端和按平台打包的二进制文件
  之前才成立。Marketplace 描述中的示例也含有 `# ...` 这类行内注释,
  这在 Ktav 中并不合法(`#` 若不在行首 `##` 之后即为普通内容)——
  已改写为独立的 `##` 行。
- 词法分析器:多行 `(` / `((` 块的正文现在是不透明的(`MULTILINE_TEXT`,
  由 `)` / `))` 关闭)——此前其中的 `{`、`[` 和 `key: value` 行会被当作
  结构处理,数组中单独的 `(` 条目会成为非法字符。`\n` 之前的 `\r` 视为
  空白,因此 CRLF 文档中的 `true` 和数字保持其类型。
- `KtavCorpusCoverageTest` 对每个 valid 样例运行词法分析器(token 流无空洞
  也无重叠;数字、关键字和 null 的数量与样例期望值一致)。三条较早的
  `KtavLexerTest` 断言比较的是平台的 `TokenType.BAD_CHARACTER` 而不是插件
  自己的类型,永远不会失败——已修复。
- LSP 同步现在按项目/文档会话保留一个订阅:共享文档分别更新每个拥有者,
  恢复的标签页使用正常的打开流程,重新打开时在新会话中发送当前内容。
  项目关闭会移除其监听器并关闭客户端,不会发布尚未完成初始化的客户端,
  也不会留下延迟启动的服务器进程。
  传输关闭时还会使等待中的请求失败,并原子地拒绝迟到的请求注册,
  而不是等待超时。
- 格式化在发送前和编辑器线程中的最终写入时,均检查文档身份、已同步
  版本/修改戳、订阅和客户端归属。旧的、已取消或过期的结果不会覆盖编辑
  或重新打开的文档;应用后的文本会同步到每个拥有者,并有独立的 Undo 步骤。
  回归覆盖使用真实 IntelliJ Documents 和平台格式化回调,以及可控制响应
  延迟的 stdio 子进程来验证响应和关闭竞态;可选的 smoke 使用真实
  `ktav-lsp` 二进制文件。

### Spec submodule

- 锁定到 `5871254`(`v0.8.0`),此前为 `04f867f`(`v0.7.0`)。

### 仓库与发布工具

- IntelliJ 重建 helper 使用当前 checkout 及显式选择的 IDE/平台路径，
  不终止进程或清理 IDE 缓存、日志、其他插件及归档。Python 3 通过 staging
  安装 ZIP 并保留 Unix 可执行权限。
- Helix、Zed 和 Sublime 指南明确实际前提：标准 Helix 的 tree-sitter 高亮、
  Zed 已注册的语言/服务器适配器，以及 Sublime 匹配 `source.ktav` 的 XML
  TextMate 语法。
- 共享语法、Emacs 和 Neovim 指南区分 TextMate JSON 宿主与独立的
  tree-sitter/font-lock 集成。规范多行形式的双闭合符冲突产生
  `BothFormsRequired`，而非拆分字符串；键中字面点需转义或引号。

- 预构建的 `ktav-lsp` 二进制不再提交到仓库:已跟踪的副本早已过时
  (五个平台内嵌 `ktav` 0.5.0,win32-x64 内嵌 0.7.1,
  `intellij/src/main/resources/bin/` 下多余的一份内嵌 0.1.5)。
  `intellij/bin/`、`vscode/bin/` 和 `intellij/src/main/resources/bin/`
  已加入 `.gitignore`;发布 workflow 从源码构建全部六个平台,
  `scripts/build-binaries.sh` 在本地做同样的事。
- TextMate 分词器测试为 VS Code 项目增加了固定版本的开发依赖
  (`vscode-textmate`、`vscode-oniguruma`)——不会打进 VSIX,同时
  `tsconfig.json` 为其 WebAssembly 类型加入 `DOM` 库。
- 发布 workflow:`vsce package` / `vsce publish` 不再传 `--no-dependencies`。
  扩展运行时需要 `vscode-languageclient`,而 0.6.1 的 VSIX 未带
  `node_modules`,因此打包后的扩展无法加载语言客户端;现在会打包生产
  依赖(不含 devDependencies)。
- CI:docs job 使用 `actions/setup-node@v6`。
- 文档:VS Code 与 LSP 的 README 现在一致说明,Marketplace 与 Open VSX
  上的扩展已为六个平台捆绑 `ktav-lsp`(仅其他情况才需要单独安装);
  `intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md` 与
  `lsp/docs/bench-baseline.md` 已标注为历史文档。

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
