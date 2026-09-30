//! Scope-sensitive formatting and semantic-token regression oracles.

use ktav_lsp::{reindent::reindent, semantic::semantic_tokens};

const COMMENT: u32 = 0;
const KEYWORD: u32 = 1;
const NUMBER: u32 = 2;
const STRING: u32 = 3;
const PROPERTY: u32 = 4;
const OPERATOR: u32 = 5;

fn tokens(text: &str) -> Vec<(u32, u32, u32, u32)> {
    let mut line = 0;
    let mut column = 0;
    semantic_tokens(text)
        .into_iter()
        .map(|token| {
            line += token.delta_line;
            column = if token.delta_line == 0 {
                column + token.delta_start
            } else {
                token.delta_start
            };
            (line, column, token.length, token.token_type)
        })
        .collect()
}

fn assert_roundtrip(source: &str, expected: &str) {
    let before = ktav::parse(source).expect("regression source must parse");
    let formatted = reindent(source);
    assert_eq!(formatted, expected, "source: {source:?}");
    assert_eq!(
        ktav::parse(&formatted).expect("formatted source must parse"),
        before,
        "formatting changed data: {source:?}"
    );
    assert_eq!(reindent(&formatted), formatted);
}

#[test]
fn formatting_preserves_colon_bearing_array_strings() {
    for (source, expected) in [
        ("hello\nname: (value)\n", "hello\nname: (value)\n"),
        (
            "[\nname: (value)\nname: ((value))\n]\n",
            "[\n    name: (value)\n    name: ((value))\n]\n",
        ),
        (
            "items: [\nname: (\nname: ((\nname: {\nname: [\ntrue\n]\ntail: (value)\n",
            "items: [\n    name: (\n    name: ((\n    name: {\n    name: [\n    true\n]\ntail:: (value)\n",
        ),
        (
            "hello\n[\n{\nname: (value)\n}\nname: (value)\n]\nname: (value)\n",
            "hello\n[\n    {\n        name:: (value)\n    }\n    name: (value)\n]\nname: (value)\n",
        ),
    ] {
        assert_roundtrip(source, expected);
    }
    // Negative control: the historical rewrite changes the second String.
    assert_ne!(
        ktav::parse("hello\nname: (value)\n").unwrap(),
        ktav::parse("hello\nname:: (value)\n").unwrap()
    );
}

#[test]
fn formatting_dispatches_the_first_content_line_only_once() {
    for (source, expected) in [
        (
            "\u{FEFF}\u{2003}## note\r\rhello\rname:\u{00A0}(value)\r",
            "\u{FEFF}## note\n\nhello\nname:\u{00A0}(value)\n",
        ),
        ("name:\u{2003}(value)\r", "name::\u{2003}(value)\n"),
        (
            "name:(value)\nname: (value)\n",
            "name:(value)\nname: (value)\n",
        ),
        (
            "http://host\nname: (value)\n",
            "http://host\nname: (value)\n",
        ),
        (
            "\"open: true\nname: (value)\n",
            "\"open: true\nname: (value)\n",
        ),
        ("a\\: true\nname: (value)\n", "a\\: true\nname: (value)\n"),
        (": true\nname: (value)\n", ": true\nname: (value)\n"),
        (":: head\nname: (value)\n", ":: head\nname: (value)\n"),
        ("\"a:b\": (value)\n", "\"a:b\":: (value)\n"),
        (
            "\u{2003}\u{FEFF}hello\nname: (value)\n",
            "\u{2003}\u{FEFF}hello\nname: (value)\n",
        ),
    ] {
        assert_roundtrip(source, expected);
    }
}

#[test]
fn formatting_keeps_multiline_and_raw_bodies_in_their_scope() {
    let source = "items: [\n:: ((\n(\n  name: (value)\n  ## content\n)\n((\n name: true\n [\n)\n))\n{\nraw:: (\nname: (value)\n}\n]\n";
    let expected = "items: [\n    :: ((\n    (\n  name: (value)\n  ## content\n    )\n    ((\n name: true\n [\n)\n    ))\n    {\n        raw:: (\n        name:: (value)\n    }\n]\n";
    assert_roundtrip(source, expected);
}

#[test]
fn bare_and_wrapped_arrays_have_exact_string_tokens() {
    assert_eq!(
        tokens("hello\nname: true\nname: 42\nname: (\nname:: ((\n"),
        vec![
            (0, 0, 5, STRING),
            (1, 0, 10, STRING),
            (2, 0, 8, STRING),
            (3, 0, 7, STRING),
            (4, 0, 9, STRING),
        ]
    );
    assert_eq!(
        tokens("[\nname: true\n]\n"),
        vec![(0, 0, 1, OPERATOR), (1, 0, 10, STRING), (2, 0, 1, OPERATOR)]
    );
}

#[test]
fn bare_array_restores_its_scope_after_nested_containers() {
    let source = "hello\n{\nname: true\nchild: {\nn: 42\n}\n}\n[\nname: true\n]\nname: true\n";
    assert!(ktav::parse(source).is_ok());
    assert_eq!(
        tokens(source),
        vec![
            (0, 0, 5, STRING),
            (1, 0, 1, OPERATOR),
            (2, 0, 4, PROPERTY),
            (2, 4, 1, OPERATOR),
            (2, 6, 4, KEYWORD),
            (3, 0, 5, PROPERTY),
            (3, 5, 1, OPERATOR),
            (3, 7, 1, OPERATOR),
            (4, 0, 1, PROPERTY),
            (4, 1, 1, OPERATOR),
            (4, 3, 2, NUMBER),
            (5, 0, 1, OPERATOR),
            (6, 0, 1, OPERATOR),
            (7, 0, 1, OPERATOR),
            (8, 0, 10, STRING),
            (9, 0, 1, OPERATOR),
            (10, 0, 10, STRING),
        ]
    );
}

#[test]
fn paren_shortcuts_and_lone_closers_have_exact_array_tokens() {
    let source = "[\n()\n(())\n)\n))\n]\n";
    assert!(ktav::parse(source).is_ok());
    assert_eq!(
        tokens(source),
        vec![
            (0, 0, 1, OPERATOR),
            (1, 0, 2, OPERATOR),
            (2, 0, 4, OPERATOR),
            (3, 0, 1, STRING),
            (4, 0, 2, STRING),
            (5, 0, 1, OPERATOR),
        ]
    );
}

#[test]
fn bare_wrapped_and_nested_objects_have_exact_property_tokens() {
    assert_eq!(
        tokens("name: true\n"),
        vec![(0, 0, 4, PROPERTY), (0, 4, 1, OPERATOR), (0, 6, 4, KEYWORD)]
    );
    assert_eq!(
        tokens("{\nname: true\n}\n"),
        vec![
            (0, 0, 1, OPERATOR),
            (1, 0, 4, PROPERTY),
            (1, 4, 1, OPERATOR),
            (1, 6, 4, KEYWORD),
            (2, 0, 1, OPERATOR),
        ]
    );
    assert_eq!(
        tokens(
            "items: [\nname: true\n{\nname: true\n}\n[\nname: true\n]\nname: true\n]\ntail: 42\n"
        ),
        vec![
            (0, 0, 5, PROPERTY),
            (0, 5, 1, OPERATOR),
            (0, 7, 1, OPERATOR),
            (1, 0, 10, STRING),
            (2, 0, 1, OPERATOR),
            (3, 0, 4, PROPERTY),
            (3, 4, 1, OPERATOR),
            (3, 6, 4, KEYWORD),
            (4, 0, 1, OPERATOR),
            (5, 0, 1, OPERATOR),
            (6, 0, 10, STRING),
            (7, 0, 1, OPERATOR),
            (8, 0, 10, STRING),
            (9, 0, 1, OPERATOR),
            (10, 0, 4, PROPERTY),
            (10, 4, 1, OPERATOR),
            (10, 6, 2, NUMBER),
        ]
    );
}

#[test]
fn first_content_dispatch_respects_sep_end_quotes_escapes_and_raw_marker() {
    for source in [
        "name:true",
        "http://host",
        "\"name: true",
        "a\\: true",
        ": true",
    ] {
        assert!(ktav::parse(source).is_ok(), "{source:?}");
        assert_eq!(tokens(source), vec![(0, 0, source.len() as u32, STRING)]);
    }
    assert_eq!(
        tokens("## note\n\n:: head\nname: true\n"),
        vec![
            (0, 0, 7, COMMENT),
            (2, 0, 2, OPERATOR),
            (2, 3, 4, STRING),
            (3, 0, 10, STRING),
        ]
    );
    assert_eq!(
        tokens("\"a:b\": true\n"),
        vec![(0, 1, 3, PROPERTY), (0, 5, 1, OPERATOR), (0, 7, 4, KEYWORD)]
    );
}

#[test]
fn raw_and_multiline_tokens_do_not_redispatch_content() {
    let source = "[\n:: ((\n(\nname: true\n## content\n)\n((\n[\n)\n))\n{\nr:: (\nn: true\n}\nname: (\ntrue\n]\n";
    assert!(ktav::parse(source).is_ok());
    assert_eq!(
        tokens(source),
        vec![
            (0, 0, 1, OPERATOR),
            (1, 0, 2, OPERATOR),
            (1, 3, 2, STRING),
            (2, 0, 1, OPERATOR),
            (3, 0, 10, STRING),
            (4, 0, 10, STRING),
            (5, 0, 1, OPERATOR),
            (6, 0, 2, OPERATOR),
            (7, 0, 1, STRING),
            (8, 0, 1, STRING),
            (9, 0, 2, OPERATOR),
            (10, 0, 1, OPERATOR),
            (11, 0, 1, PROPERTY),
            (11, 1, 2, OPERATOR),
            (11, 4, 1, STRING),
            (12, 0, 1, PROPERTY),
            (12, 1, 1, OPERATOR),
            (12, 3, 4, KEYWORD),
            (13, 0, 1, OPERATOR),
            (14, 0, 7, STRING),
            (15, 0, 4, KEYWORD),
            (16, 0, 1, OPERATOR),
        ]
    );
}

#[test]
fn unicode_whitespace_cr_and_bom_keep_exact_byte_ranges() {
    assert_eq!(
        tokens("\u{FEFF}\u{2003}## note\r\r\u{00A0}[\r\u{3000}name: true\u{2003}\r]\r"),
        vec![
            (0, 6, 7, COMMENT),
            (2, 2, 1, OPERATOR),
            (3, 3, 10, STRING),
            (4, 0, 1, OPERATOR),
        ]
    );
    assert_eq!(
        tokens("\u{FEFF}\u{2003}name:\u{00A0}true\r"),
        vec![
            (0, 6, 4, PROPERTY),
            (0, 10, 1, OPERATOR),
            (0, 13, 4, KEYWORD)
        ]
    );
}

#[test]
fn every_non_terminating_spec_whitespace_supports_root_dispatch() {
    for ws in [
        '\u{0009}', '\u{000B}', '\u{000C}', '\u{0020}', '\u{0085}', '\u{00A0}', '\u{1680}',
        '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
        '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}',
        '\u{205F}', '\u{3000}',
    ] {
        let source = format!("{ws}name:{ws}true{ws}\r");
        let width = ws.len_utf8() as u32;
        assert!(ktav::parse(&source).is_ok(), "{ws:?}");
        assert_eq!(
            tokens(&source),
            vec![
                (0, width, 4, PROPERTY),
                (0, width + 4, 1, OPERATOR),
                (0, width * 2 + 5, 4, KEYWORD)
            ],
            "{ws:?}"
        );
        assert_roundtrip(
            &format!("{ws}name:{ws}(value){ws}\r"),
            &format!("name::{ws}(value)\n"),
        );
        assert_roundtrip(
            &format!("{ws}hello\rname:{ws}(value)\r"),
            &format!("hello\nname:{ws}(value)\n"),
        );
    }
}

#[test]
fn multiline_content_guard_uses_array_scope() {
    let source = "hello\nname: (\ntrue\n(\nname: true\n)\nname: ((\ntrue\n";
    assert!(ktav::parse(source).is_ok());
    for line in 0..8 {
        assert_eq!(
            ktav_lsp::tokens::line_is_multiline_content(source, line),
            matches!(line, 4 | 5),
            "line {line}"
        );
    }
}

#[test]
fn pinned_parseable_corpus_formatting_preserves_values_and_is_idempotent() {
    use std::{fs, path::Path};

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/versions/0.8/tests");
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("manifest.json")).expect("initialise pinned spec"),
    )
    .unwrap();
    assert_eq!(manifest["schema_version"], 1);
    let mut failures = Vec::new();
    for (category, multiplier) in [
        ("valid", 2),
        ("parseable-unrepresentable", 1),
        ("strict-lossy", 1),
    ] {
        let expected =
            manifest["categories"][category]["count"].as_u64().unwrap() as usize * multiplier;
        let mut directories = vec![root.join(category)];
        let mut count = 0;
        while let Some(directory) = directories.pop() {
            for entry in fs::read_dir(directory).expect("pinned category") {
                let path = entry.expect("fixture entry").path();
                if path.is_dir() {
                    directories.push(path);
                } else if path
                    .extension()
                    .is_some_and(|extension| extension == "ktav")
                {
                    count += 1;
                    let source = fs::read_to_string(&path).expect("UTF-8 parseable fixture");
                    let original = ktav::parse(&source)
                        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                    let formatted = reindent(&source);
                    match ktav::parse(&formatted) {
                        Ok(value) if value == original => {}
                        result => {
                            failures.push(format!("{}: {result:?} != {original:?}", path.display()))
                        }
                    }
                    if reindent(&formatted) != formatted {
                        failures.push(format!("{}: not idempotent", path.display()));
                    }
                }
            }
        }
        assert_eq!(count, expected, "pinned {category} source count");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
