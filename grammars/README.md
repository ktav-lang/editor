# Ktav editor grammars

Canonical TextMate grammar and VS Code language configuration for the
[Ktav](../../spec/) plain-text configuration format (`.ktav`).

## Files

- `ktav.tmLanguage.json` — TextMate grammar. Scope name `source.ktav`,
  file extension `.ktav`. Suitable for any TextMate-compatible host
  (VS Code, Sublime Text, Atom, IntelliJ TextMate bundles, GitHub
  Linguist, `tree-sitter`-adjacent tools that consume TextMate, etc.).
- `language-configuration.json` — VS Code language configuration:
  comments, brackets, auto-closing pairs, indentation rules, word
  pattern.

## Where it is used

These two JSON files are the canonical artifacts. Downstream packages
consume them by reference (copy or symlink) — no logic is duplicated:

- `vscode/` — bundles them via `package.json` `contributes.languages`
  and `contributes.grammars`.
- `intellij/` — loads `ktav.tmLanguage.json` through the IntelliJ
  TextMate bundle API.

When you change the grammar here, both downstream packages pick up
the new behavior on their next build / packaging step. Do not fork
copies in the downstream subprojects; fix the bug here.

## Local testing

### VS Code

1. From a VS Code window, open the Command Palette and run
   `Developer: Inspect Editor Tokens and Scopes`.
2. Open any sample from `spec/versions/0.1/tests/valid/**/*.ktav`.
3. Click into a token; the panel shows the resolved scope chain. Each
   scope listed in the "Token classes" section below should appear on
   the corresponding token.

For an end-to-end check, install the `vscode/` extension via
`code --install-extension` (or `F5` from the `vscode/` workspace) and
visually verify that comments, keys, separators, markers, scalars,
keywords, and brackets all render distinctly under your color theme.

### Other editors

Any editor with TextMate-grammar support can consume
`ktav.tmLanguage.json` directly. Drop it into a TextMate bundle (or
the editor's grammar directory) and associate `*.ktav` with scope
`source.ktav`.

## Token classes

The grammar tags spans with these scope names. Themes that style
these scopes will style Ktav consistently.

| Scope                                              | Matches                                    |
| -------------------------------------------------- | ------------------------------------------ |
| `comment.line.number-sign.ktav`                    | `# …` line comments                        |
| `entity.name.tag.ktav`                             | Key segments (left of `:`)                 |
| `punctuation.accessor.dot.ktav`                    | `.` separating dotted key segments         |
| `punctuation.separator.key-value.ktav`             | The `:` of a plain pair                    |
| `keyword.operator.marker.raw.ktav`                 | `::` (raw-string marker)                   |
| `keyword.operator.marker.integer.ktav`             | `:i` (typed-Integer marker)                |
| `keyword.operator.marker.float.ktav`               | `:f` (typed-Float marker)                  |
| `constant.language.ktav`                           | `null`, `true`, `false` scalars            |
| `constant.numeric.integer.ktav`                    | Body after `:i`                            |
| `constant.numeric.float.ktav`                      | Body after `:f`                            |
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
  `pair-raw` (`::`) → `pair-integer` (`:i`) → `pair-float` (`:f`) →
  empty/open compound and multi-line forms → `pair-string` (`:`
  fallback). The `:i` / `:f` regexes use a `(?=\s|$)` lookahead so
  they cannot grab a key whose name happens to end in `i` or `f`
  (the marker only fires when followed by whitespace or EOL, matching
  the spec's mandatory-space rule, § 5.3 / § 6.10).
- Compound closers are anchored to standalone lines: `^\s*\)\s*$`,
  `^\s*\)\)\s*$`, `^\s*\}\s*$`, `^\s*\]\s*$`. A line like `) x` or
  `))suffix` does not close the block — it is content (in a multi-line
  string) or a syntax error (in an Object/Array context, which the
  grammar leaves unhighlighted).
- Array context is tracked through dedicated `array-*` repository
  rules included only inside `[ … ]` regions, so item-form lines
  (`:: foo`, `:i 42`, `:f 3.14`, bare scalars) light up only there.
- Multi-line string content is highlighted as a single string scope —
  no inner classification is applied, matching the spec's "raw
  content" semantics (§ 5.6).

## Known limitations

- Static syntax highlighting only. The grammar does not enforce
  semantic rules (duplicate-name detection, path conflicts, typed
  scalar body validation beyond the regex shape, dotted-key expansion,
  empty-key checks). Those belong to a parser/linter.
- Inline `#` is *not* a comment per the spec, and the grammar honors
  that. A `#` inside a value body is highlighted as part of the string.
- Inside multi-line strings, lines that look like `key: value` are
  **not** parsed as pairs — the `contentName` covers the whole region
  with one string scope. This matches the spec but means a misplaced
  `)` / `))` (one that isn't actually the closer the spec considers
  it) may visually break out of the string region. Authors who need
  to embed both `)` and `))` lines in the same value must split the
  string per § 5.6.1.
- The grammar uses regex-based line classification, not a real parser.
  Pathological inputs (e.g. a key segment with mid-segment `.` that
  would, by the strict grammar, not be a separator) are tokenized
  as if every `.` were a separator. This matches every implementation
  in the wild and is the only reasonable visual behavior.
