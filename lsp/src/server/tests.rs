use super::*;
use crate::diagnostics::parse_for_diagnostics;
use crate::semantic::semantic_tokens;
use crate::symbols::build_symbols;
use crate::tokens::byte_to_utf16;
use tower_lsp::LanguageServer;

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

// ---- § 3.1: leading BOM is metadata, not content ----

/// spec/versions/0.8/tests/valid/scalars/leading_bom.ktav: a raw UTF-8
/// BOM (`EF BB BF`) followed by `host: value`. Before the § 3.1 fix in
/// `crate::lines`, hover resolved the key as `"\u{feff}host"` (the BOM
/// glued onto the key by a naive line slice) — not found in the parsed
/// `Value` (whose key is plain `"host"`), so hover fell back to the
/// generic "value" message instead of "string: `value`".
#[test]
fn hover_resolves_key_on_line_with_leading_bom() {
    let text = "\u{FEFF}host: value\n";
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let (service, _socket) = tower_lsp::LspService::new(Backend::new);
    let backend = service.inner();
    let uri = Url::parse("file:///bom-fixture.ktav").unwrap();

    rt.block_on(backend.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "ktav".to_string(),
            version: 1,
            text: text.to_string(),
        },
    }));

    let hover = rt
        .block_on(backend.hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        }))
        .expect("hover must not error");
    let md = match hover.expect("hover must resolve a value").contents {
        HoverContents::Markup(m) => m.value,
        other => format!("{other:?}"),
    };
    assert!(md.contains("**host**"), "got {md:?}");
    assert!(
        md.contains("string: `value`"),
        "expected the resolved value's type, got {md:?}"
    );
}

#[test]
fn document_symbols_resolve_key_on_line_with_leading_bom() {
    let text = "\u{FEFF}host: value\n";
    let value = ktav::parse(text).expect("fixture must parse");
    let syms = build_symbols(&value, text);
    let host = syms.iter().find(|s| s.name == "host").expect("host key");
    assert_eq!(host.range.start.line, 0);
    assert_eq!(
        host.range.start.character, 3,
        "the LSP byte column must include the BOM"
    );
    assert_eq!(host.range.end.character, 3 + "host: value".len() as u32);
}

#[test]
fn semantic_tokens_property_span_counts_leading_bom_in_utf8_and_utf16() {
    let text = "\u{FEFF}host: value\n";
    let toks = semantic_tokens(text);
    let first = toks.first().expect("at least one token");
    // The parser ignores the BOM, while client-facing coordinates count it.
    assert_eq!(first.token_type, 4);
    assert_eq!(first.delta_start, 3);
    assert_eq!(first.length, 4);

    let mut utf16 = toks;
    convert_semantic_tokens_to_utf16(&mut utf16, text);
    assert_eq!(utf16[0].delta_start, 1);
    assert_eq!(utf16[0].length, 4);
}

#[test]
fn reindent_preserves_leading_bom_and_content() {
    let text = "\u{FEFF}host: value\n";
    assert_eq!(crate::reindent::reindent(text), text);
}

#[test]
fn formatting_edit_covers_bom_and_indented_document_in_both_encodings() {
    let text = "\u{FEFF}    host: value  ";
    let expected = "\u{FEFF}host: value";
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    for (encoding, flag) in [(PositionEncoding::Utf8, 0), (PositionEncoding::Utf16, 1)] {
        let (service, _socket) = tower_lsp::LspService::new(Backend::new);
        let backend = service.inner();
        backend
            .encoding
            .store(flag, std::sync::atomic::Ordering::Relaxed);
        let uri = Url::parse("file:///bom-format.ktav").unwrap();
        rt.block_on(backend.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "ktav".to_string(),
                version: 1,
                text: text.to_string(),
            },
        }));

        let edits = rt
            .block_on(backend.formatting(DocumentFormattingParams {
                text_document: TextDocumentIdentifier { uri },
                work_done_progress_params: WorkDoneProgressParams::default(),
                options: FormattingOptions {
                    tab_size: 4,
                    insert_spaces: true,
                    ..Default::default()
                },
            }))
            .expect("formatting must succeed")
            .expect("opened document must have a formatting edit");
        let edit = edits.first().expect("expected one whole-document edit");
        assert_eq!(edit.range.start, Position::new(0, 0));
        assert_eq!(edit.range.end.line, 0);

        let end_byte = match encoding {
            PositionEncoding::Utf8 => edit.range.end.character as usize,
            PositionEncoding::Utf16 => text
                .char_indices()
                .scan(0u32, |units, (byte, ch)| {
                    let here = *units;
                    *units += ch.len_utf16() as u32;
                    Some((here, byte))
                })
                .find(|(units, _)| *units == edit.range.end.character)
                .map(|(_, byte)| byte)
                .unwrap_or(text.len()),
        };
        let applied = format!(
            "{}{}{}",
            &text[..0],
            edit.new_text.as_str(),
            &text[end_byte..]
        );
        assert_eq!(
            end_byte,
            text.len(),
            "range must reach document end ({encoding:?})"
        );
        assert_eq!(
            applied, expected,
            "edit must not duplicate trailing source text"
        );
    }
}

fn complete_at_cursor(
    rt: &tokio::runtime::Runtime,
    marked: &str,
    encoding: PositionEncoding,
) -> (String, Position, Option<CompletionResponse>) {
    let cursor = marked.find('|').expect("cursor marker");
    let mut text = marked.to_string();
    text.remove(cursor);
    let lines = crate::lines::split_lines(&text);
    let (line, byte_column) = crate::lines::byte_to_line_col(&text, cursor);
    let byte_column = byte_column as usize;
    let character = match encoding {
        PositionEncoding::Utf8 => byte_column as u32,
        PositionEncoding::Utf16 => byte_to_utf16(lines[line as usize], byte_column),
    };
    let position = Position::new(line, character);
    let (service, _socket) = tower_lsp::LspService::new(Backend::new);
    let backend = service.inner();
    backend.encoding.store(encoding.as_u8(), Ordering::Relaxed);
    let uri = Url::parse("file:///completion.ktav").unwrap();
    backend
        .docs
        .insert(uri.clone(), DocEntry::new(1, text.clone()));
    let response = rt
        .block_on(backend.completion(CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        }))
        .expect("completion request");
    (text, position, response)
}

fn completion_offset(text: &str, position: Position, encoding: PositionEncoding) -> usize {
    let lines = crate::lines::split_lines(text);
    let line = lines[position.line as usize];
    let prefix = prefix_by_encoding(line, position.character, encoding);
    line.as_ptr() as usize - text.as_ptr() as usize + prefix.len()
}

#[test]
fn completion_edits_preserve_separator_and_parsed_value() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    for encoding in [PositionEncoding::Utf8, PositionEncoding::Utf16] {
        for (marked, label, expected) in [
            ("anchor: 1\nkey:|", "null", "anchor: 1\nkey: null"),
            ("key:|", "true", "key: true"),
            ("\u{FEFF}key:|", ":", "\u{FEFF}key:: "),
            ("anchor: 1\nkey::|", "null", "anchor: 1\nkey:: null"),
            ("anchor: 1\nkey: |", "false", "anchor: 1\nkey: false"),
            ("anchor: 1\nkey:\t|", "{}", "anchor: 1\nkey:\t{}"),
            (
                "anchor: 1\nkey:\u{2003}|",
                "[]",
                "anchor: 1\nkey:\u{2003}[]",
            ),
            ("anchor: 1\nkey:|  ", "null", "anchor: 1\nkey:  null"),
            ("{\n\"😀:имя\":|\n}", "()", "{\n\"😀:имя\": ()\n}"),
            ("anchor: 1\nkey:|", ":", "anchor: 1\nkey:: "),
            ("anchor: 1\nkey: \t|", ":", "anchor: 1\nkey:: \t"),
            ("anchor: 1\nkey:|  ", ":", "anchor: 1\nkey::  "),
        ] {
            for eol in ["\n", "\r", "\r\n"] {
                let marked = marked.replace('\n', eol);
                let expected = expected.replace('\n', eol);
                let (text, position, response) = complete_at_cursor(&rt, &marked, encoding);
                let Some(CompletionResponse::Array(items)) = response else {
                    panic!("missing completions: {marked:?}");
                };
                let item = items
                    .iter()
                    .find(|item| item.label == label)
                    .expect("value item");
                let mut applied = text.clone();
                match &item.text_edit {
                    Some(CompletionTextEdit::Edit(edit)) => {
                        let start = completion_offset(&text, edit.range.start, encoding);
                        let end = completion_offset(&text, edit.range.end, encoding);
                        applied.replace_range(start..end, &edit.new_text);
                    }
                    None => {
                        let offset = completion_offset(&text, position, encoding);
                        applied.insert_str(offset, item.insert_text.as_ref().expect("insert text"));
                    }
                    Some(CompletionTextEdit::InsertAndReplace(_)) => panic!("unexpected edit"),
                }
                assert_eq!(applied, expected, "{marked:?}, {label}, {encoding:?}");
                let Value::Object(object) =
                    ktav::parse(&applied).expect("accepted completion must parse")
                else {
                    panic!("completion changed the root kind");
                };
                let value = match label {
                    "null" if marked.contains("::") => Value::String("null".into()),
                    "null" => Value::Null,
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    "{}" => Value::Object(Default::default()),
                    "[]" => Value::Array(Vec::new()),
                    "()" | ":" => Value::String(Default::default()),
                    _ => panic!("missing value oracle"),
                };
                let key = if marked.contains("😀:имя") {
                    "😀:имя"
                } else {
                    "key"
                };
                assert_eq!(object.get(key), Some(&value));
                if marked.contains("anchor:") {
                    assert_eq!(object.get("anchor"), Some(&Value::Integer("1".into())));
                    assert_eq!(object.len(), 2);
                } else {
                    assert_eq!(object.len(), 1);
                }
            }
        }
    }
}

#[test]
fn completion_ignores_non_pair_context_and_existing_values() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    for marked in [
        "hello\nkey:|",
        "[\nkey:|\n]",
        "text: ((\nkey:|\n))",
        "## key:|",
        "anchor: 1\nkey:| value",
        "anchor: 1\nkey: |value",
        "anchor: 1\nkey: va|lue",
        "anchor: 1\nkey:|:",
        "anchor: 1\n::|",
    ] {
        let (_, _, response) = complete_at_cursor(&rt, marked, PositionEncoding::Utf16);
        assert!(response.is_none(), "offered pair values at {marked:?}");
    }
    let (_, _, response) = complete_at_cursor(&rt, "anchor: 1\nkey::|", PositionEncoding::Utf16);
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("raw value completions");
    };
    assert!(
        items.iter().all(|item| item.label != ":"),
        "must not create a third colon"
    );
}
