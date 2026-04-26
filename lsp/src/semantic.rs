//! Text-driven semantic-tokens producer.
//!
//! Walks each line, classifies it (comment, key+value, bare key, raw
//! marker, typed-scalar marker, keyword) and emits LSP semantic tokens
//! relative to the previous token, as the protocol requires.
//!
//! Token type indices MUST match the order returned by [`token_types`].

use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

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
        classify_line(line_idx as u32, line, &mut abs);
    }

    encode_deltas(&abs)
}

fn classify_line(line: u32, raw: &str, out: &mut Vec<AbsToken>) {
    let leading_ws = raw.len() - raw.trim_start().len();
    let trimmed = raw.trim_start();
    if trimmed.is_empty() {
        return;
    }

    // Comment: '#' starts the line (after any whitespace).
    if trimmed.starts_with('#') {
        out.push(AbsToken {
            line,
            start: leading_ws as u32,
            length: trimmed.trim_end().len() as u32,
            token_type: TOK_COMMENT,
        });
        return;
    }

    // Closing brace lines: `}`, `]`, `)` — operator
    if matches!(trimmed.chars().next(), Some('}') | Some(']') | Some(')')) && trimmed.len() == 1 {
        out.push(AbsToken {
            line,
            start: leading_ws as u32,
            length: 1,
            token_type: TOK_OPERATOR,
        });
        return;
    }

    // Array literal-string item: `:: value`
    if trimmed.starts_with("::") {
        out.push(AbsToken {
            line,
            start: leading_ws as u32,
            length: 2,
            token_type: TOK_OPERATOR,
        });
        let rest_off = leading_ws + 2;
        let rest = &raw[rest_off..];
        let rs = rest.len() - rest.trim_start().len();
        let value = rest.trim_start().trim_end();
        if !value.is_empty() {
            out.push(AbsToken {
                line,
                start: (rest_off + rs) as u32,
                length: value.len() as u32,
                token_type: TOK_STRING,
            });
        }
        return;
    }

    // Key: value form. The key runs to the first ':'.
    let Some(colon_rel) = trimmed.find(':') else {
        // No colon — bare item line in an array (scalar literal).
        emit_value(line, leading_ws as u32, trimmed.trim_end(), out);
        return;
    };

    let key = &trimmed[..colon_rel];
    if !key.is_empty() {
        out.push(AbsToken {
            line,
            start: leading_ws as u32,
            length: key.len() as u32,
            token_type: TOK_PROPERTY,
        });
    }

    let after_key = colon_rel + 1;
    // Marker detection: '::' (raw), ':i' (int typed), ':f' (float typed),
    // or plain ':'.
    let after_bytes = trimmed.as_bytes();
    let mut marker_len = 1; // the ':'
    if after_bytes.len() > after_key {
        match after_bytes[after_key] {
            b':' => marker_len = 2,
            b'i' | b'f' => {
                // typed-scalar marker only if followed by space or EOL
                let next = after_bytes.get(after_key + 1).copied();
                if matches!(next, None | Some(b' ') | Some(b'\t')) {
                    marker_len = 2;
                }
            }
            _ => {}
        }
    }
    out.push(AbsToken {
        line,
        start: (leading_ws + colon_rel) as u32,
        length: marker_len as u32,
        token_type: TOK_OPERATOR,
    });

    // Value (if present)
    let val_off = leading_ws + colon_rel + marker_len;
    if val_off >= raw.len() {
        return;
    }
    let val_chunk = &raw[val_off..];
    let vs = val_chunk.len() - val_chunk.trim_start().len();
    let value = val_chunk.trim_start().trim_end();
    if value.is_empty() {
        return;
    }

    let is_typed = marker_len == 2 && matches!(after_bytes[after_key], b'i' | b'f');
    let is_raw = marker_len == 2 && after_bytes[after_key] == b':';

    let tok_type = if is_raw {
        TOK_STRING
    } else if is_typed {
        TOK_NUMBER
    } else {
        classify_value(value)
    };

    // Compound openers — these are operators on this line.
    if matches!(value, "{" | "[" | "(" | "((" | "{}" | "[]" | "()") {
        out.push(AbsToken {
            line,
            start: (val_off + vs) as u32,
            length: value.len() as u32,
            token_type: TOK_OPERATOR,
        });
        return;
    }

    out.push(AbsToken {
        line,
        start: (val_off + vs) as u32,
        length: value.len() as u32,
        token_type: tok_type,
    });
}

fn emit_value(line: u32, start: u32, value: &str, out: &mut Vec<AbsToken>) {
    if value.is_empty() {
        return;
    }
    out.push(AbsToken {
        line,
        start,
        length: value.len() as u32,
        token_type: classify_value(value),
    });
}

fn classify_value(v: &str) -> u32 {
    match v {
        "null" | "true" | "false" => TOK_KEYWORD,
        _ if looks_numeric(v) => TOK_NUMBER,
        _ => TOK_STRING,
    }
}

fn looks_numeric(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let start = if bytes[0] == b'-' || bytes[0] == b'+' {
        1
    } else {
        0
    };
    if start >= bytes.len() {
        return false;
    }
    bytes[start..].iter().all(|b| {
        b.is_ascii_digit() || *b == b'.' || *b == b'e' || *b == b'E' || *b == b'+' || *b == b'-'
    }) && bytes[start..].iter().any(|b| b.is_ascii_digit())
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
