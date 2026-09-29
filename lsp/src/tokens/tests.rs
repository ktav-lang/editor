use super::*;

fn pair(s: &str) -> LineKind<'_> {
    classify_line(s)
}

#[test]
fn comment() {
    // Spec 0.5.0: `##` required. Single `#` is not a comment.
    match pair("  ## hello") {
        LineKind::Comment { start, length } => {
            assert_eq!(start, 2);
            assert_eq!(length, 8);
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn single_hash_is_not_comment() {
    // Spec 0.5.0: a lone `#` is an ordinary character.
    assert!(!matches!(pair("  # hello"), LineKind::Comment { .. }));
}

#[test]
fn plain_pair() {
    match pair("name: alice") {
        LineKind::Pair {
            key_start,
            key_length,
            marker,
            value_kind,
            value_text,
            ..
        } => {
            assert_eq!(key_start, 0);
            assert_eq!(key_length, 4);
            assert_eq!(marker, Marker::Plain);
            assert_eq!(value_kind, ValueKind::String);
            assert_eq!(value_text, "alice");
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn typed_int_removed_spec050() {
    // Spec 0.5.0: `:i` is no longer a typed marker — it is treated as a
    // Plain marker whose value starts with `i`.
    match pair("port:i 8080") {
        LineKind::Pair { marker, .. } => {
            assert_eq!(marker, Marker::Plain);
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn raw_marker() {
    match pair("greeting:: hello world") {
        LineKind::Pair {
            marker, value_kind, ..
        } => {
            assert_eq!(marker, Marker::Raw);
            assert_eq!(value_kind, ValueKind::String);
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn typed_letter_glued_is_plain() {
    // `port:istanbul` — `i` followed by non-ws → Plain marker, value
    // = `istanbul`. Mirrors `ktav` classify_separator.
    match pair("city:istanbul") {
        LineKind::Pair { marker, .. } => assert_eq!(marker, Marker::Plain),
        other => panic!("got {:?}", other),
    }
}

#[test]
fn array_raw_item() {
    match pair("    :: literal") {
        LineKind::RawArrayItem {
            marker_start,
            value_start,
            value_length,
        } => {
            assert_eq!(marker_start, 4);
            assert_eq!(value_start, 7);
            assert_eq!(value_length, 7);
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn close_brace() {
    assert!(matches!(pair("  }"), LineKind::CloseBrace { start: 2 }));
}

#[test]
fn compound_open() {
    match pair("server: {") {
        LineKind::Pair { value_kind, .. } => assert_eq!(value_kind, ValueKind::CompoundOpen),
        other => panic!("got {:?}", other),
    }
}

#[test]
fn ipv4_and_versions_are_not_numeric() {
    // More than one `.` ⇒ not a number; classified as String.
    assert!(!looks_numeric("127.0.0.1"));
    assert!(!looks_numeric("1.2.3"));
    assert!(!looks_numeric("1.2.3.4"));
    // Well-formed numbers still recognised.
    assert!(looks_numeric("42"));
    assert!(looks_numeric("1.3"));
    assert!(looks_numeric("1.5e3"));
    assert!(looks_numeric("0xFF_00"));
    assert_eq!(classify_value("127.0.0.1"), ValueKind::String);
    assert_eq!(classify_value("1.3"), ValueKind::Number);
}

#[test]
fn line_starting_with_bracket_is_array_item_not_pair() {
    // `{name: alice, age: 30}` is an inline-object array item, not a
    // `key: value` pair keyed on `{name`.
    match pair("{name: alice, age: 30}") {
        LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::String),
        other => panic!("got {:?}", other),
    }
    match pair("[1, 2, 3]") {
        LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::String),
        other => panic!("got {:?}", other),
    }
    // Lone opener stays CompoundOpen.
    match pair("{") {
        LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::CompoundOpen),
        other => panic!("got {:?}", other),
    }
}

#[test]
fn bare_multiline_opener_array_item_is_compound_open() {
    // § 4's `<item-value> ::= <value-start> <line-end>` dispatches a bare
    // array item through the same production as a pair's value — a lone
    // `(` / `((` opens a multi-line string body, it is not a one-char
    // String item.
    match pair("(") {
        LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::CompoundOpen),
        other => panic!("got {:?}", other),
    }
    match pair("((") {
        LineKind::ArrayItem { kind, .. } => assert_eq!(kind, ValueKind::CompoundOpen),
        other => panic!("got {:?}", other),
    }
}

#[test]
fn line_is_multiline_content_stripped_block() {
    // motd: (
    //     port: 8080     <- content, must not be re-classified as a pair
    //     ## note        <- content, must not be re-classified as a comment
    // )                  <- terminator: still "inside" entering this line
    // done: 1            <- back to ordinary lines
    let text = "motd: (\n    port: 8080\n    ## note\n)\ndone: 1\n";
    assert!(!line_is_multiline_content(text, 0)); // opener line itself
    assert!(line_is_multiline_content(text, 1));
    assert!(line_is_multiline_content(text, 2));
    assert!(line_is_multiline_content(text, 3)); // terminator line
    assert!(!line_is_multiline_content(text, 4));
}

#[test]
fn line_is_multiline_content_verbatim_block_needs_double_paren() {
    let text = "body: ((\n}\n))\n";
    assert!(!line_is_multiline_content(text, 0));
    // A lone `}` inside a verbatim block is content, not a closer — and
    // a lone `)` would not terminate it either (needs `))`).
    assert!(line_is_multiline_content(text, 1));
    assert!(line_is_multiline_content(text, 2)); // `))` terminator
}

#[test]
fn line_is_multiline_content_bare_array_item_opener() {
    let text = "items: [\n(\nhello\n)\n]\n";
    assert!(!line_is_multiline_content(text, 1)); // the `(` opener itself
    assert!(line_is_multiline_content(text, 2));
    assert!(line_is_multiline_content(text, 3)); // the `)` terminator
    assert!(!line_is_multiline_content(text, 4));
}

#[test]
fn dotted_split() {
    let segs: Vec<_> = split_dotted(2, "a.bb.ccc").collect();
    assert_eq!(segs, vec![(2, "a"), (4, "bb"), (7, "ccc")]);
}

#[test]
fn dotted_split_quoted_segment_dot_is_opaque() {
    // § 5.3.3's own example: `a."b.c".d` is THREE segments — the dot
    // inside the quoted segment does not split it.
    let segs: Vec<_> = split_dotted(0, r#"a."b.c".d"#).collect();
    assert_eq!(segs, vec![(0, "a"), (2, r#""b.c""#), (8, "d")]);
}

#[test]
fn dotted_split_mid_token_quote_is_ordinary() {
    let segs: Vec<_> = split_dotted(0, "don't").collect();
    assert_eq!(segs, vec![(0, "don't")]);
}

#[test]
fn key_escape_literal_dot_is_one_segment() {
    // Spec 0.6.0: `a\.b` is ONE segment (the `\.` is a literal dot).
    let segs: Vec<_> = split_dotted(0, r"a\.b").collect();
    assert_eq!(segs, vec![(0, r"a\.b")]);
}

#[test]
fn key_escape_mixed_path_and_literal() {
    // `x.y\.z` → `x`, `y\.z` (split on first dot, second dot is escaped).
    let segs: Vec<_> = split_dotted(0, r"x.y\.z").collect();
    assert_eq!(segs, vec![(0, "x"), (2, r"y\.z")]);
}

#[test]
fn key_escape_double_backslash_does_not_protect_dot() {
    // `a\\.b` → `\\` is an escape sequence (literal `\`), then the
    // following `.` is UNESCAPED and splits the path. Two segments:
    // `a\\` and `b`.
    let segs: Vec<_> = split_dotted(0, r"a\\.b").collect();
    assert_eq!(segs, vec![(0, r"a\\"), (4, "b")]);
}

#[test]
fn pair_with_escaped_dot_in_key() {
    // `a\.b: v` — key is `a\.b` (one slice), marker Plain, value `v`.
    match pair(r"a\.b: v") {
        LineKind::Pair {
            key_start,
            key_length,
            marker,
            value_text,
            ..
        } => {
            assert_eq!(key_start, 0);
            assert_eq!(key_length, 4); // a \ . b
            assert_eq!(marker, Marker::Plain);
            assert_eq!(value_text, "v");
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn pair_with_escaped_colon_in_key() {
    // `a\:b: v` — first `:` is escaped; key is `a\:b`, marker is the
    // SECOND `:`.
    match pair(r"a\:b: v") {
        LineKind::Pair {
            key_length,
            marker_start,
            marker,
            value_text,
            ..
        } => {
            assert_eq!(key_length, 4); // a \ : b
            assert_eq!(marker_start, 4);
            assert_eq!(marker, Marker::Plain);
            assert_eq!(value_text, "v");
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn pair_with_colon_inside_quoted_key() {
    // `"a:b": v` — the colon inside the quoted key is NOT the
    // separator (§ 5.3.3); the real separator is right after the
    // closing quote.
    match pair(r#""a:b": v"#) {
        LineKind::Pair {
            key_start,
            key_length,
            marker_start,
            marker,
            value_text,
            ..
        } => {
            assert_eq!(key_start, 0);
            assert_eq!(key_length, 5); // "a:b"
            assert_eq!(marker_start, 5);
            assert_eq!(marker, Marker::Plain);
            assert_eq!(value_text, "v");
        }
        other => panic!("got {:?}", other),
    }
}

#[test]
fn find_key_separator_basic() {
    assert_eq!(find_key_separator("a:b"), Some(1));
    assert_eq!(find_key_separator(r"a\:b:c"), Some(4));
    assert_eq!(find_key_separator(r"a\\:b"), Some(3));
    assert_eq!(find_key_separator(r"a\:b"), None);
}

#[test]
fn find_key_separator_colon_inside_quoted_segment_is_opaque() {
    // The `:` inside the quoted key is ordinary content; the real
    // separator is the one right after the closing quote.
    assert_eq!(find_key_separator(r#""a:b": 1"#), Some(5));
    assert_eq!(find_key_separator("'a:b': 1"), Some(5));
    assert_eq!(find_key_separator("`a:b`: 1"), Some(5));
}

#[test]
fn find_key_separator_mid_token_quote_is_ordinary() {
    // A quote NOT at a segment's first position never opens a
    // quoted segment (§ 5.3.3's positional rule) — unaffected.
    assert_eq!(find_key_separator("don't: 1"), Some(5));
}

#[test]
fn find_key_separator_unterminated_quote_finds_nothing() {
    // No matching closer before EOL — swallows the rest of the line,
    // indistinguishable from "no `:` on this line at all" (§ 5.3.3).
    assert_eq!(find_key_separator(r#""a: 1"#), None);
}

#[test]
fn prefix_utf8_ascii() {
    use crate::server::PositionEncoding::Utf8;
    assert_eq!(prefix_by_encoding("name: alice", 6, Utf8), "name: ");
}

#[test]
fn prefix_utf8_clamps_to_len() {
    use crate::server::PositionEncoding::Utf8;
    let line = "abc";
    assert_eq!(prefix_by_encoding(line, 999, Utf8), "abc");
}

#[test]
fn prefix_utf8_rounds_down_off_boundary() {
    use crate::server::PositionEncoding::Utf8;
    // "и" is 2 bytes (0xD0 0xB8). character=1 lands mid-codepoint.
    let line = "имя";
    let p = prefix_by_encoding(line, 1, Utf8);
    assert_eq!(p, ""); // rounded down to 0
    let p2 = prefix_by_encoding(line, 3, Utf8);
    assert_eq!(p2, "и"); // rounded down to 2
}

#[test]
fn prefix_utf16_cyrillic() {
    use crate::server::PositionEncoding::Utf16;
    // "ключ: " — 5 cyrillic chars (1 utf16 unit each) + ": " = 7 utf16 units.
    let line = "ключ: x";
    let p = prefix_by_encoding(line, 6, Utf16);
    assert_eq!(p, "ключ: ");
}

#[test]
fn prefix_utf16_emoji_surrogate_pair() {
    use crate::server::PositionEncoding::Utf16;
    // "k: 😀" — k(1) :(1) (1) =3 units, emoji = 2 units (surrogate pair).
    let line = "k: 😀";
    // Stop before the emoji.
    assert_eq!(prefix_by_encoding(line, 3, Utf16), "k: ");
    // Mid-surrogate (4) — should not split codepoint; returns up to emoji's start.
    assert_eq!(prefix_by_encoding(line, 4, Utf16), "k: ");
    // After emoji.
    assert_eq!(prefix_by_encoding(line, 5, Utf16), "k: 😀");
}

#[test]
fn byte_to_utf16_empty() {
    assert_eq!(byte_to_utf16("", 0), 0);
    assert_eq!(byte_to_utf16("", 5), 0);
}

#[test]
fn byte_to_utf16_overshoot_clamps() {
    // Past end → clamps to UTF-16 length of full string.
    assert_eq!(byte_to_utf16("a", 5), 1);
    assert_eq!(byte_to_utf16("abc", 999), 3);
}

#[test]
fn byte_to_utf16_mid_codepoint_invalid_byte_index() {
    // "é" is 2 bytes (0xC3 0xA9) and 1 UTF-16 unit. Asking for byte
    // index 1 lands mid-codepoint. The helper slices `&line[..cap]`;
    // a non-boundary cap would panic, so we verify current behaviour:
    // byte_idx==1 is NOT a char boundary in "é", so the helper would
    // panic. Pin the safe boundaries instead.
    assert_eq!(byte_to_utf16("é", 0), 0);
    assert_eq!(byte_to_utf16("é", 2), 1); // full "é"
}

#[test]
fn byte_to_utf16_4byte_utf8_surrogate_pair() {
    // "𝕏" (U+1D54F) — 4 bytes UTF-8, 2 UTF-16 units (surrogate pair).
    assert_eq!(byte_to_utf16("𝕏", 0), 0);
    assert_eq!(byte_to_utf16("𝕏", 4), 2);
    // Past end clamps to 2.
    assert_eq!(byte_to_utf16("𝕏", 99), 2);
}

#[test]
fn prefix_utf16_mid_surrogate_rounds_down() {
    use crate::server::PositionEncoding::Utf16;
    // "𝕏" is a single codepoint occupying 2 UTF-16 units. Target=1
    // lands mid-surrogate-pair: helper rounds DOWN (returns "").
    assert_eq!(prefix_by_encoding("𝕏", 1, Utf16), "");
    // Target=0 → "".
    assert_eq!(prefix_by_encoding("𝕏", 0, Utf16), "");
    // Target=2 → full "𝕏".
    assert_eq!(prefix_by_encoding("𝕏", 2, Utf16), "𝕏");
}

#[test]
fn prefix_utf8_mid_codepoint_rounds_down() {
    use crate::server::PositionEncoding::Utf8;
    // "café" — 'é' is 2 bytes at offset 3..5. byte_idx=4 lands
    // mid-codepoint; helper rounds DOWN to byte 3 → "caf".
    assert_eq!(prefix_by_encoding("café", 3, Utf8), "caf");
    assert_eq!(prefix_by_encoding("café", 4, Utf8), "caf");
    assert_eq!(prefix_by_encoding("café", 5, Utf8), "café");
}

#[test]
fn cursor_after_sep() {
    assert!(cursor_is_after_separator("name: "));
    assert!(cursor_is_after_separator("name:: "));
    assert!(!cursor_is_after_separator("name"));
    assert!(!cursor_is_after_separator("nam"));
}

// -- § 3.6 / § 5.2 exact numeric classification -----------------------

const NUMERIC_INTEGERS: &[&str] = &[
    "0", "7", "-7", "+7", "1_000", "0x1A", "0o755", "0b1010", "-0x1f", "0x1_A", "-0",
];

const NUMERIC_FLOATS: &[&str] = &[
    "0.5", "-0.5", "1e3", "1E3", "1e+3", "1_0.5_0", "6.022e23", "0e0",
];

const NOT_NUMERIC_STRINGS: &[&str] = &[
    "01234",
    "00",
    "0_7",
    "-045",
    "+007",
    "01.5",
    "05e3",
    "1_",
    "_1",
    "1__0",
    "0x",
    "0x_1",
    "0X1A",
    "0b102",
    "0o9",
    "1.",
    ".5",
    "1e",
    "1e+",
    "1.5e",
    "1_.5",
    "1._5",
    "1.2.3",
    "2026-09-28",
    "127.0.0.1",
    "-",
    "+",
    "True",
    "NULL",
    "False",
];

#[test]
fn integer_literals_are_numeric() {
    for s in NUMERIC_INTEGERS {
        assert!(looks_numeric(s), "expected numeric: {s:?}");
        assert_eq!(
            classify_value(s),
            ValueKind::Number,
            "expected Number: {s:?}"
        );
    }
}

#[test]
fn float_literals_are_numeric() {
    for s in NUMERIC_FLOATS {
        assert!(looks_numeric(s), "expected numeric: {s:?}");
        assert_eq!(
            classify_value(s),
            ValueKind::Number,
            "expected Number: {s:?}"
        );
    }
}

#[test]
fn non_numeric_spellings_are_strings() {
    for s in NOT_NUMERIC_STRINGS {
        assert!(!looks_numeric(s), "expected NOT numeric: {s:?}");
        assert_eq!(
            classify_value(s),
            ValueKind::String,
            "expected String: {s:?}"
        );
    }
}

#[test]
fn keywords_are_case_sensitive() {
    assert_eq!(classify_value("null"), ValueKind::Null);
    assert_eq!(classify_value("true"), ValueKind::Bool);
    assert_eq!(classify_value("false"), ValueKind::Bool);
    assert_eq!(classify_value("True"), ValueKind::String);
    assert_eq!(classify_value("NULL"), ValueKind::String);
    assert_eq!(classify_value("False"), ValueKind::String);
}

#[test]
fn redundant_leading_zero_exceptions() {
    // § 5.2 rule 13's own carve-outs: NOT redundant.
    assert_eq!(classify_value("0"), ValueKind::Number);
    assert_eq!(classify_value("0.5"), ValueKind::Number);
    assert_eq!(classify_value("0e0"), ValueKind::Number);
    assert_eq!(classify_value("0x1A"), ValueKind::Number);
    assert_eq!(classify_value("0o755"), ValueKind::Number);
    assert_eq!(classify_value("0b1010"), ValueKind::Number);
}

// -- § 3.3 exact whitespace set (not just space/tab/CR) ----------------

#[test]
fn trailing_ws_trims_full_spec_set() {
    // U+00A0 NBSP, U+2003 EM SPACE, U+3000 IDEOGRAPHIC SPACE.
    for ws in ['\u{00A0}', '\u{2003}', '\u{3000}', '\u{2028}', '\u{2029}'] {
        let line = format!("name: alice{ws}");
        match pair(&line) {
            LineKind::Pair {
                value_text,
                value_length,
                ..
            } => {
                assert_eq!(value_text, "alice", "ws={:?}", ws as u32);
                assert_eq!(value_length, 5, "ws={:?}", ws as u32);
            }
            other => panic!("got {:?}", other),
        }
    }
}

#[test]
fn leading_ws_full_spec_set() {
    // U+00A0 NBSP as indentation.
    match pair("\u{00A0}name: alice") {
        LineKind::Pair { key_start, .. } => {
            assert_eq!(key_start, "\u{00A0}".len() as u32);
        }
        other => panic!("got {:?}", other),
    }
}
