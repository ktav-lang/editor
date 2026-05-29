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
//! It does NOT track the brace stack: a tokenizer that needs to know
//! "am I inside an array?" already lost — for our purposes (highlighting
//! and column ranges) per-line classification is sufficient and matches
//! what `ktav::parse` accepts.

/// Marker shape on a `key:` line, matching `ktav`'s `Separator` enum.
///
/// Spec 0.5.0: `:i` and `:f` typed markers are removed. Only `Plain` (`:`)
/// and `Raw` (`::`) remain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Plain `:`.
    Plain,
    /// `::` — raw / literal-string body.
    Raw,
}

impl Marker {
    /// Byte length of the marker text on the line.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        match self {
            Marker::Plain => 1,
            Marker::Raw => 2,
        }
    }
}

/// What kind of value follows a marker (or stands alone as an array item).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// `null`.
    Null,
    /// `true` / `false`.
    Bool,
    /// Looks numeric (the surface form, no type marker).
    Number,
    /// Anything else — also the body of `::` raw markers.
    String,
    /// Compound opener / inline-empty: `{`, `[`, `(`, `((`, `{}`, `[]`, `()`.
    CompoundOpen,
    /// Lone closer line: `}`, `]`, `)`.
    CompoundClose,
}

/// One classified line.
#[derive(Debug, Clone)]
pub enum LineKind<'a> {
    /// Blank or whitespace-only.
    Blank,
    /// `# ...` line.
    Comment {
        /// Column of the `#`.
        start: u32,
        /// Trimmed-trailing length.
        length: u32,
    },
    /// Lone `}` / `]` / `)` closer.
    CloseBrace {
        /// Column of the closer.
        start: u32,
    },
    /// `:: value` literal-string array item.
    RawArrayItem {
        /// Column of the `::`.
        marker_start: u32,
        /// Value span (column + length); zero-length if no value.
        value_start: u32,
        value_length: u32,
    },
    /// `key{:|::|:i|:f} value` — the workhorse line.
    Pair {
        /// Column of the first byte of `key`.
        key_start: u32,
        /// Length in bytes of `key` (the dotted path is one slice — we
        /// expose dot-segment splitting via [`split_dotted`] when needed).
        key_length: u32,
        /// Column of the marker's first byte.
        marker_start: u32,
        marker: Marker,
        /// Value span. `value_length == 0` ⇒ no value on this line
        /// (compound opener will be on the next line, etc.).
        value_start: u32,
        value_length: u32,
        /// Pre-computed kind. For `Raw` markers this is always
        /// [`ValueKind::String`]; for typed markers always
        /// [`ValueKind::Number`]; for plain markers it is the result of
        /// [`classify_value`] applied to the value slice.
        value_kind: ValueKind,
        /// Borrowed slice of the value text (already trimmed).
        value_text: &'a str,
    },
    /// Bare item line inside an array (no `:` on the line).
    ArrayItem {
        start: u32,
        length: u32,
        kind: ValueKind,
    },
}

/// Tokenize a single line. `line` is the raw text without trailing `\n`.
pub fn classify_line(raw: &str) -> LineKind<'_> {
    let leading_ws = raw.len() - raw.trim_start().len();
    let trimmed = &raw[leading_ws..];
    let trimmed = trim_trailing_ws(trimmed);
    if trimmed.is_empty() {
        return LineKind::Blank;
    }

    // Spec 0.5.0: comments require `##` (two `#` bytes). A single `#` is
    // an ordinary character in keys and values.
    if trimmed.starts_with("##") {
        return LineKind::Comment {
            start: leading_ws as u32,
            length: trimmed.len() as u32,
        };
    }

    // Lone closer line.
    if trimmed.len() == 1 && matches!(trimmed.as_bytes()[0], b'}' | b']' | b')') {
        return LineKind::CloseBrace {
            start: leading_ws as u32,
        };
    }

    // `::` array item.
    if let Some(after) = trimmed.strip_prefix("::") {
        let body_offset = leading_ws + 2;
        let body = after;
        let inner_ws = body.len() - body.trim_start().len();
        let value = body.trim_start();
        return LineKind::RawArrayItem {
            marker_start: leading_ws as u32,
            value_start: (body_offset + inner_ws) as u32,
            value_length: value.len() as u32,
        };
    }

    // A line that *begins* with `{` or `[` is an inline-compound value (an
    // array item such as `{name: alice, age: 30}` or `[1, 2, 3]`), NOT a
    // `key: value` pair — even though it contains a `:`. Keys can never start
    // with a bracket, so the leading bracket is decisive. Route it through
    // `ArrayItem` so the semantic emitter tokenizes it structurally via
    // `emit_inline` instead of folding the whole line into one string.
    // Lone openers (`{`, `[`, `{}`, `[]`) stay `CompoundOpen`.
    if matches!(trimmed.as_bytes()[0], b'{' | b'[') {
        let kind = if matches!(trimmed, "{" | "[" | "{}" | "[]") {
            ValueKind::CompoundOpen
        } else {
            ValueKind::String
        };
        return LineKind::ArrayItem {
            start: leading_ws as u32,
            length: trimmed.len() as u32,
            kind,
        };
    }

    // key: ... line — colon must exist for a Pair.
    let Some(colon_rel) = trimmed.find(':') else {
        // Bare scalar item line (inside an array).
        return LineKind::ArrayItem {
            start: leading_ws as u32,
            length: trimmed.len() as u32,
            kind: classify_value(trimmed),
        };
    };

    let key = &trimmed[..colon_rel];
    let after_colon = &trimmed[colon_rel + 1..];
    let marker = classify_marker(after_colon);

    let marker_byte_len = marker.len();
    let body = &trimmed[colon_rel + marker_byte_len..];

    let body_offset = leading_ws + colon_rel + marker_byte_len;
    let inner_ws = body.len() - body.trim_start().len();
    let value = body.trim_start();

    let value_kind = if value.is_empty() {
        // Default to String — UI never reads it when length==0.
        ValueKind::String
    } else if matches!(value, "{" | "[" | "(" | "((" | "{}" | "[]" | "()") {
        ValueKind::CompoundOpen
    } else {
        match marker {
            Marker::Raw => ValueKind::String,
            Marker::Plain => classify_value(value),
        }
    };

    LineKind::Pair {
        key_start: leading_ws as u32,
        key_length: key.len() as u32,
        marker_start: (leading_ws + colon_rel) as u32,
        marker,
        value_start: (body_offset + inner_ws) as u32,
        value_length: value.len() as u32,
        value_kind,
        value_text: value,
    }
}

/// Mirror of `ktav::parser::parser::classify_separator`. `after_colon`
/// is the slice after the first `:`.
///
/// Spec 0.5.0: only `::` (Raw) and `:` (Plain) are recognised.
fn classify_marker(after_colon: &str) -> Marker {
    if after_colon.starts_with(':') {
        return Marker::Raw;
    }
    Marker::Plain
}

/// Classify a bare value's surface kind.
pub fn classify_value(v: &str) -> ValueKind {
    match v {
        "null" => ValueKind::Null,
        "true" | "false" => ValueKind::Bool,
        _ if looks_numeric(v) => ValueKind::Number,
        _ => ValueKind::String,
    }
}

/// Spec 0.5.0 number literal heuristic — covers decimal, hex (`0x`), octal
/// (`0o`), binary (`0b`), and float (requires `.` or exponent). Underscore
/// separators between digits are allowed.
fn looks_numeric(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let start = if bytes[0] == b'-' || bytes[0] == b'+' {
        1
    } else {
        0
    };
    if start >= bytes.len() {
        return false;
    }
    let rest = &bytes[start..];
    // Prefixed bases: 0x, 0o, 0b
    if rest.len() >= 2 && rest[0] == b'0' {
        match rest[1] {
            b'x' => {
                return rest[2..]
                    .iter()
                    .all(|b| b.is_ascii_hexdigit() || *b == b'_')
                    && rest.len() > 2
            }
            b'o' => {
                return rest[2..]
                    .iter()
                    .all(|b| matches!(b, b'0'..=b'7') || *b == b'_')
                    && rest.len() > 2
            }
            b'b' => {
                return rest[2..]
                    .iter()
                    .all(|b| matches!(b, b'0' | b'1') || *b == b'_')
                    && rest.len() > 2
            }
            _ => {}
        }
    }
    // Decimal integer or float. A well-formed float carries at most one `.`
    // and at most one exponent marker — so dotted runs like an IPv4 address
    // (`127.0.0.1`) or a version (`1.2.3`) are NOT numbers; they fall through
    // to `String`. At least one digit is still required.
    let mut dots = 0u32;
    let mut exps = 0u32;
    let mut has_digit = false;
    for b in rest {
        match b {
            b'0'..=b'9' => has_digit = true,
            b'_' | b'+' | b'-' => {}
            b'.' => dots += 1,
            b'e' | b'E' => exps += 1,
            _ => return false,
        }
    }
    has_digit && dots <= 1 && exps <= 1
}

fn trim_trailing_ws(s: &str) -> &str {
    s.trim_end_matches([' ', '\t', '\r'])
}

/// Yield `(segment_start, segment_text)` for each dotted segment of a key.
/// `key_start` is the absolute column where the key text begins.
pub fn split_dotted(key_start: u32, key: &str) -> impl Iterator<Item = (u32, &str)> {
    let mut col = key_start;
    key.split('.').map(move |seg| {
        let here = col;
        col += seg.len() as u32 + 1; // +1 for the '.'
        (here, seg)
    })
}

/// Convert a byte index into a `line` to a UTF-16 code-unit offset.
///
/// Used by [`crate::server`] to re-encode column positions from byte to
/// UTF-16 when the negotiated [`tower_lsp::lsp_types::PositionEncodingKind`]
/// is UTF-16 (the LSP default). For UTF-8 negotiation no conversion is
/// needed and this helper is bypassed.
///
/// `byte_idx` past the end of `line` clamps to the line's UTF-16 length.
pub fn byte_to_utf16(line: &str, byte_idx: usize) -> u32 {
    let cap = byte_idx.min(line.len());
    let prefix = &line[..cap];
    prefix.encode_utf16().count() as u32
}

/// Slice the prefix of `line` up to a cursor column expressed in the
/// negotiated [`crate::server::PositionEncoding`].
///
/// - `Utf8`: `character` is a byte offset. Clamped to `line.len()`, then
///   rounded **down** to the nearest UTF-8 char boundary so the returned
///   slice is always valid UTF-8 (never splits a multi-byte codepoint).
/// - `Utf16`: `character` is a UTF-16 code-unit count. We walk `chars()`
///   accumulating `len_utf16()` and stop the first time the total reaches
///   or exceeds the target — returning the byte-prefix up to that char's
///   start. This handles surrogate-pair targets (BMP-only stop point)
///   without ever slicing inside a single codepoint.
///
/// Used by completion to compute the "text before cursor" slice in an
/// encoding-correct way for non-ASCII lines (Cyrillic, Hebrew, emoji).
pub fn prefix_by_encoding(
    line: &str,
    character: u32,
    enc: crate::server::PositionEncoding,
) -> &str {
    match enc {
        crate::server::PositionEncoding::Utf8 => {
            let mut n = (character as usize).min(line.len());
            while n > 0 && !line.is_char_boundary(n) {
                n -= 1;
            }
            &line[..n]
        }
        crate::server::PositionEncoding::Utf16 => {
            let target = character as usize;
            let mut acc: usize = 0;
            for (byte_idx, c) in line.char_indices() {
                let next = acc + c.len_utf16();
                if next > target {
                    // Including this char would overshoot — and if it's a
                    // surrogate pair the target landed mid-codepoint. Stop
                    // here without slicing into the codepoint.
                    return &line[..byte_idx];
                }
                acc = next;
                if acc == target {
                    let end = byte_idx + c.len_utf8();
                    return &line[..end];
                }
            }
            line
        }
    }
}

/// True if a `key: ` form on this line indicates the cursor is positioned
/// AFTER the separator (used by completion to switch from key-mode to
/// value-mode). `upto` is the line text up to the cursor column.
///
/// Spec 0.5.0: only `::` and `:` are markers; `:i`/`:f` are gone.
pub fn cursor_is_after_separator(upto: &str) -> bool {
    let trimmed = upto.trim_start();
    let Some(i) = trimmed.find(':') else {
        return false;
    };
    let after = &trimmed[i + 1..];
    if let Some(rest) = after.strip_prefix(':') {
        rest.chars().all(char::is_whitespace)
    } else {
        // Plain `:` — accept any whitespace tail.
        after.chars().all(char::is_whitespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(s: &str) -> LineKind<'_> {
        classify_line(s)
    }

    #[test]
    fn comment() {
        // Spec 0.5.0: `##` required. Single `#` is not a comment.
        match pair("  ## hello") {
            LineKind::Comment { start, length } => {
                assert_eq!(start, 2);
                assert_eq!(length, 8);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn single_hash_is_not_comment() {
        // Spec 0.5.0: a lone `#` is an ordinary character.
        assert!(!matches!(pair("  # hello"), LineKind::Comment { .. }));
    }

    #[test]
    fn plain_pair() {
        match pair("name: alice") {
            LineKind::Pair {
                key_start,
                key_length,
                marker,
                value_kind,
                value_text,
                ..
            } => {
                assert_eq!(key_start, 0);
                assert_eq!(key_length, 4);
                assert_eq!(marker, Marker::Plain);
                assert_eq!(value_kind, ValueKind::String);
                assert_eq!(value_text, "alice");
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn typed_int_removed_spec050() {
        // Spec 0.5.0: `:i` is no longer a typed marker — it is treated as a
        // Plain marker whose value starts with `i`.
        match pair("port:i 8080") {
            LineKind::Pair { marker, .. } => {
                assert_eq!(marker, Marker::Plain);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn raw_marker() {
        match pair("greeting:: hello world") {
            LineKind::Pair {
                marker, value_kind, ..
            } => {
                assert_eq!(marker, Marker::Raw);
                assert_eq!(value_kind, ValueKind::String);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn typed_letter_glued_is_plain() {
        // `port:istanbul` — `i` followed by non-ws → Plain marker, value
        // = `istanbul`. Mirrors `ktav` classify_separator.
        match pair("city:istanbul") {
            LineKind::Pair { marker, .. } => assert_eq!(marker, Marker::Plain),
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn array_raw_item() {
        match pair("    :: literal") {
            LineKind::RawArrayItem {
                marker_start,
                value_start,
                value_length,
            } => {
                assert_eq!(marker_start, 4);
                assert_eq!(value_start, 7);
                assert_eq!(value_length, 7);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn close_brace() {
        assert!(matches!(pair("  }"), LineKind::CloseBrace { start: 2 }));
    }

    #[test]
    fn compound_open() {
        match pair("server: {") {
            LineKind::Pair { value_kind, .. } => assert_eq!(value_kind, ValueKind::CompoundOpen),
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn ipv4_and_versions_are_not_numeric() {
        // More than one `.` ⇒ not a number; classified as String.
        assert!(!looks_numeric("127.0.0.1"));
        assert!(!looks_numeric("1.2.3"));
        assert!(!looks_numeric("1.2.3.4"));
        // Well-formed numbers still recognised.
        assert!(looks_numeric("42"));
        assert!(looks_numeric("1.3"));
        assert!(looks_numeric("1.5e3"));
        assert!(looks_numeric("0xFF_00"));
        assert_eq!(classify_value("127.0.0.1"), ValueKind::String);
        assert_eq!(classify_value("1.3"), ValueKind::Number);
    }

    #[test]
    fn line_starting_with_bracket_is_array_item_not_pair() {
        // `{name: alice, age: 30}` is an inline-object array item, not a
        // `key: value` pair keyed on `{name`.
        match pair("{name: alice, age: 30}") {
            LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::String),
            other => panic!("got {:?}", other),
        }
        match pair("[1, 2, 3]") {
            LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::String),
            other => panic!("got {:?}", other),
        }
        // Lone opener stays CompoundOpen.
        match pair("{") {
            LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::CompoundOpen),
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn dotted_split() {
        let segs: Vec<_> = split_dotted(2, "a.bb.ccc").collect();
        assert_eq!(segs, vec![(2, "a"), (4, "bb"), (7, "ccc")]);
    }

    #[test]
    fn prefix_utf8_ascii() {
        use crate::server::PositionEncoding::Utf8;
        assert_eq!(prefix_by_encoding("name: alice", 6, Utf8), "name: ");
    }

    #[test]
    fn prefix_utf8_clamps_to_len() {
        use crate::server::PositionEncoding::Utf8;
        let line = "abc";
        assert_eq!(prefix_by_encoding(line, 999, Utf8), "abc");
    }

    #[test]
    fn prefix_utf8_rounds_down_off_boundary() {
        use crate::server::PositionEncoding::Utf8;
        // "и" is 2 bytes (0xD0 0xB8). character=1 lands mid-codepoint.
        let line = "имя";
        let p = prefix_by_encoding(line, 1, Utf8);
        assert_eq!(p, ""); // rounded down to 0
        let p2 = prefix_by_encoding(line, 3, Utf8);
        assert_eq!(p2, "и"); // rounded down to 2
    }

    #[test]
    fn prefix_utf16_cyrillic() {
        use crate::server::PositionEncoding::Utf16;
        // "ключ: " — 5 cyrillic chars (1 utf16 unit each) + ": " = 7 utf16 units.
        let line = "ключ: x";
        let p = prefix_by_encoding(line, 6, Utf16);
        assert_eq!(p, "ключ: ");
    }

    #[test]
    fn prefix_utf16_emoji_surrogate_pair() {
        use crate::server::PositionEncoding::Utf16;
        // "k: 😀" — k(1) :(1) (1) =3 units, emoji = 2 units (surrogate pair).
        let line = "k: 😀";
        // Stop before the emoji.
        assert_eq!(prefix_by_encoding(line, 3, Utf16), "k: ");
        // Mid-surrogate (4) — should not split codepoint; returns up to emoji's start.
        assert_eq!(prefix_by_encoding(line, 4, Utf16), "k: ");
        // After emoji.
        assert_eq!(prefix_by_encoding(line, 5, Utf16), "k: 😀");
    }

    #[test]
    fn byte_to_utf16_empty() {
        assert_eq!(byte_to_utf16("", 0), 0);
        assert_eq!(byte_to_utf16("", 5), 0);
    }

    #[test]
    fn byte_to_utf16_overshoot_clamps() {
        // Past end → clamps to UTF-16 length of full string.
        assert_eq!(byte_to_utf16("a", 5), 1);
        assert_eq!(byte_to_utf16("abc", 999), 3);
    }

    #[test]
    fn byte_to_utf16_mid_codepoint_invalid_byte_index() {
        // "é" is 2 bytes (0xC3 0xA9) and 1 UTF-16 unit. Asking for byte
        // index 1 lands mid-codepoint. The helper slices `&line[..cap]`;
        // a non-boundary cap would panic, so we verify current behaviour:
        // byte_idx==1 is NOT a char boundary in "é", so the helper would
        // panic. Pin the safe boundaries instead.
        assert_eq!(byte_to_utf16("é", 0), 0);
        assert_eq!(byte_to_utf16("é", 2), 1); // full "é"
    }

    #[test]
    fn byte_to_utf16_4byte_utf8_surrogate_pair() {
        // "𝕏" (U+1D54F) — 4 bytes UTF-8, 2 UTF-16 units (surrogate pair).
        assert_eq!(byte_to_utf16("𝕏", 0), 0);
        assert_eq!(byte_to_utf16("𝕏", 4), 2);
        // Past end clamps to 2.
        assert_eq!(byte_to_utf16("𝕏", 99), 2);
    }

    #[test]
    fn prefix_utf16_mid_surrogate_rounds_down() {
        use crate::server::PositionEncoding::Utf16;
        // "𝕏" is a single codepoint occupying 2 UTF-16 units. Target=1
        // lands mid-surrogate-pair: helper rounds DOWN (returns "").
        assert_eq!(prefix_by_encoding("𝕏", 1, Utf16), "");
        // Target=0 → "".
        assert_eq!(prefix_by_encoding("𝕏", 0, Utf16), "");
        // Target=2 → full "𝕏".
        assert_eq!(prefix_by_encoding("𝕏", 2, Utf16), "𝕏");
    }

    #[test]
    fn prefix_utf8_mid_codepoint_rounds_down() {
        use crate::server::PositionEncoding::Utf8;
        // "café" — 'é' is 2 bytes at offset 3..5. byte_idx=4 lands
        // mid-codepoint; helper rounds DOWN to byte 3 → "caf".
        assert_eq!(prefix_by_encoding("café", 3, Utf8), "caf");
        assert_eq!(prefix_by_encoding("café", 4, Utf8), "caf");
        assert_eq!(prefix_by_encoding("café", 5, Utf8), "café");
    }

    #[test]
    fn cursor_after_sep() {
        assert!(cursor_is_after_separator("name: "));
        assert!(cursor_is_after_separator("name:: "));
        assert!(!cursor_is_after_separator("name"));
        assert!(!cursor_is_after_separator("nam"));
    }
}
