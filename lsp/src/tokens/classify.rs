//! The line classifier: [`classify_line`] turns one raw line into a
//! [`LineKind`], mirroring `ktav::parser`'s line-shape rules.

use super::kinds::{LineKind, Marker, ValueKind};
use super::scan::find_key_separator;

/// § 3.3 whitespace — the exact twenty-five-code-point closed set. Never
/// delegate to `char::is_whitespace`, even though the two happen to agree
/// today: the spec pins this list independent of Unicode's own evolving
/// `White_Space` property, and the spec explicitly forbids relying on that
/// coincidence.
pub(crate) fn is_ktav_ws(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

/// Tokenize a single line. `line` is the raw text without trailing `\n`.
pub fn classify_line(raw: &str) -> LineKind<'_> {
    classify_line_in_scope(raw, true)
}

pub(super) fn classify_array_line(raw: &str) -> LineKind<'_> {
    classify_line_in_scope(raw, false)
}

fn classify_line_in_scope(raw: &str, pairs: bool) -> LineKind<'_> {
    let trimmed_start = raw.trim_start_matches(is_ktav_ws);
    let leading_ws = raw.len() - trimmed_start.len();
    let trimmed = trim_trailing_ws(trimmed_start);
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

    // Outside a multiline body, `)` is an ordinary Array String.
    if trimmed.len() == 1
        && (matches!(trimmed.as_bytes()[0], b'}' | b']') || pairs && trimmed == ")")
    {
        return LineKind::CloseBrace {
            start: leading_ws as u32,
        };
    }

    // `::` array item.
    if let Some(after) = trimmed.strip_prefix("::") {
        let body_offset = leading_ws + 2;
        let body = after;
        let inner_ws = body.len() - body.trim_start_matches(is_ktav_ws).len();
        let value = body.trim_start_matches(is_ktav_ws);
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

    // key: ... line — colon must exist for a Pair. Spec 0.6.0: the colon
    // counts only when UNESCAPED (a preceding lone `\` escapes it). Spec
    // 0.7.0: a colon inside a quoted key segment is opaque, not a
    // separator (§ 5.3.3).
    let Some(colon_rel) = pairs.then(|| find_key_separator(trimmed)).flatten() else {
        // Bare scalar item line (inside an array). Per § 4's
        // `<item-value> ::= <value-start> <line-end>`, a lone `(` / `((`
        // here dispatches through the same `<value-start>` production as
        // a pair's value — it opens a multi-line string body, not a
        // String scalar spelled "(".
        let kind = if matches!(trimmed, "(" | "((" | "()" | "(())") {
            ValueKind::CompoundOpen
        } else {
            classify_value(trimmed)
        };
        return LineKind::ArrayItem {
            start: leading_ws as u32,
            length: trimmed.len() as u32,
            kind,
        };
    };

    let key = &trimmed[..colon_rel];
    let after_colon = &trimmed[colon_rel + 1..];
    let marker = classify_marker(after_colon);

    let marker_byte_len = marker.len();
    let body = &trimmed[colon_rel + marker_byte_len..];

    let body_offset = leading_ws + colon_rel + marker_byte_len;
    let inner_ws = body.len() - body.trim_start_matches(is_ktav_ws).len();
    let value = body.trim_start_matches(is_ktav_ws);

    // § 4: `::` (Raw) is a dedicated raw-scalar production — its body is
    // ALWAYS a literal String, never dispatched through compound-opener
    // or value-shape rules. `key:: ((` is the two-byte literal string
    // "((", not a multi-line opener; checking the compound-opener shape
    // before the marker (as a prior version of this did) misclassified
    // every Raw pair whose body happened to spell out `{`, `[`, `(`,
    // `((`, `{}`, `[]` or `()`.
    let value_kind = if value.is_empty() {
        // Default to String — UI never reads it when length==0.
        ValueKind::String
    } else if marker == Marker::Raw {
        ValueKind::String
    } else if matches!(value, "{" | "[" | "(" | "((" | "{}" | "[]" | "()" | "(())") {
        ValueKind::CompoundOpen
    } else {
        classify_value(value)
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

/// Classify a bare value's surface kind, per § 5.2 rules 10–15 (rules 1–9,
/// the multi-line and inline-compound openers, are dispatched before this
/// function is ever reached — see `classify_line` and
/// `analysis::semantic::starts_inline_compound`). Keywords are matched
/// case-sensitively; numeric literals follow § 3.6's grammar plus rule
/// 13/14's redundant-leading-zero exception (see [`looks_numeric`]).
pub fn classify_value(v: &str) -> ValueKind {
    match v {
        "null" => ValueKind::Null,
        "true" | "false" => ValueKind::Bool,
        _ if looks_numeric(v) => ValueKind::Number,
        _ => ValueKind::String,
    }
}

/// § 5.2 rules 13–14: exact integer-or-float literal check (§ 3.6 grammar)
/// with the redundant-leading-zero exception applied. Highlighting only —
/// unlike the reference parser this does NOT enforce the i64 range or
/// Float finiteness: an overflowing literal still highlights as a number,
/// consistent with tree-sitter-ktav's `advance_number`, which has no
/// magnitude check either (domain checks are the parser's job, not the
/// editor's).
pub(crate) fn looks_numeric(s: &str) -> bool {
    if s.is_empty() || has_redundant_leading_zero(s) {
        return false;
    }
    matches_integer_grammar(s) || matches_float_grammar(s)
}

/// § 5.2's redundant-leading-zero exception: a base-10 digit run whose
/// first digit is `0` while at least one further digit or underscore
/// follows, sign optional (`01234`, `-045`, `00`, `0_7`). NOT redundant:
/// `0x1A` / `0o755` / `0b1010` (the `0` belongs to the base prefix), a
/// lone `0`, and `0.5` / `0e3` (the byte after `0` is `.` / `e`, never
/// another digit).
fn has_redundant_leading_zero(s: &str) -> bool {
    let bytes = s.as_bytes();
    let start = match bytes.first() {
        Some(b'+') | Some(b'-') => 1,
        Some(_) => 0,
        None => return false,
    };
    if bytes.get(start) != Some(&b'0') {
        return false;
    }
    matches!(bytes.get(start + 1), Some(b) if b.is_ascii_digit() || *b == b'_')
}

/// § 3.6 integer grammar: `sign? (hex | oct | bin | dec)`. Base prefixes
/// are lowercase-only (`0x` / `0o` / `0b`) per the grammar's literal
/// terminals — `0X1A` does not match. Underscore separators are allowed
/// only BETWEEN two digits: never leading, trailing, doubled, or directly
/// after the base prefix.
fn matches_integer_grammar(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    if i >= bytes.len() {
        return false;
    }
    if bytes[i] == b'0' && i + 1 < bytes.len() {
        match bytes[i + 1] {
            b'x' => return digit_run(&bytes[i + 2..], |b| b.is_ascii_hexdigit()),
            b'o' => return digit_run(&bytes[i + 2..], |b| matches!(b, b'0'..=b'7')),
            b'b' => return digit_run(&bytes[i + 2..], |b| matches!(b, b'0' | b'1')),
            _ => {}
        }
    }
    digit_run(&bytes[i..], |b| b.is_ascii_digit())
}

/// A full `dec_digit (("_")? dec_digit)*`-shaped run that consumes EVERY
/// byte of `digits`: the first byte must be a digit and each `_` must sit
/// strictly between two digits (no leading, trailing or doubled `_`).
fn digit_run(digits: &[u8], is_digit: impl Fn(u8) -> bool) -> bool {
    if digits.is_empty() || !is_digit(digits[0]) {
        return false;
    }
    let mut prev_underscore = false;
    for &b in &digits[1..] {
        if b == b'_' {
            if prev_underscore {
                return false;
            }
            prev_underscore = true;
        } else if is_digit(b) {
            prev_underscore = false;
        } else {
            return false;
        }
    }
    !prev_underscore
}

/// § 3.6 float grammar: `sign? dec_part "." dec_part exponent?` or
/// `sign? dec_part exponent` (`exponent ::= ("e"|"E") sign? dec_part`). A
/// pure digit run with neither `.` nor exponent is an integer, not a
/// float, and is rejected here (caller tries [`matches_integer_grammar`]
/// separately).
fn matches_float_grammar(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let (next, ok) = scan_dec_part(bytes, i);
    if !ok {
        return false;
    }
    i = next;
    if bytes.get(i) == Some(&b'.') {
        let (next, ok) = scan_dec_part(bytes, i + 1);
        if !ok {
            return false;
        }
        i = next;
        if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
            let (next, ok) = scan_exponent(bytes, i);
            if !ok {
                return false;
            }
            i = next;
        }
        return i == bytes.len();
    }
    if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
        let (next, ok) = scan_exponent(bytes, i);
        return ok && next == bytes.len();
    }
    false
}

/// Scan a `dec_part` starting at `i`: one-or-more digits with `_`
/// separators allowed only between two digits. Returns the index just
/// past the run and whether the run itself is well-formed (no leading,
/// trailing or doubled underscore up to the point it stopped) — full
/// consumption (`i == bytes.len()`) is the caller's separate check where
/// required.
fn scan_dec_part(bytes: &[u8], mut i: usize) -> (usize, bool) {
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return (i, false);
    }
    i += 1;
    let mut prev_underscore = false;
    while i < bytes.len() {
        match bytes[i] {
            b'_' if !prev_underscore => {
                prev_underscore = true;
                i += 1;
            }
            b'0'..=b'9' => {
                prev_underscore = false;
                i += 1;
            }
            _ => break,
        }
    }
    (i, !prev_underscore)
}

/// Scan `("e"|"E") sign? dec_part` starting at `i` (`bytes[i]` is already
/// known to be `e`/`E`).
fn scan_exponent(bytes: &[u8], mut i: usize) -> (usize, bool) {
    i += 1;
    if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    scan_dec_part(bytes, i)
}

fn trim_trailing_ws(s: &str) -> &str {
    s.trim_end_matches(is_ktav_ws)
}

/// Was a multi-line string body (§ 5.6) still open on entry to `text`'s
/// line `line_idx` (0-based)? True for every content line AND the
/// terminator line itself (both need special handling — a bare `key: `
/// content line or a `))` terminator line have no ordinary meaning);
/// false for the opener line (safe to classify normally, it is what sets
/// the state) and for any line outside a block.
///
/// Every consumer that classifies ONE line in isolation — hover,
/// completion — must check this before trusting [`classify_line`] on
/// that line. The document context shares scope-aware opener detection
/// with formatting and semantic tokens.
pub fn line_is_multiline_content(text: &str, line_idx: usize) -> bool {
    let mut context = super::DocumentContext::default();
    for (i, raw) in crate::lines::content_lines(text).into_iter().enumerate() {
        if i == line_idx {
            return context.in_multiline();
        }
        context.next_line(raw);
    }
    context.in_multiline()
}
