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

/// Find the first line where `key` appears as a key segment at the
/// **top level** of the document (depth 0). Brace/bracket openers and
/// closers on their own line bump the depth; nested objects are skipped
/// so a top-level `host` is not matched by an inner `host`. The matched
/// segment must be followed by `:`, `.`, or whitespace-then-`:`/`.`.
///
/// Returns the full-line range when found.
fn locate_key(text: &str, key: &str) -> Option<Range> {
    let mut depth: i32 = 0;
    for (i, line) in text.split('\n').enumerate() {
        let trimmed = line.trim_start();
        // Lone closer line bumps depth DOWN before inspection — closers
        // belong to the parent scope.
        if trimmed == "}" || trimmed == "]" || trimmed == ")" {
            depth -= 1;
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Only look for top-level keys at depth 0.
        if depth == 0 {
            if let Some(rest) = trimmed.strip_prefix(key) {
                // Boundary check: next byte must be ':' or '.', possibly
                // preceded by whitespace before ':' / '.'.
                let next = rest.trim_start_matches([' ', '\t']);
                let ok = next.starts_with(':') || next.starts_with('.');
                if ok {
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

        // Track depth: an opener at end-of-line (after trimming) starts a
        // nested compound. We look at the trimmed-trailing tail.
        let tail = trimmed.trim_end();
        // Lone openers like `{`, `[`, `(`, `((`.
        if matches!(tail, "{" | "[" | "(" | "((") {
            depth += 1;
            continue;
        }
        // Pair line ending with an opener: `key: {` / `key: [` / `key: (`
        // / `key: ((`.
        if tail.ends_with(" {")
            || tail.ends_with(" [")
            || tail.ends_with(" (")
            || tail.ends_with(" ((")
        {
            depth += 1;
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
