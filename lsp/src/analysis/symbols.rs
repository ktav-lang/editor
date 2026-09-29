//! Build a symbol tree from a parsed Value without parsing values again.
//!
//! An arena assigns each Value a stable NodeId. A single source pass follows
//! decoded object children and actual array items, attaching spans to those
//! identities. Explicit compounds save/restore their enclosing node, so dotted
//! extensions remain independent of source order and equal-depth siblings.
//! Hash-indexed children keep indexing O(source bytes + Value nodes), without
//! cloning dotted prefixes or rescanning the document. Block spans remain
//! whole-line approximations; inline keys/items retain their own byte spans.

use std::borrow::Cow;
use std::collections::HashMap;

use ktav::Value;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

use crate::analysis::semantic::{walk_inline, InlineEvent};
use crate::tokens::{find_key_separator, is_ktav_ws, split_dotted};

/// Build an outline for the parsed Object or Array root.
/// Columns are bytes; the server converts them to the negotiated encoding.
pub fn build_symbols(value: &Value, text: &str) -> Vec<DocumentSymbol> {
    let mut index = SourceIndex { nodes: Vec::new() };
    let root = index.add_node(value);
    index.scan(root, text);
    index.build_children(root).unwrap_or_default()
}

type NodeId = usize;

struct Node<'a> {
    value: &'a Value,
    children: Children<'a>,
    range: Option<Range>,
}

enum Children<'a> {
    Object(HashMap<&'a str, NodeId>),
    Array(Vec<NodeId>),
    Scalar,
}

enum Frame {
    Object { id: NodeId },
    Array { id: NodeId, next_item: usize },
    Multiline { closer: &'static str },
}

struct SourceIndex<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> SourceIndex<'a> {
    fn add_node(&mut self, value: &'a Value) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node {
            value,
            children: Children::Scalar,
            range: None,
        });
        let children = match value {
            Value::Object(map) => {
                let mut children = HashMap::with_capacity(map.len());
                for (key, child) in map {
                    children.insert(key.as_str(), self.add_node(child));
                }
                Children::Object(children)
            }
            Value::Array(items) => {
                Children::Array(items.iter().map(|child| self.add_node(child)).collect())
            }
            _ => Children::Scalar,
        };
        self.nodes[id].children = children;
        id
    }

    fn frame(&self, id: NodeId) -> Option<Frame> {
        match self.nodes[id].children {
            Children::Object(_) => Some(Frame::Object { id }),
            Children::Array(_) => Some(Frame::Array { id, next_item: 0 }),
            Children::Scalar => None,
        }
    }

    fn record_key(
        &mut self,
        mut object: NodeId,
        raw: &str,
        line: u32,
        col: u32,
        whole_line: Option<Range>,
    ) -> Option<NodeId> {
        for (start, segment) in split_dotted(col, raw) {
            let name = decode_symbol_key(segment)?;
            let Children::Object(children) = &self.nodes[object].children else {
                return None;
            };
            object = *children.get(name.as_ref())?;
            let range = whole_line
                .unwrap_or_else(|| inline_range(line, start, start + segment.len() as u32));
            // A reopened dotted prefix keeps its first source occurrence.
            self.nodes[object].range.get_or_insert(range);
        }
        Some(object)
    }

    fn next_item(&self, frame: &mut Frame) -> Option<NodeId> {
        let Frame::Array { id, next_item } = frame else {
            return None;
        };
        let Children::Array(items) = &self.nodes[*id].children else {
            return None;
        };
        let child = *items.get(*next_item)?;
        *next_item += 1;
        Some(child)
    }

    fn scan(&mut self, root: NodeId, text: &str) {
        let Some(root_frame) = self.frame(root) else {
            return;
        };
        let mut frames = vec![root_frame];
        let mut first_content = true;
        let bom = crate::lines::leading_bom_len(text) as u32;
        for (line_no, raw) in crate::lines::content_lines(text).into_iter().enumerate() {
            let tail = raw.trim_matches(is_ktav_ws);
            if let Some(Frame::Multiline { closer }) = frames.last() {
                if tail == *closer {
                    frames.pop();
                }
                continue;
            }
            if tail.is_empty() || tail.starts_with("##") {
                continue;
            }
            let line = line_no as u32;
            let bom = if line_no == 0 { bom } else { 0 };
            let col = bom + (raw.len() - raw.trim_start_matches(is_ktav_ws).len()) as u32;
            let line_end = bom + raw.len() as u32;
            let is_root_wrapper = first_content
                && matches!(
                    (self.nodes[root].value, tail.as_bytes().first()),
                    (Value::Object(_), Some(b'{')) | (Value::Array(_), Some(b'['))
                );
            first_content = false;
            if is_root_wrapper {
                if !matches!(tail, "{" | "[") {
                    self.scan_inline(root, tail, line, col);
                }
                continue;
            }
            if matches!(tail, "}" | "]") {
                frames.pop();
                continue;
            }
            let Some(frame) = frames.last_mut() else {
                continue;
            };
            match frame {
                Frame::Object { id } => {
                    let Some(colon) = find_key_separator(tail) else {
                        continue;
                    };
                    let key = tail[..colon].trim_end_matches(is_ktav_ws);
                    let Some(value) = self.record_key(
                        *id,
                        key,
                        line,
                        col,
                        Some(inline_range(line, bom, line_end)),
                    ) else {
                        continue;
                    };
                    // Spec 0.8 section 5.3: raw bodies never open a scope.
                    if tail.as_bytes().get(colon + 1) == Some(&b':') {
                        continue;
                    }
                    let body = &tail[colon + 1..];
                    let trimmed = body.trim_start_matches(is_ktav_ws);
                    let value_col = col + (colon + 1 + body.len() - trimmed.len()) as u32;
                    self.scan_value(value, trimmed, line, value_col, &mut frames);
                }
                Frame::Array { .. } => {
                    let Some(value) = self.next_item(frame) else {
                        continue;
                    };
                    self.nodes[value].range = Some(inline_range(line, 0, line_end));
                    if !tail.starts_with("::") {
                        self.scan_value(value, tail, line, col, &mut frames);
                    }
                }
                Frame::Multiline { .. } => unreachable!("multiline content handled above"),
            }
        }
    }

    fn scan_value(&mut self, id: NodeId, body: &str, line: u32, col: u32, frames: &mut Vec<Frame>) {
        match body {
            // Spec 0.8 sections 5.2/5.4: only an exact body opens a block.
            "{" | "[" => {
                if let Some(frame) = self.frame(id) {
                    frames.push(frame);
                }
            }
            "(" => frames.push(Frame::Multiline { closer: ")" }),
            "((" => frames.push(Frame::Multiline { closer: "))" }),
            _ if matches!(body.as_bytes().first(), Some(b'{') | Some(b'[')) => {
                self.scan_inline(id, body, line, col);
            }
            _ => {}
        }
    }

    fn scan_inline(&mut self, root: NodeId, text: &str, line: u32, col: u32) {
        // The optional ID is an inline array item whose end is its closer.
        let mut frames: Vec<(Frame, Option<NodeId>)> = Vec::new();
        let mut pending_value = None;
        let mut first_open = true;
        walk_inline(text, |event| match event {
            InlineEvent::Open { pos } => {
                let (id, array_item) = if first_open {
                    first_open = false;
                    (Some(root), None)
                } else if let Some(id) = pending_value.take() {
                    (Some(id), None)
                } else {
                    let item = frames
                        .last_mut()
                        .and_then(|(frame, _)| self.next_item(frame));
                    (item, item)
                };
                if let Some(id) = id {
                    if let Some(item) = array_item {
                        self.nodes[item].range =
                            Some(inline_range(line, col + pos as u32, col + pos as u32 + 1));
                    }
                    if let Some(frame) = self.frame(id) {
                        frames.push((frame, array_item));
                    }
                }
            }
            InlineEvent::Close { pos } => {
                if let Some((_, Some(item))) = frames.pop() {
                    if let Some(range) = &mut self.nodes[item].range {
                        range.end.character = col + pos as u32 + 1;
                    }
                }
                pending_value = None;
            }
            InlineEvent::Key { start, text } => {
                if let Some((Frame::Object { id }, _)) = frames.last() {
                    pending_value = self.record_key(*id, text, line, col + start as u32, None);
                }
            }
            InlineEvent::Value { start, text, .. } => {
                if pending_value.take().is_none() {
                    if let Some(item) = frames
                        .last_mut()
                        .and_then(|(frame, _)| self.next_item(frame))
                    {
                        self.nodes[item].range = Some(inline_range(
                            line,
                            col + start as u32,
                            col + (start + text.len()) as u32,
                        ));
                    }
                }
            }
            InlineEvent::Comma { .. } => pending_value = None,
            InlineEvent::Sep { .. } => {}
        });
    }

    fn build_children(&self, id: NodeId) -> Option<Vec<DocumentSymbol>> {
        let node = &self.nodes[id];
        match (&node.children, node.value) {
            (Children::Object(children), Value::Object(map)) => Some(
                map.iter()
                    .map(|(key, _)| self.build_symbol(children[key.as_str()], key.to_string()))
                    .collect(),
            ),
            (Children::Array(items), _) => Some(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, &child)| self.build_symbol(child, format!("[{i}]")))
                    .collect(),
            ),
            _ => None,
        }
    }

    fn build_symbol(&self, id: NodeId, name: String) -> DocumentSymbol {
        let node = &self.nodes[id];
        let range = node.range.unwrap_or_else(zero_range);
        #[allow(deprecated)]
        DocumentSymbol {
            name,
            detail: Some(value_kind(node.value).to_string()),
            kind: kind_for(node.value),
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: self.build_children(id),
        }
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
