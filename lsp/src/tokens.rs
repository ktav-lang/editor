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
//! (`# comment`, `key: value`, `key:: value`, `key:i N`, `key:f N`,
//! `:: value` array literal-string item, lone `}` / `]` / `)` closers,
//! compound openers `{` `[` `(` `((` `{}` `[]` `()`).
//!
//! It does NOT track the brace stack: a tokenizer that needs to know
//! "am I inside an array?" already lost — for our purposes (highlighting
//! and column ranges) per-line classification is sufficient and matches
//! what `ktav::parse` accepts.

/// Marker shape on a `key:` line, matching `ktav`'s `Separator` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Plain `:`.
    Plain,
    /// `::` — raw / literal-string body.
    Raw,
    /// `:i` — typed integer body.
    TypedInt,
    /// `:f` — typed float body.
    TypedFloat,
}

impl Marker {
    /// Byte length of the marker text on the line.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> usize {
        match self {
            Marker::Plain => 1,
            _ => 2,
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

    if trimmed.starts_with('#') {
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
            Marker::TypedInt | Marker::TypedFloat => ValueKind::Number,
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
fn classify_marker(after_colon: &str) -> Marker {
    if after_colon.starts_with(':') {
        return Marker::Raw;
    }
    if let Some(rest) = after_colon.strip_prefix('i') {
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            return Marker::TypedInt;
        }
    }
    if let Some(rest) = after_colon.strip_prefix('f') {
        if rest.is_empty() || rest.starts_with(char::is_whitespace) {
            return Marker::TypedFloat;
        }
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
    bytes[start..].iter().all(|b| {
        b.is_ascii_digit() || *b == b'.' || *b == b'e' || *b == b'E' || *b == b'+' || *b == b'-'
    }) && bytes[start..].iter().any(|b| b.is_ascii_digit())
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
pub fn cursor_is_after_separator(upto: &str) -> bool {
    let trimmed = upto.trim_start();
    let Some(i) = trimmed.find(':') else {
        return false;
    };
    let after = &trimmed[i + 1..];
    if let Some(rest) = after.strip_prefix(':') {
        rest.chars().all(char::is_whitespace)
    } else {
        // Plain `:`, `:i`, `:f` — accept any whitespace tail (and the
        // typed-marker letter itself).
        after
            .chars()
            .all(|c| c == ' ' || c == '\t' || c == 'i' || c == 'f')
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
        match pair("  # hello") {
            LineKind::Comment { start, length } => {
                assert_eq!(start, 2);
                assert_eq!(length, 7);
            }
            other => panic!("got {:?}", other),
        }
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
    fn typed_int() {
        match pair("port:i 8080") {
            LineKind::Pair {
                marker, value_kind, ..
            } => {
                assert_eq!(marker, Marker::TypedInt);
                assert_eq!(value_kind, ValueKind::Number);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn typed_int_eol() {
        // `port:i` with no body — still a TypedInt marker (parser will
        // reject the empty body, but classification is the same).
        match pair("port:i") {
            LineKind::Pair { marker, .. } => assert_eq!(marker, Marker::TypedInt),
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
    fn cursor_after_sep() {
        assert!(cursor_is_after_separator("name: "));
        assert!(cursor_is_after_separator("name:: "));
        assert!(cursor_is_after_separator("name:i "));
        assert!(!cursor_is_after_separator("name"));
        assert!(!cursor_is_after_separator("nam"));
    }
}
