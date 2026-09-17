//! Single source of truth for Ktav line tokenization in the LSP.
//!
//! `ktav::parse` does not surface source spans, and we cannot afford to
//! disagree with it on edge cases (what counts as a typed-scalar marker,
//! how dotted keys are split, where the value text begins). This module
//! re-implements **the same line-shape rules** the `ktav` parser uses
//! (`classify_separator` / `require_sep_end` in `ktav::parser::parser`)
//! so that semantic tokens, hover, completion and diagnostic-range
//! tightening all share one classifier.
//!
//! The tokenizer is purely line-oriented — Ktav's grammar is too
//! (`## comment`, `key: value`, `key:: value`,
//! `:: value` array literal-string item, lone `}` / `]` / `)` closers,
//! compound openers `{` `[` `(` `((` `{}` `[]` `()`).
//!
//! Spec 0.5.0: typed markers `:i` and `:f` are removed; type is inferred
//! from the lexical form of the scalar. Comments now require `##` (two `#`
//! bytes); a single `#` is an ordinary character.
//!
//! Spec 0.6.0: keys process the full §3.7 escape set. `\.` keeps a literal
//! dot inside a key segment (does NOT split a dotted path), `\:` keeps a
//! literal colon inside a key (does NOT act as the key/value separator),
//! `\\` is a literal backslash, and any other `\x` from the §3.7 table
//! is an escape sequence in the key. The line classifier therefore finds
//! the first UNESCAPED `:` as the marker, and dotted-path segmentation
//! splits only on UNESCAPED `.`. The escape lead `\` is itself a regular
//! key character (highlight stays on the key scope; structured decoding
//! is the parser's job).
//!
//! Spec 0.7.0: a key segment may also be a `<quoted-segment>` (§ 5.3.3) —
//! `"`, `'` or `` ` `` opening at the FIRST code point of a segment (start
//! of key, or right after an unescaped `.`), running to the first
//! unescaped occurrence of that SAME character. While inside a quoted
//! segment, `:` / `.` / `,` / `{` / `}` / `[` / `]` are opaque ordinary
//! content — [`find_key_separator`] and [`split_dotted`] both track
//! segment-start position so a `:` or `.` inside `"a:b.c"` never splits
//! the separator search or the dotted path. A quote character NOT at a
//! segment's first position (`don't: 1`) is unaffected — same as before
//! 0.7.0.
//!
//! It does NOT track the brace stack: a tokenizer that needs to know
//! "am I inside an array?" already lost — for our purposes (highlighting
//! and column ranges) per-line classification is sufficient and matches
//! what `ktav::parse` accepts.

mod classify;
mod encoding;
mod key_paths;
mod kinds;

#[cfg(test)]
pub(crate) use classify::looks_numeric;
pub use classify::{classify_line, classify_value};
pub use encoding::{byte_to_utf16, prefix_by_encoding};
pub(crate) use key_paths::find_key_separator;
pub use key_paths::{cursor_is_after_separator, split_dotted};
pub use kinds::{LineKind, Marker, ValueKind};

#[cfg(test)]
mod tests;
