//! Run the `ktav` parser over a document and turn any [`ktav::Error`]
//! into LSP [`Diagnostic`]s.
//!
//! `ktav 0.1.4` only carries free-form `Error::Syntax(String)` messages.
//! We recover both the line number and (where possible) a tighter
//! column range by re-scanning the offending line through the shared
//! [`crate::tokens`] classifier.
//!
//! Recognised shapes (each produces a tightened range):
//!   - `"Line N: MissingSeparatorSpace: ..."`           → marker + first non-ws char
//!   - `"Line N: InvalidTypedScalar: ..."`              → value span
//!   - `"Line N: Duplicate key '<k>' ..."`              → key segment
//!   - `"Empty key at line N"`                          → leading ws → first ':'
//!   - `"Invalid key at line N: '<k>'"`                 → key segment
//!   - `"Unclosed { at end of input"` (no line)         → last line
//!   - everything else                                  → full line 0

use std::sync::OnceLock;

use ktav::Error;
use regex::Regex;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::tokens::{classify_line, LineKind};

/// Parse `text` and return diagnostics. Empty vec on success.
pub fn parse_for_diagnostics(text: &str) -> Vec<Diagnostic> {
    match ktav::parse(text) {
        Ok(_) => Vec::new(),
        Err(Error::Syntax(msg)) => vec![syntax_to_diagnostic(text, &msg)],
        Err(Error::Io(_)) | Err(Error::Message(_)) => vec![Diagnostic {
            range: full_line_range(text, 0),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("ktav".to_string()),
            message: "ktav: parse failed".to_string(),
            ..Default::default()
        }],
    }
}

fn syntax_to_diagnostic(text: &str, msg: &str) -> Diagnostic {
    let range = compute_range(text, msg);
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ktav".to_string()),
        message: msg.to_string(),
        ..Default::default()
    }
}

/// Best-effort tightened range for a syntax-error message.
fn compute_range(text: &str, msg: &str) -> Range {
    let line = match extract_line_number(msg) {
        Some(n) => n.saturating_sub(1),
        None => {
            // No "line N" info — try last non-blank for "Unclosed" shapes.
            if msg.contains("Unclosed") {
                last_non_blank_line(text)
            } else {
                0
            }
        }
    };

    let line_text = nth_line(text, line as usize).unwrap_or("");

    // Shape-specific narrowing.
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
    // Highlight marker + the next non-whitespace character (which is the
    // glued body that violates the rule).
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
        // Locate the requested segment within the dotted path.
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
        // Fall back to the whole key.
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

/// Locate `seg` as a dot-bounded segment inside `dotted`.
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

/// Pull the 1-based line number out of an `Error::Syntax` message.
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

/// Pull a single-quoted key segment (e.g. `'foo.bar'`) out of a message.
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

/// Full-line range for a 0-based line index.
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
        let text = "key:value\n";
        let diags = parse_for_diagnostics(text);
        assert_eq!(diags.len(), 1);
        let r = diags[0].range;
        assert_eq!(r.start.line, 0);
        assert_eq!(r.start.character, 3); // `:` column
                                          // marker (':') + glued body ("value") = end column 9.
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
        // Tightened to the key segment `port` (cols 0..4), not the whole line.
        assert_eq!(r.start.character, 0);
        assert_eq!(r.end.character, 4);
    }

    #[test]
    fn empty_doc_is_valid() {
        let diags = parse_for_diagnostics("");
        assert!(diags.is_empty());
    }

    #[test]
    fn line_extraction_regex_shapes() {
        assert_eq!(extract_line_number("Invalid key at line 7: 'x.'"), Some(7));
        assert_eq!(extract_line_number("Empty key at line 3"), Some(3));
        assert_eq!(extract_line_number("Line 12: something"), Some(12));
        assert_eq!(extract_line_number("Unclosed { at end of input"), None);
    }
}
