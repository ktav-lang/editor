//! Consumer-visible declaration envelopes and precise outline navigation anchors.

use ktav::Value;
use ktav_lsp::symbols::build_symbols;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

type Span = (u32, u32, u32, u32);

fn range((line, character, end_line, end_character): Span) -> Range {
    Range {
        start: Position { line, character },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

fn parse(text: &str) -> Value {
    ktav::parse(text).expect("fixture must parse")
}

fn find<'a>(symbols: &'a [DocumentSymbol], name: &str) -> &'a DocumentSymbol {
    symbols
        .iter()
        .find(|symbol| symbol.name == name)
        .unwrap_or_else(|| panic!("no symbol named '{name}' among {symbols:?}"))
}

fn find_path<'a>(symbols: &'a [DocumentSymbol], path: &[&str]) -> &'a DocumentSymbol {
    let mut symbol = find(symbols, path[0]);
    for name in &path[1..] {
        symbol = find(symbol.children.as_ref().expect("compound children"), name);
    }
    symbol
}

fn assert_spans(symbol: &DocumentSymbol, declaration: Span, selection: Span) {
    assert_eq!(
        symbol.range,
        range(declaration),
        "declaration for {}",
        symbol.name
    );
    assert_eq!(
        symbol.selection_range,
        range(selection),
        "navigation for {}",
        symbol.name
    );
}

fn assert_contains(parent: Range, child: Range) {
    assert!(
        parent.start <= child.start && child.end <= parent.end,
        "{parent:?} must contain {child:?}"
    );
}

fn assert_tree_contains(symbols: &[DocumentSymbol]) {
    for symbol in symbols {
        assert_contains(symbol.range, symbol.selection_range);
        if let Some(children) = &symbol.children {
            for child in children {
                assert_contains(symbol.range, child.range);
            }
            assert_tree_contains(children);
        }
    }
}

#[test]
fn inline_compounds_cover_values_and_closers_without_adjacent_siblings() {
    let text = "outer: {nested: {child: 1}, empty: {}, list: [], after: 2}\nnext: 3\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "outer"), (0, 0, 0, 58), (0, 0, 0, 5));
    assert_spans(
        find_path(&symbols, &["outer", "nested"]),
        (0, 8, 0, 26),
        (0, 8, 0, 14),
    );
    assert_spans(
        find_path(&symbols, &["outer", "nested", "child"]),
        (0, 17, 0, 25),
        (0, 17, 0, 22),
    );
    assert_spans(
        find_path(&symbols, &["outer", "empty"]),
        (0, 28, 0, 37),
        (0, 28, 0, 33),
    );
    assert_spans(
        find_path(&symbols, &["outer", "list"]),
        (0, 39, 0, 47),
        (0, 39, 0, 43),
    );
    assert_spans(
        find_path(&symbols, &["outer", "after"]),
        (0, 49, 0, 57),
        (0, 49, 0, 54),
    );
    assert_spans(find(&symbols, "next"), (1, 0, 1, 7), (1, 0, 1, 4));
    let children = find(&symbols, "outer").children.as_ref().unwrap();
    for pair in children.windows(2) {
        assert!(pair[0].range.end <= pair[1].range.start);
    }
    assert_tree_contains(&symbols);
}

#[test]
fn block_parent_includes_indented_children_comments_and_exact_closer() {
    let text =
        "outer:\t{\n    child: 1\n    ## inside\n    empty: [\n    ]  \n    }   \nafter: 2\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "outer"), (0, 0, 5, 5), (0, 0, 0, 5));
    assert_spans(
        find_path(&symbols, &["outer", "child"]),
        (1, 4, 1, 12),
        (1, 4, 1, 9),
    );
    assert_spans(
        find_path(&symbols, &["outer", "empty"]),
        (3, 4, 4, 5),
        (3, 4, 3, 9),
    );
    assert_spans(find(&symbols, "after"), (6, 0, 6, 8), (6, 0, 6, 5));
    assert_tree_contains(&symbols);
}

#[test]
fn reopened_dotted_definitions_enclose_all_occurrences_and_keep_first_navigation() {
    let text = "a.b: 1\nother: {x: {key: 2}}\na.c: {items: [{key: 4}, {key: 5}]}\na.c.extra: 6\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "a"), (0, 0, 3, 12), (0, 0, 0, 1));
    assert_spans(find_path(&symbols, &["a", "b"]), (0, 2, 0, 6), (0, 2, 0, 3));
    assert_spans(
        find_path(&symbols, &["a", "c"]),
        (2, 2, 3, 12),
        (2, 2, 2, 3),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "items"]),
        (2, 6, 2, 33),
        (2, 6, 2, 11),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "items", "[0]"]),
        (2, 14, 2, 22),
        (2, 14, 2, 15),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "items", "[1]"]),
        (2, 24, 2, 32),
        (2, 24, 2, 25),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "extra"]),
        (3, 4, 3, 12),
        (3, 4, 3, 9),
    );
    assert_spans(find(&symbols, "other"), (1, 0, 1, 20), (1, 0, 1, 5));
    assert_spans(
        find_path(&symbols, &["other", "x"]),
        (1, 8, 1, 19),
        (1, 8, 1, 9),
    );
    assert_spans(
        find_path(&symbols, &["other", "x", "key"]),
        (1, 12, 1, 18),
        (1, 12, 1, 15),
    );
    assert_tree_contains(&symbols);
}

#[test]
fn inline_interleaved_dotted_prefix_envelopes_may_overlap() {
    let text = "cfg: {a.x: 1, b.x: 2, b.y: 3, a.y: 4}\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(
        find_path(&symbols, &["cfg", "a"]),
        (0, 6, 0, 36),
        (0, 6, 0, 7),
    );
    assert_spans(
        find_path(&symbols, &["cfg", "b"]),
        (0, 14, 0, 28),
        (0, 14, 0, 15),
    );
    for (parent, key, start, end) in [
        ("a", "x", 8, 12),
        ("b", "x", 16, 20),
        ("b", "y", 24, 28),
        ("a", "y", 32, 36),
    ] {
        assert_spans(
            find_path(&symbols, &["cfg", parent, key]),
            (0, start, 0, end),
            (0, start, 0, start + 1),
        );
    }
    assert_tree_contains(&symbols);
}

#[test]
fn block_reopened_explicit_objects_keep_closers_and_later_children() {
    let text = "a.b: 1\nother: {\nx: {\nkey: 2\n}\n}\na.c: {\nkey: 3\n}\na.c.extra: 4\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "a"), (0, 0, 9, 12), (0, 0, 0, 1));
    assert_spans(
        find_path(&symbols, &["a", "c"]),
        (6, 2, 9, 12),
        (6, 2, 6, 3),
    );
    assert_spans(find(&symbols, "other"), (1, 0, 5, 1), (1, 0, 1, 5));
    assert_spans(
        find_path(&symbols, &["other", "x"]),
        (2, 0, 4, 1),
        (2, 0, 2, 1),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "key"]),
        (7, 0, 7, 6),
        (7, 0, 7, 3),
    );
    assert_spans(
        find_path(&symbols, &["a", "c", "extra"]),
        (9, 4, 9, 12),
        (9, 4, 9, 9),
    );
    assert_tree_contains(&symbols);
}

#[test]
fn inline_array_items_have_small_anchors_and_disjoint_complete_bodies() {
    let text = "items: [{}, [], {key: 2}, [3, 4], tail]\n";
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "items"), (0, 0, 0, 39), (0, 0, 0, 5));
    let items = find(&symbols, "items").children.as_ref().unwrap();
    for (i, (start, end, anchor_end)) in [
        (8, 10, 9),
        (12, 14, 13),
        (16, 24, 17),
        (26, 32, 27),
        (34, 38, 38),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(items[i].name, format!("[{i}]"));
        assert_spans(&items[i], (0, start, 0, end), (0, start, 0, anchor_end));
    }
    assert_spans(
        find(items[2].children.as_ref().unwrap(), "key"),
        (0, 17, 0, 23),
        (0, 17, 0, 20),
    );
    for pair in items.windows(2) {
        assert!(pair[0].range.end < pair[1].range.start);
    }
    assert_tree_contains(&symbols);
}

#[test]
fn top_level_arrays_enclose_block_inline_and_multiline_items() {
    let text = "[\n    {\n        name: alice\n    }  \n    []\n    [1, 2]\n    (\n        child: {\n    )  \n    last\n]\n";
    let symbols = build_symbols(&parse(text), text);
    assert_eq!(symbols.len(), 5);
    assert_spans(&symbols[0], (1, 4, 3, 5), (1, 4, 1, 5));
    assert_spans(
        find(symbols[0].children.as_ref().unwrap(), "name"),
        (2, 8, 2, 19),
        (2, 8, 2, 12),
    );
    assert_spans(&symbols[1], (4, 4, 4, 6), (4, 4, 4, 5));
    assert_spans(&symbols[2], (5, 4, 5, 10), (5, 4, 5, 5));
    assert_spans(&symbols[3], (6, 4, 8, 5), (6, 4, 6, 5));
    assert_eq!(symbols[3].kind, SymbolKind::STRING);
    assert_spans(&symbols[4], (9, 4, 9, 8), (9, 4, 9, 8));
    assert_tree_contains(&symbols);
}

#[test]
fn multiline_string_ranges_include_matching_closer_not_body_shaped_symbols() {
    for (opener, closer) in [("(", ")"), ("((", "))")] {
        let text = format!(
            "text: {opener}\nchild: {{\n## literal string content\n  {closer}   \nafter: 2\n"
        );
        let symbols = build_symbols(&parse(&text), &text);
        assert_eq!(symbols.len(), 2);
        let string = find(&symbols, "text");
        assert_eq!(string.kind, SymbolKind::STRING);
        assert!(string.children.is_none());
        assert_spans(string, (0, 0, 3, 2 + closer.len() as u32), (0, 0, 0, 4));
        assert_spans(find(&symbols, "after"), (4, 0, 4, 8), (4, 0, 4, 5));
        assert_tree_contains(&symbols);
    }
}

#[test]
fn exact_body_dispatch_preserves_bom_whitespace_and_all_line_endings() {
    let whitespace = [
        '\t', '\u{000b}', '\u{000c}', ' ', '\u{0085}', '\u{00a0}', '\u{1680}', '\u{2000}',
        '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}',
        '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}',
        '\u{3000}',
    ];
    for ws in whitespace {
        let width = ws.len_utf8() as u32;
        for eol in ["\n", "\r", "\r\n"] {
            let text = format!(
                "\u{feff}{ws}outer:{ws}{{{ws}{eol}{ws}child: 1{eol}{ws}}}{ws}{eol}after: 2{eol}"
            );
            let symbols = build_symbols(&parse(&text), &text);
            assert_spans(
                find(&symbols, "outer"),
                (0, 3 + width, 2, width + 1),
                (0, 3 + width, 0, 8 + width),
            );
            assert_spans(
                find_path(&symbols, &["outer", "child"]),
                (1, width, 1, width + 8),
                (1, width, 1, width + 5),
            );
            assert_spans(find(&symbols, "after"), (3, 0, 3, 8), (3, 0, 3, 5));
            assert_tree_contains(&symbols);
        }
    }
}

#[test]
fn escaped_quoted_and_spaced_segments_navigate_to_exact_source_keys() {
    let text = "cfg: {\"a\": 1, b\\.c: 2, \"u\\u0041\": 3, tail: 4}\nroot . \"a\\\"b\" : 1\n\\uD83D\\uDE00: 1\n";
    let symbols = build_symbols(&parse(text), text);
    for (name, start, key_end, end) in [
        ("a", 6, 9, 12),
        ("b.c", 14, 18, 21),
        ("uA", 23, 32, 35),
        ("tail", 37, 41, 44),
    ] {
        assert_spans(
            find_path(&symbols, &["cfg", name]),
            (0, start, 0, end),
            (0, start, 0, key_end),
        );
    }
    assert_spans(find(&symbols, "root"), (1, 0, 1, 17), (1, 0, 1, 4));
    assert_spans(
        find_path(&symbols, &["root", "a\"b"]),
        (1, 7, 1, 17),
        (1, 7, 1, 13),
    );
    assert_spans(find(&symbols, "\u{1f600}"), (2, 0, 2, 15), (2, 0, 2, 12));
    assert_tree_contains(&symbols);
    let inline = "cfg: {a. \"b,c\": 1, tail: 2}\n";
    let symbols = build_symbols(&parse(inline), inline);
    assert_spans(
        find_path(&symbols, &["cfg", "a"]),
        (0, 6, 0, 17),
        (0, 6, 0, 7),
    );
    assert_spans(
        find_path(&symbols, &["cfg", "a", "b,c"]),
        (0, 9, 0, 17),
        (0, 9, 0, 14),
    );
    assert_spans(
        find_path(&symbols, &["cfg", "tail"]),
        (0, 19, 0, 26),
        (0, 19, 0, 23),
    );
    assert_tree_contains(&symbols);
}

#[test]
fn scalar_suffixes_raw_openers_and_pair_shaped_array_items_do_not_open_scopes() {
    for token in ["{", "[", "(", "(("] {
        for marker in [":", "::"] {
            let body = if marker == "::" {
                token.to_string()
            } else {
                format!("hello {token}")
            };
            let text = format!("before: 0\ntext{marker}\t{body}\nafter: 1\n");
            let symbols = build_symbols(&parse(&text), &text);
            assert_spans(
                find(&symbols, "text"),
                (1, 0, 1, (5 + marker.len() + body.len()) as u32),
                (1, 0, 1, 4),
            );
            assert_spans(find(&symbols, "after"), (2, 0, 2, 8), (2, 0, 2, 5));
            assert_tree_contains(&symbols);
        }
    }
    let text = ":: start\nfoo: {\nbar\n{\nchild: 1\n}\n";
    let symbols = build_symbols(&parse(text), text);
    assert_eq!(symbols.len(), 4);
    assert_spans(&symbols[1], (1, 0, 1, 6), (1, 0, 1, 6));
    assert_spans(&symbols[3], (3, 0, 5, 1), (3, 0, 3, 1));
    assert_spans(
        find(symbols[3].children.as_ref().unwrap(), "child"),
        (4, 0, 4, 8),
        (4, 0, 4, 5),
    );
    assert_tree_contains(&symbols);
}

#[test]
fn spec_reopened_explicit_object_keeps_first_selection_and_extends_envelope() {
    let text =
        include_str!("../../spec/versions/0.8/tests/valid/dotted_keys/extend_explicit_object.ktav");
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find(&symbols, "a"), (0, 0, 3, 6), (0, 0, 0, 1));
    assert_spans(find_path(&symbols, &["a", "x"]), (1, 4, 1, 8), (1, 4, 1, 5));
    assert_spans(find_path(&symbols, &["a", "y"]), (3, 2, 3, 6), (3, 2, 3, 3));
    assert_tree_contains(&symbols);
}

#[test]
fn pinned_parseable_symbol_corpus_has_source_boundaries_and_containing_parents() {
    fn check(symbols: &[DocumentSymbol], fixture: &std::path::Path, lines: &[&str]) {
        for symbol in symbols {
            for span in [symbol.range, symbol.selection_range] {
                assert!(
                    span.start < span.end,
                    "{}: {}: {span:?}",
                    fixture.display(),
                    symbol.name
                );
                for position in [span.start, span.end] {
                    let line = lines.get(position.line as usize).expect("source line");
                    assert!(
                        position.character as usize <= line.len(),
                        "{}: {}: {span:?}",
                        fixture.display(),
                        symbol.name
                    );
                    assert!(line.is_char_boundary(position.character as usize));
                }
            }
            assert_contains(symbol.range, symbol.selection_range);
            if let Some(children) = &symbol.children {
                for child in children {
                    assert_contains(symbol.range, child.range);
                }
                check(children, fixture, lines);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/versions/0.8/tests");
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
                    check(
                        &build_symbols(&value, &text),
                        &path,
                        &ktav_lsp::lines::split_lines(&text),
                    );
                    fixtures += 1;
                }
            }
        }
        assert_eq!(fixtures, expected, "pinned {category} source count");
    }
}

#[test]
fn empty_inline_values_include_separator_but_select_only_key() {
    let text = include_str!("../../spec/versions/0.8/tests/valid/inline/object/empty_value.ktav");
    let symbols = build_symbols(&parse(text), text);
    assert_spans(find_path(&symbols, &["a", "x"]), (0, 4, 0, 6), (0, 4, 0, 5));
    assert_spans(
        find_path(&symbols, &["a", "y"]),
        (0, 8, 0, 12),
        (0, 8, 0, 9),
    );
    assert_spans(
        find_path(&symbols, &["b", "empty"]),
        (1, 4, 1, 10),
        (1, 4, 1, 9),
    );
    assert_tree_contains(&symbols);
}

#[test]
fn raw_structural_items_keep_precise_scalar_navigation_inside_array_envelope() {
    let text =
        include_str!("../../spec/versions/0.8/tests/valid/raw_marker/structural_item_tokens.ktav");
    let symbols = build_symbols(&parse(text), text);
    let items = find(&symbols, "items");
    assert_spans(items, (0, 0, 7, 1), (0, 0, 0, 5));
    for (i, end) in [8, 8, 8, 9, 9, 12].into_iter().enumerate() {
        let item = &items.children.as_ref().unwrap()[i];
        assert_eq!(item.kind, SymbolKind::STRING);
        assert!(item.children.is_none());
        assert_spans(
            item,
            (i as u32 + 1, 4, i as u32 + 1, end),
            (i as u32 + 1, 4, i as u32 + 1, end),
        );
    }
    assert_tree_contains(&symbols);
}

#[test]
fn non_spec_whitespace_is_scalar_content_not_a_compound_opener() {
    for ch in ['\u{180e}', '\u{200b}', '\u{feff}'] {
        let text = format!("text: {ch}{{\nafter: 1\n");
        let symbols = build_symbols(&parse(&text), &text);
        assert_spans(
            find(&symbols, "text"),
            (0, 0, 0, 7 + ch.len_utf8() as u32),
            (0, 0, 0, 4),
        );
        assert_spans(find(&symbols, "after"), (1, 0, 1, 8), (1, 0, 1, 5));
        assert_tree_contains(&symbols);
    }
}

#[tokio::test]
async fn document_symbol_ranges_follow_negotiated_encoding() {
    use ktav_lsp::Backend;
    use tower_lsp::{lsp_types::*, LanguageServer, LspService};

    for eol in ["\n", "\r", "\r\n"] {
        let text = format!("\u{feff}outer: {{nested: {{\u{1f600}: 1}}, empty: []}}   {eol}value: ({eol}\u{1f600}{eol}  )  {eol}next: 2{eol}");
        for (
            encoding,
            outer_start,
            outer_end,
            nested_start,
            nested_end,
            key_start,
            key_end,
            key_selection_end,
        ) in [
            (PositionEncodingKind::UTF8, 3, 40, 11, 28, 20, 27, 24),
            (PositionEncodingKind::UTF16, 1, 36, 9, 24, 18, 23, 20),
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
                        text: text.clone(),
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
            let DocumentSymbolResponse::Nested(symbols) = response else {
                panic!("expected nested symbols");
            };
            assert_spans(
                find(&symbols, "outer"),
                (0, outer_start, 0, outer_end),
                (0, outer_start, 0, outer_start + 5),
            );
            assert_spans(
                find_path(&symbols, &["outer", "nested"]),
                (0, nested_start, 0, nested_end),
                (0, nested_start, 0, nested_start + 6),
            );
            assert_spans(
                find_path(&symbols, &["outer", "nested", "\u{1f600}"]),
                (0, key_start, 0, key_end),
                (0, key_start, 0, key_selection_end),
            );
            assert_spans(find(&symbols, "value"), (1, 0, 3, 3), (1, 0, 1, 5));
            assert_spans(find(&symbols, "next"), (4, 0, 4, 7), (4, 0, 4, 4));
            assert_tree_contains(&symbols);
        }
    }
}
