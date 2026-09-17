//! Byte→UTF-16 column conversion — applied only when the negotiated
//! position encoding is UTF-16.

use tower_lsp::lsp_types::{Diagnostic, DocumentSymbol, Position, SemanticToken};

use crate::tokens::byte_to_utf16;

pub(super) fn convert_diagnostics_to_utf16(diags: &mut [Diagnostic], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    for d in diags {
        convert_position_to_utf16(&mut d.range.start, &lines);
        convert_position_to_utf16(&mut d.range.end, &lines);
    }
}

pub(super) fn convert_symbols_to_utf16(symbols: &mut [DocumentSymbol], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    fn walk(syms: &mut [DocumentSymbol], lines: &[&str]) {
        for s in syms {
            convert_position_to_utf16(&mut s.range.start, lines);
            convert_position_to_utf16(&mut s.range.end, lines);
            convert_position_to_utf16(&mut s.selection_range.start, lines);
            convert_position_to_utf16(&mut s.selection_range.end, lines);
            if let Some(kids) = s.children.as_mut() {
                walk(kids, lines);
            }
        }
    }
    walk(symbols, &lines);
}

fn convert_position_to_utf16(pos: &mut Position, lines: &[&str]) {
    let line = lines.get(pos.line as usize).copied().unwrap_or("");
    pos.character = byte_to_utf16(line, pos.character as usize);
}

/// Re-encode the absolute (line, start, length) implied by a delta-encoded
/// `SemanticToken` stream from byte offsets to UTF-16 code units.
pub(super) fn convert_semantic_tokens_to_utf16(toks: &mut [SemanticToken], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut abs_line: u32 = 0;
    let mut abs_start_bytes: u32 = 0;
    let mut prev_emit_line: u32 = 0;
    let mut prev_emit_start_u16: u32 = 0;

    for t in toks.iter_mut() {
        // Reconstruct absolute byte position on this token.
        abs_line += t.delta_line;
        if t.delta_line == 0 {
            abs_start_bytes += t.delta_start;
        } else {
            abs_start_bytes = t.delta_start;
        }

        let line = lines.get(abs_line as usize).copied().unwrap_or("");
        let start_bytes = abs_start_bytes as usize;
        let end_bytes = start_bytes + t.length as usize;
        let start_u16 = byte_to_utf16(line, start_bytes);
        let end_u16 = byte_to_utf16(line, end_bytes);

        let new_delta_line = abs_line - prev_emit_line;
        let new_delta_start = if new_delta_line == 0 {
            start_u16 - prev_emit_start_u16
        } else {
            start_u16
        };

        t.delta_line = new_delta_line;
        t.delta_start = new_delta_start;
        t.length = end_u16 - start_u16;

        prev_emit_line = abs_line;
        prev_emit_start_u16 = start_u16;
    }
}
