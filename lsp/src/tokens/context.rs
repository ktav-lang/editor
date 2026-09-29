//! Stateful spec 5.0.1 / 5.1 dispatch for document-level consumers.

use super::classify::{classify_array_line, classify_line, is_ktav_ws};
use super::{find_key_separator, LineKind, ValueKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Object,
    Array,
}

#[derive(Default)]
pub(crate) struct DocumentContext {
    root: Option<Scope>,
    scopes: Vec<Scope>,
    terminator: Option<&'static str>,
}

pub(crate) struct ContextLine<'a> {
    pub kind: LineKind<'a>,
    pub depth: usize,
    pub multiline_content: bool,
}

impl DocumentContext {
    pub fn in_multiline(&self) -> bool {
        self.terminator.is_some()
    }

    /// Positions are byte offsets into a BOM-free, unterminated line.
    pub fn next_line<'a>(&mut self, raw: &'a str) -> ContextLine<'a> {
        let trimmed = raw.trim_matches(is_ktav_ws);
        let start = raw.len() - raw.trim_start_matches(is_ktav_ws).len();
        let mut depth = self.scopes.len();

        if let Some(terminator) = self.terminator {
            let closed = trimmed == terminator;
            if closed {
                self.terminator = None;
            }
            return ContextLine {
                kind: if trimmed.is_empty() {
                    LineKind::Blank
                } else {
                    LineKind::ArrayItem {
                        start: start as u32,
                        length: trimmed.len() as u32,
                        kind: if closed {
                            ValueKind::CompoundClose
                        } else {
                            ValueKind::String
                        },
                    }
                },
                depth,
                multiline_content: !closed,
            };
        }

        // Blank/comment lines do not decide the root kind.
        if trimmed.is_empty() || trimmed.starts_with("##") {
            return ContextLine {
                kind: classify_line(raw),
                depth,
                multiline_content: false,
            };
        }
        if self.root.is_none() {
            self.root = Some(if trimmed.starts_with('{') || is_pair_candidate(trimmed) {
                Scope::Object
            } else {
                Scope::Array
            });
        }
        let scope = self.scopes.last().copied().or(self.root);
        let kind = if scope == Some(Scope::Array) {
            classify_array_line(raw)
        } else {
            classify_line(raw)
        };

        // Only a matching container closer changes the stack on incomplete input.
        let closing = match trimmed {
            "}" => Some(Scope::Object),
            "]" => Some(Scope::Array),
            _ => None,
        };
        if closing.is_some() && closing == self.scopes.last().copied() {
            self.scopes.pop();
            depth = self.scopes.len();
        }

        let opener = match &kind {
            LineKind::Pair {
                value_kind: ValueKind::CompoundOpen,
                value_text,
                ..
            } => Some(*value_text),
            LineKind::ArrayItem {
                kind: ValueKind::CompoundOpen,
                start,
                length,
            } => Some(&raw[*start as usize..*start as usize + *length as usize]),
            _ => None,
        };
        match opener {
            Some("{") => self.scopes.push(Scope::Object),
            Some("[") => self.scopes.push(Scope::Array),
            Some("(") => self.terminator = Some(")"),
            Some("((") => self.terminator = Some("))"),
            _ => {}
        }
        ContextLine {
            kind,
            depth,
            multiline_content: false,
        }
    }
}

/// Spec 5.0.1 rule 6 is lexical, not key validation; leading brackets win.
fn is_pair_candidate(trimmed: &str) -> bool {
    if matches!(trimmed.as_bytes().first(), Some(b'{') | Some(b'[')) {
        return false;
    }
    let Some(colon) = find_key_separator(trimmed) else {
        return false;
    };
    if trimmed[..colon].trim_end_matches(is_ktav_ws).is_empty() {
        return false;
    }
    let after = &trimmed[colon + 1..];
    after.is_empty() || after.starts_with(':') || after.starts_with(is_ktav_ws)
}
