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

use crate::analysis::semantic::{walk_inline, InlineEvent};
use crate::tokens::{find_key_separator, is_ktav_ws, split_dotted};

/// Build a tree of `DocumentSymbol`s for the top-level value.
///
/// Per Ktav spec § 5.0.1, the top-level value may be an Object or an
/// Array. For Object roots the outline is built from key hits; for
/// Array roots the items render as `[0]`, `[1]`, … entries — same
/// shape we already use for nested arrays.
pub fn build_symbols(value: &Value, text: &str) -> Vec<DocumentSymbol> {
    let root_is_array = matches!(value, Value::Array(_));
    let hits = collect_key_hits(text, root_is_array);
    let mut cursor = Cursor {
        hits: &hits,
        pos: 0,
    };
    match value {
        Value::Object(map) => build_object_at(map, &mut cursor, 0),
        Value::Array(items) => build_array_items(items, &mut cursor, 0, text),
        _ => Vec::new(),
    }
}

/// Build symbols for a top-level Array, where items appear at depth 0
/// (no enclosing `[ ... ]` brackets in the source). Each item renders
/// as `[i]`. For object items the line range is the line of the first
/// key (depth 0); for bare-scalar items we fall back to the i-th
/// non-blank, non-comment, non-closer line.
fn build_array_items(
    items: &[Value],
    cursor: &mut Cursor<'_>,
    depth: u32,
    text: &str,
) -> Vec<DocumentSymbol> {
    let item_lines = collect_top_array_item_lines(text);
    let mut out = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        // Default range covers the i-th item line if we found one.
        let range = item_lines
            .get(i)
            .copied()
            .map(|(line, line_len)| Range {
                start: Position { line, character: 0 },
                end: Position {
                    line,
                    character: line_len,
                },
            })
            .unwrap_or_else(zero_range);
        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: format!("[{}]", i),
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

/// Walk the source line by line and collect `(line, line_len)` for
/// each top-level Array item — i.e. lines at virtual depth 0 that are
/// not blank, comments, or lone closers.
///
/// This is a separate pass from `collect_key_hits` because top-level
/// Array items may be bare scalars without keys (e.g. `foo` on a line
/// of its own), which `collect_key_hits` skips.
fn collect_top_array_item_lines(text: &str) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    let mut multi = Multi::None;
    let mut depth: u32 = 0;
    let mut compound_pushes: Vec<u32> = Vec::new();

    for (i, line) in crate::lines::split_lines(text).into_iter().enumerate() {
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
                    depth = depth.saturating_sub(n);
                }
            }
            continue;
        }

        let trimmed = line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tail = trimmed.trim_end();

        if tail == "}" || tail == "]" {
            if let Some(n) = compound_pushes.pop() {
                depth = depth.saturating_sub(n);
            }
            continue;
        }

        // Record this line as a top-level item if we're at depth 0.
        if depth == 0 {
            out.push((i as u32, line.len() as u32));
        }

        // Now apply depth changes for the NEXT line.
        if matches!(tail, "{" | "[" | "(" | "((") {
            match tail {
                "(" => multi = Multi::Stripped,
                "((" => multi = Multi::Verbatim,
                _ => {}
            }
            compound_pushes.push(1);
            depth += 1;
            continue;
        }

        // Pair with trailing opener? Push compound depth for it.
        let pushed_multi = if tail.ends_with(" ((") {
            Some(Multi::Verbatim)
        } else if tail.ends_with(" (") {
            Some(Multi::Stripped)
        } else {
            None
        };
        let opens_compound = pushed_multi.is_some() || tail.ends_with(" {") || tail.ends_with(" [");
        if opens_compound {
            // For top-level item tracking we only care about physical
            // depth, so push 1 regardless of dotted-key segment count.
            compound_pushes.push(1);
            depth += 1;
            if let Some(m) = pushed_multi {
                multi = m;
            }
        }
    }

    out
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
    /// Column span of this hit within `line`, in bytes. `(0, line_len)`
    /// for a hit that names its own top-level physical line (the
    /// existing whole-line approximation, unchanged); a tighter
    /// `(start, end)` for a key nested inside an inline compound that
    /// shares a line with other hits — see [`collect_inline_hits`].
    col_start: u32,
    col_end: u32,
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
                        character: h.col_start,
                    },
                    end: Position {
                        line: h.line,
                        character: h.col_end,
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
///
/// `root_is_array` tells the scanner whether the document's own root
/// wrapper — a lone `{`/`}` (or complete single-line `{...}`) for an
/// Object root, `[`/`]` for an Array root — is the FIRST content line.
/// That wrapper is transparent (§ 5.0.1): `build_symbols` dispatches the
/// root's own keys/items at depth 0, so the wrapper must not consume a
/// virtual-depth level here either, or every root-level hit ends up one
/// level too deep and falls back to `zero_range()`.
fn collect_key_hits(text: &str, root_is_array: bool) -> Vec<KeyHit<'_>> {
    let mut hits: Vec<KeyHit<'_>> = Vec::new();
    let mut multi = Multi::None;
    let mut virtual_depth: u32 = 0;
    // Per physical compound (object/array/multi-line) on the stack: how
    // many virtual depths we pushed for it. A pair-with-opener `a.b: {`
    // pushes `segment_count` (2 for `a.b`); a lone opener pushes 1.
    let mut compound_pushes: Vec<u32> = Vec::new();
    let mut first_content_line = true;

    for (i, line) in crate::lines::split_lines(text).into_iter().enumerate() {
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

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let is_first = first_content_line;
        first_content_line = false;
        let leading_ws = (line.len() - trimmed.len()) as u32;

        // Lone closer line drops one physical compound off the stack.
        if trimmed == "}" || trimmed == "]" {
            if let Some(n) = compound_pushes.pop() {
                virtual_depth = virtual_depth.saturating_sub(n);
            }
            continue;
        }

        let tail = trimmed.trim_end();

        // Lone opener line — pushes a compound, no key recorded. The
        // FIRST content line's own bracket is transparent when it
        // matches the root's shape (see doc comment above).
        if matches!(tail, "{" | "[" | "(" | "((") {
            let is_root_wrapper =
                is_first && ((!root_is_array && tail == "{") || (root_is_array && tail == "["));
            if !is_root_wrapper {
                match tail {
                    "(" => multi = Multi::Stripped,
                    "((" => multi = Multi::Verbatim,
                    _ => {}
                }
                compound_pushes.push(1);
                virtual_depth += 1;
            }
            continue;
        }

        // A line beginning with `{` / `[` is an inline-compound value —
        // never a `key: value` pair even if it contains a `:` (keys can
        // never start with a bracket; mirrors `classify_line`'s own
        // precedence rule). This covers a bare inline compound array
        // item (`{a: 1}` on its own line) AND the top-level root's own
        // complete single-line literal (`{host: 1, port: 2}` as the
        // WHOLE document) — the latter is exactly as transparent as the
        // lone-opener wrapper above.
        if matches!(trimmed.as_bytes()[0], b'{' | b'[') {
            let is_root_wrapper = is_first
                && ((!root_is_array && trimmed.as_bytes()[0] == b'{')
                    || (root_is_array && trimmed.as_bytes()[0] == b'['));
            let base_depth = if is_root_wrapper {
                virtual_depth
            } else {
                virtual_depth + 1
            };
            collect_inline_hits(trimmed, leading_ws, base_depth, &mut hits, i as u32);
            continue;
        }

        // Pair / array-item lines: anything else. Spec 0.6.0: the
        // separator is the first UNESCAPED `:`; `\:` inside the key is
        // literal content. Spec 0.7.0: a `:` inside a quoted key segment
        // (§ 5.3.3) is opaque too.
        let Some(colon) = find_key_separator(trimmed) else {
            // Array item without `:` — no key recorded.
            continue;
        };

        let key_part = trimmed[..colon].trim_end();
        if key_part.is_empty() {
            continue;
        }

        // Split dotted key into segments. Empty segments (from `..`)
        // would be invalid input — the parser would have already
        // failed, so we just defensively filter them out. Spec 0.6.0:
        // a `\.` inside a segment is a literal dot, not a separator;
        // split on UNESCAPED `.` only. Spec 0.7.0: a `.` inside a quoted
        // segment (§ 5.3.3) does not split it.
        let line_len = line.len() as u32;
        let line_no = i as u32;
        let mut seg_count: u32 = 0;
        for (_, seg) in split_dotted(0, key_part) {
            if seg.is_empty() {
                continue;
            }
            hits.push(KeyHit {
                depth: virtual_depth + seg_count,
                name: seg,
                line: line_no,
                col_start: 0,
                col_end: line_len,
            });
            seg_count += 1;
        }

        if seg_count == 0 {
            continue;
        }

        // § 4: a Raw (`::`) value is always a literal string, never
        // structurally inline, however it looks — never scan it below.
        let is_raw_marker = tail.as_bytes().get(colon + 1) == Some(&b':');
        if !is_raw_marker {
            let (value_col, value) = pair_value_after_marker(leading_ws, tail, colon);
            // An inline compound value (`key: {a: 1}` / `key: [1, 2]`)
            // embeds its own keys on THIS line. Excludes the six exact
            // compound-opener/empty forms, handled by the trailing-opener
            // check below.
            if matches!(value.as_bytes().first(), Some(b'{') | Some(b'['))
                && !matches!(value, "{" | "[" | "(" | "((" | "{}" | "[]" | "()")
            {
                collect_inline_hits(
                    value,
                    value_col,
                    virtual_depth + seg_count,
                    &mut hits,
                    line_no,
                );
            }
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

/// For a Plain-marker Pair line, the value substring after the marker
/// and its own leading whitespace, plus its absolute byte column within
/// the physical line (`leading_ws` is that line's own indent — the
/// column where `tail` begins). `tail` is `trimmed` with only trailing
/// whitespace stripped, so indices up to and including `colon` line up
/// with `trimmed`/`leading_ws` unchanged.
fn pair_value_after_marker(leading_ws: u32, tail: &str, colon: usize) -> (u32, &str) {
    let marker_len = if tail.as_bytes().get(colon + 1) == Some(&b':') {
        2
    } else {
        1
    };
    let start = colon + marker_len;
    let body = &tail[start..];
    let inner_ws = body.len() - body.trim_start_matches(is_ktav_ws).len();
    (leading_ws + (start + inner_ws) as u32, &body[inner_ws..])
}

/// Depth-aware key-hit collector for an inline compound (`{...}` /
/// `[...]`) found on a single line — reuses [`walk_inline`] (the same
/// quote/escape-aware scanner `emit_inline` tokenizes with) rather than
/// re-deriving the structural rules a third time.
///
/// `base_depth` is the virtual depth of keys directly inside the
/// OUTERMOST bracket of `text` — the caller has already accounted for
/// how this inline text is reached: a key's value gets
/// `virtual_depth + seg_count`; a bare compound array item gets
/// `virtual_depth + 1`; the transparent top-level root wrapper gets
/// `virtual_depth` unchanged (see `collect_key_hits`). Every further
/// nested `{`/`[` inside `text` adds one level per dotted-key segment
/// that opened it, or flatly one level when no key precedes it (a bare
/// compound array item nested inline, e.g. `[{a: 1}, {b: 2}]`) — mirrors
/// `collect_key_hits`'s own line-level rule exactly.
///
/// `col_base` is the absolute byte column where `text` begins on `line`,
/// used to turn `walk_inline`'s text-relative offsets into real
/// [`KeyHit`] columns via [`split_dotted`]'s own `key_start` parameter.
fn collect_inline_hits<'a>(
    text: &'a str,
    col_base: u32,
    base_depth: u32,
    hits: &mut Vec<KeyHit<'a>>,
    line: u32,
) {
    let mut depth = base_depth;
    let mut frame_bumps: Vec<u32> = Vec::new();
    let mut pending_bump: Option<u32> = None;
    let mut first_open_seen = false;

    walk_inline(text, |ev| match ev {
        InlineEvent::Open { .. } => {
            if !first_open_seen {
                // The outermost bracket is already accounted for by
                // `base_depth` — never a bump of its own.
                first_open_seen = true;
            } else {
                let bump = pending_bump.unwrap_or(1);
                frame_bumps.push(bump);
                depth += bump;
            }
            pending_bump = None;
        }
        InlineEvent::Close { .. } => {
            if let Some(b) = frame_bumps.pop() {
                depth = depth.saturating_sub(b);
            }
            pending_bump = None;
        }
        InlineEvent::Key {
            start,
            text: key_text,
        } => {
            let mut n = 0u32;
            for (seg_col, seg) in split_dotted(col_base + start as u32, key_text) {
                if seg.is_empty() {
                    continue;
                }
                hits.push(KeyHit {
                    depth: depth + n,
                    name: seg,
                    line,
                    col_start: seg_col,
                    col_end: seg_col + seg.len() as u32,
                });
                n += 1;
            }
            pending_bump = if n > 0 { Some(n) } else { None };
        }
        InlineEvent::Value { .. } => {
            pending_bump = None;
        }
        InlineEvent::Comma { .. } | InlineEvent::Sep { .. } => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Value {
        ktav::parse(text).expect("fixture must parse")
    }

    fn find<'a>(syms: &'a [DocumentSymbol], name: &str) -> &'a DocumentSymbol {
        syms.iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("no symbol named '{name}' among {syms:?}"))
    }

    #[test]
    fn inline_object_value_keys_get_precise_ranges() {
        // `a: {name: alice}` (valid/inline/object/single_pair.ktav) —
        // before the fix, `name` had no hit at all (only `a` did) and
        // fell back to zero_range().
        let text = "a: {name: alice}\n";
        let syms = build_symbols(&parse(text), text);
        let a = find(&syms, "a");
        let kids = a.children.as_ref().expect("a has children");
        let name = find(kids, "name");
        assert_ne!(
            name.range,
            zero_range(),
            "name must not fall back to zero_range"
        );
        assert_eq!(name.range.start.line, 0);
        assert_eq!(name.range.start.character, 4);
        assert_eq!(name.range.end.character, 8);
    }

    #[test]
    fn inline_object_multiple_keys_each_get_own_range() {
        // valid/inline/object/multiple_pairs.ktav.
        let text = "server: {host: localhost, port: 8080, tls: true}\n";
        let syms = build_symbols(&parse(text), text);
        let server = find(&syms, "server");
        let kids = server.children.as_ref().unwrap();
        for (name, want_col) in [("host", 9), ("port", 26), ("tls", 38)] {
            let sym = find(kids, name);
            assert_eq!(sym.range.start.character, want_col, "for key {name}");
            assert_eq!(sym.range.end.character, want_col + name.len() as u32);
        }
    }

    #[test]
    fn inline_nested_array_of_objects_keys_resolve() {
        // valid/inline/nested/mixed.ktav's first line — inline array of
        // inline objects, two nesting levels inside one key's value.
        let text = "users: [{name: alice, age: 30}, {name: bob, age: 25}]\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        let users = find(&syms, "users");
        let kids = users.children.as_ref().expect("users has children");
        assert_eq!(kids.len(), 2);
        let alice = find(kids[0].children.as_ref().unwrap(), "name");
        assert_ne!(alice.range, zero_range());
        let bob = find(kids[1].children.as_ref().unwrap(), "name");
        assert_ne!(bob.range, zero_range());
        assert_ne!(alice.range, bob.range);
    }

    #[test]
    fn inline_dotted_key_creates_nested_symbols() {
        // valid/inline/object/dotted_keys.ktav: `cfg: {a.b: 1, a.c: 2}`
        // → cfg.a.b / cfg.a.c, matching top-level dotted-key nesting.
        let text = "cfg: {a.b: 1, a.c: 2}\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        let cfg = find(&syms, "cfg");
        let a = find(cfg.children.as_ref().unwrap(), "a");
        let a_kids = a.children.as_ref().expect("a has children");
        assert_ne!(find(a_kids, "b").range, zero_range());
        assert_ne!(find(a_kids, "c").range, zero_range());
    }

    #[test]
    fn raw_marker_inside_inline_object_does_not_corrupt_hits() {
        // valid/raw_marker/in_inline.ktav.
        let text = "cfg: {real_num: 42, str_num:: 42, real_bool: true, str_bool:: true}\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        let cfg = find(&syms, "cfg");
        let kids = cfg.children.as_ref().unwrap();
        for k in ["real_num", "str_num", "real_bool", "str_bool"] {
            assert_ne!(find(kids, k).range, zero_range(), "for key {k}");
        }
    }

    #[test]
    fn top_level_object_brace_wrapper_is_transparent() {
        // valid/top_level/multiline_object_opener.ktav.
        let text = "{\n    name: alice\n    port: 8080\n}\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        let name = find(&syms, "name");
        assert_eq!(name.range.start.line, 1);
        let port = find(&syms, "port");
        assert_eq!(port.range.start.line, 2);
    }

    #[test]
    fn top_level_inline_object_literal_is_transparent() {
        // valid/top_level_inline/object.ktav — the WHOLE document is one
        // inline object literal; before the fix `find_key_separator`
        // matched the colon inside it and produced a garbage key.
        let text = "{host: localhost, port: 8080, tls: true}\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        assert_eq!(syms.len(), 3);
        for name in ["host", "port", "tls"] {
            assert_ne!(find(&syms, name).range, zero_range(), "for key {name}");
        }
    }

    #[test]
    fn top_level_array_bracket_wrapper_is_transparent() {
        // document_symbols_top_level_array_of_objects_have_children
        // (integration.rs) only checks names; this also pins positions.
        let text = "[\n{\n    name: alice\n}\n{\n    name: bob\n}\n]\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        assert_eq!(syms.len(), 2);
        let name0 = find(syms[0].children.as_ref().unwrap(), "name");
        assert_eq!(name0.range.start.line, 2);
        let name1 = find(syms[1].children.as_ref().unwrap(), "name");
        assert_eq!(name1.range.start.line, 5);
    }

    #[test]
    fn top_level_inline_array_of_objects_keys_resolve() {
        // valid/top_level_inline/unquoted_key_context_from_active_scope.ktav.
        let text = "[{a: 1, b: [2]}]\n";
        let value = parse(text);
        let syms = build_symbols(&value, text);
        assert_eq!(syms.len(), 1);
        let obj_kids = syms[0].children.as_ref().expect("[0] has children");
        assert_ne!(find(obj_kids, "a").range, zero_range());
        // `b`'s own hit resolves too, even though its value is itself an
        // inline array (no keys inside it to check further).
        assert_ne!(find(obj_kids, "b").range, zero_range());
    }
}
