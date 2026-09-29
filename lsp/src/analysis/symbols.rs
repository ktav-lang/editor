//! Build a `Vec<DocumentSymbol>` tree from a parsed [`ktav::Value`].
//!
//! Position information is approximate: `ktav::Value` does not carry
//! source spans, so we make a single line-by-line pass over the raw
//! text collecting `(virtual_depth, key_name, line_range)` for every
//! pair we see, then walk the parsed `Value` and take hits from
//! depth/name queues. The earlier implementation rescanned
//! the entire document inside `locate_key` for every key — at large
//! sizes that was a hard O(N²) (11.5s on a 500 KiB doc with thousands
//! of top-level keys); indexing avoids repeated source scans.
//!
//! The parser preserves first-insertion order in `ObjectMap`, but a
//! dotted object can reopen after a sibling. DFS order then differs
//! from source order, so one global forward cursor would skip hits.
//!
//! The resulting outline is best-effort: clients only need positions
//! accurate enough to jump near the symbol; precise highlighting is
//! the semantic-tokens path.

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};

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
    let (hits, arrays, root_array) = collect_key_hits(text, root_is_array);
    let mut cursor = Cursor::new(&hits);
    let mut prefix = Vec::new();
    match value {
        Value::Object(map) => build_object_at(map, &mut cursor, &arrays, 0, &mut prefix),
        Value::Array(items) => build_array_items(items, &mut cursor, &arrays, root_array, 0, text),
        _ => Vec::new(),
    }
}

/// Build `[i]` symbols for a top-level Array. Inline items use their
/// own spans; multiline and bare items retain the physical-line fallback.
fn build_array_items(
    items: &[Value],
    cursor: &mut Cursor<'_>,
    arrays: &[InlineArray],
    inline_id: Option<usize>,
    depth: u32,
    text: &str,
) -> Vec<DocumentSymbol> {
    let item_lines = if inline_id.is_none() {
        collect_top_array_item_lines(text)
    } else {
        Vec::new()
    };
    let mut out = Vec::with_capacity(items.len());
    for (i, v) in items.iter().enumerate() {
        let inline_item = inline_id.and_then(|id| arrays[id].items.get(i));
        let child_array_id = inline_item.and_then(|item| item.array_id).or_else(|| {
            if !matches!(v, Value::Array(_)) {
                return None;
            }
            let line = item_lines.get(i)?.0;
            // The first inline array on this physical item line is its outer value.
            let id = arrays.partition_point(|array| array.line < line);
            arrays
                .get(id)
                .filter(|array| array.line == line)
                .map(|_| id)
        });
        let range = inline_item
            .map(|item| item.range)
            .or_else(|| {
                item_lines.get(i).copied().map(|(line, line_len)| Range {
                    start: Position { line, character: 0 },
                    end: Position {
                        line,
                        character: line_len,
                    },
                })
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
            children: build_children(
                v,
                cursor,
                arrays,
                child_array_id,
                depth + 1,
                &mut Vec::new(),
            ),
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
    #[derive(Clone, Copy, PartialEq)]
    enum Compound {
        Object,
        Other,
    }

    let mut out: Vec<(u32, u32)> = Vec::new();
    let mut multi = Multi::None;
    let mut compounds: Vec<Compound> = Vec::new();
    let mut first_content_line = true;

    for (i, line) in crate::lines::content_lines(text).into_iter().enumerate() {
        if multi != Multi::None {
            let trimmed = line.trim();
            let is_term = match multi {
                Multi::Stripped => trimmed == ")",
                Multi::Verbatim => trimmed == "))",
                Multi::None => false,
            };
            if is_term {
                multi = Multi::None;
                compounds.pop();
            }
            continue;
        }

        let trimmed = line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with("##") {
            continue;
        }

        let tail = trimmed.trim_end();

        if first_content_line && tail == "[" {
            first_content_line = false;
            continue;
        }
        first_content_line = false;

        if tail == "}" || tail == "]" {
            compounds.pop();
            continue;
        }

        // Record this line as a top-level item if we're at depth 0.
        if compounds.is_empty() {
            let bom = if i == 0 {
                crate::lines::leading_bom_len(text) as u32
            } else {
                0
            };
            out.push((i as u32, bom + line.len() as u32));
        }

        // Now apply depth changes for the NEXT line.
        if matches!(tail, "{" | "[" | "(" | "((") {
            match tail {
                "(" => multi = Multi::Stripped,
                "((" => multi = Multi::Verbatim,
                _ => {}
            }
            compounds.push(if tail == "{" {
                Compound::Object
            } else {
                Compound::Other
            });
            continue;
        }

        // Only pairs inside an Object can have a trailing opener. In an
        // Array, `foo: {` is a scalar item, not an Object opener.
        if compounds.last() != Some(&Compound::Object) {
            continue;
        }
        let Some(colon) = find_key_separator(tail) else {
            continue;
        };
        if colon == 0 || tail.as_bytes().get(colon + 1) == Some(&b':') {
            continue;
        }

        let pushed_multi = if tail.ends_with(" ((") {
            Some(Multi::Verbatim)
        } else if tail.ends_with(" (") {
            Some(Multi::Stripped)
        } else {
            None
        };
        let opens_compound = pushed_multi.is_some() || tail.ends_with(" {") || tail.ends_with(" [");
        if opens_compound {
            compounds.push(if tail.ends_with(" {") {
                Compound::Object
            } else {
                Compound::Other
            });
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
    arrays: &[InlineArray],
    depth: u32,
    prefix: &mut Vec<String>,
) -> Vec<DocumentSymbol> {
    let mut out = Vec::with_capacity(map.len());
    for (k, v) in map {
        let (range, array_id, reset_prefix) =
            cursor
                .lookup(depth, prefix, k.as_str())
                .unwrap_or((zero_range(), None, false));
        let children = if matches!(v, Value::Object(_)) && !reset_prefix {
            prefix.push(k.to_string());
            let children = build_children(v, cursor, arrays, array_id, depth + 1, prefix);
            prefix.pop();
            children
        } else {
            build_children(v, cursor, arrays, array_id, depth + 1, &mut Vec::new())
        };
        #[allow(deprecated)]
        out.push(DocumentSymbol {
            name: k.to_string(),
            detail: Some(value_kind(v).to_string()),
            kind: kind_for(v),
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children,
        });
    }
    out
}

fn build_children(
    value: &Value,
    cursor: &mut Cursor<'_>,
    arrays: &[InlineArray],
    inline_id: Option<usize>,
    depth: u32,
    prefix: &mut Vec<String>,
) -> Option<Vec<DocumentSymbol>> {
    match value {
        Value::Object(map) => Some(build_object_at(map, cursor, arrays, depth, prefix)),
        Value::Array(items) => {
            let mut kids = Vec::with_capacity(items.len());
            for (i, v) in items.iter().enumerate() {
                let inline_item = inline_id.and_then(|id| arrays[id].items.get(i));
                let r = inline_item.map_or_else(zero_range, |item| item.range);
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
                    children: build_children(
                        v,
                        cursor,
                        arrays,
                        inline_item.and_then(|item| item.array_id),
                        depth + 1,
                        &mut Vec::new(),
                    ),
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
    dotted_prefix: Vec<String>,
    /// An explicit object value starts a fresh dotted-key scope.
    resets_prefix: bool,
    line: u32,
    /// Column span of this hit within `line`, in bytes. `(0, line_len)`
    /// for a hit that names its own top-level physical line (the
    /// existing whole-line approximation, unchanged); a tighter
    /// `(start, end)` for a key nested inside an inline compound that
    /// shares a line with other hits — see [`collect_inline_hits`].
    col_start: u32,
    col_end: u32,
    array_id: Option<usize>,
}

struct InlineItem {
    range: Range,
    array_id: Option<usize>,
}

// Inline and block arrays use the same item-span table.
struct InlineArray {
    line: u32,
    items: Vec<InlineItem>,
}

struct Cursor<'a> {
    hits: &'a [KeyHit<'a>],
    by_depth: HashMap<u32, HashMap<Vec<String>, HashMap<Cow<'a, str>, VecDeque<usize>>>>,
}

impl<'a> Cursor<'a> {
    fn new(hits: &'a [KeyHit<'a>]) -> Self {
        let mut by_depth: HashMap<
            u32,
            HashMap<Vec<String>, HashMap<Cow<'a, str>, VecDeque<usize>>>,
        > = HashMap::new();
        for (index, hit) in hits.iter().enumerate() {
            if let Some(name) = decode_symbol_key(hit.name) {
                by_depth
                    .entry(hit.depth)
                    .or_default()
                    .entry(hit.dotted_prefix.clone())
                    .or_default()
                    .entry(name)
                    .or_default()
                    .push_back(index);
            }
        }
        Self { hits, by_depth }
    }

    /// Take the next source occurrence at this depth and dotted-key scope.
    fn lookup(
        &mut self,
        depth: u32,
        prefix: &[String],
        name: &str,
    ) -> Option<(Range, Option<usize>, bool)> {
        let index = self
            .by_depth
            .get_mut(&depth)?
            .get_mut(prefix)?
            .get_mut(name)?
            .pop_front()?;
        let h = &self.hits[index];
        Some((
            Range {
                start: Position {
                    line: h.line,
                    character: h.col_start,
                },
                end: Position {
                    line: h.line,
                    character: h.col_end,
                },
            },
            h.array_id,
            h.resets_prefix,
        ))
    }
}

// The Value tree has decoded keys, but hit spans must remain in source bytes.
fn decode_symbol_key(raw: &str) -> Option<Cow<'_, str>> {
    let raw = raw.trim_matches(is_ktav_ws);
    let input = match raw.as_bytes().first() {
        Some(quote @ (b'"' | b'\'' | b'`')) => {
            if raw.len() < 2 || raw.as_bytes().last() != Some(quote) {
                return None;
            }
            &raw[1..raw.len() - 1]
        }
        _ => raw,
    };
    if !input.contains('\\') {
        return Some(Cow::Borrowed(input));
    }

    let mut decoded = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }
        decoded.push_str(&input[start..i]);
        let next = *bytes.get(i + 1)?;
        let ch = match next {
            b'\\' => '\\',
            b',' => ',',
            b'}' => '}',
            b']' => ']',
            b'{' => '{',
            b'[' => '[',
            b'n' => '\n',
            b'r' => '\r',
            b'.' => '.',
            b':' => ':',
            b'"' => '"',
            b'\'' => '\'',
            b'`' => '`',
            b'u' => {
                let hex = std::str::from_utf8(bytes.get(i + 2..i + 6)?).ok()?;
                let high = u16::from_str_radix(hex, 16).ok()?;
                if (0xD800..=0xDBFF).contains(&high) {
                    if bytes.get(i + 6..i + 8)? != b"\\u" {
                        return None;
                    }
                    let low_hex = std::str::from_utf8(bytes.get(i + 8..i + 12)?).ok()?;
                    let low = u16::from_str_radix(low_hex, 16).ok()?;
                    if !(0xDC00..=0xDFFF).contains(&low) {
                        return None;
                    }
                    i += 6;
                    char::from_u32(0x10000 + ((high as u32 - 0xD800) << 10) + low as u32 - 0xDC00)?
                } else {
                    char::from_u32(high as u32)?
                }
            }
            _ => return None,
        };
        decoded.push(ch);
        i += if next == b'u' { 6 } else { 2 };
        start = i;
    }
    decoded.push_str(&input[start..]);
    Some(Cow::Owned(decoded))
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
fn collect_key_hits(
    text: &str,
    root_is_array: bool,
) -> (Vec<KeyHit<'_>>, Vec<InlineArray>, Option<usize>) {
    let mut hits: Vec<KeyHit<'_>> = Vec::new();
    let mut arrays: Vec<InlineArray> = Vec::new();
    let mut root_array = None;
    let mut multi = Multi::None;
    let mut virtual_depth: u32 = 0;
    // Per physical compound (object/array/multi-line) on the stack: how
    // many virtual depths we pushed for it. A pair-with-opener `a.b: {`
    // pushes `segment_count` (2 for `a.b`); a lone opener pushes 1.
    let mut compound_pushes: Vec<(u32, Option<usize>)> = Vec::new();
    let mut first_content_line = true;

    for (i, line) in crate::lines::content_lines(text).into_iter().enumerate() {
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
                if let Some((n, _)) = compound_pushes.pop() {
                    virtual_depth = virtual_depth.saturating_sub(n);
                }
            }
            continue;
        }

        let trimmed = line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with("##") {
            continue;
        }

        let is_first = first_content_line;
        first_content_line = false;
        let bom_offset = if i == 0 {
            crate::lines::leading_bom_len(text) as u32
        } else {
            0
        };
        let leading_ws = bom_offset + (line.len() - trimmed.len()) as u32;
        let tail = trimmed.trim_end();

        // Lone closer line drops one physical compound off the stack.
        if tail == "}" || tail == "]" {
            if let Some((n, _)) = compound_pushes.pop() {
                virtual_depth = virtual_depth.saturating_sub(n);
            }
            continue;
        }

        let line_no = i as u32;
        let line_len = bom_offset + line.len() as u32;
        let block_item = compound_pushes
            .last()
            .and_then(|(_, array_id)| *array_id)
            .map(|array_id| {
                let item = arrays[array_id].items.len();
                arrays[array_id].items.push(InlineItem {
                    range: inline_range(line_no, 0, line_len),
                    array_id: None,
                });
                (array_id, item)
            });

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
                let array_id = if tail == "[" {
                    let id = arrays.len();
                    arrays.push(InlineArray {
                        line: line_no,
                        items: Vec::new(),
                    });
                    if let Some((parent, item)) = block_item {
                        arrays[parent].items[item].array_id = Some(id);
                    }
                    Some(id)
                } else {
                    None
                };
                compound_pushes.push((1, array_id));
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
            let id = collect_inline_hits(
                trimmed,
                leading_ws,
                base_depth,
                &mut hits,
                &mut arrays,
                line_no,
            );
            if trimmed.starts_with('[') {
                if let Some((parent, item)) = block_item {
                    arrays[parent].items[item].array_id = id;
                }
            }
            if is_root_wrapper && root_is_array {
                root_array = id;
            }
            continue;
        }

        // Pair-shaped text in an Array is still a scalar item.
        let in_object = match compound_pushes.last() {
            Some((_, array_id)) => array_id.is_none(),
            None => !root_is_array,
        };
        if !in_object {
            continue;
        }

        // Pair lines: anything else. Spec 0.6.0: the
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
        let mut seg_count: u32 = 0;
        let mut dotted_prefix = Vec::new();
        for (_, seg) in split_dotted(0, key_part) {
            if seg.is_empty() {
                continue;
            }
            hits.push(KeyHit {
                depth: virtual_depth + seg_count,
                name: seg,
                dotted_prefix: dotted_prefix.clone(),
                resets_prefix: false,
                line: line_no,
                col_start: bom_offset,
                col_end: line_len,
                array_id: None,
            });
            if let Some(decoded) = decode_symbol_key(seg) {
                dotted_prefix.push(decoded.into_owned());
            }
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
                let key_hit = hits.len() - 1;
                let id = collect_inline_hits(
                    value,
                    value_col,
                    virtual_depth + seg_count,
                    &mut hits,
                    &mut arrays,
                    line_no,
                );
                if value.starts_with('[') {
                    hits[key_hit].array_id = id;
                } else {
                    hits[key_hit].resets_prefix = true;
                }
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

        if opens_compound && !is_raw_marker {
            if tail.ends_with(" {") {
                hits.last_mut().expect("pair has a key hit").resets_prefix = true;
            }
            let array_id = if tail.ends_with(" [") {
                let id = arrays.len();
                arrays.push(InlineArray {
                    line: line_no,
                    items: Vec::new(),
                });
                hits.last_mut().expect("pair has a key hit").array_id = Some(id);
                Some(id)
            } else {
                None
            };
            compound_pushes.push((seg_count, array_id));
            virtual_depth += seg_count;
            if let Some(m) = pushed_multi {
                multi = m;
            }
        }
        // Otherwise it's a scalar pair: hits are recorded, but no
        // compound push happens.
    }

    (hits, arrays, root_array)
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
    arrays: &mut Vec<InlineArray>,
    line: u32,
) -> Option<usize> {
    let mut depth = base_depth;
    let mut frame_bumps: Vec<u32> = Vec::new();
    let mut pending_bump: Option<u32> = None;
    let mut pending_key: Option<usize> = None;
    let mut frames: Vec<(Option<usize>, Option<(usize, usize)>)> = Vec::new();
    let mut root_array = None;
    let mut first_open_seen = false;

    walk_inline(text, |ev| match ev {
        InlineEvent::Open { pos } => {
            let parent_item = frames.last().and_then(|frame| frame.0).map(|parent| {
                let item = arrays[parent].items.len();
                arrays[parent].items.push(InlineItem {
                    range: inline_range(line, col_base + pos as u32, col_base + pos as u32 + 1),
                    array_id: None,
                });
                (parent, item)
            });
            let array_id = if text.as_bytes()[pos] == b'[' {
                let id = arrays.len();
                arrays.push(InlineArray {
                    line,
                    items: Vec::new(),
                });
                if !first_open_seen {
                    root_array = Some(id);
                }
                if let Some((parent, item)) = parent_item {
                    arrays[parent].items[item].array_id = Some(id);
                }
                if let Some(key) = pending_key {
                    hits[key].array_id = Some(id);
                }
                Some(id)
            } else {
                if let Some(key) = pending_key {
                    hits[key].resets_prefix = true;
                }
                None
            };
            frames.push((array_id, parent_item));
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
            pending_key = None;
        }
        InlineEvent::Close { pos } => {
            if let Some((_, Some((parent, item)))) = frames.pop() {
                arrays[parent].items[item].range.end.character = col_base + pos as u32 + 1;
            }
            if let Some(b) = frame_bumps.pop() {
                depth = depth.saturating_sub(b);
            }
            pending_bump = None;
            pending_key = None;
        }
        InlineEvent::Key {
            start,
            text: key_text,
        } => {
            let mut n = 0u32;
            let mut dotted_prefix = Vec::new();
            for (seg_col, seg) in split_dotted(col_base + start as u32, key_text) {
                if seg.is_empty() {
                    continue;
                }
                hits.push(KeyHit {
                    depth: depth + n,
                    name: seg,
                    dotted_prefix: dotted_prefix.clone(),
                    resets_prefix: false,
                    line,
                    col_start: seg_col,
                    col_end: seg_col + seg.len() as u32,
                    array_id: None,
                });
                if let Some(decoded) = decode_symbol_key(seg) {
                    dotted_prefix.push(decoded.into_owned());
                }
                n += 1;
            }
            pending_bump = if n > 0 { Some(n) } else { None };
            pending_key = if n > 0 { Some(hits.len() - 1) } else { None };
        }
        InlineEvent::Value { start, text, .. } => {
            if let Some(Some(array_id)) = frames.last().map(|frame| frame.0) {
                arrays[array_id].items.push(InlineItem {
                    range: inline_range(
                        line,
                        col_base + start as u32,
                        col_base + (start + text.len()) as u32,
                    ),
                    array_id: None,
                });
            }
            pending_bump = None;
            pending_key = None;
        }
        InlineEvent::Comma { .. } => {
            pending_key = None;
        }
        InlineEvent::Sep { .. } => {}
    });
    root_array
}

fn inline_range(line: u32, start: u32, end: u32) -> Range {
    Range {
        start: Position {
            line,
            character: start,
        },
        end: Position {
            line,
            character: end,
        },
    }
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

    fn assert_range(sym: &DocumentSymbol, line: u32, start: u32, end: u32) {
        let expected = Range {
            start: Position {
                line,
                character: start,
            },
            end: Position {
                line,
                character: end,
            },
        };
        assert_eq!(sym.range, expected, "range for {}", sym.name);
        assert_eq!(
            sym.selection_range, expected,
            "selectionRange for {}",
            sym.name
        );
    }

    #[test]
    fn spec_reopened_dotted_key_keeps_sibling_and_child_positions() {
        let text = include_str!(
            "../../../spec/versions/0.8/tests/valid/dotted_keys/reopen_after_sibling.ktav"
        );
        let syms = build_symbols(&parse(text), text);
        assert_eq!(
            syms.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["a", "c"]
        );
        let a = find(&syms, "a");
        assert_range(a, 0, 0, 6);
        let children = a.children.as_ref().unwrap();
        assert_range(find(children, "b"), 0, 0, 6);
        assert_range(find(children, "d"), 2, 0, 6);
        assert_range(find(&syms, "c"), 1, 0, 4);
    }

    #[test]
    fn reopened_dotted_key_with_inline_sibling_keeps_each_scope() {
        let text = "a.b: 1\nc: {b: 2}\na.d: 3\nnext: 4\n";
        let syms = build_symbols(&parse(text), text);
        let a = find(&syms, "a");
        assert_range(find(a.children.as_ref().unwrap(), "b"), 0, 0, 6);
        assert_range(find(a.children.as_ref().unwrap(), "d"), 2, 0, 6);
        let c = find(&syms, "c");
        assert_range(c, 1, 0, 9);
        assert_range(find(c.children.as_ref().unwrap(), "b"), 1, 4, 5);
        assert_range(find(&syms, "next"), 3, 0, 7);
    }

    #[test]
    fn reopened_dotted_parents_do_not_exchange_same_named_children() {
        let text = "a.x: 1\nb.x: 2\nb.y: 3\na.y: 4\n";
        let syms = build_symbols(&parse(text), text);
        let a = find(&syms, "a").children.as_ref().unwrap();
        let b = find(&syms, "b").children.as_ref().unwrap();
        assert_range(find(a, "x"), 0, 0, 6);
        assert_range(find(a, "y"), 3, 0, 6);
        assert_range(find(b, "x"), 1, 0, 6);
        assert_range(find(b, "y"), 2, 0, 6);
    }

    #[test]
    fn inline_reopened_dotted_parents_keep_same_named_children() {
        let text = "cfg: {a.x: 1, b.x: 2, b.y: 3, a.y: 4}\n";
        let syms = build_symbols(&parse(text), text);
        let cfg = find(&syms, "cfg").children.as_ref().unwrap();
        let a = find(cfg, "a").children.as_ref().unwrap();
        let b = find(cfg, "b").children.as_ref().unwrap();
        assert_range(find(a, "x"), 0, 8, 9);
        assert_range(find(a, "y"), 0, 32, 33);
        assert_range(find(b, "x"), 0, 16, 17);
        assert_range(find(b, "y"), 0, 24, 25);
    }

    #[test]
    fn spec_block_array_scalars_have_item_positions() {
        let text = include_str!("../../../spec/versions/0.8/tests/valid/arrays/scalars.ktav");
        let syms = build_symbols(&parse(text), text);
        let tags = find(&syms, "tags");
        assert_range(tags, 0, 0, 7);
        let items = tags.children.as_ref().unwrap();
        assert_eq!(items.len(), 3);
        for (item, (line, end)) in items.iter().zip([(1, 11), (2, 6), (3, 8)]) {
            assert_range(item, line, 0, end);
        }
    }

    #[test]
    fn block_array_skips_comments_and_links_nested_array_items() {
        let text = "tags: [\n    ## note\n\n    one\n    [two, three]\n    [\n        four\n    ]\n    five\n]\nafter: 1\n";
        let syms = build_symbols(&parse(text), text);
        let items = find(&syms, "tags").children.as_ref().unwrap();
        assert_eq!(items.len(), 4);
        assert_range(&items[0], 3, 0, 7);
        assert_range(&items[1], 4, 0, 16);
        let inline_items = items[1].children.as_ref().unwrap();
        assert_range(&inline_items[0], 4, 5, 8);
        assert_range(&inline_items[1], 4, 10, 15);
        assert_range(&items[2], 5, 0, 5);
        assert_range(&items[2].children.as_ref().unwrap()[0], 6, 0, 12);
        assert_range(&items[3], 8, 0, 8);
        assert_range(find(&syms, "after"), 10, 0, 8);
    }

    #[test]
    fn block_array_closer_with_trailing_space_is_not_an_item() {
        let text = "tags: [\n    one\n]   \nafter: 1\n";
        let syms = build_symbols(&parse(text), text);
        let items = find(&syms, "tags").children.as_ref().unwrap();
        assert_eq!(items.len(), 1);
        assert_range(&items[0], 1, 0, 7);
        assert_range(find(&syms, "after"), 3, 0, 8);
    }

    #[test]
    fn quoted_key_keeps_its_range_and_does_not_consume_sibling() {
        let text = "\"a\": 1\nb: 2\n";
        let syms = build_symbols(&parse(text), text);
        assert_range(find(&syms, "a"), 0, 0, 6);
        assert_range(find(&syms, "b"), 1, 0, 4);
    }

    #[test]
    fn escaped_and_quoted_dotted_segments_keep_source_spans() {
        let text = "a\\.b: 1\nroot.\"x\\u0041\": 2\ntail: 3\n";
        let syms = build_symbols(&parse(text), text);
        assert_range(find(&syms, "a.b"), 0, 0, 7);
        let root = find(&syms, "root");
        assert_range(root, 1, 0, 17);
        assert_range(find(root.children.as_ref().unwrap(), "xA"), 1, 0, 17);
        assert_range(find(&syms, "tail"), 2, 0, 7);
    }

    #[test]
    fn spaced_quoted_dotted_segments_match_decoded_keys() {
        let text = "root . \"a\\\"b\" : 1\nafter: 2\n";
        let syms = build_symbols(&parse(text), text);
        let root = find(&syms, "root");
        assert_range(root, 0, 0, 17);
        assert_range(find(root.children.as_ref().unwrap(), "a\"b"), 0, 0, 17);
        assert_range(find(&syms, "after"), 1, 0, 8);
    }

    #[test]
    fn spaced_quoted_dotted_segments_keep_colon_and_dot_in_names() {
        let text = "root . \"a:b\": 1\nroot . \"a.b\": 2\n";
        let syms = build_symbols(&parse(text), text);
        let root = find(&syms, "root");
        let kids = root.children.as_ref().unwrap();
        assert_range(find(kids, "a:b"), 0, 0, 15);
        assert_range(find(kids, "a.b"), 1, 0, 15);
    }

    #[test]
    fn quoted_inline_dotted_segment_keeps_sibling_ranges() {
        let text = "cfg: {a.\"b,c\": 1, tail: 2}\n";
        let syms = build_symbols(&parse(text), text);
        let kids = find(&syms, "cfg").children.as_ref().unwrap();
        let a = find(kids, "a");
        assert_range(a, 0, 6, 7);
        assert_range(find(a.children.as_ref().unwrap(), "b,c"), 0, 8, 13);
        assert_range(find(kids, "tail"), 0, 18, 22);

        let text = "cfg: {a. \"b,c\": 1, tail: 2}\n";
        let syms = build_symbols(&parse(text), text);
        let kids = find(&syms, "cfg").children.as_ref().unwrap();
        let a = find(kids, "a");
        assert_range(find(a.children.as_ref().unwrap(), "b,c"), 0, 8, 14);
        assert_range(find(kids, "tail"), 0, 19, 23);
    }

    #[test]
    fn unicode_surrogate_escape_key_matches_value_name() {
        let text = "\\uD83D\\uDE00: 1\nnext: 2\n";
        let syms = build_symbols(&parse(text), text);
        assert_range(find(&syms, "\u{1f600}"), 0, 0, 15);
        assert_range(find(&syms, "next"), 1, 0, 7);
    }

    #[test]
    fn inline_quoted_and_escaped_keys_keep_raw_byte_columns() {
        let text = "cfg: {\"a\": 1, b\\.c: 2, \"u\\u0041\": 3, tail: 4}\nnext: 5\n";
        let syms = build_symbols(&parse(text), text);
        let kids = find(&syms, "cfg").children.as_ref().unwrap();
        assert_range(find(kids, "a"), 0, 6, 9);
        assert_range(find(kids, "b.c"), 0, 14, 18);
        assert_range(find(kids, "uA"), 0, 23, 32);
        assert_range(find(kids, "tail"), 0, 37, 41);
        assert_range(find(&syms, "next"), 1, 0, 7);
    }

    #[test]
    fn hash_keys_and_array_items_are_not_comments() {
        let object = "#child: 1\n## comment\n#value: 2\nend: 3\n";
        let syms = build_symbols(&parse(object), object);
        assert_range(find(&syms, "#child"), 0, 0, 9);
        assert_range(find(&syms, "#value"), 2, 0, 9);
        assert_range(find(&syms, "end"), 3, 0, 6);

        let array = "#value\n## comment\nother\n";
        let syms = build_symbols(&parse(array), array);
        assert_range(&syms[0], 0, 0, 6);
        assert_range(&syms[1], 2, 0, 5);
    }

    #[test]
    fn bom_quoted_hash_and_inline_keys_keep_original_columns() {
        let text = "\u{feff}\"#child\": {\"a\\u0041\": 1, #value: 2}\nnext: 3\n";
        let syms = build_symbols(&parse(text), text);
        let child = find(&syms, "#child");
        assert_range(child, 0, 3, 38);
        let kids = child.children.as_ref().unwrap();
        assert_range(find(kids, "aA"), 0, 14, 23);
        assert_range(find(kids, "#value"), 0, 28, 34);
        assert_range(find(&syms, "next"), 1, 0, 7);
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
    fn top_level_multiline_array_items_have_exact_ranges() {
        let text = "[\n    foo\n    bar\n    baz\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 3);
        for (sym, line) in syms.iter().zip(1..=3) {
            assert_range(sym, line, 0, 7);
        }
    }

    #[test]
    fn pair_shaped_array_scalars_do_not_open_compounds() {
        for scalar in ["foo: {", "foo: [", "foo: (", "foo: ((", "foo:: {", "foo {"] {
            let text = format!(":: start\n{scalar}\nbar\n");
            let syms = build_symbols(&parse(&text), &text);
            assert_eq!(syms.len(), 3, "{scalar}");
            assert_range(&syms[0], 0, 0, 8);
            assert_range(&syms[1], 1, 0, scalar.len() as u32);
            assert_range(&syms[2], 2, 0, 3);
        }
    }

    #[test]
    fn spec_pair_shaped_top_level_array_items_keep_ranges() {
        let text = include_str!(
            "../../../spec/versions/0.8/tests/valid/top_level_array/pair_shaped_first_item.ktav"
        );
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 4);
        for (sym, (line, end)) in syms.iter().zip([(0, 18), (1, 2), (2, 10), (3, 10)]) {
            assert_range(sym, line, 0, end);
        }
    }

    #[test]
    fn nested_array_scalar_opener_does_not_hide_following_top_level_item() {
        let text = "[\n[\nfoo: {\nbar\n]\nafter\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 1, 0, 1);
        let nested = syms[0].children.as_ref().unwrap();
        assert_range(&nested[0], 2, 0, 6);
        assert_range(&nested[1], 3, 0, 3);
        assert_range(&syms[1], 5, 0, 5);
    }

    #[test]
    fn scalar_opener_in_root_array_does_not_hide_later_object_key() {
        let text = ":: start\nfoo: {\nbar\n{\nchild: 1\n}\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 4);
        assert_range(&syms[3], 3, 0, 1);
        assert_range(find(syms[3].children.as_ref().unwrap(), "child"), 4, 0, 8);
    }

    #[test]
    fn raw_pair_opener_inside_object_remains_scalar() {
        let text = "[\n{\nraw:: {\nchild: 1\n}\nafter\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        let object = syms[0].children.as_ref().unwrap();
        assert_range(find(object, "raw"), 2, 0, 7);
        assert_range(find(object, "child"), 3, 0, 8);
        assert_range(&syms[1], 5, 0, 5);
    }

    #[test]
    fn object_pair_opener_in_top_level_array_keeps_following_item() {
        let text = "[\n{\nchild: {\nleaf: one\n}\n}\nafter\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 1, 0, 1);
        assert_range(&syms[1], 6, 0, 5);
    }

    #[test]
    fn top_level_array_wrapper_skips_comments_and_preserves_bom_cr_positions() {
        let text = "\u{feff}## lead\r[\r    #value\r    ## comment\r    bar\r]\r";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 2, 0, 10);
        assert_range(&syms[1], 4, 0, 7);
    }

    #[test]
    fn top_level_array_nested_compound_does_not_shift_following_item() {
        let text = "[\n    first\n    [\n        child\n    ]\n    last\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 3);
        assert_range(&syms[0], 1, 0, 9);
        assert_range(&syms[1], 2, 0, 5);
        assert_range(&syms[2], 5, 0, 8);
    }

    #[test]
    fn multiline_top_level_array_links_inline_array_items() {
        let text = "[\n    [1, 2]\n    [3, 4]\n]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        for (i, line) in [1, 2].into_iter().enumerate() {
            assert_range(&syms[i], line, 0, 10);
            let items = syms[i].children.as_ref().expect("inline array children");
            assert_eq!(items.len(), 2);
            assert_range(&items[0], line, 5, 6);
            assert_range(&items[1], line, 8, 9);
        }
    }

    #[test]
    fn bare_top_level_array_links_inline_array_item() {
        let text = "first\n[1, 2]\n";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 0, 0, 5);
        assert_range(&syms[1], 1, 0, 6);
        let items = syms[1].children.as_ref().expect("inline array children");
        assert_eq!(items.len(), 2);
        assert_range(&items[0], 1, 1, 2);
        assert_range(&items[1], 1, 4, 5);
    }

    #[test]
    fn bare_and_inline_top_level_arrays_keep_item_lines() {
        let bare = "foo\nbar\n";
        let syms = build_symbols(&parse(bare), bare);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 0, 0, 3);
        assert_range(&syms[1], 1, 0, 3);

        let inline = "[foo]\n";
        let syms = build_symbols(&parse(inline), inline);
        assert_eq!(syms.len(), 1);
        assert_range(&syms[0], 0, 1, 4);
    }

    #[test]
    fn spec_top_level_inline_array_has_distinct_item_ranges() {
        let text =
            include_str!("../../../spec/versions/0.8/tests/valid/top_level_inline/array.ktav");
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 5);
        for (i, (start, end)) in [(1, 2), (4, 5), (7, 8), (10, 11), (13, 14)]
            .into_iter()
            .enumerate()
        {
            assert_eq!(syms[i].name, format!("[{i}]"));
            assert_range(&syms[i], 0, start, end);
        }
    }

    #[test]
    fn spec_inline_nested_array_items_have_exact_ranges() {
        let text = include_str!("../../../spec/versions/0.8/tests/valid/inline/nested/mixed.ktav");
        let syms = build_symbols(&parse(text), text);
        let users = find(&syms, "users").children.as_ref().unwrap();
        assert_eq!(users.len(), 2);
        assert_range(&users[0], 0, 8, 30);
        assert_range(&users[1], 0, 32, 52);

        let data = find(&syms, "data").children.as_ref().unwrap();
        let names = find(data, "names").children.as_ref().unwrap();
        let ages = find(data, "ages").children.as_ref().unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(ages.len(), 2);
        assert_range(&names[0], 1, 15, 20);
        assert_range(&names[1], 1, 22, 25);
        assert_range(&ages[0], 1, 35, 37);
        assert_range(&ages[1], 1, 39, 41);
    }

    #[test]
    fn spec_top_level_inline_nested_array_uses_own_item_range() {
        let text = include_str!("../../../spec/versions/0.8/tests/valid/top_level_inline/unquoted_key_context_from_active_scope.ktav");
        let syms = build_symbols(&parse(text), text);
        assert_range(&syms[0], 0, 1, 15);
        let object = syms[0].children.as_ref().unwrap();
        let b_items = find(object, "b").children.as_ref().unwrap();
        assert_range(&b_items[0], 0, 12, 13);
    }

    #[test]
    fn inline_array_ranges_keep_bom_hash_quoted_key_and_cr_columns() {
        let text = "\u{feff}[#value, {\"#key\": [1, 2]}]\r";
        let syms = build_symbols(&parse(text), text);
        assert_eq!(syms.len(), 2);
        assert_range(&syms[0], 0, 4, 10);
        assert_range(&syms[1], 0, 12, 28);
        let key = find(syms[1].children.as_ref().unwrap(), "#key");
        assert_range(key, 0, 13, 19);
        let items = key.children.as_ref().unwrap();
        assert_range(&items[0], 0, 22, 23);
        assert_range(&items[1], 0, 25, 26);
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
