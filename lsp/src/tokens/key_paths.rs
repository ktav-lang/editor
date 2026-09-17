//! Key-path scanners: locate the unescaped `:` key/value separator,
//! split dotted key paths into segments, and detect whether a cursor
//! sits after the separator. Quote-aware per spec 0.7 § 5.3.3.

/// Find the byte index of the key/value separator `:` on `trimmed`,
/// treating a `<quoted-segment>` (§ 5.3.3) as opaque — a `:` inside
/// `"a:b"` / `'a:b'` / `` `a:b` `` is ordinary content, not the
/// separator, exactly as `ktav`'s own quote-aware scan treats it. A
/// quote character opens a segment only at a segment's first code
/// point (line start, or right after an unescaped `.`); elsewhere it is
/// an ordinary key byte and does not affect the scan.
///
/// If a quote opens a segment with no matching unescaped closer before
/// end of line, the whole rest of the line is swallowed (quote-opaque)
/// and no separator is found — mirroring § 5.3.3's "Unterminated quoted
/// segments" rule, which is indistinguishable from a line with no `:`
/// at all.
pub(crate) fn find_key_separator(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut at_segment_start = true;
    while i < bytes.len() {
        let c = bytes[i];
        if at_segment_start && matches!(c, b'"' | b'\'' | b'`') {
            let quote = c;
            let mut j = i + 1;
            let mut closed = false;
            while j < bytes.len() {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if bytes[j] == quote {
                    closed = true;
                    j += 1;
                    break;
                }
                j += 1;
            }
            if !closed {
                return None;
            }
            i = j;
            at_segment_start = false;
            continue;
        }
        if c == b'\\' {
            i += 2;
            at_segment_start = false;
            continue;
        }
        if c == b'.' {
            i += 1;
            at_segment_start = true;
            continue;
        }
        if c == b':' {
            return Some(i);
        }
        i += 1;
        at_segment_start = false;
    }
    None
}

/// Yield `(segment_start, segment_text)` for each dotted segment of a key.
/// `key_start` is the absolute column where the key text begins.
///
/// Spec 0.6.0: the separator is an UNESCAPED `.`. A `\.` inside a segment
/// is a literal dot (part of the segment text); `\\` escapes a single
/// backslash and does not protect the following byte from being a
/// separator. Segments are returned with their escape sequences still
/// embedded — callers that need the decoded text must unescape themselves
/// (highlighting / column ranges only need byte positions).
///
/// Spec 0.7.0: a `<quoted-segment>` (§ 5.3.3) is opaque to this split — a
/// `.` inside `"b.c"` does not start a new segment, so
/// `a."b.c".d` is three segments (`a`, `"b.c"`, `d`), not four. As with
/// [`find_key_separator`], a quote opens a segment only at a segment's
/// first code point (key start, or right after an unescaped `.`).
pub fn split_dotted(key_start: u32, key: &str) -> impl Iterator<Item = (u32, &str)> {
    let bytes = key.as_bytes();
    let mut segs: Vec<(u32, &str)> = Vec::new();
    let mut seg_start = 0usize;
    let mut i = 0usize;
    let mut at_segment_start = true;
    while i < bytes.len() {
        let c = bytes[i];
        if at_segment_start && matches!(c, b'"' | b'\'' | b'`') {
            let quote = c;
            let mut j = i + 1;
            while j < bytes.len() {
                if bytes[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if bytes[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            i = j;
            at_segment_start = false;
            continue;
        }
        if c == b'\\' {
            i += 2;
            at_segment_start = false;
            continue;
        }
        if c == b'.' {
            segs.push((key_start + seg_start as u32, &key[seg_start..i]));
            seg_start = i + 1;
            i += 1;
            at_segment_start = true;
            continue;
        }
        i += 1;
        at_segment_start = false;
    }
    segs.push((key_start + seg_start as u32, &key[seg_start..]));
    segs.into_iter()
}

/// True if a `key: ` form on this line indicates the cursor is positioned
/// AFTER the separator (used by completion to switch from key-mode to
/// value-mode). `upto` is the line text up to the cursor column.
///
/// Spec 0.5.0: only `::` and `:` are markers; `:i`/`:f` are gone.
pub fn cursor_is_after_separator(upto: &str) -> bool {
    let trimmed = upto.trim_start();
    // Spec 0.6.0: only an UNESCAPED `:` is the separator (`\:` is a
    // literal colon inside the key). Spec 0.7.0: a `:` inside a quoted
    // key segment is opaque too (§ 5.3.3).
    let Some(i) = find_key_separator(trimmed) else {
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
