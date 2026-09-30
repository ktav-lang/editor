//! LSP semantic-tokens producer, driven by the shared
//! [`crate::tokens`] line classifier (the single source of truth that
//! mirrors `ktav::parser`'s separator rules).
//!
//! Token type indices MUST match the order returned by [`token_types`].

use tower_lsp::lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType};

use crate::tokens::{
    classify_value, is_ktav_ws, split_dotted, DocumentContext, LineKind, Marker, ValueKind,
};

/// Token types we expose, in the order their indices are referenced
/// from the deltas.
pub fn token_types() -> Vec<SemanticTokenType> {
    vec![
        SemanticTokenType::COMMENT,     // 0
        SemanticTokenType::KEYWORD,     // 1  — booleans (true / false)
        SemanticTokenType::NUMBER,      // 2
        SemanticTokenType::STRING,      // 3
        SemanticTokenType::PROPERTY,    // 4  — keys
        SemanticTokenType::OPERATOR,    // 5
        SemanticTokenType::new("null"), // 6  — null literal (distinct hue)
    ]
}

/// Token modifiers, in bit order. `quoted` marks a key segment written
/// in quoted form (§ 5.3.3) so clients can colour it apart from bare keys.
pub fn token_modifiers() -> Vec<SemanticTokenModifier> {
    vec![SemanticTokenModifier::new("quoted")]
}

const MOD_QUOTED: u32 = 1 << 0;

/// Push the token for one key segment. A bare segment is one PROPERTY; a
/// quoted segment (§ 5.3.3) emits only its content, with the `quoted`
/// modifier, so the paired delimiters keep the grammar's punctuation colour.
fn push_key_segment(out: &mut Vec<AbsToken>, line: u32, col: u32, seg: &str) {
    let lead = seg.len() - seg.trim_start_matches(is_ktav_ws).len();
    let body = seg[lead..].trim_end_matches(is_ktav_ws);
    if let Some(q @ ('"' | '\'' | '`')) = body.chars().next() {
        if body.len() >= 2 && body.ends_with(q) {
            if body.len() > 2 {
                out.push(AbsToken {
                    line,
                    start: col + lead as u32 + 1,
                    length: (body.len() - 2) as u32,
                    token_type: TOK_PROPERTY,
                    modifiers: MOD_QUOTED,
                });
            }
            return;
        }
    }
    if !seg.is_empty() {
        out.push(AbsToken {
            line,
            start: col,
            length: seg.len() as u32,
            token_type: TOK_PROPERTY,
            modifiers: 0,
        });
    }
}

const TOK_COMMENT: u32 = 0;
const TOK_KEYWORD: u32 = 1;
const TOK_NUMBER: u32 = 2;
const TOK_STRING: u32 = 3;
const TOK_PROPERTY: u32 = 4;
const TOK_OPERATOR: u32 = 5;
const TOK_NULL: u32 = 6;

#[derive(Clone, Copy)]
struct AbsToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

/// Produce semantic tokens for `text`. Output is encoded in the
/// LSP-mandated delta format.
pub fn semantic_tokens(text: &str) -> Vec<SemanticToken> {
    let mut abs: Vec<AbsToken> = Vec::new();
    let mut context = DocumentContext::default();

    // § 3.2: LF, CR and CRLF are all valid line terminators — go through
    // the shared splitter so semantic-token line numbers never desync
    // from diagnostics / symbols / hover on a non-LF document.
    let bom = crate::lines::leading_bom_len(text) as u32;
    for (line_idx, line) in crate::lines::content_lines(text).into_iter().enumerate() {
        let line_idx = line_idx as u32;
        let first_token = abs.len();
        let classified = context.next_line(line);
        if classified.multiline_content {
            if let LineKind::ArrayItem { start, length, .. } = classified.kind {
                abs.push(AbsToken {
                    line: line_idx,
                    start,
                    length,
                    token_type: TOK_STRING,
                    modifiers: 0,
                });
            }
        } else {
            emit_line(line_idx, line, classified.kind, &mut abs);
        }
        if line_idx == 0 {
            for token in &mut abs[first_token..] {
                token.start += bom;
            }
        }
    }

    encode_deltas(&abs)
}

fn emit_line(line: u32, raw: &str, kind: LineKind<'_>, out: &mut Vec<AbsToken>) {
    match kind {
        LineKind::Blank => {}
        LineKind::Comment { start, length } => {
            out.push(AbsToken {
                line,
                start,
                length,
                token_type: TOK_COMMENT,
                modifiers: 0,
            });
        }
        LineKind::CloseBrace { start } => {
            out.push(AbsToken {
                line,
                start,
                length: 1,
                token_type: TOK_OPERATOR,
                modifiers: 0,
            });
        }
        LineKind::RawArrayItem {
            marker_start,
            value_start,
            value_length,
        } => {
            out.push(AbsToken {
                line,
                start: marker_start,
                length: 2,
                token_type: TOK_OPERATOR,
                modifiers: 0,
            });
            if value_length > 0 {
                out.push(AbsToken {
                    line,
                    start: value_start,
                    length: value_length,
                    token_type: TOK_STRING,
                    modifiers: 0,
                });
            }
        }
        LineKind::Pair {
            key_start,
            key_length,
            marker_start,
            marker,
            value_start,
            value_length,
            value_kind,
            value_text,
            ..
        } => {
            // Emit dotted-key segments as PROPERTY tokens (one per segment).
            // The dot itself we leave as a gap — clients render the gap
            // with default colour, matching the tree-sitter highlights.scm
            // recipe (`(key) @property`, `"." @punctuation.delimiter`).
            let key_lo = key_start as usize;
            let key_hi = key_lo + key_length as usize;
            let raw_key = &raw[key_lo..key_hi];
            for (col, seg) in split_dotted(key_start, raw_key) {
                push_key_segment(out, line, col, seg);
            }
            out.push(AbsToken {
                line,
                start: marker_start,
                length: marker.len() as u32,
                token_type: TOK_OPERATOR,
                modifiers: 0,
            });
            if value_length == 0 {
                return;
            }
            // An inline compound value (`{…}` / `[…]` on a `:` line) is
            // tokenized structurally so its brackets read as operators
            // (and thus bracket-match) rather than disappearing into one
            // opaque string. `::` (Raw) bodies stay literal per spec.
            if marker == Marker::Plain
                && value_kind == ValueKind::String
                && starts_inline_compound(value_text)
            {
                emit_inline(line, value_start, value_text, out);
                return;
            }
            let tt = match value_kind {
                ValueKind::Bool => TOK_KEYWORD,
                ValueKind::Null => TOK_NULL,
                ValueKind::Number => TOK_NUMBER,
                ValueKind::String => TOK_STRING,
                ValueKind::CompoundOpen => TOK_OPERATOR,
                ValueKind::CompoundClose => TOK_OPERATOR,
            };
            out.push(AbsToken {
                line,
                start: value_start,
                length: value_length,
                token_type: tt,
                modifiers: 0,
            });
        }
        LineKind::ArrayItem {
            start,
            length,
            kind,
        } => {
            let item_text = &raw[start as usize..start as usize + length as usize];
            if matches!(kind, ValueKind::String) && starts_inline_compound(item_text) {
                emit_inline(line, start, item_text, out);
                return;
            }
            let tt = match kind {
                ValueKind::Bool => TOK_KEYWORD,
                ValueKind::Null => TOK_NULL,
                ValueKind::Number => TOK_NUMBER,
                ValueKind::CompoundOpen | ValueKind::CompoundClose => TOK_OPERATOR,
                _ => TOK_STRING,
            };
            out.push(AbsToken {
                line,
                start,
                length,
                token_type: tt,
                modifiers: 0,
            });
        }
    }
}

/// True if `text` (already trimmed) opens an inline compound — begins with
/// `{` or `[`. Callers gate this on `ValueKind::String` so the lone multiline
/// openers (`{`, `[`, `(`, `((`) and empty forms (`{}`, `[]`) — classified
/// [`ValueKind::CompoundOpen`] — never reach the inline tokenizer.
fn starts_inline_compound(text: &str) -> bool {
    matches!(text.as_bytes().first(), Some(b'{') | Some(b'['))
}

/// One structural event produced while walking an inline compound's text
/// (`{...}` / `[...]`) — shared by [`emit_inline`] (semantic tokens) and
/// `analysis::symbols`'s inline-key collector, so the quote/escape-aware
/// scanning rules (§ 5.3.3 quoted segments, § 3.7 escapes, § 5.8.5 raw
/// values) never diverge between the two. All positions are byte offsets
/// into the `text` passed to [`walk_inline`].
pub(crate) enum InlineEvent<'a> {
    /// `{` or `[`.
    Open { pos: usize },
    /// `}` or `]`.
    Close { pos: usize },
    /// `,`.
    Comma { pos: usize },
    /// `:` or `::` between a key and its value.
    Sep { pos: usize, raw: bool },
    /// An object key run, already trimmed of trailing whitespace.
    Key { start: usize, text: &'a str },
    /// A scalar value run, already trimmed of surrounding whitespace.
    /// `forced_string` is set for a raw (`::`) value or one containing a
    /// recognised escape — § 5.2's typing never applies to either.
    Value {
        start: usize,
        text: &'a str,
        forced_string: bool,
    },
}

/// Walk a single-line inline compound (`{k: v, …}` / `[v1, v2]`, nesting
/// allowed), emitting one [`InlineEvent`] per structural token in source
/// order. The grammar's head/rest rule is preserved: `{` / `[` open a
/// nested compound only at a value position; once a scalar run has begun
/// they are ordinary literal content (`hello{world`).
pub(crate) fn walk_inline<'a>(text: &'a str, mut emit: impl FnMut(InlineEvent<'a>)) {
    let b = text.as_bytes();
    let mut i = 0usize;
    // Container stack: true = object, false = array.
    let mut stack: Vec<bool> = Vec::new();
    // Inside an object, the next scalar-shaped run is a key until the `:`.
    let mut expect_key = false;
    // Set by a `::` separator; cleared once the following value is consumed.
    let mut raw_value = false;

    // § 3.3: whitespace inside an inline compound is the full 25-code-point
    // set, not just ASCII space/tab. ASCII structural bytes (`{}[]:,` and
    // `\`) are never a UTF-8 continuation byte, so scanning those by raw
    // byte value is safe even inside a multi-byte scalar run; only the
    // "is this a separator" check needs to decode a char.
    let ws_len_at = |i: usize| -> usize {
        text[i..]
            .chars()
            .next()
            .filter(|&c| is_ktav_ws(c))
            .map_or(0, char::len_utf8)
    };

    while i < b.len() {
        let ws = ws_len_at(i);
        if ws > 0 {
            i += ws;
            continue;
        }
        match b[i] {
            // § 5.8.5: the `::` branch is a raw scalar, not an inline
            // value — a leading `{` / `[` there is literal content, never
            // a nested-compound opener. Falls through to the `_` arm.
            c @ (b'{' | b'[') if !raw_value => {
                emit(InlineEvent::Open { pos: i });
                let is_obj = c == b'{';
                stack.push(is_obj);
                expect_key = is_obj;
                raw_value = false;
                i += 1;
            }
            b'}' | b']' => {
                emit(InlineEvent::Close { pos: i });
                stack.pop();
                expect_key = false;
                raw_value = false;
                i += 1;
            }
            b',' => {
                emit(InlineEvent::Comma { pos: i });
                expect_key = matches!(stack.last(), Some(true));
                raw_value = false;
                i += 1;
            }
            b':' if expect_key => {
                // Pair separator inside an object (`:` or `::`).
                let len = if i + 1 < b.len() && b[i + 1] == b':' {
                    2
                } else {
                    1
                };
                emit(InlineEvent::Sep {
                    pos: i,
                    raw: len == 2,
                });
                expect_key = false;
                raw_value = len == 2;
                i += len;
            }
            _ => {
                let start = i;
                if expect_key {
                    let mut at_segment_start = true;
                    // A quote opens after edge whitespace in each segment.
                    while i < b.len() {
                        if at_segment_start {
                            let ws = ws_len_at(i);
                            if ws > 0 {
                                i += ws;
                                continue;
                            }
                        }
                        match b[i] {
                            q @ (b'"' | b'\'' | b'`') if at_segment_start => {
                                i += 1;
                                while i < b.len() {
                                    if b[i] == b'\\' && i + 1 < b.len() {
                                        i += 2;
                                        continue;
                                    }
                                    if b[i] == q {
                                        i += 1;
                                        break;
                                    }
                                    i += 1;
                                }
                                at_segment_start = false;
                            }
                            // § 3.7: `\X` escapes a structural byte —
                            // `\[`, `\]`, `\{`, `\}`, `\,`, `\:` stay
                            // ordinary key content, not a delimiter.
                            b'\\' if i + 1 < b.len() => {
                                i += 2;
                                at_segment_start = false;
                            }
                            b'.' => {
                                i += 1;
                                at_segment_start = true;
                            }
                            b':' | b',' | b'{' | b'}' | b'[' | b']' => break,
                            _ => {
                                i += 1;
                                at_segment_start = false;
                            }
                        }
                    }
                    let run = &text[start..i];
                    let run = run.trim_end_matches(is_ktav_ws);
                    emit(InlineEvent::Key { start, text: run });
                    // expect_key stays set; the `:` arm clears it.
                } else {
                    // Scalar value: run until an unescaped `,` / `}` / `]`.
                    // `{` / `[` mid-run are literal content (head/rest rule).
                    let mut escaped = false;
                    while i < b.len() {
                        match b[i] {
                            b'\\' => {
                                escaped = true;
                                i = (i + 2).min(b.len());
                            }
                            b',' | b'}' | b']' => break,
                            _ => i += 1,
                        }
                    }
                    let run = &text[start..i];
                    let body = run.trim_matches(is_ktav_ws);
                    let lead = run.len() - run.trim_start_matches(is_ktav_ws).len();
                    emit(InlineEvent::Value {
                        start: start + lead,
                        text: body,
                        forced_string: raw_value || escaped,
                    });
                    raw_value = false;
                }
            }
        }
    }
}

/// Tokenize an inline compound into structural + scalar sub-tokens via
/// [`walk_inline`]. `base` is the absolute (byte) column of `text[0]`.
///
/// Brackets, commas and `:` / `::` separators become OPERATOR tokens so the
/// editor treats them as real brackets (enabling bracket matching) instead of
/// folding the whole value into one opaque string. Object keys become
/// PROPERTY; scalar values are classified (STRING / NUMBER / KEYWORD).
fn emit_inline(line: u32, base: u32, text: &str, out: &mut Vec<AbsToken>) {
    let push = |out: &mut Vec<AbsToken>, start: usize, len: usize, tt: u32, mods: u32| {
        if len > 0 {
            out.push(AbsToken {
                line,
                start: base + start as u32,
                length: len as u32,
                token_type: tt,
                modifiers: mods,
            });
        }
    };
    walk_inline(text, |ev| match ev {
        InlineEvent::Open { pos } => push(out, pos, 1, TOK_OPERATOR, 0),
        InlineEvent::Close { pos } => push(out, pos, 1, TOK_OPERATOR, 0),
        InlineEvent::Comma { pos } => push(out, pos, 1, TOK_OPERATOR, 0),
        InlineEvent::Sep { pos, raw } => push(out, pos, if raw { 2 } else { 1 }, TOK_OPERATOR, 0),
        InlineEvent::Key { start, text } => {
            for (col, seg) in split_dotted(base + start as u32, text) {
                push_key_segment(out, line, col, seg);
            }
        }
        InlineEvent::Value {
            start,
            text,
            forced_string,
        } => {
            let tt = if forced_string {
                TOK_STRING
            } else {
                match classify_value(text) {
                    ValueKind::Bool => TOK_KEYWORD,
                    ValueKind::Null => TOK_NULL,
                    ValueKind::Number => TOK_NUMBER,
                    _ => TOK_STRING,
                }
            };
            push(out, start, text.len(), tt, 0);
        }
    });
}

fn encode_deltas(toks: &[AbsToken]) -> Vec<SemanticToken> {
    let mut out = Vec::with_capacity(toks.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for t in toks {
        let delta_line = t.line - prev_line;
        let delta_start = if delta_line == 0 {
            t.start - prev_start
        } else {
            t.start
        };
        out.push(SemanticToken {
            delta_line,
            delta_start,
            length: t.length,
            token_type: t.token_type,
            token_modifiers_bitset: t.modifiers,
        });
        prev_line = t.line;
        prev_start = t.start;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode the delta stream into absolute `(line, start, length,
    /// token_type)` quadruples.
    fn toks(text: &str) -> Vec<(u32, u32, u32, u32)> {
        let mut line = 0u32;
        let mut col = 0u32;
        semantic_tokens(text)
            .into_iter()
            .map(|t| {
                line += t.delta_line;
                col = if t.delta_line == 0 {
                    col + t.delta_start
                } else {
                    t.delta_start
                };
                (line, col, t.length, t.token_type)
            })
            .collect()
    }

    #[test]
    fn raw_marker_inline_value_is_untyped_string() {
        // `r::` is raw — never dispatched through § 5.2 typing, even
        // though "42" looks numeric. `s:` is plain — typed normally.
        // `z:` is plain but leading-zero — String per rule 13.
        let text = "line: {r:: 42, s: 42, z: 042}";
        let t = toks(text);
        assert!(
            t.contains(&(0, 11, 2, TOK_STRING)),
            "r's raw value should be String: {t:?}"
        );
        assert!(
            t.contains(&(0, 18, 2, TOK_NUMBER)),
            "s's plain value should be Number: {t:?}"
        );
        assert!(
            t.contains(&(0, 25, 3, TOK_STRING)),
            "z's leading-zero value should be String: {t:?}"
        );
    }

    #[test]
    fn quoted_inline_keys_are_opaque_to_structural_bytes() {
        // The `,` inside `"a,b"` and the `:` inside `'x:y'` must not
        // split the key (§ 5.3.3) — each quoted segment's content is one
        // PROPERTY token; the paired quotes are left to the grammar.
        let text = "line: {\"a,b\": 1, 'x:y': 2}";
        let t = toks(text);
        assert!(
            t.contains(&(0, 8, 3, TOK_PROPERTY)),
            "\"a,b\" content should be one PROPERTY token: {t:?}"
        );
        assert!(
            t.contains(&(0, 18, 3, TOK_PROPERTY)),
            "'x:y' content should be one PROPERTY token: {t:?}"
        );
        // Exactly 3 PROPERTY tokens on the line: the outer `line` key plus
        // the two quoted inline keys — no spurious split (e.g. just `"a`).
        let property_count = t
            .iter()
            .filter(|&&(l, _, _, tt)| l == 0 && tt == TOK_PROPERTY)
            .count();
        assert_eq!(property_count, 3, "unexpected PROPERTY token split: {t:?}");
    }

    #[test]
    fn quoted_key_content_carries_the_quoted_modifier() {
        let mods = |text: &str| -> Vec<(u32, u32, u32)> {
            let mut col = 0u32;
            semantic_tokens(text)
                .into_iter()
                .map(|t| {
                    col = if t.delta_line == 0 {
                        col + t.delta_start
                    } else {
                        t.delta_start
                    };
                    (col, t)
                })
                .filter(|(_, t)| t.token_type == TOK_PROPERTY)
                .map(|(col, t)| (col, t.length, t.token_modifiers_bitset))
                .collect()
        };
        // Bare segment: whole segment, no modifier. Quoted: content only.
        assert_eq!(
            mods("a.\"b.c\".`d`: 1"),
            vec![(0, 1, 0), (3, 3, MOD_QUOTED), (9, 1, MOD_QUOTED)]
        );
        assert_eq!(
            mods("'k': {\"x\": 1, y: 2}"),
            vec![(1, 1, MOD_QUOTED), (7, 1, MOD_QUOTED), (14, 1, 0)]
        );
        assert_eq!(token_modifiers().len(), 1);
    }

    #[test]
    fn spaced_quoted_dotted_keys_keep_highlight_ranges() {
        let t = toks("root . \"a.b\": 1\nroot . \"a:b\": 2");
        assert_eq!(
            t,
            vec![
                (0, 0, 5, TOK_PROPERTY),
                (0, 8, 3, TOK_PROPERTY),
                (0, 12, 1, TOK_OPERATOR),
                (0, 14, 1, TOK_NUMBER),
                (1, 0, 5, TOK_PROPERTY),
                (1, 8, 3, TOK_PROPERTY),
                (1, 12, 1, TOK_OPERATOR),
                (1, 14, 1, TOK_NUMBER),
            ]
        );
    }

    #[test]
    fn quoted_inline_dotted_segment_keeps_comma_opaque() {
        let t = toks("cfg: {a.\"b,c\": 1, tail: 2}");
        assert_eq!(
            t,
            vec![
                (0, 0, 3, TOK_PROPERTY),
                (0, 3, 1, TOK_OPERATOR),
                (0, 5, 1, TOK_OPERATOR),
                (0, 6, 1, TOK_PROPERTY),
                (0, 9, 3, TOK_PROPERTY),
                (0, 13, 1, TOK_OPERATOR),
                (0, 15, 1, TOK_NUMBER),
                (0, 16, 1, TOK_OPERATOR),
                (0, 18, 4, TOK_PROPERTY),
                (0, 22, 1, TOK_OPERATOR),
                (0, 24, 1, TOK_NUMBER),
                (0, 25, 1, TOK_OPERATOR),
            ]
        );

        let t = toks("cfg: {a. \"b,c\": 1, tail: 2}");
        assert!(t.contains(&(0, 6, 1, TOK_PROPERTY)), "bare segment: {t:?}");
        assert!(
            t.contains(&(0, 10, 3, TOK_PROPERTY)),
            "quoted content: {t:?}"
        );
        assert!(
            t.contains(&(0, 14, 1, TOK_OPERATOR)),
            "pair separator: {t:?}"
        );
        assert!(t.contains(&(0, 19, 4, TOK_PROPERTY)), "sibling: {t:?}");
    }

    #[test]
    fn trailing_nbsp_after_keyword_is_trimmed() {
        // § 3.3: NBSP (U+00A0) is whitespace, not just space/tab.
        // "line: {a: true<NBSP>}" — value run starts right after "a: ".
        let text = "line: {a: true\u{00A0}}";
        let t = toks(text);
        assert!(
            t.contains(&(0, 10, 4, TOK_KEYWORD)),
            "trailing NBSP must not be part of the `true` token: {t:?}"
        );
    }

    #[test]
    fn escaped_value_is_string() {
        // § 5.2: an inline scalar body containing a recognised escape is
        // always String, regardless of what the raw digits look like.
        let text = "line: {a: 1\\.0}";
        let t = toks(text);
        assert!(
            t.contains(&(0, 10, 4, TOK_STRING)),
            "escaped value must classify as String: {t:?}"
        );
    }

    #[test]
    fn escaped_open_bracket_in_inline_key_is_one_property_token() {
        // spec/versions/0.8/tests/valid/key_escaping/
        // escaped_open_bracket_in_inline_pair_key_after_comma.ktav — the
        // key is `\[a`, decoding to `[a`; before the fix the `\` alone
        // became a PROPERTY token, `[` opened a bogus nested array, and
        // `a: 1` collapsed into one String.
        let text = "obj: {x: 0, \\[a: 1}";
        let t = toks(text);
        assert!(
            t.contains(&(0, 12, 3, TOK_PROPERTY)),
            "\\[a must be one PROPERTY token: {t:?}"
        );
        assert!(
            t.contains(&(0, 17, 1, TOK_NUMBER)),
            "1 must be a NUMBER token: {t:?}"
        );
        // No spurious extra OPERATOR from treating the escaped `[` as a
        // nested-array opener: the outer `:` plus `{`, `:`, `,`, `:`, `}`
        // inline = 6 operators, not 7.
        let operator_count = t
            .iter()
            .filter(|&&(l, _, _, tt)| l == 0 && tt == TOK_OPERATOR)
            .count();
        assert_eq!(operator_count, 6, "unexpected OPERATOR token: {t:?}");
    }

    #[test]
    fn multiline_stripped_block_content_is_plain_strings() {
        // § 5.6: inside an open `(` block, ordinary line-shape rules are
        // suspended — a content line that looks like a pair, a comment or
        // a lone closer must still become one trimmed STRING token.
        let text = "motd: (\n    port: 8080\n    ## note\n    true\n)\n";
        let t = toks(text);
        assert_eq!(
            t,
            vec![
                (0, 0, 4, TOK_PROPERTY), // motd
                (0, 4, 1, TOK_OPERATOR), // :
                (0, 6, 1, TOK_OPERATOR), // (
                (1, 4, 10, TOK_STRING),  // port: 8080
                (2, 4, 7, TOK_STRING),   // ## note
                (3, 4, 4, TOK_STRING),   // true
                (4, 0, 1, TOK_OPERATOR), // )
            ]
        );
    }

    #[test]
    fn multiline_content_line_that_looks_like_a_closer_stays_string() {
        // A content line trimming to `}` must not be treated as a
        // CloseBrace — only the block's own terminator (`)`) ends it.
        let text = "key: (\n}\n)\n";
        let t = toks(text);
        assert_eq!(
            t,
            vec![
                (0, 0, 3, TOK_PROPERTY),
                (0, 3, 1, TOK_OPERATOR),
                (0, 5, 1, TOK_OPERATOR),
                (1, 0, 1, TOK_STRING),
                (2, 0, 1, TOK_OPERATOR),
            ]
        );
    }

    #[test]
    fn multiline_verbatim_block_requires_double_paren_to_close() {
        // Inside a verbatim `((` block a lone `)` is ordinary content;
        // only `))` closes it. Content lines nest brace-looking text.
        let text = "body: ((\n {\n \"qwe\": 1\n }\n))\n";
        let t = toks(text);
        assert_eq!(
            t,
            vec![
                (0, 0, 4, TOK_PROPERTY), // body
                (0, 4, 1, TOK_OPERATOR), // :
                (0, 6, 2, TOK_OPERATOR), // ((
                (1, 1, 1, TOK_STRING),   // {
                (2, 1, 8, TOK_STRING),   // "qwe": 1
                (3, 1, 1, TOK_STRING),   // }
                (4, 0, 2, TOK_OPERATOR), // ))
            ]
        );
    }

    #[test]
    fn multiline_blank_content_line_has_no_token() {
        let text = "key: (\n\n)\n";
        let t = toks(text);
        assert_eq!(
            t,
            vec![
                (0, 0, 3, TOK_PROPERTY),
                (0, 3, 1, TOK_OPERATOR),
                (0, 5, 1, TOK_OPERATOR),
                (2, 0, 1, TOK_OPERATOR),
            ]
        );
    }

    #[test]
    fn tokens_agree_across_line_terminators() {
        // § 3.2: LF, CR and CRLF are equivalent line terminators — the
        // decoded (line, col, length, type) stream must be identical
        // regardless of which one the document uses.
        let lf = "a: 1\nb: {\n    c: 2\n}\n";
        let cr = "a: 1\rb: {\r    c: 2\r}\r";
        let crlf = "a: 1\r\nb: {\r\n    c: 2\r\n}\r\n";
        let want = toks(lf);
        assert_eq!(toks(cr), want);
        assert_eq!(toks(crlf), want);
    }

    #[test]
    fn multiline_array_item_opener_is_operator_not_string() {
        // A bare `(` / `((` array item opens the same multi-line block a
        // pair's value would — before the fix `classify_line` had no
        // CompoundOpen case for it here, so the opener rendered as a
        // one-char STRING instead of an OPERATOR.
        let text = "items: [\n(\nhello\n)\n]\n";
        let t = toks(text);
        assert!(
            t.contains(&(1, 0, 1, TOK_OPERATOR)),
            "lone `(` array item must be OPERATOR: {t:?}"
        );
        assert!(
            t.contains(&(2, 0, 5, TOK_STRING)),
            "block content must be STRING: {t:?}"
        );
        assert!(
            t.contains(&(3, 0, 1, TOK_OPERATOR)),
            "terminator `)` must be OPERATOR: {t:?}"
        );
    }
}
