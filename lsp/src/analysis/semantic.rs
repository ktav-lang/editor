//! LSP semantic-tokens producer, driven by the shared
//! [`crate::tokens`] line classifier (the single source of truth that
//! mirrors `ktav::parser`'s separator rules).
//!
//! Token type indices MUST match the order returned by [`token_types`].

use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

use crate::tokens::{classify_line, classify_value, split_dotted, LineKind, Marker, ValueKind};

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
}

/// Produce semantic tokens for `text`. Output is encoded in the
/// LSP-mandated delta format.
pub fn semantic_tokens(text: &str) -> Vec<SemanticToken> {
    let mut abs: Vec<AbsToken> = Vec::new();

    for (line_idx, line) in text.split('\n').enumerate() {
        emit_line(line_idx as u32, line, &mut abs);
    }

    encode_deltas(&abs)
}

fn emit_line(line: u32, raw: &str, out: &mut Vec<AbsToken>) {
    match classify_line(raw) {
        LineKind::Blank => {}
        LineKind::Comment { start, length } => {
            out.push(AbsToken {
                line,
                start,
                length,
                token_type: TOK_COMMENT,
            });
        }
        LineKind::CloseBrace { start } => {
            out.push(AbsToken {
                line,
                start,
                length: 1,
                token_type: TOK_OPERATOR,
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
            });
            if value_length > 0 {
                out.push(AbsToken {
                    line,
                    start: value_start,
                    length: value_length,
                    token_type: TOK_STRING,
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
                if !seg.is_empty() {
                    out.push(AbsToken {
                        line,
                        start: col,
                        length: seg.len() as u32,
                        token_type: TOK_PROPERTY,
                    });
                }
            }
            out.push(AbsToken {
                line,
                start: marker_start,
                length: marker.len() as u32,
                token_type: TOK_OPERATOR,
            });
            if value_length > 0 {
                // An inline compound value (`{…}` / `[…]` on a `:` line) is
                // tokenized structurally so its brackets read as operators
                // (and thus bracket-match) rather than disappearing into one
                // opaque string. `::` (Raw) bodies stay literal per spec.
                if marker == Marker::Plain
                    && value_kind == ValueKind::String
                    && starts_inline_compound(value_text)
                {
                    emit_inline(line, value_start, value_text, out);
                } else {
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
                    });
                }
            }
        }
        LineKind::ArrayItem {
            start,
            length,
            kind,
        } => {
            let item_text = &raw[start as usize..start as usize + length as usize];
            if matches!(kind, ValueKind::String) && starts_inline_compound(item_text) {
                emit_inline(line, start, item_text, out);
            } else {
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
                });
            }
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

/// Tokenize a single-line inline compound (`{k: v, …}` / `[v1, v2]`, nesting
/// allowed) into structural + scalar sub-tokens. `base` is the absolute
/// (byte) column of `text[0]`.
///
/// Brackets, commas and `:` / `::` separators become OPERATOR tokens so the
/// editor treats them as real brackets (enabling bracket matching) instead of
/// folding the whole value into one opaque string. Object keys become
/// PROPERTY; scalar values are classified (STRING / NUMBER / KEYWORD).
///
/// The grammar's head/rest rule is preserved: `{` / `[` open a nested compound
/// only at a value position; once a scalar run has begun they are ordinary
/// literal content (`hello{world`).
fn emit_inline(line: u32, base: u32, text: &str, out: &mut Vec<AbsToken>) {
    let b = text.as_bytes();
    let mut i = 0usize;
    // Container stack: true = object, false = array.
    let mut stack: Vec<bool> = Vec::new();
    // Inside an object, the next scalar-shaped run is a key until the `:`.
    let mut expect_key = false;

    let push = |out: &mut Vec<AbsToken>, start: usize, len: usize, tt: u32| {
        if len > 0 {
            out.push(AbsToken {
                line,
                start: base + start as u32,
                length: len as u32,
                token_type: tt,
            });
        }
    };

    while i < b.len() {
        match b[i] {
            b' ' | b'\t' => i += 1,
            c @ (b'{' | b'[') => {
                push(out, i, 1, TOK_OPERATOR);
                let is_obj = c == b'{';
                stack.push(is_obj);
                expect_key = is_obj;
                i += 1;
            }
            b'}' | b']' => {
                push(out, i, 1, TOK_OPERATOR);
                stack.pop();
                expect_key = false;
                i += 1;
            }
            b',' => {
                push(out, i, 1, TOK_OPERATOR);
                expect_key = matches!(stack.last(), Some(true));
                i += 1;
            }
            b':' if expect_key => {
                // Pair separator inside an object (`:` or `::`).
                let len = if i + 1 < b.len() && b[i + 1] == b':' {
                    2
                } else {
                    1
                };
                push(out, i, len, TOK_OPERATOR);
                expect_key = false;
                i += len;
            }
            _ => {
                let start = i;
                if expect_key {
                    // Key run: up to the separator / structural delimiters.
                    while i < b.len() && !matches!(b[i], b':' | b',' | b'{' | b'}' | b'[' | b']') {
                        i += 1;
                    }
                    let run = &text[start..i];
                    push(out, start, run.trim_end().len(), TOK_PROPERTY);
                    // expect_key stays set; the `:` arm clears it.
                } else {
                    // Scalar value: run until an unescaped `,` / `}` / `]`.
                    // `{` / `[` mid-run are literal content (head/rest rule).
                    while i < b.len() {
                        match b[i] {
                            b'\\' => i = (i + 2).min(b.len()),
                            b',' | b'}' | b']' => break,
                            _ => i += 1,
                        }
                    }
                    let run = &text[start..i];
                    let lead = run.len() - run.trim_start().len();
                    let body = run.trim();
                    let tt = match classify_value(body) {
                        ValueKind::Bool => TOK_KEYWORD,
                        ValueKind::Null => TOK_NULL,
                        ValueKind::Number => TOK_NUMBER,
                        _ => TOK_STRING,
                    };
                    push(out, start + lead, body.len(), tt);
                }
            }
        }
    }
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
            token_modifiers_bitset: 0,
        });
        prev_line = t.line;
        prev_start = t.start;
    }
    out
}
