//! LSP semantic-tokens producer, driven by the shared
//! [`crate::tokens`] line classifier (the single source of truth that
//! mirrors `ktav::parser`'s separator rules).
//!
//! Token type indices MUST match the order returned by [`token_types`].

use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

use crate::tokens::{classify_line, split_dotted, LineKind, ValueKind};

/// Token types we expose, in the order their indices are referenced
/// from the deltas.
pub fn token_types() -> Vec<SemanticTokenType> {
    vec![
        SemanticTokenType::COMMENT,  // 0
        SemanticTokenType::KEYWORD,  // 1
        SemanticTokenType::NUMBER,   // 2
        SemanticTokenType::STRING,   // 3
        SemanticTokenType::PROPERTY, // 4
        SemanticTokenType::OPERATOR, // 5
    ]
}

const TOK_COMMENT: u32 = 0;
const TOK_KEYWORD: u32 = 1;
const TOK_NUMBER: u32 = 2;
const TOK_STRING: u32 = 3;
const TOK_PROPERTY: u32 = 4;
const TOK_OPERATOR: u32 = 5;

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
                let tt = match value_kind {
                    ValueKind::Null | ValueKind::Bool => TOK_KEYWORD,
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
        LineKind::ArrayItem {
            start,
            length,
            kind,
        } => {
            let tt = match kind {
                ValueKind::Null | ValueKind::Bool => TOK_KEYWORD,
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
