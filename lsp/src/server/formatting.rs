//! Whole-document formatting support: the end-of-document position,
//! expressed in the negotiated position encoding.

use crate::tokens::byte_to_utf16;

use super::PositionEncoding;

/// `(line, character)` of the end of `text`, in the given position
/// encoding — i.e. the position one past the last byte, suitable as the
/// `end` of a whole-document replace `Range`.
///
/// `character` must be in the negotiated encoding, same as every other
/// position this file emits: byte length (`str::len`) for `Utf8`,
/// UTF-16 code-unit count (`byte_to_utf16`) for `Utf16`. Using
/// `chars().count()` here (as a prior version of this function did)
/// undercounts BOTH: it undercounts UTF-8 byte length for any
/// multi-byte scalar, and undercounts UTF-16 length for any
/// astral-plane / surrogate-pair character — under-shooting the real
/// end-of-line offset and leaving trailing bytes of the last line
/// unreplaced by a formatting edit.
pub(super) fn end_of_document(text: &str, encoding: PositionEncoding) -> (u32, u32) {
    // § 3.2: LF, CR and CRLF are equivalent line terminators — go
    // through the shared splitter so the replaced range's end matches
    // the client's own line count on a non-LF document.
    let lines = crate::lines::split_lines(text);
    let last_line = lines.len().saturating_sub(1) as u32;
    let last_line_text = lines.last().copied().unwrap_or("");
    let last_col = match encoding {
        PositionEncoding::Utf8 => last_line_text.len() as u32,
        PositionEncoding::Utf16 => byte_to_utf16(last_line_text, last_line_text.len()),
    };
    (last_line, last_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_of_document_agrees_across_line_terminators() {
        // § 3.2: LF, CR and CRLF are equivalent line terminators.
        let lf = "a: 1\nb: 2\n";
        let cr = "a: 1\rb: 2\r";
        let crlf = "a: 1\r\nb: 2\r\n";
        let want = end_of_document(lf, PositionEncoding::Utf8);
        assert_eq!(end_of_document(cr, PositionEncoding::Utf8), want);
        assert_eq!(end_of_document(crlf, PositionEncoding::Utf8), want);
        assert_eq!(want, (2, 0));
    }

    #[test]
    fn end_of_document_no_trailing_terminator_cr() {
        let text = "a: 1\rb: 2";
        assert_eq!(end_of_document(text, PositionEncoding::Utf8), (1, 4));
    }
}
