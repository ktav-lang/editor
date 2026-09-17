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
    let last_line = text.split('\n').count().saturating_sub(1) as u32;
    let last_line_text = text.split('\n').next_back().unwrap_or("");
    let last_col = match encoding {
        PositionEncoding::Utf8 => last_line_text.len() as u32,
        PositionEncoding::Utf16 => byte_to_utf16(last_line_text, last_line_text.len()),
    };
    (last_line, last_col)
}
