//! Source spans for the parsed symbol tree, independent of DFS/source ordering.

use ktav::Value;
use ktav_lsp::symbols::build_symbols;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

fn zero_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

fn parse(text: &str) -> Value {
    ktav::parse(text).expect("fixture must parse")
}

fn find<'a>(syms: &'a [DocumentSymbol], name: &str) -> &'a DocumentSymbol {
    syms.iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no symbol named '{name}' among {syms:?}"))
}

fn find_path<'a>(syms: &'a [DocumentSymbol], path: &[&str]) -> &'a DocumentSymbol {
    let mut symbol = find(syms, path[0]);
    for name in &path[1..] {
        symbol = find(symbol.children.as_ref().unwrap(), name);
    }
    symbol
}

fn assert_range(sym: &DocumentSymbol, line: u32, start: u32, end: u32) {
    let expected = Range {
        start: Position {
            line,
            character: start,
        },
        end: Position {
            line,
            character: end,
        },
    };
    assert_eq!(sym.range, expected, "range for {}", sym.name);
    assert_eq!(
        sym.selection_range, expected,
        "selectionRange for {}",
        sym.name
    );
}

#[test]
fn scalar_body_suffix_is_not_a_block_opener() {
    let text = "before: 0\ntext: hello {\nafter: 1\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 3);
    assert_range(find(&syms, "before"), 0, 0, 9);
    assert_range(find(&syms, "text"), 1, 0, 13);
    assert_range(find(&syms, "after"), 2, 0, 8);
}

#[test]
fn tab_separated_block_opener_enters_child_object() {
    let text = "outer:\t{\nchild: 1\n}\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 1);
    let outer = find(&syms, "outer");
    assert_range(outer, 0, 0, 8);
    assert_range(find(outer.children.as_ref().unwrap(), "child"), 1, 0, 8);
}

#[test]
fn dotted_explicit_objects_use_node_identity_not_depth() {
    let text = "a.b: 1\nother: {x: {key: 2}}\na.c: {key: 3}\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    let a = find(&syms, "a");
    assert_range(a, 0, 0, 6);
    let a_children = a.children.as_ref().unwrap();
    assert_range(find(a_children, "b"), 0, 0, 6);
    let c = find(a_children, "c");
    assert_range(c, 2, 0, 13);
    assert_range(find(c.children.as_ref().unwrap(), "key"), 2, 6, 9);
    let other = find(&syms, "other");
    assert_range(other, 1, 0, 20);
    let x = find(other.children.as_ref().unwrap(), "x");
    assert_range(x, 1, 8, 9);
    assert_range(find(x.children.as_ref().unwrap(), "key"), 1, 12, 15);
}

#[test]
fn spec_dotted_extension_uses_existing_explicit_object_identity() {
    let text =
        include_str!("../../spec/versions/0.8/tests/valid/dotted_keys/extend_explicit_object.ktav");
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 1);
    let a = find(&syms, "a");
    assert_range(a, 0, 0, 4);
    let children = a.children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_range(find(children, "x"), 1, 0, 8);
    assert_range(find(children, "y"), 3, 0, 6);
}

#[test]
fn spec_raw_structural_item_tokens_remain_array_scalars() {
    let text =
        include_str!("../../spec/versions/0.8/tests/valid/raw_marker/structural_item_tokens.ktav");
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 1);
    let items = find(&syms, "items");
    assert_range(items, 0, 0, 8);
    let children = items.children.as_ref().unwrap();
    assert_eq!(children.len(), 6);
    for (i, end) in [8, 8, 8, 9, 9, 12].into_iter().enumerate() {
        assert_eq!(children[i].name, format!("[{i}]"));
        assert_eq!(children[i].kind, SymbolKind::STRING);
        assert!(children[i].children.is_none());
        assert_range(&children[i], i as u32 + 1, 0, end);
    }
}

#[test]
fn exact_body_dispatch_uses_spec_whitespace_with_bom_and_cr() {
    let whitespace = [
        '\t', '\u{000b}', '\u{000c}', ' ', '\u{0085}', '\u{00a0}', '\u{1680}', '\u{2000}',
        '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}',
        '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}',
        '\u{3000}',
    ];
    for ws in whitespace {
        let width = ws.len_utf8() as u32;
        for eol in ["\r", "\r\n", "\n"] {
            let text = format!(
                "\u{feff}{ws}outer:{ws}{{{ws}{eol}child: 1{eol}{ws}}}{ws}{eol}after: 2{eol}"
            );
            let syms = build_symbols(&parse(&text), &text);
            assert_eq!(syms.len(), 2);
            let outer = find(&syms, "outer");
            assert_range(outer, 0, 3, 10 + 3 * width);
            assert_range(find(outer.children.as_ref().unwrap(), "child"), 1, 0, 8);
            assert_range(find(&syms, "after"), 3, 0, 8);
        }
    }
}

#[test]
fn scalar_suffixes_and_raw_openers_never_change_object_context() {
    for token in ["{", "[", "(", "(("] {
        for marker in [":", "::"] {
            let body = if marker == "::" {
                token.to_string()
            } else {
                format!("hello {token}")
            };
            let text = format!("before: 0\ntext{marker}\t{body}\nafter: 1\n");
            let syms = build_symbols(&parse(&text), &text);
            assert_eq!(syms.len(), 3);
            assert_range(find(&syms, "before"), 0, 0, 9);
            assert_range(
                find(&syms, "text"),
                1,
                0,
                (5 + marker.len() + body.len()) as u32,
            );
            assert_range(find(&syms, "after"), 2, 0, 8);
        }
    }
}

#[test]
fn block_object_closers_restore_the_enclosing_node_identity() {
    let text = "a.b: 1\nother: {\nx: {\nkey: 2\n}\n}\na.c: {\nkey: 3\n}\na.c.extra: 4\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(find(&syms, "a"), 0, 0, 6);
    assert_range(find_path(&syms, &["a", "b"]), 0, 0, 6);
    assert_range(find_path(&syms, &["a", "c"]), 6, 0, 6);
    assert_range(find_path(&syms, &["a", "c", "key"]), 7, 0, 6);
    assert_range(find_path(&syms, &["a", "c", "extra"]), 9, 0, 12);
    assert_range(find(&syms, "other"), 1, 0, 8);
    assert_range(find_path(&syms, &["other", "x"]), 2, 0, 4);
    assert_range(find_path(&syms, &["other", "x", "key"]), 3, 0, 6);
}

#[test]
fn reordered_dotted_objects_link_their_actual_array_children() {
    let text = "a.b: 1\nother: {x: {items: [{key: 2}, {key: 3}]}}\na.c: {items: [{key: 4}, {key: 5}]}\na.c.extra: 6\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(find_path(&syms, &["a", "c"]), 2, 0, 34);
    assert_range(find_path(&syms, &["a", "c", "items"]), 2, 6, 11);
    assert_range(find_path(&syms, &["a", "c", "items", "[0]"]), 2, 14, 22);
    assert_range(find_path(&syms, &["a", "c", "items", "[1]"]), 2, 24, 32);
    assert_range(
        find_path(&syms, &["a", "c", "items", "[0]", "key"]),
        2,
        15,
        18,
    );
    assert_range(
        find_path(&syms, &["a", "c", "items", "[1]", "key"]),
        2,
        25,
        28,
    );
    assert_range(find_path(&syms, &["a", "c", "extra"]), 3, 0, 12);
    assert_range(find_path(&syms, &["other", "x", "items"]), 1, 12, 17);
    assert_range(find_path(&syms, &["other", "x", "items", "[0]"]), 1, 20, 28);
    assert_range(find_path(&syms, &["other", "x", "items", "[1]"]), 1, 30, 38);
    assert_range(
        find_path(&syms, &["other", "x", "items", "[0]", "key"]),
        1,
        21,
        24,
    );
    assert_range(
        find_path(&syms, &["other", "x", "items", "[1]", "key"]),
        1,
        31,
        34,
    );
}

#[test]
fn array_and_multiline_openers_use_whitespace_trimmed_entire_body() {
    for ws in ['\t', '\u{00a0}', '\u{2003}', '\u{2028}', '\u{3000}'] {
        let width = ws.len_utf8() as u32;
        for (opener, closer) in [("[", "]"), ("(", ")"), ("((", "))")] {
            let text =
                format!("\u{feff}value:{ws}{opener}{ws}\rchild: 1\r{ws}{closer}{ws}\rafter: 2\r");
            let syms = build_symbols(&parse(&text), &text);
            assert_eq!(syms.len(), 2);
            let value = find(&syms, "value");
            assert_range(value, 0, 3, 9 + opener.len() as u32 + 2 * width);
            if opener == "[" {
                let items = value.children.as_ref().unwrap();
                assert_eq!(items.len(), 1);
                assert_range(&items[0], 1, 0, 8);
            } else {
                assert_eq!(value.kind, SymbolKind::STRING);
                assert!(value.children.is_none());
            }
            assert_range(find(&syms, "after"), 3, 0, 8);
        }
    }
}

#[test]
fn non_spec_whitespace_before_opener_is_scalar_content() {
    for ch in ['\u{180e}', '\u{200b}', '\u{feff}'] {
        let text = format!("text: {ch}{{\nafter: 1\n");
        let syms = build_symbols(&parse(&text), &text);
        assert_eq!(syms.len(), 2);
        assert_range(find(&syms, "text"), 0, 0, 7 + ch.len_utf8() as u32);
        assert_range(find(&syms, "after"), 1, 0, 8);
    }
}

#[test]
fn pinned_parseable_symbol_corpus_has_no_zero_range_fallbacks() {
    fn check(symbols: &[DocumentSymbol], fixture: &std::path::Path, lines: &[&str]) {
        for symbol in symbols {
            assert_ne!(
                symbol.range,
                zero_range(),
                "{}: {}",
                fixture.display(),
                symbol.name
            );
            assert_eq!(symbol.selection_range, symbol.range);
            let range = symbol.range;
            assert_eq!(range.start.line, range.end.line);
            let line = lines.get(range.start.line as usize).expect("source line");
            assert!(range.start.character <= range.end.character);
            assert!(range.end.character as usize <= line.len());
            assert!(line.is_char_boundary(range.start.character as usize));
            assert!(line.is_char_boundary(range.end.character as usize));
            if let Some(children) = &symbol.children {
                check(children, fixture, lines);
            }
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/versions/0.8/tests");
    // Include canonical sources and all lax-parseable categories.
    for (category, expected) in [
        ("valid", 446),
        ("parseable-unrepresentable", 4),
        ("strict-lossy", 13),
    ] {
        let mut directories = vec![root.join(category)];
        let mut fixtures = 0;
        while let Some(directory) = directories.pop() {
            for entry in std::fs::read_dir(&directory).expect("pinned spec directory") {
                let path = entry.expect("spec entry").path();
                if path.is_dir() {
                    directories.push(path);
                } else if path
                    .extension()
                    .is_some_and(|extension| extension == "ktav")
                {
                    let text = std::fs::read_to_string(&path).expect("UTF-8 fixture");
                    let value = ktav::parse(&text)
                        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                    let lines = ktav_lsp::lines::split_lines(&text);
                    check(&build_symbols(&value, &text), &path, &lines);
                    fixtures += 1;
                }
            }
        }
        assert_eq!(fixtures, expected, "pinned {category} source count");
    }
}

#[test]
fn empty_inline_compounds_do_not_shift_later_array_items() {
    let text = "items: [{}, [], {key: 2}, [3, 4], tail]\n";
    let syms = build_symbols(&parse(text), text);
    let items = find(&syms, "items").children.as_ref().unwrap();
    assert_eq!(items.len(), 5);
    for (item, (start, end)) in items
        .iter()
        .zip([(8, 10), (12, 14), (16, 24), (26, 32), (34, 38)])
    {
        assert_range(item, 0, start, end);
    }
    assert_range(find(items[2].children.as_ref().unwrap(), "key"), 0, 17, 20);
    let nested = items[3].children.as_ref().unwrap();
    assert_range(&nested[0], 0, 27, 28);
    assert_range(&nested[1], 0, 30, 31);
}

#[tokio::test]
async fn document_symbol_ranges_follow_negotiated_encoding() {
    use ktav_lsp::Backend;
    use tower_lsp::{lsp_types::*, LanguageServer, LspService};

    let text = "\u{feff}a.b: 1\rother: {x: {\u{1f600}: 2}}\ra.c:\t{\u{1f600}: 3}\r";
    for (encoding, first_start, first_end, key_end) in [
        (PositionEncodingKind::UTF8, 3, 9, 10),
        (PositionEncodingKind::UTF16, 1, 7, 8),
    ] {
        let (service, _socket) = LspService::new(Backend::new);
        let backend = service.inner();
        let initialized = backend
            .initialize(InitializeParams {
                capabilities: ClientCapabilities {
                    general: Some(GeneralClientCapabilities {
                        position_encodings: Some(vec![encoding.clone()]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .expect("initialize");
        assert_eq!(initialized.capabilities.position_encoding, Some(encoding));
        let uri = Url::parse("file:///symbol-ranges.ktav").unwrap();
        backend
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "ktav".into(),
                    version: 1,
                    text: text.into(),
                },
            })
            .await;
        let response = backend
            .document_symbol(DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            })
            .await
            .expect("documentSymbol")
            .expect("symbol response");
        let DocumentSymbolResponse::Nested(syms) = response else {
            panic!("expected nested symbols");
        };
        assert_range(find_path(&syms, &["a", "b"]), 0, first_start, first_end);
        assert_range(find_path(&syms, &["a", "c", "\u{1f600}"]), 2, 6, key_end);
        assert_range(
            find_path(&syms, &["other", "x", "\u{1f600}"]),
            1,
            12,
            key_end + 6,
        );
    }
}

#[test]
fn spec_reopened_dotted_key_keeps_sibling_and_child_positions() {
    let text =
        include_str!("../../spec/versions/0.8/tests/valid/dotted_keys/reopen_after_sibling.ktav");
    let syms = build_symbols(&parse(text), text);
    assert_eq!(
        syms.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["a", "c"]
    );
    let a = find(&syms, "a");
    assert_range(a, 0, 0, 6);
    let children = a.children.as_ref().unwrap();
    assert_range(find(children, "b"), 0, 0, 6);
    assert_range(find(children, "d"), 2, 0, 6);
    assert_range(find(&syms, "c"), 1, 0, 4);
}

#[test]
fn reopened_dotted_key_with_inline_sibling_keeps_each_scope() {
    let text = "a.b: 1\nc: {b: 2}\na.d: 3\nnext: 4\n";
    let syms = build_symbols(&parse(text), text);
    let a = find(&syms, "a");
    assert_range(find(a.children.as_ref().unwrap(), "b"), 0, 0, 6);
    assert_range(find(a.children.as_ref().unwrap(), "d"), 2, 0, 6);
    let c = find(&syms, "c");
    assert_range(c, 1, 0, 9);
    assert_range(find(c.children.as_ref().unwrap(), "b"), 1, 4, 5);
    assert_range(find(&syms, "next"), 3, 0, 7);
}

#[test]
fn reopened_dotted_parents_do_not_exchange_same_named_children() {
    let text = "a.x: 1\nb.x: 2\nb.y: 3\na.y: 4\n";
    let syms = build_symbols(&parse(text), text);
    let a = find(&syms, "a").children.as_ref().unwrap();
    let b = find(&syms, "b").children.as_ref().unwrap();
    assert_range(find(a, "x"), 0, 0, 6);
    assert_range(find(a, "y"), 3, 0, 6);
    assert_range(find(b, "x"), 1, 0, 6);
    assert_range(find(b, "y"), 2, 0, 6);
}

#[test]
fn inline_reopened_dotted_parents_keep_same_named_children() {
    let text = "cfg: {a.x: 1, b.x: 2, b.y: 3, a.y: 4}\n";
    let syms = build_symbols(&parse(text), text);
    let cfg = find(&syms, "cfg").children.as_ref().unwrap();
    let a = find(cfg, "a").children.as_ref().unwrap();
    let b = find(cfg, "b").children.as_ref().unwrap();
    assert_range(find(a, "x"), 0, 8, 9);
    assert_range(find(a, "y"), 0, 32, 33);
    assert_range(find(b, "x"), 0, 16, 17);
    assert_range(find(b, "y"), 0, 24, 25);
}

#[test]
fn spec_block_array_scalars_have_item_positions() {
    let text = include_str!("../../spec/versions/0.8/tests/valid/arrays/scalars.ktav");
    let syms = build_symbols(&parse(text), text);
    let tags = find(&syms, "tags");
    assert_range(tags, 0, 0, 7);
    let items = tags.children.as_ref().unwrap();
    assert_eq!(items.len(), 3);
    for (item, (line, end)) in items.iter().zip([(1, 11), (2, 6), (3, 8)]) {
        assert_range(item, line, 0, end);
    }
}

#[test]
fn block_array_skips_comments_and_links_nested_array_items() {
    let text = "tags: [\n    ## note\n\n    one\n    [two, three]\n    [\n        four\n    ]\n    five\n]\nafter: 1\n";
    let syms = build_symbols(&parse(text), text);
    let items = find(&syms, "tags").children.as_ref().unwrap();
    assert_eq!(items.len(), 4);
    assert_range(&items[0], 3, 0, 7);
    assert_range(&items[1], 4, 0, 16);
    let inline_items = items[1].children.as_ref().unwrap();
    assert_range(&inline_items[0], 4, 5, 8);
    assert_range(&inline_items[1], 4, 10, 15);
    assert_range(&items[2], 5, 0, 5);
    assert_range(&items[2].children.as_ref().unwrap()[0], 6, 0, 12);
    assert_range(&items[3], 8, 0, 8);
    assert_range(find(&syms, "after"), 10, 0, 8);
}

#[test]
fn block_array_closer_with_trailing_space_is_not_an_item() {
    let text = "tags: [\n    one\n]   \nafter: 1\n";
    let syms = build_symbols(&parse(text), text);
    let items = find(&syms, "tags").children.as_ref().unwrap();
    assert_eq!(items.len(), 1);
    assert_range(&items[0], 1, 0, 7);
    assert_range(find(&syms, "after"), 3, 0, 8);
}

#[test]
fn quoted_key_keeps_its_range_and_does_not_consume_sibling() {
    let text = "\"a\": 1\nb: 2\n";
    let syms = build_symbols(&parse(text), text);
    assert_range(find(&syms, "a"), 0, 0, 6);
    assert_range(find(&syms, "b"), 1, 0, 4);
}

#[test]
fn escaped_and_quoted_dotted_segments_keep_source_spans() {
    let text = "a\\.b: 1\nroot.\"x\\u0041\": 2\ntail: 3\n";
    let syms = build_symbols(&parse(text), text);
    assert_range(find(&syms, "a.b"), 0, 0, 7);
    let root = find(&syms, "root");
    assert_range(root, 1, 0, 17);
    assert_range(find(root.children.as_ref().unwrap(), "xA"), 1, 0, 17);
    assert_range(find(&syms, "tail"), 2, 0, 7);
}

#[test]
fn spaced_quoted_dotted_segments_match_decoded_keys() {
    let text = "root . \"a\\\"b\" : 1\nafter: 2\n";
    let syms = build_symbols(&parse(text), text);
    let root = find(&syms, "root");
    assert_range(root, 0, 0, 17);
    assert_range(find(root.children.as_ref().unwrap(), "a\"b"), 0, 0, 17);
    assert_range(find(&syms, "after"), 1, 0, 8);
}

#[test]
fn spaced_quoted_dotted_segments_keep_colon_and_dot_in_names() {
    let text = "root . \"a:b\": 1\nroot . \"a.b\": 2\n";
    let syms = build_symbols(&parse(text), text);
    let root = find(&syms, "root");
    let kids = root.children.as_ref().unwrap();
    assert_range(find(kids, "a:b"), 0, 0, 15);
    assert_range(find(kids, "a.b"), 1, 0, 15);
}

#[test]
fn quoted_inline_dotted_segment_keeps_sibling_ranges() {
    let text = "cfg: {a.\"b,c\": 1, tail: 2}\n";
    let syms = build_symbols(&parse(text), text);
    let kids = find(&syms, "cfg").children.as_ref().unwrap();
    let a = find(kids, "a");
    assert_range(a, 0, 6, 7);
    assert_range(find(a.children.as_ref().unwrap(), "b,c"), 0, 8, 13);
    assert_range(find(kids, "tail"), 0, 18, 22);

    let text = "cfg: {a. \"b,c\": 1, tail: 2}\n";
    let syms = build_symbols(&parse(text), text);
    let kids = find(&syms, "cfg").children.as_ref().unwrap();
    let a = find(kids, "a");
    assert_range(find(a.children.as_ref().unwrap(), "b,c"), 0, 8, 14);
    assert_range(find(kids, "tail"), 0, 19, 23);
}

#[test]
fn unicode_surrogate_escape_key_matches_value_name() {
    let text = "\\uD83D\\uDE00: 1\nnext: 2\n";
    let syms = build_symbols(&parse(text), text);
    assert_range(find(&syms, "\u{1f600}"), 0, 0, 15);
    assert_range(find(&syms, "next"), 1, 0, 7);
}

#[test]
fn inline_quoted_and_escaped_keys_keep_raw_byte_columns() {
    let text = "cfg: {\"a\": 1, b\\.c: 2, \"u\\u0041\": 3, tail: 4}\nnext: 5\n";
    let syms = build_symbols(&parse(text), text);
    let kids = find(&syms, "cfg").children.as_ref().unwrap();
    assert_range(find(kids, "a"), 0, 6, 9);
    assert_range(find(kids, "b.c"), 0, 14, 18);
    assert_range(find(kids, "uA"), 0, 23, 32);
    assert_range(find(kids, "tail"), 0, 37, 41);
    assert_range(find(&syms, "next"), 1, 0, 7);
}

#[test]
fn hash_keys_and_array_items_are_not_comments() {
    let object = "#child: 1\n## comment\n#value: 2\nend: 3\n";
    let syms = build_symbols(&parse(object), object);
    assert_range(find(&syms, "#child"), 0, 0, 9);
    assert_range(find(&syms, "#value"), 2, 0, 9);
    assert_range(find(&syms, "end"), 3, 0, 6);

    let array = "#value\n## comment\nother\n";
    let syms = build_symbols(&parse(array), array);
    assert_range(&syms[0], 0, 0, 6);
    assert_range(&syms[1], 2, 0, 5);
}

#[test]
fn bom_quoted_hash_and_inline_keys_keep_original_columns() {
    let text = "\u{feff}\"#child\": {\"a\\u0041\": 1, #value: 2}\nnext: 3\n";
    let syms = build_symbols(&parse(text), text);
    let child = find(&syms, "#child");
    assert_range(child, 0, 3, 38);
    let kids = child.children.as_ref().unwrap();
    assert_range(find(kids, "aA"), 0, 14, 23);
    assert_range(find(kids, "#value"), 0, 28, 34);
    assert_range(find(&syms, "next"), 1, 0, 7);
}

#[test]
fn inline_object_value_keys_get_precise_ranges() {
    // `a: {name: alice}` (valid/inline/object/single_pair.ktav) —
    // before the fix, `name` had no hit at all (only `a` did) and
    // fell back to zero_range().
    let text = "a: {name: alice}\n";
    let syms = build_symbols(&parse(text), text);
    let a = find(&syms, "a");
    let kids = a.children.as_ref().expect("a has children");
    let name = find(kids, "name");
    assert_range(name, 0, 4, 8);
}

#[test]
fn inline_object_multiple_keys_each_get_own_range() {
    // valid/inline/object/multiple_pairs.ktav.
    let text = "server: {host: localhost, port: 8080, tls: true}\n";
    let syms = build_symbols(&parse(text), text);
    let server = find(&syms, "server");
    let kids = server.children.as_ref().unwrap();
    for (name, want_col) in [("host", 9), ("port", 26), ("tls", 38)] {
        let sym = find(kids, name);
        assert_range(sym, 0, want_col, want_col + name.len() as u32);
    }
}

#[test]
fn inline_nested_array_of_objects_keys_resolve() {
    // valid/inline/nested/mixed.ktav's first line — inline array of
    // inline objects, two nesting levels inside one key's value.
    let text = "users: [{name: alice, age: 30}, {name: bob, age: 25}]\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    let users = find(&syms, "users");
    let kids = users.children.as_ref().expect("users has children");
    assert_eq!(kids.len(), 2);
    let alice = find(kids[0].children.as_ref().unwrap(), "name");
    assert_range(alice, 0, 9, 13);
    let bob = find(kids[1].children.as_ref().unwrap(), "name");
    assert_range(bob, 0, 33, 37);
}

#[test]
fn inline_dotted_key_creates_nested_symbols() {
    // valid/inline/object/dotted_keys.ktav: `cfg: {a.b: 1, a.c: 2}`
    // → cfg.a.b / cfg.a.c, matching top-level dotted-key nesting.
    let text = "cfg: {a.b: 1, a.c: 2}\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    let cfg = find(&syms, "cfg");
    let a = find(cfg.children.as_ref().unwrap(), "a");
    let a_kids = a.children.as_ref().expect("a has children");
    assert_range(a, 0, 6, 7);
    assert_range(find(a_kids, "b"), 0, 8, 9);
    assert_range(find(a_kids, "c"), 0, 16, 17);
}

#[test]
fn raw_marker_inside_inline_object_does_not_corrupt_hits() {
    // valid/raw_marker/in_inline.ktav.
    let text = "cfg: {real_num: 42, str_num:: 42, real_bool: true, str_bool:: true}\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    let cfg = find(&syms, "cfg");
    let kids = cfg.children.as_ref().unwrap();
    for (key, start, end) in [
        ("real_num", 6, 14),
        ("str_num", 20, 27),
        ("real_bool", 34, 43),
        ("str_bool", 51, 59),
    ] {
        assert_range(find(kids, key), 0, start, end);
    }
}

#[test]
fn top_level_object_brace_wrapper_is_transparent() {
    // valid/top_level/multiline_object_opener.ktav.
    let text = "{\n    name: alice\n    port: 8080\n}\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    let name = find(&syms, "name");
    assert_range(name, 1, 0, 15);
    let port = find(&syms, "port");
    assert_range(port, 2, 0, 14);
}

#[test]
fn top_level_inline_object_literal_is_transparent() {
    // valid/top_level_inline/object.ktav — the WHOLE document is one
    // inline object literal; before the fix `find_key_separator`
    // matched the colon inside it and produced a garbage key.
    let text = "{host: localhost, port: 8080, tls: true}\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    assert_eq!(syms.len(), 3);
    for (name, start, end) in [("host", 1, 5), ("port", 18, 22), ("tls", 30, 33)] {
        assert_range(find(&syms, name), 0, start, end);
    }
}

#[test]
fn top_level_array_bracket_wrapper_is_transparent() {
    // document_symbols_top_level_array_of_objects_have_children
    // (integration.rs) only checks names; this also pins positions.
    let text = "[\n{\n    name: alice\n}\n{\n    name: bob\n}\n]\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    assert_eq!(syms.len(), 2);
    let name0 = find(syms[0].children.as_ref().unwrap(), "name");
    assert_range(name0, 2, 0, 15);
    let name1 = find(syms[1].children.as_ref().unwrap(), "name");
    assert_range(name1, 5, 0, 13);
}

#[test]
fn top_level_multiline_array_items_have_exact_ranges() {
    let text = "[\n    foo\n    bar\n    baz\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 3);
    for (sym, line) in syms.iter().zip(1..=3) {
        assert_range(sym, line, 0, 7);
    }
}

#[test]
fn pair_shaped_array_scalars_do_not_open_compounds() {
    for scalar in ["foo: {", "foo: [", "foo: (", "foo: ((", "foo:: {", "foo {"] {
        let text = format!(":: start\n{scalar}\nbar\n");
        let syms = build_symbols(&parse(&text), &text);
        assert_eq!(syms.len(), 3, "{scalar}");
        assert_range(&syms[0], 0, 0, 8);
        assert_range(&syms[1], 1, 0, scalar.len() as u32);
        assert_range(&syms[2], 2, 0, 3);
    }
}

#[test]
fn spec_pair_shaped_top_level_array_items_keep_ranges() {
    let text = include_str!(
        "../../spec/versions/0.8/tests/valid/top_level_array/pair_shaped_first_item.ktav"
    );
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 4);
    for (sym, (line, end)) in syms.iter().zip([(0, 18), (1, 2), (2, 10), (3, 10)]) {
        assert_range(sym, line, 0, end);
    }
}

#[test]
fn nested_array_scalar_opener_does_not_hide_following_top_level_item() {
    let text = "[\n[\nfoo: {\nbar\n]\nafter\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 1, 0, 1);
    let nested = syms[0].children.as_ref().unwrap();
    assert_range(&nested[0], 2, 0, 6);
    assert_range(&nested[1], 3, 0, 3);
    assert_range(&syms[1], 5, 0, 5);
}

#[test]
fn scalar_opener_in_root_array_does_not_hide_later_object_key() {
    let text = ":: start\nfoo: {\nbar\n{\nchild: 1\n}\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 4);
    assert_range(&syms[3], 3, 0, 1);
    assert_range(find(syms[3].children.as_ref().unwrap(), "child"), 4, 0, 8);
}

#[test]
fn raw_pair_opener_inside_object_remains_scalar() {
    let text = "[\n{\nraw:: {\nchild: 1\n}\nafter\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    let object = syms[0].children.as_ref().unwrap();
    assert_range(find(object, "raw"), 2, 0, 7);
    assert_range(find(object, "child"), 3, 0, 8);
    assert_range(&syms[1], 5, 0, 5);
}

#[test]
fn object_pair_opener_in_top_level_array_keeps_following_item() {
    let text = "[\n{\nchild: {\nleaf: one\n}\n}\nafter\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 1, 0, 1);
    assert_range(&syms[1], 6, 0, 5);
}

#[test]
fn top_level_array_wrapper_skips_comments_and_preserves_bom_cr_positions() {
    let text = "\u{feff}## lead\r[\r    #value\r    ## comment\r    bar\r]\r";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 2, 0, 10);
    assert_range(&syms[1], 4, 0, 7);
}

#[test]
fn top_level_array_nested_compound_does_not_shift_following_item() {
    let text = "[\n    first\n    [\n        child\n    ]\n    last\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 3);
    assert_range(&syms[0], 1, 0, 9);
    assert_range(&syms[1], 2, 0, 5);
    assert_range(&syms[2], 5, 0, 8);
}

#[test]
fn multiline_top_level_array_links_inline_array_items() {
    let text = "[\n    [1, 2]\n    [3, 4]\n]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    for (i, line) in [1, 2].into_iter().enumerate() {
        assert_range(&syms[i], line, 0, 10);
        let items = syms[i].children.as_ref().expect("inline array children");
        assert_eq!(items.len(), 2);
        assert_range(&items[0], line, 5, 6);
        assert_range(&items[1], line, 8, 9);
    }
}

#[test]
fn bare_top_level_array_links_inline_array_item() {
    let text = "first\n[1, 2]\n";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 0, 0, 5);
    assert_range(&syms[1], 1, 0, 6);
    let items = syms[1].children.as_ref().expect("inline array children");
    assert_eq!(items.len(), 2);
    assert_range(&items[0], 1, 1, 2);
    assert_range(&items[1], 1, 4, 5);
}

#[test]
fn bare_and_inline_top_level_arrays_keep_item_lines() {
    let bare = "foo\nbar\n";
    let syms = build_symbols(&parse(bare), bare);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 0, 0, 3);
    assert_range(&syms[1], 1, 0, 3);

    let inline = "[foo]\n";
    let syms = build_symbols(&parse(inline), inline);
    assert_eq!(syms.len(), 1);
    assert_range(&syms[0], 0, 1, 4);
}

#[test]
fn spec_top_level_inline_array_has_distinct_item_ranges() {
    let text = include_str!("../../spec/versions/0.8/tests/valid/top_level_inline/array.ktav");
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 5);
    for (i, (start, end)) in [(1, 2), (4, 5), (7, 8), (10, 11), (13, 14)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(syms[i].name, format!("[{i}]"));
        assert_range(&syms[i], 0, start, end);
    }
}

#[test]
fn spec_inline_nested_array_items_have_exact_ranges() {
    let text = include_str!("../../spec/versions/0.8/tests/valid/inline/nested/mixed.ktav");
    let syms = build_symbols(&parse(text), text);
    let users = find(&syms, "users").children.as_ref().unwrap();
    assert_eq!(users.len(), 2);
    assert_range(&users[0], 0, 8, 30);
    assert_range(&users[1], 0, 32, 52);

    let data = find(&syms, "data").children.as_ref().unwrap();
    let names = find(data, "names").children.as_ref().unwrap();
    let ages = find(data, "ages").children.as_ref().unwrap();
    assert_eq!(names.len(), 2);
    assert_eq!(ages.len(), 2);
    assert_range(&names[0], 1, 15, 20);
    assert_range(&names[1], 1, 22, 25);
    assert_range(&ages[0], 1, 35, 37);
    assert_range(&ages[1], 1, 39, 41);
}

#[test]
fn spec_top_level_inline_nested_array_uses_own_item_range() {
    let text = include_str!("../../spec/versions/0.8/tests/valid/top_level_inline/unquoted_key_context_from_active_scope.ktav");
    let syms = build_symbols(&parse(text), text);
    assert_range(&syms[0], 0, 1, 15);
    let object = syms[0].children.as_ref().unwrap();
    let b_items = find(object, "b").children.as_ref().unwrap();
    assert_range(&b_items[0], 0, 12, 13);
}

#[test]
fn inline_array_ranges_keep_bom_hash_quoted_key_and_cr_columns() {
    let text = "\u{feff}[#value, {\"#key\": [1, 2]}]\r";
    let syms = build_symbols(&parse(text), text);
    assert_eq!(syms.len(), 2);
    assert_range(&syms[0], 0, 4, 10);
    assert_range(&syms[1], 0, 12, 28);
    let key = find(syms[1].children.as_ref().unwrap(), "#key");
    assert_range(key, 0, 13, 19);
    let items = key.children.as_ref().unwrap();
    assert_range(&items[0], 0, 22, 23);
    assert_range(&items[1], 0, 25, 26);
}

#[test]
fn top_level_inline_array_of_objects_keys_resolve() {
    // valid/top_level_inline/unquoted_key_context_from_active_scope.ktav.
    let text = "[{a: 1, b: [2]}]\n";
    let value = parse(text);
    let syms = build_symbols(&value, text);
    assert_eq!(syms.len(), 1);
    let obj_kids = syms[0].children.as_ref().expect("[0] has children");
    assert_range(&syms[0], 0, 1, 15);
    assert_range(find(obj_kids, "a"), 0, 2, 3);
    // `b`'s own hit resolves too, even though its value is itself an
    // inline array (no keys inside it to check further).
    assert_range(find(obj_kids, "b"), 0, 8, 9);
}
