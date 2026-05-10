//! Per-variant assertions on the structured-diagnostic path.
//!
//! For each [`ktav::ErrorKind`] variant we exercise (10 of them — all
//! the spec-defined categories plus the two `0.1.6` newcomers
//! `UnbalancedBracket` / `InlineNonEmptyCompound` / `MissingSeparator`),
//! we feed the parser a known-bad document, take the resulting
//! [`tower_lsp::lsp_types::Diagnostic`] from
//! [`ktav_lsp::diagnostics::parse_for_diagnostics`], and assert:
//!
//! 1. `severity == ERROR`, `source == "ktav"`,
//! 2. `message` is byte-identical to `kind.to_string()` (the Display
//!    contract pinned in `tests/error_format_pinning.rs`),
//! 3. `range` matches `span.line_col(text)` for the start, with the
//!    same conversion applied to the span's end offset.
//!
//! UTF-16 conversion of the column is exercised via [`byte_to_utf16`]
//! on a Cyrillic fixture — diagnostics emitted from this module are
//! BYTE-indexed (the server applies the UTF-16 re-encoding only when
//! the negotiated encoding requires it).

use ktav::{Error, ErrorKind, Span};
use ktav_lsp::diagnostics::parse_for_diagnostics;
use ktav_lsp::tokens::byte_to_utf16;
use tower_lsp::lsp_types::DiagnosticSeverity;

/// Run the parser, expect a Structured error, return its kind.
fn structured(text: &str) -> ErrorKind {
    match ktav::parse(text) {
        Err(Error::Structured(k)) => k,
        Err(other) => panic!("expected Structured error, got {:?}", other),
        Ok(_) => panic!("expected parse to fail for: {:?}", text),
    }
}

/// 1-based line/0-based byte-column for the START of a span; same for
/// the end offset (encoded as a zero-length span at `span.end`).
fn span_start_end_line_col(text: &str, span: Span) -> ((u32, u32), (u32, u32)) {
    let s = span.line_col(text);
    let e = Span::new(span.end, span.end).line_col(text);
    (s, e)
}

/// Boilerplate-free "diagnostic matches kind" assertion. Returns the
/// diagnostic for further per-test assertions.
fn assert_diag_matches_kind(text: &str, kind: &ErrorKind) -> tower_lsp::lsp_types::Diagnostic {
    let diags = parse_for_diagnostics(text);
    assert_eq!(diags.len(), 1, "expected exactly one diagnostic");
    let d = diags.into_iter().next().unwrap();

    assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(d.source.as_deref(), Some("ktav"));
    assert_eq!(
        d.message,
        kind.to_string(),
        "diagnostic message must equal kind.to_string()"
    );

    let span = kind.span();
    if span.start == 0 && span.end == 0 {
        // Empty span — kind has no source range; we can't pin precise
        // start/end here. The diagnostic should still be on a non-
        // negative line.
        return d;
    }

    let ((sl, sc), (el, ec)) = span_start_end_line_col(text, span);
    assert_eq!(d.range.start.line, sl - 1, "start line");
    assert_eq!(d.range.start.character, sc, "start col (bytes)");
    assert_eq!(d.range.end.line, el - 1, "end line");
    assert_eq!(d.range.end.character, ec, "end col (bytes)");
    d
}

#[test]
fn missing_separator_space_variant() {
    // First line forces Object root (per spec 0.1.1 a bare
    // `key:value` at the document start is a top-level Array string
    // item, not a malformed pair).
    let text = "anchor: 1\nkey:value\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::MissingSeparatorSpace { .. }));
    let d = assert_diag_matches_kind(text, &k);
    // Concrete fixture: span covers `value` on line 2 (cols 4..9).
    assert_eq!(d.range.start.line, 1);
    assert_eq!(d.range.start.character, 4);
    assert_eq!(d.range.end.character, 9);
}

#[test]
fn invalid_typed_scalar_variant() {
    let text = "port:i abc\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::InvalidTypedScalar { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn duplicate_key_variant() {
    let text = "port: 80\nport: 443\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::DuplicateKey { .. }));
    let d = assert_diag_matches_kind(text, &k);
    assert_eq!(d.range.start.line, 1);
}

#[test]
fn key_path_conflict_variant() {
    let text = "db: 1\ndb.x: 2\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::KeyPathConflict { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn empty_key_variant() {
    // First line forces Object root (per spec 0.1.1).
    let text = "anchor: 1\n: value\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::EmptyKey { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn invalid_key_variant() {
    let text = "a.: 1\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::InvalidKey { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn unclosed_compound_variant() {
    let text = "obj: {\n  a: 1\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::UnclosedCompound { .. }));
    let d = assert_diag_matches_kind(text, &k);
    assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn unbalanced_bracket_variant() {
    // `}` with no matching opener.
    let text = "key: 1\n}\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::UnbalancedBracket { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn inline_nonempty_compound_variant() {
    // Object with inline non-empty entries — forbidden by spec § 6.7.
    let text = "obj: { a: 1, b: 2 }\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::InlineNonEmptyCompound { .. }));
    assert_diag_matches_kind(text, &k);
}

#[test]
fn missing_separator_variant() {
    // A line that's not blank/comment/closer/array-item but lacks `:`.
    let text = "obj: {\n  bareword\n}\n";
    let k = structured(text);
    assert!(matches!(k, ErrorKind::MissingSeparator { .. }));
    assert_diag_matches_kind(text, &k);
}

// ---------------------------------------------------------------------------
// UTF-16 conversion path — `parse_for_diagnostics` emits BYTE columns;
// the server's `convert_diagnostics_to_utf16` re-encodes when needed.
// We exercise both paths against a Cyrillic fixture.
// ---------------------------------------------------------------------------

#[test]
fn cyrillic_key_diagnostic_byte_columns() {
    // `имя:значение` — `имя` is 6 bytes (3 chars × 2 bytes each); `:`
    // is at byte 6; the glued body `значение` starts at byte 7.
    // MissingSeparatorSpace span is `body_off..trimmed_span.end`.
    // First line forces Object root (per spec 0.1.1).
    let text = "anchor: 1\nимя:значение\n";
    let k = structured(text);
    let span = k.span();
    let d = assert_diag_matches_kind(text, &k);

    // Sanity: byte columns match the structured span's line/col.
    // The diagnostic is on line 2 (1-indexed), so we extract that line.
    let line = text.lines().nth(1).unwrap();

    // The span is in document-byte offsets; convert to line-relative
    // bytes by subtracting the byte offset where line 2 starts. Line 1
    // is "anchor: 1\n" = 10 bytes.
    let line2_start = "anchor: 1\n".len() as u32;
    let start_byte = (span.start - line2_start) as usize;
    let end_byte = (span.end - line2_start) as usize;
    assert_eq!(d.range.start.character as usize, start_byte);
    assert_eq!(d.range.end.character as usize, end_byte);

    // UTF-16 conversion: each Cyrillic char is 1 UTF-16 unit. `имя` is
    // 6 bytes / 3 code units; the `:` and body that follow are also
    // 1 unit per char. `byte_to_utf16` is what
    // `convert_diagnostics_to_utf16` uses, so we exercise it directly.
    let start_u16 = byte_to_utf16(line, start_byte);
    let end_u16 = byte_to_utf16(line, end_byte);
    // Body starts after `имя:` = 4 UTF-16 units; body `значение` is 8
    // chars / 8 UTF-16 units, so end = 4 + 8 = 12.
    assert_eq!(start_u16, 4);
    assert_eq!(end_u16, 12);
}
