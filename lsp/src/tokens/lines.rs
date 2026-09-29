//! Single source of truth for splitting a document into lines.
//!
//! Ktav § 3.2 terminates a line on LF, CR, **or** CRLF — all three are
//! equivalent. The LSP specification's own `Position.line` counts lines
//! the same way (`\n`, `\r\n` and `\r` are all line breaks for position
//! purposes), so a client editing a CR-only or CRLF file expects our
//! `line`/`character` positions to line up with its own.
//!
//! Every feature that carves `text` into per-line pieces — hover,
//! completion, semantic tokens, diagnostics, symbols, the reindenter,
//! UTF-16 re-encoding — MUST go through [`split_lines`] / [`byte_to_line_col`]
//! instead of `text.split('\n')`, or it silently desyncs from every other
//! feature (and from the client) on a non-LF document.
//!
//! § 3.1: a conformant reader skips exactly one leading byte-order mark
//! (U+FEFF, raw UTF-8 `EF BB BF`) if it is the very first code point of
//! the document — it is metadata, not content. [`leading_bom_len`] is
//! the single place that detects it; [`split_lines`] excludes it from
//! line 0's own slice and [`byte_to_line_col`] starts counting after it,
//! so every consumer that goes through either function (which is all of
//! them, per the module's own contract) sees a document whose line 0
//! starts at the first real character, with no per-feature BOM handling
//! needed. `ktav::parse` does the same skip before its own scan (mirrored
//! here so the LSP never disagrees with the reference parser about where
//! a key or token starts), and its `Span` byte offsets stay in
//! *original*-text coordinates — matching what `byte_to_line_col` expects.
//! A U+FEFF anywhere else in the document is ordinary content, never
//! stripped.

/// Byte length of the UTF-8 byte-order mark (`EF BB BF`) at the very
/// start of `text`, if present — 3 or 0. Mirrors `ktav`'s own
/// `leading_bom_len` (not reachable from here — a private fn in a
/// separate crate) so the two never disagree on what counts as a
/// leading BOM.
pub fn leading_bom_len(text: &str) -> usize {
    if text.as_bytes().starts_with(b"\xEF\xBB\xBF") {
        3
    } else {
        0
    }
}

/// Byte length of the line terminator starting at `bytes[i]`, or 0 if
/// `bytes[i]` isn't a terminator byte. `\r\n` is ONE two-byte terminator;
/// a lone `\r` or lone `\n` is one byte — this is the only place that
/// needs to know the three shapes are equivalent.
fn terminator_len(bytes: &[u8], i: usize) -> usize {
    match bytes.get(i) {
        Some(b'\n') => 1,
        Some(b'\r') => {
            if bytes.get(i + 1) == Some(&b'\n') {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

/// Split `text` into line contents with terminators stripped.
///
/// Same shape as `text.split('\n')` on an LF-only document — including a
/// trailing empty slice when `text` ends with a terminator — but also
/// accepts a lone `\r` and `\r\n` as terminators (§ 3.2). For an
/// LF-only document this is byte-for-byte what `text.split('\n')`
/// already produced, so switching a caller over never changes its
/// behaviour on LF input.
pub fn split_lines(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let bom = leading_bom_len(text);
    let mut out = Vec::new();
    let mut start = bom;
    let mut i = start;
    while i < bytes.len() {
        let tlen = terminator_len(bytes, i);
        if tlen == 0 {
            i += 1;
            continue;
        }
        out.push(&text[start..i]);
        i += tlen;
        start = i;
    }
    out.push(&text[start..]);
    out
}

/// Does `text` end with a line terminator (LF, CR or CRLF)? Used to
/// decide whether a re-emitted document should keep a trailing newline.
pub fn ends_with_terminator(text: &str) -> bool {
    text.ends_with('\n') || text.ends_with('\r')
}

/// Convert an absolute byte offset into `text` into a 0-based
/// `(line, byte_column)` pair, splitting on LF/CR/CRLF (§ 3.2). `column`
/// is the byte distance from the start of that line. `offset` past the
/// end of `text` clamps to the last line.
pub fn byte_to_line_col(text: &str, offset: usize) -> (u32, u32) {
    let bytes = text.as_bytes();
    let bom = leading_bom_len(text);
    // An offset landing inside the BOM itself (never produced by
    // `ktav::parse`'s own spans, which start scanning at `bom`) clamps
    // to the first real column rather than underflowing below it.
    let offset = offset.max(bom).min(bytes.len());
    let mut line: u32 = 0;
    let mut line_start = bom;
    let mut i = bom;
    while i < offset {
        let tlen = terminator_len(bytes, i);
        if tlen == 0 {
            i += 1;
            continue;
        }
        line += 1;
        i += tlen;
        line_start = i;
    }
    (line, (offset - line_start) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_std_split_on_lf_only_docs() {
        for text in ["", "a", "a\n", "a\nb", "a\nb\n", "\n\n\n", "a\n\nb\n"] {
            let want: Vec<&str> = text.split('\n').collect();
            assert_eq!(split_lines(text), want, "text={:?}", text);
        }
    }

    #[test]
    fn cr_lf_crlf_are_equivalent() {
        let lf = "a\nb\nc";
        let cr = "a\rb\rc";
        let crlf = "a\r\nb\r\nc";
        let want = split_lines(lf);
        assert_eq!(split_lines(cr), want);
        assert_eq!(split_lines(crlf), want);
    }

    #[test]
    fn mixed_terminators_in_one_document() {
        let text = "a\nb\rc\r\nd";
        assert_eq!(split_lines(text), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn trailing_terminator_yields_trailing_empty_line() {
        assert_eq!(split_lines("a\r"), vec!["a", ""]);
        assert_eq!(split_lines("a\r\n"), vec!["a", ""]);
        assert_eq!(split_lines("a\n"), vec!["a", ""]);
    }

    #[test]
    fn ends_with_terminator_recognises_all_three_forms() {
        assert!(ends_with_terminator("a\n"));
        assert!(ends_with_terminator("a\r"));
        assert!(ends_with_terminator("a\r\n"));
        assert!(!ends_with_terminator("a"));
        assert!(!ends_with_terminator(""));
    }

    #[test]
    fn byte_to_line_col_matches_across_terminators() {
        // "key2" starts right after the first line in all three variants.
        let lf = "key1: 1\nkey2: 2\n";
        let cr = "key1: 1\rkey2: 2\r";
        let crlf = "key1: 1\r\nkey2: 2\r\n";
        let offset_lf = lf.find("key2").unwrap();
        let offset_cr = cr.find("key2").unwrap();
        let offset_crlf = crlf.find("key2").unwrap();
        assert_eq!(byte_to_line_col(lf, offset_lf), (1, 0));
        assert_eq!(byte_to_line_col(cr, offset_cr), (1, 0));
        assert_eq!(byte_to_line_col(crlf, offset_crlf), (1, 0));
    }

    #[test]
    fn byte_to_line_col_offset_past_end_clamps() {
        let text = "a\nbb\n";
        assert_eq!(
            byte_to_line_col(text, 1000),
            byte_to_line_col(text, text.len())
        );
    }

    // ---- § 3.1: leading BOM is metadata, never content ----

    #[test]
    fn leading_bom_len_detects_and_excludes() {
        assert_eq!(leading_bom_len("\u{FEFF}host: value"), 3);
        assert_eq!(leading_bom_len("host: value"), 0);
        // Not the first code point — ordinary content, not a BOM.
        assert_eq!(leading_bom_len("a\u{FEFF}b"), 0);
        assert_eq!(leading_bom_len(""), 0);
    }

    #[test]
    fn split_lines_excludes_leading_bom_from_first_line() {
        let text = "\u{FEFF}host: value\nport: 1\n";
        assert_eq!(split_lines(text), vec!["host: value", "port: 1", ""]);
    }

    #[test]
    fn split_lines_bom_only_document_is_one_empty_line() {
        assert_eq!(split_lines("\u{FEFF}"), vec![""]);
    }

    #[test]
    fn byte_to_line_col_excludes_leading_bom() {
        // Same absolute byte offset as the non-BOM sibling test above,
        // shifted by the BOM's 3 bytes — must land on the same column.
        let bom = "\u{FEFF}key1: 1\nkey2: 2\n";
        let offset = bom.find("key2").unwrap();
        assert_eq!(byte_to_line_col(bom, offset), (1, 0));
        // A byte inside the document's own first "real" line — 3 bytes
        // ahead of the BOM-less sibling's offset for the same content.
        let offset_key1 = bom.find("key1").unwrap();
        assert_eq!(byte_to_line_col(bom, offset_key1), (0, 0));
    }

    #[test]
    fn byte_to_line_col_offset_inside_bom_clamps_to_first_column() {
        let text = "\u{FEFF}host: value\n";
        assert_eq!(byte_to_line_col(text, 0), (0, 0));
        assert_eq!(byte_to_line_col(text, 1), (0, 0));
        assert_eq!(byte_to_line_col(text, 2), (0, 0));
    }
}
