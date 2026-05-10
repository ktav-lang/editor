//! Run the `ktav` parser over a document and turn any [`ktav::Error`]
//! into LSP [`Diagnostic`]s.
//!
//! Since `ktav 0.1.5` (and especially `0.1.6`, which adds byte-offset
//! [`ktav::Span`]s on every variant), the parser emits
//! [`ktav::Error::Structured`] with a tight source span we can map
//! directly to an LSP [`Range`]. The legacy `Error::Syntax(String)`
//! path is kept as a defence-in-depth fallback (no live code path under
//! `0.1.6+`, but the `#[non_exhaustive]` `Error` enum lets future
//! callers / wrappers still produce it, and we used to depend on it
//! exclusively).
//!
//! Encoding contract: `Diagnostic.range.character` is emitted in BYTES
//! here; if the negotiated [`crate::server::PositionEncoding`] is UTF-16,
//! [`crate::server::convert_diagnostics_to_utf16`] re-encodes the columns
//! after this function returns.

use std::sync::OnceLock;

use ktav::{Error, ErrorKind, Span};
use regex::Regex;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::tokens::{classify_line, LineKind};

/// Parse `text` and return diagnostics. Empty vec on success.
pub fn parse_for_diagnostics(text: &str) -> Vec<Diagnostic> {
    match ktav::parse(text) {
        Ok(_) => Vec::new(),
        Err(Error::Structured(kind)) => vec![structured_to_diagnostic(text, &kind)],
        Err(Error::Syntax(msg)) => vec![syntax_to_diagnostic_legacy(text, &msg)],
        Err(_) => vec![generic_io_diagnostic()],
    }
}

/// Build a [`Diagnostic`] from a structured [`ErrorKind`]. The message
/// text is `kind.to_string()` (byte-identical to what `Error::Syntax`
/// used to produce per the Display contract); only the `range` is now
/// derived from the structured `span` instead of regex-extracted line
/// + classifier-derived columns.
fn structured_to_diagnostic(text: &str, kind: &ErrorKind) -> Diagnostic {
    let range = range_from_span(text, kind.span(), kind.line());
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ktav".to_string()),
        message: kind.to_string(),
        ..Default::default()
    }
}

/// Convert a [`Span`] into an LSP [`Range`]. Falls back to a full-line
/// or last-non-blank-line range when the span is empty (`Span::EMPTY`),
/// which currently happens for some `UnclosedCompound` and `Other`
/// internal-state failures where no source range is meaningful.
fn range_from_span(text: &str, span: Span, fallback_line: Option<u32>) -> Range {
    if span.start == 0 && span.end == 0 {
        // Empty span — fall back. Prefer the kind's own line if known,
        // otherwise the last non-blank line (matches the legacy
        // "Unclosed { at end of input" behaviour).
        let line = fallback_line
            .map(|n| n.saturating_sub(1))
            .unwrap_or_else(|| last_non_blank_line(text));
        return full_line_range(text, line);
    }

    let (start_line, start_col) = span.line_col(text);
    let end_span = Span::new(span.end, span.end);
    let (end_line, end_col) = end_span.line_col(text);

    Range {
        start: Position {
            line: start_line.saturating_sub(1),
            character: start_col,
        },
        end: Position {
            line: end_line.saturating_sub(1),
            character: end_col,
        },
    }
}

/// Legacy fallback for `Error::Syntax(String)`. Kept as defence-in-depth
/// against downstream wrappers that may still construct it; the parser
/// itself no longer does under `0.1.5+`.
fn syntax_to_diagnostic_legacy(text: &str, msg: &str) -> Diagnostic {
    let range = compute_range_legacy(text, msg);
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ktav".to_string()),
        message: msg.to_string(),
        ..Default::default()
    }
}

/// Generic catch-all for `Error::Io` / `Error::Message` and any future
/// `#[non_exhaustive]` variants.
fn generic_io_diagnostic() -> Diagnostic {
    Diagnostic {
        range: Range::default(),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ktav".to_string()),
        message: "ktav: parse failed".to_string(),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Legacy regex-based range computation. Pre-0.1.5 path. NOT exercised under
// the path-dep ktav 0.1.7, but kept as a regression net for any future
// `Error::Syntax(_)` that escapes through downstream wrappers.
// ---------------------------------------------------------------------------

fn compute_range_legacy(text: &str, msg: &str) -> Range {
    let line = match extract_line_number(msg) {
        Some(n) => n.saturating_sub(1),
        None => {
            if msg.contains("Unclosed") {
                last_non_blank_line(text)
            } else {
                0
            }
        }
    };

    let line_text = nth_line(text, line as usize).unwrap_or("");

    if msg.contains("MissingSeparatorSpace") {
        if let Some(r) = range_for_missing_separator(line, line_text) {
            return r;
        }
    } else if msg.contains("InvalidTypedScalar") {
        if let Some(r) = range_for_value_span(line, line_text) {
            return r;
        }
    } else if msg.contains("duplicate key")
        || msg.contains("Duplicate key")
        || msg.contains("conflicts with")
    {
        if let Some(key) = extract_quoted_key(msg) {
            if let Some(r) = range_for_key_segment(line, line_text, &key) {
                return r;
            }
        }
        if let Some(r) = range_for_key(line, line_text) {
            return r;
        }
    } else if msg.contains("Empty key at line") {
        if let Some(r) = range_for_leading_to_colon(line, line_text) {
            return r;
        }
    } else if msg.contains("Invalid key at line") {
        if let Some(key) = extract_quoted_key(msg) {
            if let Some(r) = range_for_key_segment(line, line_text, &key) {
                return r;
            }
        }
        if let Some(r) = range_for_key(line, line_text) {
            return r;
        }
    }

    full_line_range(text, line)
}

fn range_for_missing_separator(line: u32, line_text: &str) -> Option<Range> {
    let LineKind::Pair {
        marker_start,
        marker,
        ..
    } = classify_line(line_text)
    else {
        return None;
    };
    let mlen = marker.len() as u32;
    let after = (marker_start + mlen) as usize;
    let extra = line_text[after..]
        .chars()
        .take_while(|c| !c.is_whitespace())
        .map(|c| c.len_utf8())
        .sum::<usize>() as u32;
    Some(Range {
        start: Position {
            line,
            character: marker_start,
        },
        end: Position {
            line,
            character: marker_start + mlen + extra,
        },
    })
}

fn range_for_value_span(line: u32, line_text: &str) -> Option<Range> {
    if let LineKind::Pair {
        value_start,
        value_length,
        ..
    } = classify_line(line_text)
    {
        if value_length > 0 {
            return Some(Range {
                start: Position {
                    line,
                    character: value_start,
                },
                end: Position {
                    line,
                    character: value_start + value_length,
                },
            });
        }
    }
    None
}

fn range_for_key(line: u32, line_text: &str) -> Option<Range> {
    if let LineKind::Pair {
        key_start,
        key_length,
        ..
    } = classify_line(line_text)
    {
        return Some(Range {
            start: Position {
                line,
                character: key_start,
            },
            end: Position {
                line,
                character: key_start + key_length,
            },
        });
    }
    None
}

fn range_for_key_segment(line: u32, line_text: &str, key: &str) -> Option<Range> {
    if let LineKind::Pair {
        key_start,
        key_length,
        ..
    } = classify_line(line_text)
    {
        let lo = key_start as usize;
        let hi = lo + key_length as usize;
        let key_text = line_text.get(lo..hi)?;
        if let Some(rel) = find_segment(key_text, key) {
            let abs = key_start + rel as u32;
            return Some(Range {
                start: Position {
                    line,
                    character: abs,
                },
                end: Position {
                    line,
                    character: abs + key.len() as u32,
                },
            });
        }
        return Some(Range {
            start: Position {
                line,
                character: key_start,
            },
            end: Position {
                line,
                character: key_start + key_length,
            },
        });
    }
    None
}

fn find_segment(dotted: &str, seg: &str) -> Option<usize> {
    let mut col = 0usize;
    for s in dotted.split('.') {
        if s == seg {
            return Some(col);
        }
        col += s.len() + 1;
    }
    None
}

fn range_for_leading_to_colon(line: u32, line_text: &str) -> Option<Range> {
    let leading_ws = line_text.len() - line_text.trim_start().len();
    let colon = line_text.find(':')?;
    Some(Range {
        start: Position {
            line,
            character: leading_ws as u32,
        },
        end: Position {
            line,
            character: colon as u32 + 1,
        },
    })
}

fn extract_line_number(msg: &str) -> Option<u32> {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    let res = RE.get_or_init(|| {
        vec![
            Regex::new(r"^Line (\d+):").unwrap(),
            Regex::new(r"at line (\d+)").unwrap(),
        ]
    });
    for re in res {
        if let Some(c) = re.captures(msg) {
            if let Some(m) = c.get(1) {
                if let Ok(n) = m.as_str().parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn extract_quoted_key(msg: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"'([^']+)'").unwrap());
    re.captures(msg)?.get(1).map(|m| m.as_str().to_string())
}

fn nth_line(text: &str, idx: usize) -> Option<&str> {
    text.split('\n').nth(idx)
}

fn last_non_blank_line(text: &str) -> u32 {
    let lines: Vec<&str> = text.split('\n').collect();
    for (i, l) in lines.iter().enumerate().rev() {
        if !l.trim().is_empty() {
            return i as u32;
        }
    }
    0
}

fn full_line_range(text: &str, line: u32) -> Range {
    let lines: Vec<&str> = text.split('\n').collect();
    let idx = line as usize;
    let len = lines.get(idx).map(|s| s.len() as u32).unwrap_or(0);
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: len,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_doc_no_diagnostics() {
        let text = "port: 8080\nhost: example.com\n";
        assert!(parse_for_diagnostics(text).is_empty());
    }

    #[test]
    fn missing_separator_space_tight_range() {
        // First line forces Object root (per spec 0.1.1 a bare
        // `key:value` at the document start is a top-level Array
        // string item, not a malformed pair). The malformed pair on
        // line 2 then yields MissingSeparatorSpace.
        let text = "anchor: 1\nkey:value\n";
        let diags = parse_for_diagnostics(text);
        assert_eq!(diags.len(), 1);
        let r = diags[0].range;
        assert_eq!(r.start.line, 1);
        // Structured span covers the glued body `value` on line 2,
        // bytes 14..19 of the document, which converts to columns
        // 4..9 on the line.
        assert_eq!(r.start.character, 4);
        assert_eq!(r.end.character, 9);
        assert!(diags[0].message.contains("MissingSeparatorSpace"));
    }

    #[test]
    fn duplicate_key_diagnostic_on_line_2() {
        let text = "port: 80\nport: 443\n";
        let diags = parse_for_diagnostics(text);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range.start.line, 1);
        let r = diags[0].range;
        // Tightened to the key segment `port` (cols 0..4).
        assert_eq!(r.start.character, 0);
        assert_eq!(r.end.character, 4);
    }

    #[test]
    fn empty_doc_is_valid() {
        let diags = parse_for_diagnostics("");
        assert!(diags.is_empty());
    }

    #[test]
    fn legacy_line_extraction_regex_shapes() {
        assert_eq!(extract_line_number("Invalid key at line 7: 'x.'"), Some(7));
        assert_eq!(extract_line_number("Empty key at line 3"), Some(3));
        assert_eq!(extract_line_number("Line 12: something"), Some(12));
        assert_eq!(extract_line_number("Unclosed { at end of input"), None);
    }
}
