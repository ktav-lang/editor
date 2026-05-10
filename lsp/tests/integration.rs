//! End-to-end tests: feed text through the public modules and check
//! that diagnostics, semantic tokens, symbols and the shared tokenizer
//! behave correctly. All tests use the real `ktav::parse` — no mocks.

use ktav_lsp::diagnostics::parse_for_diagnostics;
use ktav_lsp::semantic::{semantic_tokens, token_types};
use ktav_lsp::server::{DocEntry, PositionEncoding};
use ktav_lsp::symbols::build_symbols;
use ktav_lsp::tokens::{
    byte_to_utf16, classify_line, cursor_is_after_separator, prefix_by_encoding, LineKind, Marker,
    ValueKind,
};

// ---- Diagnostics ----

#[test]
fn valid_document_yields_no_diagnostics() {
    let text = "name: example\nport: 8080\n";
    assert!(parse_for_diagnostics(text).is_empty());
}

#[test]
fn missing_separator_space_tight_range_covers_marker_and_glued_body() {
    // First line forces Object root (per spec 0.1.1 a bare
    // `key:value` at the document start is a top-level Array string
    // item, not a malformed pair). The malformed pair on line 2 then
    // yields MissingSeparatorSpace.
    let text = "anchor: 1\nkey:value\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    let r = d[0].range;
    assert_eq!(r.start.line, 1);
    // Structured span: just the glued body `value` (bytes 4..9 on
    // line 2). The legacy classifier-derived range used to start at
    // the colon (col 3); the structured span is even tighter.
    assert_eq!(r.start.character, 4);
    assert_eq!(r.end.character, 9);
    assert!(d[0].message.contains("MissingSeparatorSpace"));
}

#[test]
fn missing_separator_space_for_typed_marker() {
    // First line forces Object root (per spec 0.1.1).
    let text = "anchor: 1\nport:i8080\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    // `:i` is NOT classified as TypedInt because no whitespace follows
    // — it's a Plain marker covering just `:`. The diagnostic still
    // tightens to the marker + glued body.
    let r = d[0].range;
    assert_eq!(r.start.line, 1);
    assert!(r.end.character > r.start.character);
}

#[test]
fn duplicate_key_tight_to_key() {
    let text = "port: 80\nport: 443\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].range.start.line, 1);
    assert_eq!(d[0].range.start.character, 0);
    assert_eq!(d[0].range.end.character, 4);
}

#[test]
fn empty_input_is_valid() {
    assert!(parse_for_diagnostics("").is_empty());
}

#[test]
fn invalid_typed_scalar_value_span_tightened() {
    // `:i NaN` — typed-int body is invalid.
    let text = "port:i abc\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    let r = d[0].range;
    assert_eq!(r.start.line, 0);
    // Structured span covers the body region (incl. leading space):
    // `port:i abc` — body starts at byte 6, ends at 10.
    assert_eq!(r.start.character, 6);
    assert_eq!(r.end.character, 10);
    assert!(d[0].message.contains("InvalidTypedScalar"));
}

// ---- Semantic tokens ----

#[test]
fn token_types_index_layout_stable() {
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
fn semantic_tokens_emit_for_simple_doc() {
    let text = "# comment\nname: alice\nflag: true\ncount:i 42\n";
    let toks = semantic_tokens(text);
    assert!(!toks.is_empty());
    assert_eq!(toks[0].delta_line, 0);
    assert_eq!(toks[0].token_type, 0); // COMMENT
}

#[test]
fn semantic_tokens_dotted_key_emits_one_property_per_segment() {
    let text = "a.b.c: 10\n";
    let toks = semantic_tokens(text);
    let property_count = toks.iter().filter(|t| t.token_type == 4).count();
    assert_eq!(
        property_count, 3,
        "expected 3 PROPERTY tokens, got {:?}",
        toks
    );
}

#[test]
fn semantic_tokens_typed_value_is_number() {
    let text = "n:i 42\n";
    let toks = semantic_tokens(text);
    // Last token should be NUMBER (index 2).
    assert_eq!(toks.last().unwrap().token_type, 2);
}

#[test]
fn semantic_tokens_raw_value_is_string() {
    let text = "g:: foo bar\n";
    let toks = semantic_tokens(text);
    assert_eq!(toks.last().unwrap().token_type, 3);
}

#[test]
fn semantic_tokens_compound_open_is_operator() {
    let text = "obj: {\n  x: 1\n}\n";
    let toks = semantic_tokens(text);
    // `obj` PROPERTY, `:` OPERATOR, `{` OPERATOR — line 0 has 3 tokens
    // ending with operator.
    let line0: Vec<_> = toks
        .iter()
        .scan(0u32, |line, t| {
            *line += t.delta_line;
            Some((*line, t.token_type))
        })
        .filter(|(l, _)| *l == 0)
        .collect();
    assert_eq!(line0.last().unwrap().1, 5);
}

#[test]
fn semantic_tokens_for_keywords() {
    for (text, _) in [
        ("flag: true\n", 1u32),
        ("flag: false\n", 1),
        ("flag: null\n", 1),
    ] {
        let toks = semantic_tokens(text);
        assert_eq!(toks.last().unwrap().token_type, 1, "for {}", text);
    }
}

// ---- Symbols ----

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

#[test]
fn document_symbols_dotted_key_creates_nested_outline() {
    let text = "db.host: localhost\ndb.port:i 5432\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    // Top-level should expose `db`.
    assert!(syms.iter().any(|s| s.name == "db"));
    let db = syms.iter().find(|s| s.name == "db").unwrap();
    let kids = db.children.as_ref().expect("db has children");
    assert!(kids.iter().any(|k| k.name == "host"));
    assert!(kids.iter().any(|k| k.name == "port"));
}

// ---- Tokens (shared classifier) ----

#[test]
fn tokens_pair_basic() {
    match classify_line("port:i 8080") {
        LineKind::Pair {
            marker, value_kind, ..
        } => {
            assert_eq!(marker, Marker::TypedInt);
            assert_eq!(value_kind, ValueKind::Number);
        }
        other => panic!("expected pair, got {:?}", other),
    }
}

#[test]
fn tokens_cursor_after_sep() {
    assert!(cursor_is_after_separator("name: "));
    assert!(cursor_is_after_separator("name:"));
    assert!(cursor_is_after_separator("name:: "));
    assert!(!cursor_is_after_separator("nam"));
}

// ---- Non-ASCII / UTF-16 conversion ----

#[test]
fn byte_to_utf16_ascii_is_identity() {
    let line = "name: alice";
    assert_eq!(byte_to_utf16(line, 0), 0);
    assert_eq!(byte_to_utf16(line, 5), 5);
    assert_eq!(byte_to_utf16(line, line.len()), 11);
}

#[test]
fn byte_to_utf16_cyrillic_two_bytes_one_unit() {
    // `имя` = 3 chars × 2 bytes = 6 bytes, 3 UTF-16 code units.
    let line = "имя: x";
    // Byte length of "имя" is 6, UTF-16 length is 3.
    assert_eq!(byte_to_utf16(line, 6), 3);
    // ':' column: byte 6 = utf16 3.
    assert_eq!(byte_to_utf16(line, 7), 4);
}

#[test]
fn byte_to_utf16_emoji_is_surrogate_pair() {
    // 😀 is U+1F600 → 4 UTF-8 bytes, 2 UTF-16 code units (surrogate pair).
    let line = "k: 😀";
    let after_emoji = line.len();
    // "k: " is 3 ASCII bytes / 3 UTF-16 units, then 4 bytes / 2 units for emoji.
    assert_eq!(byte_to_utf16(line, after_emoji), 5);
}

#[test]
fn semantic_tokens_with_cyrillic_key_emit_byte_columns() {
    // The internal `semantic_tokens` produces BYTE-indexed columns;
    // the server post-processes them when UTF-16 is negotiated. Here
    // we assert the byte-indexed shape so future changes to the
    // converter don't accidentally double-encode.
    let text = "имя: значение\n";
    let toks = semantic_tokens(text);
    // First token = property "имя" at column 0, byte length 6.
    assert_eq!(toks[0].delta_line, 0);
    assert_eq!(toks[0].delta_start, 0);
    assert_eq!(toks[0].length, 6);
    assert_eq!(toks[0].token_type, 4); // PROPERTY
}

#[test]
fn diagnostics_byte_columns_for_cyrillic_pre_conversion() {
    // `имя:значение` → MissingSeparatorSpace. `имя` = 6 bytes, `:` at
    // byte 6, glued body `значение` starts at byte 7. Structured span
    // covers the body alone, so start column = 7 in BYTES.
    // First line forces Object root (per spec 0.1.1 a bare
    // `имя:значение` at the document start would parse as a
    // top-level Array string item).
    let text = "anchor: 1\nимя:значение\n";
    let d = parse_for_diagnostics(text);
    assert_eq!(d.len(), 1);
    let r = d[0].range;
    assert_eq!(r.start.line, 1);
    assert_eq!(r.start.character, 7);
}

// ---- Symbols depth/boundary fixes ----

#[test]
fn symbols_locate_key_does_not_match_prefix() {
    // `db` and `database` share a prefix; the strip-prefix bug would
    // have located `database`'s line for the `db` key. With the
    // boundary check, both find their own lines.
    let text = "database: x\ndb: y\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    let db = syms.iter().find(|s| s.name == "db").unwrap();
    let database = syms.iter().find(|s| s.name == "database").unwrap();
    assert_eq!(db.range.start.line, 1, "db should be on line 1");
    assert_eq!(database.range.start.line, 0, "database on line 0");
}

#[test]
fn symbols_locate_key_skips_nested_same_name() {
    // Top-level `host` shares a name with `server.host`. With depth
    // tracking, the top-level `host` matches its own line and
    // `server.host` resolves via children of `server`.
    let text = "server: {\n    host: nested\n}\nhost: top\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    let top_host = syms.iter().find(|s| s.name == "host").unwrap();
    // The top-level `host` lives on line 3 (0-indexed).
    assert_eq!(top_host.range.start.line, 3);
}

// ---- Top-level Array (spec § 5.0.1) ----

#[test]
fn document_symbols_built_from_top_level_array_of_scalars() {
    // Bare scalar items at the document root. Per spec 0.1.1 these
    // form a top-level Array; the outline should expose `[0]`, `[1]`,
    // `[2]` with each item's range pointing to its own line.
    let text = "first\nsecond\nthird\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    assert_eq!(syms.len(), 3, "three top-level array items");
    assert_eq!(syms[0].name, "[0]");
    assert_eq!(syms[1].name, "[1]");
    assert_eq!(syms[2].name, "[2]");
    assert_eq!(syms[0].range.start.line, 0);
    assert_eq!(syms[1].range.start.line, 1);
    assert_eq!(syms[2].range.start.line, 2);
}

#[test]
fn document_symbols_top_level_array_of_objects_have_children() {
    // Each `{ ... }` block is one item of the top-level Array.
    let text = "{\n    name: alice\n}\n{\n    name: bob\n}\n";
    let value = ktav::parse(text).expect("parse");
    let syms = build_symbols(&value, text);
    assert_eq!(syms.len(), 2);
    assert_eq!(syms[0].name, "[0]");
    let kids = syms[0].children.as_ref().expect("[0] has children");
    assert!(kids.iter().any(|k| k.name == "name"));
}

// ---- Encoding-aware completion prefix + hover via classifier ----

#[test]
fn completion_prefix_after_cyrillic_key_utf16() {
    // Cursor at end of "ключ: " — UTF-16 units = 4 (key) + 1 (':') + 1 (' ') = 6.
    let line = "ключ: ";
    let upto = prefix_by_encoding(line, 6, PositionEncoding::Utf16);
    assert_eq!(upto, "ключ: ");
    assert!(cursor_is_after_separator(upto));
}

#[test]
fn completion_prefix_after_cyrillic_key_utf8() {
    // Same line in UTF-8: each cyrillic char is 2 bytes. "ключ" = 8 bytes,
    // ":" = 1, " " = 1, total 10.
    let line = "ключ: ";
    let upto = prefix_by_encoding(line, 10, PositionEncoding::Utf8);
    assert_eq!(upto, "ключ: ");
    assert!(cursor_is_after_separator(upto));
}

#[test]
fn completion_mid_key_returns_no_value_completions() {
    // Cursor inside the key (before ':') — completion handler should bail.
    let line = "ключ: ";
    // After 2 cyrillic chars, no colon yet.
    let upto = prefix_by_encoding(line, 2, PositionEncoding::Utf16);
    assert_eq!(upto, "кл");
    assert!(!cursor_is_after_separator(upto));
}

#[test]
fn hover_classifier_dotted_non_ascii_key_yields_pair() {
    // Hover handler routes through classify_line → uses key from Pair.
    let line = "сервер.host: example.com";
    match classify_line(line) {
        LineKind::Pair {
            key_start,
            key_length,
            value_kind,
            ..
        } => {
            let s = key_start as usize;
            let e = s + key_length as usize;
            assert_eq!(&line[s..e], "сервер.host");
            assert_eq!(value_kind, ValueKind::String);
        }
        other => panic!("expected Pair, got {:?}", other),
    }
    // Sanity: real parser exposes the dotted key as nested object.
    let text = format!("{}\n", line);
    let value = ktav::parse(&text).expect("parse");
    if let ktav::Value::Object(m) = &value {
        assert!(m.contains_key("сервер"));
    } else {
        panic!("expected object root");
    }
}

#[test]
fn hover_classifier_skips_raw_array_item() {
    // `:: literal` line — empty key, hover must be None.
    assert!(matches!(
        classify_line("    :: literal"),
        LineKind::RawArrayItem { .. }
    ));
}

#[test]
fn hover_classifier_skips_comment_line() {
    assert!(matches!(
        classify_line("# just a note"),
        LineKind::Comment { .. }
    ));
}

// ---- DocEntry parsed-value cache ----

#[test]
fn doc_entry_caches_parsed_value() {
    // Two reads of the same `DocEntry.parsed` must point to the SAME
    // `Arc<Value>` allocation — i.e. the parse happens once at insert
    // and subsequent handlers (hover, document_symbol) share it.
    let entry = DocEntry::new(1, "name: alice\nport: 8080\n".to_string());
    let p1 = entry.parsed.clone().expect("doc parsed ok");
    let p2 = entry.parsed.clone().expect("doc parsed ok");
    assert!(
        std::sync::Arc::ptr_eq(&p1, &p2),
        "DocEntry::parsed should be a cached Arc — both clones must alias",
    );
    // And: a parse of the same text yields the same observable shape
    // (sanity — we're not corrupting the cache).
    let fresh = ktav::parse("name: alice\nport: 8080\n").expect("re-parse");
    match (&*p1, &fresh) {
        (ktav::Value::Object(a), ktav::Value::Object(b)) => assert_eq!(a.len(), b.len()),
        _ => panic!("expected object root"),
    }
}

#[test]
fn doc_entry_invalidates_on_text_change() {
    // Simulate did_change replacing the text: a new DocEntry must
    // reflect the new content (no stale cache).
    let v1 = DocEntry::new(1, "k: 1\n".to_string());
    let v2 = DocEntry::new(2, "k: 2\n".to_string());
    let p1 = v1.parsed.expect("v1 parsed");
    let p2 = v2.parsed.expect("v2 parsed");
    // Different Arcs.
    assert!(!std::sync::Arc::ptr_eq(&p1, &p2));
    // And the values differ in surface form.
    let s1 = match &*p1 {
        ktav::Value::Object(m) => m.get("k").and_then(|v| match v {
            ktav::Value::Integer(s) => Some(s.as_str().to_string()),
            ktav::Value::String(s) => Some(s.to_string()),
            _ => None,
        }),
        _ => None,
    };
    let s2 = match &*p2 {
        ktav::Value::Object(m) => m.get("k").and_then(|v| match v {
            ktav::Value::Integer(s) => Some(s.as_str().to_string()),
            ktav::Value::String(s) => Some(s.to_string()),
            _ => None,
        }),
        _ => None,
    };
    assert_ne!(s1, s2, "expected different parsed values across versions");
}

#[test]
fn doc_entry_unparseable_text_yields_none_cache() {
    // Bad input → cache is `None`; handlers must degrade gracefully.
    let entry = DocEntry::new(1, "key: {\n".to_string()); // unclosed brace
    assert!(
        entry.parsed.is_none(),
        "unparseable text should leave parsed=None",
    );
}

#[test]
fn tokens_compound_close() {
    assert!(matches!(
        classify_line("}"),
        LineKind::CloseBrace { start: 0 }
    ));
}
