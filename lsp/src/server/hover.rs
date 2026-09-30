//! Hover support: resolving the `Value` a hovered `key:` line points at,
//! and rendering that value as hover markdown.
//!
//! `ktav::Value` carries no source spans, so finding "the value for the
//! key on this line" needs two things `lookup_dotted`'s old flat
//! `dotted.split('.')` against the root object did not have:
//!
//! 1. **The enclosing path.** A key nested inside `key: {` / `key: [`
//!    blocks (including array-of-objects items) is only reachable by
//!    walking the document from the top, tracking which compound is
//!    currently open — see [`enclosing_path`].
//! 2. **Decoded key segments.** A quoted segment (`"a.b"`) or an
//!    escaped one (`a\.b`) must be decoded per spec § 3.7 / § 5.3.3
//!    before it is looked up — `ktav::Value`'s `ObjectMap` keys are
//!    already decoded — see [`decode_segment`].
//!
//! Both are best-effort: any ambiguity or malformed intermediate line
//! falls back to `None` (the hover handler then shows the generic
//! "value" message). Never panics, never returns a mismatched `Value`.

use ktav::Value;

use crate::tokens::{classify_line, find_key_separator, split_dotted, LineKind, Marker, ValueKind};

/// One path segment: a decoded object key, or an array index.
enum Seg {
    Key(String),
    Index(usize),
}

/// What kind of compound a `key: X` / bare `X` opener line starts.
/// `Raw` is a multi-line **String** body (stripped `(` / verbatim
/// `((`) — not a Value-tree nesting level, just a scalar whose text
/// spans multiple source lines.
enum Opener {
    Object,
    Array,
    Raw(bool),
}

/// Structural replacement for a trailing-suffix opener check: classify
/// the WHOLE line via [`classify_line`] (the same classifier semantic
/// tokens / the reindenter use — see `reindent::opener_of`) and read off
/// its value's compound-opener shape, rather than pattern-matching the
/// line's tail. A Raw (`::`) marker's value is always a literal string
/// (§ 4) — never an opener, however it looks (`key:: ((` is the two-byte
/// string `"(("`, not a verbatim block); a value that merely ends in
/// `{`/`[`/`(`/`((` without BEING exactly that (a longer string, or an
/// escaped separator like `a\: {`) is not one either, unlike a suffix
/// match on the line's tail.
fn detect_opener(line: &str) -> Option<Opener> {
    let LineKind::Pair {
        marker,
        value_kind,
        value_text,
        ..
    } = classify_line(line)
    else {
        return None;
    };
    if marker == Marker::Raw || value_kind != ValueKind::CompoundOpen {
        return None;
    }
    match value_text {
        "{" => Some(Opener::Object),
        "[" => Some(Opener::Array),
        "(" => Some(Opener::Raw(false)),
        "((" => Some(Opener::Raw(true)),
        // "{}" / "[]" / "()" — empty inline value, nothing to nest into.
        _ => None,
    }
}

/// One entry on the scanner's open-compound stack.
enum Open {
    /// An open Object/Array. `seg_count` is how many `Seg`s to pop off
    /// the path when the matching lone `}`/`]` closer is seen.
    Compound {
        array: bool,
        seg_count: usize,
        next_index: usize,
    },
    /// An open multi-line String body. Pops `seg_count` `Seg`s (pushed
    /// by the key that introduced it, if any) when the terminator line
    /// (`)` / `))`) is seen. No index bookkeeping of its own.
    Raw { seg_count: usize },
}

/// Consume one item slot from whichever array is currently open (the
/// top-of-stack compound if it is an array, else the implicit root
/// array), returning the index assigned to that item.
fn bump_index(stack: &mut [Open], root_index: &mut usize) -> usize {
    if let Some(Open::Compound {
        array: true,
        next_index,
        ..
    }) = stack.last_mut()
    {
        let idx = *next_index;
        *next_index += 1;
        idx
    } else {
        let idx = *root_index;
        *root_index += 1;
        idx
    }
}

/// Scan `text` up to (not including) `line`, tracking open Object/
/// Array compounds, to compute the path from the document root down to
/// whatever is enclosing `line`. Returns `(path, in_array)`:
/// `in_array` is whether `line` itself sits directly inside an array
/// body — the caller bails out to the generic hover message in that
/// case, since a `key:`-shaped line there can only be the ambiguous
/// fully-inline array-item form (`{a: 1, b: 2}` as one bare item),
/// which cannot be resolved to a single key without guessing.
///
/// Returns `None` only if a key segment on an intervening line fails
/// to decode — defensive; does not happen against a document that
/// already parsed successfully via `ktav::parse`, since hover only
/// runs against the cached parse result.
fn enclosing_path(text: &str, line: usize, root_is_array: bool) -> Option<(Vec<Seg>, bool)> {
    let mut path: Vec<Seg> = Vec::new();
    let mut stack: Vec<Open> = Vec::new();
    // Some(verbatim) while inside an open multi-line String body.
    let mut multi: Option<bool> = None;
    let mut root_index: usize = 0;
    let mut first_content_line = true;

    for (i, raw_line) in crate::lines::content_lines(text).into_iter().enumerate() {
        if i >= line {
            break;
        }

        if let Some(verbatim) = multi {
            let t = raw_line.trim();
            let terminated = if verbatim { t == "))" } else { t == ")" };
            if terminated {
                multi = None;
                if let Some(Open::Raw { seg_count }) = stack.pop() {
                    let keep = path.len().saturating_sub(seg_count);
                    path.truncate(keep);
                }
            }
            continue;
        }

        let trimmed = raw_line.trim_start();
        // Spec 0.5.0: a comment is a LEADING `##` — a lone `#` is an
        // ordinary character (e.g. `#child: {` is a valid key, not a
        // comment). Delegate to the shared classifier so this never
        // disagrees with semantic tokens / the reindenter on what
        // counts as a comment.
        if trimmed.is_empty() || matches!(classify_line(trimmed), LineKind::Comment { .. }) {
            continue;
        }
        let tail = trimmed.trim_end();

        let in_array = match stack.last() {
            Some(Open::Compound { array, .. }) => *array,
            _ => root_is_array,
        };

        if first_content_line {
            first_content_line = false;
            if root_is_array && stack.is_empty() && tail == "[" {
                // Transparent top-level wrapper around a root Array's
                // own items (spec § 5.0.1) — not a nesting level.
                continue;
            }
        }

        if tail == "}" || tail == "]" {
            if let Some(Open::Compound { seg_count, .. }) = stack.pop() {
                let keep = path.len().saturating_sub(seg_count);
                path.truncate(keep);
            }
            continue;
        }

        if matches!(tail, "{" | "[" | "(" | "((") {
            if !in_array {
                // Malformed for a document that parsed successfully —
                // ignore defensively rather than guess.
                continue;
            }
            match tail {
                "{" => {
                    let idx = bump_index(&mut stack, &mut root_index);
                    path.push(Seg::Index(idx));
                    stack.push(Open::Compound {
                        array: false,
                        seg_count: 1,
                        next_index: 0,
                    });
                }
                "[" => {
                    let idx = bump_index(&mut stack, &mut root_index);
                    path.push(Seg::Index(idx));
                    stack.push(Open::Compound {
                        array: true,
                        seg_count: 1,
                        next_index: 0,
                    });
                }
                "(" => {
                    bump_index(&mut stack, &mut root_index);
                    stack.push(Open::Raw { seg_count: 0 });
                    multi = Some(false);
                }
                "((" => {
                    bump_index(&mut stack, &mut root_index);
                    stack.push(Open::Raw { seg_count: 0 });
                    multi = Some(true);
                }
                _ => unreachable!("matches! above restricts tail to these four forms"),
            }
            continue;
        }

        if in_array {
            // Bare scalar item, `:: literal` item, or a fully-inline
            // compound written on one line — the whole item lives on
            // this line; nothing carries forward to later lines.
            bump_index(&mut stack, &mut root_index);
            continue;
        }

        // Object context: this must be a pair line (arrays are the
        // only other body shape, handled above).
        let Some(colon) = find_key_separator(trimmed) else {
            continue;
        };
        let key_part = trimmed[..colon].trim_end();
        if key_part.is_empty() {
            continue;
        }
        let mut seg_texts: Vec<String> = Vec::new();
        for (_, seg) in split_dotted(0, key_part) {
            seg_texts.push(decode_segment(seg)?);
        }
        if seg_texts.is_empty() {
            continue;
        }
        let seg_count = seg_texts.len();
        for s in seg_texts {
            path.push(Seg::Key(s));
        }

        match detect_opener(trimmed) {
            Some(Opener::Object) => stack.push(Open::Compound {
                array: false,
                seg_count,
                next_index: 0,
            }),
            Some(Opener::Array) => stack.push(Open::Compound {
                array: true,
                seg_count,
                next_index: 0,
            }),
            Some(Opener::Raw(verbatim)) => {
                stack.push(Open::Raw { seg_count });
                multi = Some(verbatim);
            }
            None => {
                // Scalar, or a fully-inline compound value entirely on
                // this line — nothing nests under it for later lines.
                let keep = path.len().saturating_sub(seg_count);
                path.truncate(keep);
            }
        }
    }

    let in_array_at_target = match stack.last() {
        Some(Open::Compound { array, .. }) => *array,
        _ => root_is_array,
    };
    Some((path, in_array_at_target))
}

/// Walk `path` through `root`, matching Object keys against `Seg::Key`
/// and Array indices against `Seg::Index`. A kind mismatch or
/// out-of-range index is `None`, never a wrong value.
fn walk_path<'a>(root: &'a Value, path: &[Seg]) -> Option<&'a Value> {
    let mut cur = root;
    for seg in path {
        cur = match (cur, seg) {
            (Value::Object(map), Seg::Key(k)) => map.get(k.as_str())?,
            (Value::Array(items), Seg::Index(i)) => items.get(*i)?,
            _ => return None,
        };
    }
    Some(cur)
}

/// Decode one dotted-key segment (as returned by
/// [`crate::tokens::split_dotted`]) per spec § 3.7 (escapes) and
/// § 5.3.3 (quoting): trim the segment's outer whitespace, strip a
/// `"`/`'`/`` ` `` quoted segment's delimiters (content between them is
/// never trimmed), then decode escapes. `None` on a malformed escape —
/// should not happen against an already-parsed document, but hover
/// must never guess, so the caller falls back to the generic message.
fn decode_segment(raw: &str) -> Option<String> {
    let trimmed = raw.trim_matches(char::is_whitespace);
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    match bytes[0] {
        b'"' | b'\'' | b'`' => {
            let quote = bytes[0];
            let inner = &trimmed[1..];
            let ib = inner.as_bytes();
            let mut i = 0usize;
            let mut end = None;
            while i < ib.len() {
                if ib[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if ib[i] == quote {
                    end = Some(i);
                    break;
                }
                i += 1;
            }
            let content = match end {
                Some(e) => inner.get(..e)?,
                None => inner,
            };
            decode_escapes(content)
        }
        _ => decode_escapes(trimmed),
    }
}

/// Decode the fourteen § 3.7 escape sequences over `s`. `None` on any
/// unrecognised or malformed escape (`BadEscapeSequence`), including
/// an unpaired UTF-16 surrogate from `\uXXXX` (§ 3.7.1).
fn decode_escapes(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            let ch = s[i..].chars().next()?;
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let next = *bytes.get(i + 1)?;
        match next {
            b'\\' => {
                out.push('\\');
                i += 2;
            }
            b',' => {
                out.push(',');
                i += 2;
            }
            b'}' => {
                out.push('}');
                i += 2;
            }
            b']' => {
                out.push(']');
                i += 2;
            }
            b'{' => {
                out.push('{');
                i += 2;
            }
            b'[' => {
                out.push('[');
                i += 2;
            }
            b'n' => {
                out.push('\n');
                i += 2;
            }
            b'r' => {
                out.push('\r');
                i += 2;
            }
            b'.' => {
                out.push('.');
                i += 2;
            }
            b':' => {
                out.push(':');
                i += 2;
            }
            b'"' => {
                out.push('"');
                i += 2;
            }
            b'\'' => {
                out.push('\'');
                i += 2;
            }
            b'`' => {
                out.push('`');
                i += 2;
            }
            b'u' => {
                let unit = read_hex4(s, i + 2)?;
                if (0xD800..=0xDBFF).contains(&unit) {
                    // High surrogate: must be immediately followed by a
                    // low-surrogate `\uXXXX` (§ 3.7.1).
                    if bytes.get(i + 6) != Some(&b'\\') || bytes.get(i + 7) != Some(&b'u') {
                        return None;
                    }
                    let low = read_hex4(s, i + 8)?;
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        return None;
                    }
                    let cp = 0x10000 + (unit - 0xD800) * 0x400 + (low - 0xDC00);
                    out.push(char::from_u32(cp)?);
                    i += 12;
                } else if (0xDC00..=0xDFFF).contains(&unit) {
                    // Lone low surrogate.
                    return None;
                } else {
                    out.push(char::from_u32(unit)?);
                    i += 6;
                }
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Read exactly four ASCII hex digits at byte offset `at` in `s`.
fn read_hex4(s: &str, at: usize) -> Option<u32> {
    let slice = s.get(at..at + 4)?;
    if !slice.is_ascii() {
        return None;
    }
    u32::from_str_radix(slice, 16).ok()
}

/// Resolve the `Value` that the `key` on `text`'s line `line` points
/// at: decode `key`'s own dotted segments, prepend the enclosing path
/// computed by scanning the preceding lines, and walk `root`. `None`
/// on any ambiguity, mismatch, or decode failure — the hover handler
/// then shows the generic "value" message instead of a wrong one.
pub(super) fn resolve_value<'a>(
    root: &'a Value,
    text: &str,
    line: usize,
    key: &str,
) -> Option<&'a Value> {
    let root_is_array = matches!(root, Value::Array(_));
    let (mut full_path, in_array) = enclosing_path(text, line, root_is_array)?;
    if in_array {
        return None;
    }
    for (_, seg) in split_dotted(0, key) {
        full_path.push(Seg::Key(decode_segment(seg)?));
    }
    walk_path(root, &full_path)
}

pub(super) fn describe_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => format!("bool: `{}`", b),
        Value::Integer(s) => format!("integer: `{}`", s.as_str()),
        Value::Float(s) => format!("float: `{}`", s.as_str()),
        Value::String(s) => {
            // Truncate on a char boundary — a byte slice (`&s[..80]`)
            // panics whenever byte 80 lands inside a multi-byte code
            // point, and with `panic = "abort"` in release that kills
            // the whole server process on one hover request.
            let shown = if s.chars().count() > 80 {
                let cut: String = s.chars().take(80).collect();
                format!("{}…", cut)
            } else {
                s.to_string()
            };
            format!("string: `{}`", shown)
        }
        Value::Array(a) => format!("array of {} items", a.len()),
        Value::Object(o) => format!("object with {} keys", o.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_value_truncates_long_cyrillic_string_without_panicking() {
        // 90 Cyrillic letters = 180 bytes; byte 80 lands mid-character
        // for the old `&s[..80]` slice — this used to panic.
        let s: String = "я".repeat(90);
        let out = describe_value(&Value::String(s.clone().into()));
        assert!(out.starts_with("string: `"));
        assert!(out.contains('…'));
        // Exactly 80 chars kept, plus the ellipsis marker.
        let inner = out
            .strip_prefix("string: `")
            .and_then(|s| s.strip_suffix("`"))
            .unwrap();
        let kept = inner.strip_suffix('…').unwrap();
        assert_eq!(kept.chars().count(), 80);
        assert!(s.starts_with(kept));
    }

    #[test]
    fn describe_value_truncates_long_astral_emoji_string_without_panicking() {
        // Each 😀 is 4 UTF-8 bytes / 1 char (astral plane) — byte 80
        // also lands mid-character here, a second panic shape the old
        // code hit (surrogate-pair-width chars, not just 2-byte ones).
        let s: String = "😀".repeat(90);
        let out = describe_value(&Value::String(s.into()));
        assert!(out.starts_with("string: `"));
        assert!(out.contains('…'));
    }

    fn parse(text: &str) -> Value {
        ktav::parse(text).expect("fixture must parse")
    }

    #[test]
    fn resolve_top_level_key() {
        let text = "name: alice\nport: 8080\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 0, "name").expect("value");
        assert_eq!(describe_value(v), "string: `alice`");
    }

    #[test]
    fn resolve_nested_object_key() {
        let text = "server: {\n    port: 80\n}\n";
        let root = parse(text);
        // Line 1 is `    port: 80`.
        let v = resolve_value(&root, text, 1, "port").expect("value");
        assert_eq!(describe_value(v), "integer: `80`");
    }

    #[test]
    fn resolve_dotted_key_inside_nested_object() {
        let text = "server: {\n    net.port: 80\n}\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 1, "net.port").expect("value");
        assert_eq!(describe_value(v), "integer: `80`");
    }

    #[test]
    fn resolve_quoted_key_with_a_dot() {
        let text = "\"a.b\": 1\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 0, "\"a.b\"").expect("value");
        assert_eq!(describe_value(v), "integer: `1`");
    }

    #[test]
    fn resolve_escaped_dot_key() {
        let text = "a\\.b: 1\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 0, "a\\.b").expect("value");
        assert_eq!(describe_value(v), "integer: `1`");
    }

    #[test]
    fn resolve_array_of_objects_by_index() {
        let text = "items: [\n{\n    name: alice\n}\n{\n    name: bob\n}\n]\n";
        let root = parse(text);
        // Line 5 is `    name: bob` — the second item.
        let v = resolve_value(&root, text, 5, "name").expect("value");
        assert_eq!(describe_value(v), "string: `bob`");
    }

    #[test]
    fn resolve_non_ascii_value() {
        let text = "имя: Мария\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 0, "имя").expect("value");
        assert_eq!(describe_value(v), "string: `Мария`");
    }

    #[test]
    fn resolve_never_panics_on_stale_out_of_range_line() {
        // Defensive: an LSP client can send a hover position from a line
        // number that no longer exists (a stale request racing a
        // `did_change`) — this must report `None`, never panic or index
        // out of bounds.
        let text = "items: [\n{\n    name: alice\n}\n]\n";
        let root = parse(text);
        assert!(resolve_value(&root, text, 99, "name").is_none());
    }

    #[test]
    fn resolve_nested_object_key_agrees_across_line_terminators() {
        // § 3.2: LF, CR and CRLF are equivalent line terminators — the
        // enclosing-path scan must land on the same nesting regardless
        // of which one the document uses. Line 1 is `port: 80` in every
        // variant, since a document's own parse (which IS CR-aware, see
        // `ktav`'s parser) agrees on the Value tree either way.
        let cr = "server: {\r    port: 80\r}\r";
        let crlf = "server: {\r\n    port: 80\r\n}\r\n";
        for text in [cr, crlf] {
            let root = parse(text);
            let v = resolve_value(&root, text, 1, "port").expect("value");
            assert_eq!(describe_value(v), "integer: `80`");
        }
    }

    #[test]
    fn resolve_single_hash_key_is_not_treated_as_a_comment() {
        // Spec 0.5.0: a comment is a LEADING `##` — `#child: v` is a
        // valid key `#child`, not a comment. Before the fix,
        // `enclosing_path`'s own `trimmed.starts_with('#')` swallowed
        // this line, so a sibling key after it resolved with a missing
        // enclosing-path entry.
        let text = "#child: v\nnext: 1\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 0, "#child").expect("value");
        assert_eq!(describe_value(v), "string: `v`");
        let v2 = resolve_value(&root, text, 1, "next").expect("value");
        assert_eq!(describe_value(v2), "integer: `1`");
    }

    #[test]
    fn resolve_double_colon_raw_value_is_not_treated_as_an_opener() {
        // § 4: `key:: ((` is the two-byte literal string "((", not a
        // verbatim multi-line opener — `outer`'s enclosing path must
        // not gain a bogus Raw frame that swallows `sibling`.
        let text = "outer: {\n    key:: ((\n    sibling: 1\n}\n";
        let root = parse(text);
        let v = resolve_value(&root, text, 2, "sibling").expect("value");
        assert_eq!(describe_value(v), "integer: `1`");
    }

    #[test]
    fn resolve_falls_back_on_ambiguous_inline_array_item() {
        // A fully-inline object as one bare array item (`{name: alice}`
        // on a single line) is ambiguous for line-based key resolution —
        // must fall back to the generic message, never guess wrong.
        let text = "items: [\n{name: alice}\n{name: bob}\n]\n";
        let root = parse(text);
        assert!(resolve_value(&root, text, 1, "{name").is_none());
    }
}
