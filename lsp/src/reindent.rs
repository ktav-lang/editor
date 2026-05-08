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
        push_indent(&mut out, depth);
        out.push_str(trimmed);
        out.push('\n');

        // ---- Update depth / detect multi-line opener for the NEXT line ----
        // We classify the line by its trailing token. Order matters —
        // `((` must be checked before `(` so we don't mis-match the
        // first paren of `((`.
        if trimmed.ends_with("((") && !trimmed.ends_with(")((") {
            multi = Multi::Verbatim;
        } else if ends_with_lone_lparen(trimmed) {
            multi = Multi::Stripped;
        } else if trimmed == "{" || trimmed.ends_with(": {") {
            depth += 1;
        } else if trimmed == "[" || trimmed.ends_with(": [") {
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
}
