//! End-to-end tests for the Format Document pipeline.
//!
//! Format = `reindent(text)` (line-based canonical re-indenter that also
//! auto-fixes inline paren scalars). These tests pin the user-facing
//! contract so we catch regressions BEFORE shipping a new ktav-lsp
//! binary into the IDE plugins. To run: `cargo test --test format_pipeline`.
//!
//! `reindent` is `pub mod` in lib.rs so integration tests reach it the
//! same way external consumers would.

use ktav_lsp::reindent::reindent;

// ---- Indentation ----

#[test]
fn nested_object_canonical_indent() {
    let src = "outer: {\n  inner: 1\n}\n";
    let want = "outer: {\n    inner: 1\n}\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn over_indented_value_pulled_back() {
    let src = "a: {\n            b: 1\n            }\n";
    let want = "a: {\n    b: 1\n}\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn deeply_nested_indent_correct() {
    let src = "a: {\nb: {\nc: {\nd: 1\n}\n}\n}\n";
    let want = "a: {\n    b: {\n        c: {\n            d: 1\n        }\n    }\n}\n";
    assert_eq!(reindent(src), want);
}

// ---- Blank lines ----

#[test]
fn blank_lines_between_sections_preserved() {
    let src = "name: a\n\nother: b\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn blank_lines_inside_object_preserved() {
    let src = "obj: {\n    a: 1\n\n    b: 2\n}\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn blank_lines_inside_array_preserved() {
    let src = "items: [\n    one\n\n    two\n]\n";
    assert_eq!(reindent(src), src);
}

// ---- Comments ----

#[test]
fn top_level_comment_preserved() {
    let src = "# this is the config\nname: a\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn nested_comment_preserved_with_indent() {
    let src = "obj: {\n# inner comment\nb: 2\n}\n";
    let want = "obj: {\n    # inner comment\n    b: 2\n}\n";
    assert_eq!(reindent(src), want);
}

// ---- Multi-line strings ----

#[test]
fn stripped_multiline_form_preserved() {
    let src = "key: (\n    line1\n    line2\n)\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn verbatim_multiline_form_preserved() {
    let src = "key: ((\nline1\nline2\n))\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn verbatim_multiline_content_kept_byte_exact() {
    // Verbatim mode preserves user's leading whitespace inside content
    // (it's part of the value).
    let src = "key: ((\n    indented\n        deeper\n    back\n))\n";
    assert_eq!(reindent(src), src);
}

// ---- Auto-fix: inline paren scalar → raw marker ----

#[test]
fn inline_paren_scalar_rewritten_to_raw() {
    // `name: (value)` looks like a multi-line opener but isn't. Format
    // rewrites the separator to `::` so the value is unambiguous.
    let src = "valx: (фывфыв)\n";
    let want = "valx:: (фывфыв)\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn inline_double_paren_scalar_rewritten_to_raw() {
    let src = "name: ((wrapped))\n";
    let want = "name:: ((wrapped))\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn inline_paren_scalar_with_dotted_key_rewritten() {
    let src = "a.b.c: (value)\n";
    let want = "a.b.c:: (value)\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn already_raw_paren_scalar_unchanged() {
    let src = "name:: (value)\n";
    assert_eq!(reindent(src), src);
}

#[test]
fn typed_marker_with_paren_body_not_rewritten() {
    // `:i` / `:f` are typed markers — leave them alone. Their bodies
    // are validated by the parser, not by the formatter.
    let src = "x:i (5)\n";
    assert_eq!(reindent(src), src);
}

// ---- Integration: realistic mixed document ----

#[test]
fn complex_document_canonicalised() {
    let src = "\
# main config
name: server-1

valx: (фывфыв)

connection: {
    host: localhost
    port:i 8080

    # nested comment
    timeout:f 5.0
}

queries: [
    SELECT 1
    SELECT 2
]
";
    // Expected: `valx: (фывфыв)` rewritten to `valx:: (фывфыв)`,
    // everything else preserved (it's already canonical).
    let want = "\
# main config
name: server-1

valx:: (фывфыв)

connection: {
    host: localhost
    port:i 8080

    # nested comment
    timeout:f 5.0
}

queries: [
    SELECT 1
    SELECT 2
]
";
    assert_eq!(reindent(src), want);
}

// ---- Edge cases ----

#[test]
fn empty_document_no_change() {
    assert_eq!(reindent(""), "");
}

#[test]
fn only_blank_lines_no_change() {
    let src = "\n\n\n";
    // Trailing newlines are preserved as blank lines.
    assert_eq!(reindent(src), src);
}

#[test]
fn no_trailing_newline_in_input_preserved() {
    let src = "a: 1\nb: 2";
    assert_eq!(reindent(src), src);
}

#[test]
fn crlf_line_endings_normalised_to_lf() {
    let src = "a: 1\r\nb: 2\r\n";
    let want = "a: 1\nb: 2\n";
    assert_eq!(reindent(src), want);
}

#[test]
fn mixed_indent_in_input_normalised() {
    // User has tabs / 2-space / 8-space indent — all collapse to 4.
    let src = "a: {\n\tb: 1\n  c: 2\n        d: 3\n}\n";
    let want = "a: {\n    b: 1\n    c: 2\n    d: 3\n}\n";
    assert_eq!(reindent(src), want);
}
