use super::*;
use crate::diagnostics::parse_for_diagnostics;
use crate::semantic::semantic_tokens;
use crate::tokens::byte_to_utf16;

/// Decode a delta-encoded token stream into absolute
/// `(line, start_col, length)` triples — easier to assert against.
fn absolutize(toks: &[SemanticToken]) -> Vec<(u32, u32, u32)> {
    let mut out = Vec::with_capacity(toks.len());
    let mut line: u32 = 0;
    let mut col: u32 = 0;
    for t in toks {
        line += t.delta_line;
        if t.delta_line == 0 {
            col += t.delta_start;
        } else {
            col = t.delta_start;
        }
        out.push((line, col, t.length));
    }
    out
}

#[test]
fn convert_semantic_tokens_to_utf16_multibyte_multiline() {
    // Line 0: ASCII pair so we have a baseline.
    // Line 1: Cyrillic key (each cyrillic letter = 2 bytes UTF-8 / 1 UTF-16 unit).
    // Line 2: emoji in the value (4 bytes UTF-8 / 2 UTF-16 units).
    let text = "name: alice\nимя: bob\ngreeting: 😀\n";

    let mut byte_toks = semantic_tokens(text);
    // Sanity: byte-encoded stream should reflect UTF-8 byte columns.
    let abs_bytes = absolutize(&byte_toks);
    // Expect at least one token per line.
    assert!(abs_bytes.iter().any(|(l, _, _)| *l == 0));
    assert!(abs_bytes.iter().any(|(l, _, _)| *l == 1));
    assert!(abs_bytes.iter().any(|(l, _, _)| *l == 2));

    // Pick the key-token on line 1 (the Cyrillic "имя"). In bytes it
    // is 6 long; in UTF-16 it is 3 long.
    let line1_key_bytes = abs_bytes
        .iter()
        .find(|(l, c, _)| *l == 1 && *c == 0)
        .copied()
        .expect("key token on line 1");
    assert_eq!(line1_key_bytes.2, 6, "cyrillic key in bytes");

    convert_semantic_tokens_to_utf16(&mut byte_toks, text);
    let abs_u16 = absolutize(&byte_toks);

    // Same token, now in UTF-16: column still 0, length 3.
    let line1_key_u16 = abs_u16
        .iter()
        .find(|(l, c, _)| *l == 1 && *c == 0)
        .copied()
        .expect("key token on line 1 (utf16)");
    assert_eq!(line1_key_u16.2, 3, "cyrillic key in utf-16 units");

    // Line 2: the value "😀" is at byte column 10 ("greeting: " = 10
    // bytes). In UTF-16 the value column is also 10 (ASCII prefix),
    // but the *length* is 2 (surrogate pair), not 4 bytes.
    let line2_val_u16 = abs_u16
        .iter()
        .find(|(l, c, len)| *l == 2 && *c == 10 && *len == 2)
        .copied();
    assert!(
        line2_val_u16.is_some(),
        "expected line-2 value token of utf-16 length 2 (emoji surrogate pair); got {:?}",
        abs_u16
    );

    // Cross-line delta invariant: when delta_line > 0, delta_start is
    // an absolute column. This re-deltaing bug used to surface as
    // negative/overshoot deltas — pin it.
    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;
    for t in &byte_toks {
        let cur_line = prev_line + t.delta_line;
        let cur_col = if t.delta_line == 0 {
            prev_col + t.delta_start
        } else {
            t.delta_start
        };
        // No underflow asserts implicitly happened above. Also assert
        // monotonic-by-line column on same-line tokens.
        if t.delta_line == 0 {
            assert!(
                cur_col >= prev_col,
                "non-monotonic same-line column: {} < {}",
                cur_col,
                prev_col,
            );
        }
        prev_line = cur_line;
        prev_col = cur_col;
    }
}

#[test]
fn diagnostic_range_utf16_cyrillic_key() {
    // Duplicate-key error on a line whose key is Cyrillic. Without
    // UTF-16 conversion the column would be in bytes (each cyrillic
    // letter = 2 bytes); after conversion it must reflect UTF-16 code
    // units (1 unit per BMP cyrillic letter).
    let text = "имя: a\nимя: b\n";
    let mut diags = parse_for_diagnostics(text);
    assert!(!diags.is_empty(), "expected a duplicate-key diagnostic");

    // Find the diagnostic on line 1 (the second occurrence). Some
    // upstream parser versions report on line 0 — accept either, but
    // pin the byte→utf16 conversion below regardless.
    let d = diags
        .iter()
        .find(|d| d.range.start.line == 1)
        .or_else(|| diags.first())
        .cloned()
        .unwrap();

    let byte_start_col = d.range.start.character;
    let byte_end_col = d.range.end.character;

    convert_diagnostics_to_utf16(&mut diags, text);
    let d2 = diags
        .iter()
        .find(|d| d.range.start.line == 1)
        .or_else(|| diags.first())
        .cloned()
        .unwrap();

    // The line text on whichever line the diagnostic points to.
    let line_text: &str = text.lines().nth(d2.range.start.line as usize).unwrap_or("");
    let expected_start = byte_to_utf16(line_text, byte_start_col as usize);
    let expected_end = byte_to_utf16(line_text, byte_end_col as usize);
    assert_eq!(d2.range.start.character, expected_start);
    assert_eq!(d2.range.end.character, expected_end);

    // And: for a Cyrillic-only key the UTF-16 column must be strictly
    // less than the byte column (whenever the byte column is > 0).
    if byte_start_col > 0 {
        assert!(
            d2.range.start.character < byte_start_col,
            "utf-16 col {} should be < byte col {} for cyrillic input",
            d2.range.start.character,
            byte_start_col,
        );
    }
}

#[test]
fn diagnostic_range_utf16_emoji_value() {
    // Value containing an emoji (surrogate pair) on a syntactically
    // invalid line. Use an unclosed brace — guaranteed to produce a
    // diagnostic in current parser versions.
    let text = "greeting: 😀\nbroken: {\n";
    let mut diags = parse_for_diagnostics(text);
    assert!(!diags.is_empty(), "expected a diagnostic");

    // Snapshot pre-conversion byte columns.
    let pre: Vec<_> = diags
        .iter()
        .map(|d| {
            (
                d.range.start.line,
                d.range.start.character,
                d.range.end.character,
            )
        })
        .collect();

    convert_diagnostics_to_utf16(&mut diags, text);

    for (i, d) in diags.iter().enumerate() {
        let (line_no, byte_start, byte_end) = pre[i];
        let line_text = text.lines().nth(line_no as usize).unwrap_or("");
        assert_eq!(
            d.range.start.character,
            byte_to_utf16(line_text, byte_start as usize),
        );
        assert_eq!(
            d.range.end.character,
            byte_to_utf16(line_text, byte_end as usize),
        );
    }
}

#[test]
fn end_of_document_ascii() {
    let text = "a: 1\nb: 2\n";
    // Trailing newline: last "line" per split('\n') is empty.
    assert_eq!(end_of_document(text, PositionEncoding::Utf8), (2, 0));
    assert_eq!(end_of_document(text, PositionEncoding::Utf16), (2, 0));
}

#[test]
fn end_of_document_multibyte_last_line_utf8() {
    // Last line "café" is 5 bytes (é is 2 bytes), 4 Unicode scalars.
    // `character` for Utf8 encoding is a byte offset — must be 5, not
    // the scalar count 4 that `chars().count()` would give.
    let text = "a: 1\ncafé";
    assert_eq!(end_of_document(text, PositionEncoding::Utf8), (1, 5));
}

#[test]
fn end_of_document_astral_last_line_utf16() {
    // Last line "k: 😀" — k,:,space = 3 UTF-16 units, 😀 is a surrogate
    // pair = 2 more units, total 5. `chars().count()` would give 4
    // (each scalar counted once, undercounting the surrogate pair).
    let text = "a: 1\nk: 😀";
    assert_eq!(end_of_document(text, PositionEncoding::Utf16), (1, 5));
}
