//! Run the `ktav` parser over a document and turn any [`ktav::Error`]
//! into LSP [`Diagnostic`]s.
//!
//! The crate's `Error::Syntax` carries a free-form message; we recover a
//! line number with a small set of regexes covering all shapes emitted
//! by `ktav` 0.1.4: `"Line N: ..."`, `"Invalid key at line N: ..."`,
//! `"Empty key at line N"`. Anything else falls back to line 0.

use std::sync::OnceLock;

use ktav::Error;
use regex::Regex;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

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
    let line = extract_line_number(msg)
        .map(|n| n.saturating_sub(1))
        .unwrap_or(0);
    Diagnostic {
        range: full_line_range(text, line),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ktav".to_string()),
        message: msg.to_string(),
        ..Default::default()
    }
}

/// Pull the 1-based line number out of an `Error::Syntax` message.
/// Recognises:
///   - `"Line N: ..."` (most cases)
///   - `"Invalid key at line N: ..."`
///   - `"Empty key at line N"`
///   - `"... at line N ..."` (catch-all)
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

/// Build a `Range` covering the entirety of `line` in `text`. If the
/// line is past EOF, return a zero-width range at the very end.
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
    fn missing_separator_space_line_1() {
        let text = "key:value\n";
        let diags = parse_for_diagnostics(text);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range.start.line, 0);
        assert!(
            diags[0].message.contains("MissingSeparatorSpace"),
            "got: {}",
            diags[0].message
        );
    }

    #[test]
    fn duplicate_key_line_2() {
        let text = "port: 80\nport: 443\n";
        let diags = parse_for_diagnostics(text);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].range.start.line, 1);
        assert!(
            diags[0].message.to_lowercase().contains("duplicate"),
            "got: {}",
            diags[0].message
        );
    }

    #[test]
    fn empty_doc_is_valid() {
        let diags = parse_for_diagnostics("");
        assert!(diags.is_empty());
    }

    #[test]
    fn extracts_line_from_invalid_key_form() {
        // Synthetic — verifies the regex covers the alternate shape.
        assert_eq!(extract_line_number("Invalid key at line 7: 'x.'"), Some(7));
        assert_eq!(extract_line_number("Empty key at line 3"), Some(3));
        assert_eq!(extract_line_number("Line 12: something"), Some(12));
        assert_eq!(extract_line_number("Unclosed { at end of input"), None);
    }
}
