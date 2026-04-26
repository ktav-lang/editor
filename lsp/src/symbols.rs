//! Build a `Vec<DocumentSymbol>` tree from a parsed [`ktav::Value`].
//!
//! Position information is approximate: `ktav::Value` does not carry
//! source spans, so we scan the raw text line by line for `key:` matches
//! at the depth implied by the brace stack. This is a best-effort
//! navigator — clients only need positions accurate enough to jump near
//! the symbol; precise highlighting is the semantic-tokens path.

use ktav::Value;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// Build a tree of `DocumentSymbol`s for the top-level object.
pub fn build_symbols(value: &Value, text: &str) -> Vec<DocumentSymbol> {
    let Value::Object(map) = value else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(map.len());
    for (k, v) in map {
        let range = locate_key(text, k.as_str()).unwrap_or_else(zero_range);
        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: k.to_string(),
            detail: Some(value_kind(v).to_string()),
            kind: kind_for(v),
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: build_children(v, text),
        });
    }
    out
}

fn build_children(value: &Value, text: &str) -> Option<Vec<DocumentSymbol>> {
    match value {
        Value::Object(map) => {
            let mut kids = Vec::with_capacity(map.len());
            for (k, v) in map {
                let range = locate_key(text, k.as_str()).unwrap_or_else(zero_range);
                #[allow(deprecated)]
                kids.push(DocumentSymbol {
                    name: k.to_string(),
                    detail: Some(value_kind(v).to_string()),
                    kind: kind_for(v),
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range: range,
                    children: build_children(v, text),
                });
            }
            Some(kids)
        }
        Value::Array(items) => {
            let mut kids = Vec::with_capacity(items.len());
            for (i, v) in items.iter().enumerate() {
                let r = zero_range();
                #[allow(deprecated)]
                kids.push(DocumentSymbol {
                    name: format!("[{}]", i),
                    detail: Some(value_kind(v).to_string()),
                    kind: kind_for(v),
                    tags: None,
                    deprecated: None,
                    range: r,
                    selection_range: r,
                    children: build_children(v, text),
                });
            }
            Some(kids)
        }
        _ => None,
    }
}

fn kind_for(v: &Value) -> SymbolKind {
    match v {
        Value::Null => SymbolKind::NULL,
        Value::Bool(_) => SymbolKind::BOOLEAN,
        Value::Integer(_) => SymbolKind::NUMBER,
        Value::Float(_) => SymbolKind::NUMBER,
        Value::String(_) => SymbolKind::STRING,
        Value::Array(_) => SymbolKind::ARRAY,
        Value::Object(_) => SymbolKind::MODULE,
    }
}

fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Find the first line where `key:` appears at the start of trimmed
/// content (ignoring `#` comments). Returns full-line range.
fn locate_key(text: &str, key: &str) -> Option<Range> {
    for (i, line) in text.split('\n').enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        // Match either `key:` or `key.` (dotted-key prefix).
        if let Some(rest) = trimmed.strip_prefix(key) {
            if rest.starts_with(':') || rest.starts_with('.') {
                return Some(Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: line.len() as u32,
                    },
                });
            }
        }
    }
    None
}

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
