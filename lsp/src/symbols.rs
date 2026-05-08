//! Build a `Vec<DocumentSymbol>` tree from a parsed [`ktav::Value`].
//!
//! Position information is approximate: `ktav::Value` does not carry
//! source spans, so we make a single line-by-line pass over the raw
//! text collecting `(virtual_depth, key_name, line_range)` for every
//! pair we see, then walk the parsed `Value` and pop hits in order with
//! a depth-aware sequential cursor. The earlier implementation rescanned
//! the entire document inside `locate_key` for every key — at large
//! sizes that was a hard O(N²) (11.5s on a 500 KiB doc with thousands
//! of top-level keys); the cursor walk is O(N).
//!
//! The cursor only advances forward, and the parser preserves
//! insertion order in `ObjectMap` (it is an `IndexMap`), so the DFS
//! over `Value` and the linear scan visit keys in the same source
//! order — hits line up by construction.
//!
//! The resulting outline is best-effort: clients only need positions
//! accurate enough to jump near the symbol; precise highlighting is
//! the semantic-tokens path.

use ktav::Value;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// Build a tree of `DocumentSymbol`s for the top-level object.
pub fn build_symbols(value: &Value, text: &str) -> Vec<DocumentSymbol> {
    let Value::Object(map) = value else {
        return Vec::new();
    };
    let hits = collect_key_hits(text);
    let mut cursor = Cursor {
        hits: &hits,
        pos: 0,
    };
    build_object_at(map, &mut cursor, 0)
}

fn build_object_at(
    map: &ktav::value::ObjectMap,
    cursor: &mut Cursor<'_>,
    depth: u32,
) -> Vec<DocumentSymbol> {
    let mut out = Vec::with_capacity(map.len());
    for (k, v) in map {
        let range = cursor.lookup(depth, k.as_str()).unwrap_or_else(zero_range);
        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: k.to_string(),
            detail: Some(value_kind(v).to_string()),
            kind: kind_for(v),
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: build_children(v, cursor, depth + 1),
        });
    }
    out
}

fn build_children(
    value: &Value,
    cursor: &mut Cursor<'_>,
    depth: u32,
) -> Option<Vec<DocumentSymbol>> {
    match value {
        Value::Object(map) => Some(build_object_at(map, cursor, depth)),
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
                    // An object inside an array sits one virtual level
                    // deeper than the array itself (the lone `{` opener
                    // bumps the scanner's depth too).
                    children: build_children(v, cursor, depth + 1),
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

// ---------------------------------------------------------------------------
// Single-pass scanner
// ---------------------------------------------------------------------------

struct KeyHit<'a> {
    depth: u32,
    name: &'a str,
    line: u32,
    /// Byte length of the source line (used for `range.end.character`).
    line_len: u32,
}

struct Cursor<'a> {
    hits: &'a [KeyHit<'a>],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Advance forward through the hit list until we find a hit at the
    /// requested depth and matching name. Returns the line range or
    /// `None` if we walked off the end (defensive: shouldn't happen for
    /// docs that parsed successfully, but a stale cache from
    /// `did_change` could still in principle slip through).
    fn lookup(&mut self, depth: u32, name: &str) -> Option<Range> {
        while self.pos < self.hits.len() {
            let h = &self.hits[self.pos];
            self.pos += 1;
            if h.depth == depth && h.name == name {
                return Some(Range {
                    start: Position {
                        line: h.line,
                        character: 0,
                    },
                    end: Position {
                        line: h.line,
                        character: h.line_len,
                    },
                });
            }
        }
        None
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Multi {
    None,
    Stripped,
    Verbatim,
}

/// Scan the document once, recording one `KeyHit` per key segment in
/// source order. Tracks "virtual depth" — the depth in the parsed
/// `Value` tree, which is bumped both by physical openers (`{`/`[`/
/// multi-line) and by every additional segment of a dotted key (a
/// pair-with-opener `a.b: {` pushes 2 virtual levels for one physical
/// brace).
fn collect_key_hits(text: &str) -> Vec<KeyHit<'_>> {
    let mut hits: Vec<KeyHit<'_>> = Vec::new();
    let mut multi = Multi::None;
    let mut virtual_depth: u32 = 0;
    // Per physical compound (object/array/multi-line) on the stack: how
    // many virtual depths we pushed for it. A pair-with-opener `a.b: {`
    // pushes `segment_count` (2 for `a.b`); a lone opener pushes 1.
    let mut compound_pushes: Vec<u32> = Vec::new();

    for (i, line) in text.split('\n').enumerate() {
        // Inside a multi-line block, the only line that matters is the
        // terminator. Comments / brackets / pseudo-keys in content are
        // not parsed (mirrors `Parser::handle_line` collecting branch).
        if multi != Multi::None {
            let trimmed = line.trim();
            let is_term = match multi {
                Multi::Stripped => trimmed == ")",
                Multi::Verbatim => trimmed == "))",
                Multi::None => false,
            };
            if is_term {
                multi = Multi::None;
                if let Some(n) = compound_pushes.pop() {
                    virtual_depth = virtual_depth.saturating_sub(n);
                }
            }
            continue;
        }

        let trimmed = line.trim_start();

        // Lone closer line drops one physical compound off the stack.
        if trimmed == "}" || trimmed == "]" {
            if let Some(n) = compound_pushes.pop() {
                virtual_depth = virtual_depth.saturating_sub(n);
            }
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tail = trimmed.trim_end();

        // Lone opener line — pushes a compound, no key recorded.
        if matches!(tail, "{" | "[" | "(" | "((") {
            match tail {
                "(" => multi = Multi::Stripped,
                "((" => multi = Multi::Verbatim,
                _ => {}
            }
            compound_pushes.push(1);
            virtual_depth += 1;
            continue;
        }

        // Pair / array-item lines: anything else.
        let Some(colon) = trimmed.find(':') else {
            // Array item without `:` — no key recorded.
            continue;
        };

        let key_part = trimmed[..colon].trim_end();
        if key_part.is_empty() {
            continue;
        }

        // Split dotted key into segments. Empty segments (from `..`)
        // would be invalid input — the parser would have already
        // failed, so we just defensively filter them out.
        let line_len = line.len() as u32;
        let line_no = i as u32;
        let mut seg_count: u32 = 0;
        for seg in key_part.split('.') {
            if seg.is_empty() {
                continue;
            }
            hits.push(KeyHit {
                depth: virtual_depth + seg_count,
                name: seg,
                line: line_no,
                line_len,
            });
            seg_count += 1;
        }

        if seg_count == 0 {
            continue;
        }

        // Detect a trailing opener on the same line: `key: {` /
        // `key: [` / `key: (` / `key: ((`.
        let pushed_multi = if tail.ends_with(" ((") {
            Some(Multi::Verbatim)
        } else if tail.ends_with(" (") {
            Some(Multi::Stripped)
        } else {
            None
        };
        let opens_compound = pushed_multi.is_some() || tail.ends_with(" {") || tail.ends_with(" [");

        if opens_compound {
            compound_pushes.push(seg_count);
            virtual_depth += seg_count;
            if let Some(m) = pushed_multi {
                multi = m;
            }
        }
        // Otherwise it's a scalar pair: hits are recorded, but no
        // compound push happens.
    }

    hits
}
