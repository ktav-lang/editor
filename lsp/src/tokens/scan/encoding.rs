//! Column conversions between byte offsets and UTF-16 code units,
//! used when the negotiated LSP position encoding is UTF-16.

/// Convert a byte index into a `line` to a UTF-16 code-unit offset.
///
/// Used by [`crate::server`] to re-encode column positions from byte to
/// UTF-16 when the negotiated [`tower_lsp::lsp_types::PositionEncodingKind`]
/// is UTF-16 (the LSP default). For UTF-8 negotiation no conversion is
/// needed and this helper is bypassed.
///
/// `byte_idx` past the end of `line` clamps to the line's UTF-16 length.
pub fn byte_to_utf16(line: &str, byte_idx: usize) -> u32 {
    let cap = byte_idx.min(line.len());
    let prefix = &line[..cap];
    prefix.encode_utf16().count() as u32
}

/// Slice the prefix of `line` up to a cursor column expressed in the
/// negotiated [`crate::server::PositionEncoding`].
///
/// - `Utf8`: `character` is a byte offset. Clamped to `line.len()`, then
///   rounded **down** to the nearest UTF-8 char boundary so the returned
///   slice is always valid UTF-8 (never splits a multi-byte codepoint).
/// - `Utf16`: `character` is a UTF-16 code-unit count. We walk `chars()`
///   accumulating `len_utf16()` and stop the first time the total reaches
///   or exceeds the target — returning the byte-prefix up to that char's
///   start. This handles surrogate-pair targets (BMP-only stop point)
///   without ever slicing inside a single codepoint.
///
/// Used by completion to compute the "text before cursor" slice in an
/// encoding-correct way for non-ASCII lines (Cyrillic, Hebrew, emoji).
pub fn prefix_by_encoding(
    line: &str,
    character: u32,
    enc: crate::server::PositionEncoding,
) -> &str {
    match enc {
        crate::server::PositionEncoding::Utf8 => {
            let mut n = (character as usize).min(line.len());
            while n > 0 && !line.is_char_boundary(n) {
                n -= 1;
            }
            &line[..n]
        }
        crate::server::PositionEncoding::Utf16 => {
            let target = character as usize;
            let mut acc: usize = 0;
            for (byte_idx, c) in line.char_indices() {
                let next = acc + c.len_utf16();
                if next > target {
                    // Including this char would overshoot — and if it's a
                    // surrogate pair the target landed mid-codepoint. Stop
                    // here without slicing into the codepoint.
                    return &line[..byte_idx];
                }
                acc = next;
                if acc == target {
                    let end = byte_idx + c.len_utf8();
                    return &line[..end];
                }
            }
            line
        }
    }
}
