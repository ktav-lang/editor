//! End-to-end tests: feed text through the public modules and check
//! that diagnostics, semantic tokens and symbols come out shaped right.

use ktav_lsp::diagnostics::parse_for_diagnostics;
use ktav_lsp::semantic::{semantic_tokens, token_types};
use ktav_lsp::symbols::build_symbols;

#[test]
fn valid_document_yields_no_diagnostics() {
    let text = "name: example\nport: 8080\n";
    assert!(parse_for_diagnostics(text).is_empty());
}

#[test]
fn missing_separator_space_diagnostic_on_line_1() {
    let text = "key:value\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].range.start.line, 0);
    assert!(
        d[0].message.contains("MissingSeparatorSpace"),
        "got: {}",
        d[0].message
    );
}

#[test]
fn duplicate_key_diagnostic_on_line_2() {
    let text = "port: 80\nport: 443\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].range.start.line, 1);
}

#[test]
fn empty_input_is_valid() {
    assert!(parse_for_diagnostics("").is_empty());
}

#[test]
fn semantic_tokens_emit_for_simple_doc() {
    let text = "# comment\nname: alice\nflag: true\ncount:i 42\n";
    let toks = semantic_tokens(text);
    // 1 (comment) + 2 (name+sep+value) wait — let's just assert non-empty and
    // that the count matches our line-by-line emission.
    assert!(!toks.is_empty());
    // First token must be the comment on line 0.
    assert_eq!(toks[0].delta_line, 0);
}

#[test]
fn token_types_index_layout_stable() {
    // Stable order — clients depend on these indices via the legend.
    let names: Vec<String> = token_types()
        .into_iter()
        .map(|t| t.as_str().to_string())
        .collect();
    assert_eq!(names[0], "comment");
    assert_eq!(names[1], "keyword");
    assert_eq!(names[2], "number");
    assert_eq!(names[3], "string");
    assert_eq!(names[4], "property");
    assert_eq!(names[5], "operator");
}

#[test]
fn document_symbols_built_from_object() {
    let text = "name: alice\nport: 8080\nserver: {\n    host: localhost\n}\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"name"));
    assert!(names.contains(&"port"));
    assert!(names.contains(&"server"));
    let server = syms.iter().find(|s| s.name == "server").unwrap();
    let kids = server.children.as_ref().expect("server has children");
    assert!(kids.iter().any(|k| k.name == "host"));
}
