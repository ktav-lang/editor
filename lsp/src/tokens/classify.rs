//! The line classifier: [`classify_line`] turns one raw line into a
//! [`LineKind`], mirroring `ktav::parser`'s line-shape rules.

use super::key_paths::find_key_separator;
use super::kinds::{LineKind, Marker, ValueKind};

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

    // key: ... line — colon must exist for a Pair. Spec 0.6.0: the colon
    // counts only when UNESCAPED (a preceding lone `\` escapes it). Spec
    // 0.7.0: a colon inside a quoted key segment is opaque, not a
    // separator (§ 5.3.3).
    let Some(colon_rel) = find_key_separator(trimmed) else {
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
pub(crate) fn looks_numeric(s: &str) -> bool {
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
