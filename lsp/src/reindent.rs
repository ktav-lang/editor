//! Line-based canonical re-indenter.
//!
//! The previous formatting approach was `parse → render(value)`. That
//! produces a fully canonical Ktav document — but it discards anything
//! the parser doesn't store in the `Value` tree:
//!
//!   * blank lines (visual separators between sections)
//!   * `#` comments
//!   * the user's choice of multi-line form (`( ... )` vs `(( ... ))`)
//!
//! All three are pure user-controlled formatting choices and should
//! survive a "format document" pass. So instead of round-tripping
//! through `Value`, we walk the source line by line, track nesting
//! depth from structural tokens, and emit each line with canonical
//! indent (`depth * 4` spaces). Inside multi-line string blocks
//! (`(` ... `)` and `((` ... `))`) lines are copied verbatim — both
//! forms are content-preserving on the parser side.
//!
//! The output is byte-equal to the input when every line is already
//! at the right indent.

const INDENT: &str = "    ";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Multi {
    None,
    /// Inside `(` ... `)` — stripped form. Parser dedents on read; we
    /// keep the original indent of the contents.
    Stripped,
    /// Inside `((` ... `))` — verbatim. Bytes preserved exactly.
    Verbatim,
}

/// Re-emit `src` with canonical indentation. Blank lines, comments,
/// and multi-line string contents are preserved.
pub fn reindent(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 32);
    let mut depth: usize = 0;
    let mut multi = Multi::None;

    // `split('\n')` on `"a\n"` yields `["a", ""]` — the trailing empty
    // entry represents "no characters after the final newline", not a
    // blank line. Drop it so we don't emit an extra `\n`. (If the user
    // actually wrote `a\n\n`, that's `["a", "", ""]` — we still drop
    // only the last, preserving the explicit blank.)
    let lines: Vec<&str> = {
        let mut v: Vec<&str> = src.split('\n').collect();
        if v.last().map(|s| s.is_empty()).unwrap_or(false) {
            v.pop();
        }
        v
    };
    let trailing_newline = src.ends_with('\n');

    for raw_line in lines {
        // Strip a single trailing CR (Windows line endings) — we always
        // emit `\n` and let the editor reapply CRLF on save if it wants.
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();

        // ---- Inside multi-line string: copy verbatim ----
        if multi != Multi::None {
            if (multi == Multi::Stripped && trimmed == ")")
                || (multi == Multi::Verbatim && trimmed == "))")
            {
                // Closing line — emit at current depth, exit multi mode.
                push_indent(&mut out, depth);
                out.push_str(trimmed);
                out.push('\n');
                multi = Multi::None;
            } else {
                // Content line — copy as-is (preserves user's leading
                // whitespace, which is part of the value).
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        // ---- Blank line: keep as visual separator ----
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // ---- Closing structural line: dedent BEFORE emit ----
        // A line that is just `}` / `]` (or with leading whitespace
        // only) reduces nesting before its own indent is computed.
        if matches!(trimmed, "}" | "]") {
            depth = depth.saturating_sub(1);
            push_indent(&mut out, depth);
            out.push_str(trimmed);
            out.push('\n');
            continue;
        }

        // ---- Comment: keep at current depth ----
        if trimmed.starts_with('#') {
            push_indent(&mut out, depth);
            out.push_str(trimmed);
            out.push('\n');
            continue;
        }

        // ---- Regular line: emit at current depth ----
        // Auto-disambiguate values that visually look like multi-line
        // openers but aren't: `name: (value)` and `name: ((value))`
        // are inline scalars starting with `(` / `((`. The parser
        // accepts them, but the reader is left to wonder whether `(`
        // is the start of a multi-line block. Canonical form is the
        // raw marker `::`, which carries the same semantics
        // (`name:: (value)` parses identically). Rewrite on format.
        let line_to_emit = canonicalise_paren_scalar(trimmed);
        push_indent(&mut out, depth);
        out.push_str(&line_to_emit);
        out.push('\n');

        // ---- Update depth / detect multi-line opener for the NEXT line ----
        // We classify the line by its trailing token. Order matters —
        // `((` must be checked before `(` so we don't mis-match the
        // first paren of `((`.
        if trimmed.ends_with("((") && !trimmed.ends_with(")((") {
            multi = Multi::Verbatim;
        } else if ends_with_lone_lparen(trimmed) {
            multi = Multi::Stripped;
        } else if trimmed == "{"
            || trimmed == "["
            || trimmed.ends_with(": {")
            || trimmed.ends_with(": [")
        {
            depth += 1;
        }
    }

    // Strip the trailing `\n` if the input didn't have one — otherwise
    // every line emitted with `\n` produces a synthetic final newline.
    if !trailing_newline && out.ends_with('\n') {
        out.pop();
    }

    out
}

/// If [trimmed] is `<key>: <value>` where `<value>` is an inline
/// scalar starting with `(` or `((` (i.e. NOT a multi-line opener:
/// the line has more text after the open paren on the same line),
/// rewrite the separator to `::` so the value is unambiguous.
///
/// Examples (input → output):
///     `name: (value)`     → `name:: (value)`
///     `name: ((value))`   → `name:: ((value))`
///     `name: (value`      → `name:: (value`     (still ambiguous, but
///                                                we make the `::`
///                                                intent explicit)
///     `name: (`           → unchanged (multi-line stripped opener)
///     `name: ((`          → unchanged (multi-line verbatim opener)
///     `name: text`        → unchanged
///     `# name: (value)`   → unchanged (caller handled comments)
///
/// Semantically `name: (value)` and `name:: (value)` produce identical
/// values via the current parser — both yield `String("(value)")`. So
/// this rewrite is safe (no observable behaviour change), it only
/// removes visual confusion with multi-line openers.
fn canonicalise_paren_scalar(trimmed: &str) -> std::borrow::Cow<'_, str> {
    // Find the FIRST `:` separator. We accept `:`, `::`, `:i`, `:f` as
    // markers; only the plain `:` is the case we rewrite (the others
    // already mean something specific and aren't ambiguous).
    let bytes = trimmed.as_bytes();
    let colon = match bytes.iter().position(|&b| b == b':') {
        Some(p) => p,
        None => return std::borrow::Cow::Borrowed(trimmed),
    };

    // Reject `::` / `:i` / `:f` markers — they're already explicit.
    if colon + 1 < bytes.len() {
        let next = bytes[colon + 1];
        if next == b':' || next == b'i' || next == b'f' {
            return std::borrow::Cow::Borrowed(trimmed);
        }
    }

    // The `:` must be followed by at least one whitespace character to
    // be a valid pair separator (Ktav § 6.10).
    if colon + 1 >= bytes.len() || bytes[colon + 1] != b' ' && bytes[colon + 1] != b'\t' {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // The value starts after the leading whitespace.
    let mut value_start = colon + 1;
    while value_start < bytes.len()
        && (bytes[value_start] == b' ' || bytes[value_start] == b'\t')
    {
        value_start += 1;
    }
    let value = &trimmed[value_start..];

    // Empty value — leave alone (Ktav represents this as `name:` /
    // `name: ` and the parser keeps the empty-string semantics).
    if value.is_empty() {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // Must start with `(` (single or double).
    if !value.starts_with('(') {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // Detect multi-line opener: the opener is the WHOLE value (just `(`
    // or `((`, possibly trailing whitespace already trimmed by caller).
    // In that case we leave it alone — `:` + multi-line block is the
    // canonical multi-line form.
    if value == "(" || value == "((" {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // Inline `(`-starting value: rewrite `:` → `::`. Keep the same
    // whitespace shape: replace exactly the single `:` token.
    let key_part = &trimmed[..colon];
    let after_colon = &trimmed[colon + 1..];
    std::borrow::Cow::Owned(format!("{}::{}", key_part, after_colon))
}

/// Does the line end with a single `(` (stripped multi-line opener)
/// rather than `((`? A lone `(` after `:`+space, or a bare `(` line.
fn ends_with_lone_lparen(s: &str) -> bool {
    if s == "(" {
        return true;
    }
    if !s.ends_with('(') {
        return false;
    }
    // Look at the char before the trailing `(`. If it's another `(` we
    // have a `((` opener — handled separately.
    let bytes = s.as_bytes();
    if bytes.len() >= 2 && bytes[bytes.len() - 2] == b'(' {
        return false;
    }
    // Otherwise (e.g. `body: (`) it's a single-paren opener.
    true
}

fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str(INDENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_preserved() {
        let src = "name: a\n\nother: b\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn comments_preserved() {
        let src = "# top\nname: a\n# inline\nother: b\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn nested_object_indent_canonicalised() {
        let src = "outer: {\n  inner: 1\n}\n";
        let want = "outer: {\n    inner: 1\n}\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn nested_object_dedent_correct() {
        let src = "a: {\n        b: 1\n        }\n";
        let want = "a: {\n    b: 1\n}\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn double_nested() {
        let src = "a: {\nb: {\nc: 1\n}\n}\n";
        let want = "a: {\n    b: {\n        c: 1\n    }\n}\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn stripped_multiline_content_kept() {
        // Content lines are copied verbatim (they may carry leading WS
        // that's part of the value); closing `)` is re-indented.
        let src = "key: (\n    line1\n    line2\n     )\n";
        let want = "key: (\n    line1\n    line2\n)\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn verbatim_multiline_content_kept_verbatim() {
        let src = "key: ((\n  weird   indent\n      here\n))\n";
        let want = "key: ((\n  weird   indent\n      here\n))\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn array_with_blank_lines_preserved() {
        let src = "items: [\nfirst\n\nsecond\n]\n";
        let want = "items: [\n    first\n\n    second\n]\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn crlf_normalised_to_lf() {
        let src = "a: 1\r\nb: 2\r\n";
        let want = "a: 1\nb: 2\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn inline_paren_scalar_gets_raw_marker() {
        // `name: (value)` is an inline scalar starting with `(` — visually
        // confusing with multi-line opener. Format rewrites to `::`.
        let src = "name: (value)\n";
        let want = "name:: (value)\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn inline_double_paren_scalar_gets_raw_marker() {
        let src = "name: ((value))\n";
        let want = "name:: ((value))\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn multi_line_opener_paren_is_not_touched() {
        // Just `(` at end of line — that's the multi-line opener.
        let src = "name: (\n    body\n)\n";
        let want = "name: (\n    body\n)\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn multi_line_opener_double_paren_is_not_touched() {
        let src = "name: ((\nbody\n))\n";
        let want = "name: ((\nbody\n))\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn raw_marker_paren_value_unchanged() {
        // Already explicit raw — leave alone.
        let src = "name:: (value)\n";
        let want = "name:: (value)\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn typed_marker_paren_value_unchanged() {
        // `:i` / `:f` are typed markers, not a plain `:` — don't rewrite.
        // (Such bodies would fail typed-scalar validation, but that's
        //  the parser's job to flag, not the formatter's.)
        let src = "x:i (5)\n";
        let want = "x:i (5)\n";
        assert_eq!(reindent(src), want);
    }
}
