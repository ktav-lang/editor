# Ktav editor grammars

Canonical TextMate grammar and VS Code language configuration for the
[Ktav](../spec/) plain-text configuration format (`.ktav`).

**Languages:** **English** · [Русский](docs/README.ru.md) · [简体中文](docs/README.zh.md)

## Files

- `ktav.tmLanguage.json` — TextMate grammar. Scope name `source.ktav`,
  file extension `.ktav`. Direct JSON loading requires a host that
  accepts this format, such as VS Code. Sublime Text needs the XML
  export described in [its installation guide](../docs/sublime.md).
  This is not a tree-sitter grammar.
- `language-configuration.json` — VS Code language configuration:
  comments, brackets, auto-closing pairs, indentation rules, word
  pattern.

## Where it is used

These two JSON files are the canonical TextMate and VS Code configuration
artifacts:

- `vscode/` — bundles them via `package.json` `contributes.languages`
  and `contributes.grammars`.
- `intellij/` — uses its own `KtavLexer` and
  `KtavSyntaxHighlighterFactory` for native highlighting; it does not load
  this TextMate grammar.

Grammar changes reach the VS Code extension on its next build / packaging
step. Update the IntelliJ lexer separately when the same behavior is needed.

## Local testing

### VS Code

1. From a VS Code window, open the Command Palette and run
   `Developer: Inspect Editor Tokens and Scopes`.
2. Open any sample from `spec/versions/0.8/tests/valid/**/*.ktav`.
3. Click into a token; the panel shows the resolved scope chain. Each
   scope listed in the "Token classes" section below should appear on
   the corresponding token.

For an end-to-end check, install the `vscode/` extension via
`code --install-extension` (or `F5` from the `vscode/` workspace) and
visually verify that comments, keys, separators, markers, scalars,
keywords, and brackets all render distinctly under your color theme.

### Other editors

Direct use of `ktav.tmLanguage.json` requires an editor that accepts
TextMate grammars in JSON format, such as VS Code; TextMate support
alone does not guarantee that. Follow the host's grammar-registration
instructions and associate `*.ktav` with scope `source.ktav`.
Sublime Text requires XML: use the existing
`grammars/scripts/export-tmlanguage.js` exporter and follow the
[Sublime installation recipe](../docs/sublime.md#file-type-association).
Tree-sitter requires a separate Ktav grammar and highlight queries;
the shared TextMate JSON cannot be installed as a tree-sitter grammar.

## Token classes

The grammar tags spans with these scope names. Themes that style
these scopes will style Ktav consistently.

| Scope                                              | Matches                                    |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `## …` line comments                       |
| `entity.name.tag.ktav`                             | Bare key segments (left of `:`)            |
| `string.quoted.double.key.ktav`                    | Double-quoted key segments                 |
| `string.quoted.single.key.ktav`                    | Single-quoted key segments                 |
| `string.quoted.backtick.key.ktav`                  | Backtick-quoted key segments               |
| `punctuation.accessor.dot.ktav`                    | `.` separating dotted key segments         |
| `punctuation.separator.key-value.ktav`             | The `:` of a plain pair                    |
| `keyword.operator.marker.raw.ktav`                 | `::` (raw-string marker)                   |
| `constant.language.boolean.ktav`                   | `true`, `false` scalars                    |
| `constant.language.null.ktav`                      | `null` scalars                             |
| `constant.numeric.integer.ktav`                    | Integer literal (§ 3.6; no redundant leading zero) |
| `constant.numeric.float.ktav`                      | Float literal (§ 3.6; has `.` / exponent)  |
| `constant.numeric.ktav`                            | Whole-line Array item number literals      |
| `invalid.illegal.escape.unicode.ktav`              | Lone surrogate or malformed `\uXXXX`        |
| `string.unquoted.ktav`                             | Ordinary string scalars                    |
| `string.unquoted.raw.ktav`                         | Body after `::`                            |
| `string.quoted.multiline.stripped.ktav`            | Content inside `( … )`                     |
| `string.quoted.multiline.verbatim.ktav`            | Content inside `(( … ))`                   |
| `punctuation.section.braces.begin.ktav`            | `{`                                        |
| `punctuation.section.braces.end.ktav`              | `}`                                        |
| `punctuation.section.brackets.begin.ktav`          | `[`                                        |
| `punctuation.section.brackets.end.ktav`            | `]`                                        |
| `punctuation.section.parens.begin.ktav`            | `(`, `((`                                  |
| `punctuation.section.parens.end.ktav`              | `)`, `))`                                  |

## Notes on the implementation

- Marker disambiguation. Inside `pair`, alternatives are ordered
  `pair-raw` (`::`) → empty/open compound and multi-line forms →
  `pair-value` (`:` fallback). After a plain `:`, a bare body is
  classified by lexical form (§ 3.6 / § 5.2): an integer literal →
  `constant.numeric.integer`, a float literal → `constant.numeric.float`,
  anything else — including redundant-leading-zero decimals such as
  `01234` — → `string.unquoted` (matching the spec's mandatory
  space-after-separator rule, § 5.3 / § 6.10).
- The classification lives in the directly scanned pattern, not in a
  rule reached through `captures`: `^` / `$` there anchor to the line,
  not to the capture, so a capture-level check silently fails off
  column 0.
- Compound closers are anchored to standalone lines: `^\s*\)\s*$`,
  `^\s*\)\)\s*$`, `^\s*\}\s*$`, `^\s*\]\s*$`. A line like `) x` or
  `))suffix` does not close the block — it is content (in a multi-line
  string) or a syntax error (in an Object/Array context, which the
  grammar leaves unhighlighted).
- Array context is tracked through dedicated `array-*` repository
  rules included only inside `[ … ]` regions, so item-form lines
  (`:: foo`, bare scalars) light up only there.
- Multi-line string content is highlighted as a single string scope —
  no inner classification is applied, matching the spec's "raw
  content" semantics (§ 5.6).

## Known limitations

- Static syntax highlighting only. The grammar does not enforce
  semantic rules (duplicate-name detection, path conflicts, numeric
  validity beyond the regex shape, dotted-key expansion, empty-key
  checks). Those belong to a parser/linter.
- Inline `#` is *not* a comment per the spec, and the grammar honors
  that. A `#` inside a value body is highlighted as part of the string.
- Multi-line string content has one string scope; lines such as
  `key: value` are not pairs. A line trimming to exactly `)` closes
  stripped form, and one trimming to exactly `))` closes verbatim
  form (§ 5.6.1). To include one colliding closer as content, switch
  forms; other canonical representability checks still apply.
  If a String has both kinds of colliding line, the canonical
  multi-line writer must reject it with `BothFormsRequired` (§ 5.9.7).
  Adjacent blocks do not concatenate into one String. This is a
  canonical-output restriction, not a ban on every parseable spelling:
  the inline document `{s: ))\n)}` yields one String containing both
  lines, but has no canonical multi-line representation.
- Key dots follow § 4 and § 5.3.2–5.3.3: an unescaped `.` outside a
  quoted segment is a path separator. A literal dot in a bare segment
  requires `\.`; inside a quoted segment it is ordinary content.
  `example.com: 1` is a dotted path; `example\.com: 1` and
  `"example.com": 1` each use one literal key. `x.y\.z: v` has the
  path segments `x` and `y.z`. Regex highlighting is not validation
  of these paths' semantic effects.
