//! LSP-level line-model tests.
//!
//! Ktav § 3.2 (and the LSP position spec itself) treat LF, CR and CRLF
//! as equivalent line terminators. These tests pin that every
//! position-producing feature — diagnostics, semantic tokens, symbols,
//! format — agrees on line/column for the same logical document no
//! matter which terminator it uses, and that plain LF behaviour is
//! unchanged. `ktav_lsp::lines` is the shared splitter all of them go
//! through; see its own unit tests for the primitive-level coverage.

use ktav_lsp::diagnostics::parse_for_diagnostics;
use ktav_lsp::lines::split_lines;
use ktav_lsp::reindent::reindent;
use ktav_lsp::semantic::semantic_tokens;
use ktav_lsp::symbols::build_symbols;

// ---- `lines::split_lines` — the shared primitive ----

#[test]
fn split_lines_matches_std_split_on_lf_only_docs() {
    for text in ["", "a", "a\n", "a\nb", "a\nb\n", "\n\n\n", "a\n\nb\n"] {
        let want: Vec<&str> = text.split('\n').collect();
        assert_eq!(split_lines(text), want, "text={:?}", text);
    }
}

#[test]
fn split_lines_treats_cr_lf_crlf_as_equivalent() {
    let lf = "a\nb\nc";
    let cr = "a\rb\rc";
    let crlf = "a\r\nb\r\nc";
    let want = split_lines(lf);
    assert_eq!(split_lines(cr), want);
    assert_eq!(split_lines(crlf), want);
}

// ---- Diagnostics ----

#[test]
fn diagnostics_agree_across_line_terminators() {
    let lf = "a: 1\nbroken:2\n";
    let cr = "a: 1\rbroken:2\r";
    let crlf = "a: 1\r\nbroken:2\r\n";

    let want = parse_for_diagnostics(lf);
    assert_eq!(want.len(), 1);
    assert_eq!(want[0].range.start.line, 1, "error must land on line 2");

    let got_cr = parse_for_diagnostics(cr);
    let got_crlf = parse_for_diagnostics(crlf);
    assert_eq!(got_cr.len(), 1);
    assert_eq!(got_crlf.len(), 1);
    assert_eq!(got_cr[0].range, want[0].range, "CR must match LF");
    assert_eq!(got_crlf[0].range, want[0].range, "CRLF must match LF");
}

// ---- Semantic tokens ----

#[test]
fn semantic_tokens_agree_across_line_terminators() {
    let lf = "a: 1\nb: {\n    c: 2\n}\n";
    let cr = "a: 1\rb: {\r    c: 2\r}\r";
    let crlf = "a: 1\r\nb: {\r\n    c: 2\r\n}\r\n";

    let want = semantic_tokens(lf);
    assert!(!want.is_empty());

    for (variant, text) in [("cr", cr), ("crlf", crlf)] {
        let got = semantic_tokens(text);
        assert_eq!(got.len(), want.len(), "variant={variant}");
        for (a, b) in got.iter().zip(want.iter()) {
            assert_eq!(
                (a.delta_line, a.delta_start, a.length, a.token_type),
                (b.delta_line, b.delta_start, b.length, b.token_type),
                "variant={variant}"
            );
        }
    }
}

// ---- Symbols ----

#[test]
fn symbols_agree_across_line_terminators() {
    let lf = "outer: {\n    inner: 1\n}\n";
    let cr = "outer: {\r    inner: 1\r}\r";
    let crlf = "outer: {\r\n    inner: 1\r\n}\r\n";

    for text in [lf, cr, crlf] {
        let root = ktav::parse(text).expect("fixture must parse");
        let syms = build_symbols(&root, text);
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].range.start.line, 0);
        let children = syms[0].children.as_ref().expect("outer has children");
        assert_eq!(children.len(), 1);
        assert_eq!(
            children[0].range.start.line, 1,
            "inner key must resolve to line 2 regardless of terminator"
        );
    }
}

// ---- Format ----

#[test]
fn reindent_formats_cr_and_crlf_like_lf_with_lf_output() {
    let lf = "outer: {\n    inner: 1\n}\n";
    let cr = "outer: {\rinner: 1\r}\r";
    let crlf = "outer: {\r\ninner: 1\r\n}\r\n";

    let want = reindent(lf);
    assert_eq!(want, lf, "LF input already canonical");
    assert_eq!(reindent(cr), want);
    assert_eq!(reindent(crlf), want);
}
