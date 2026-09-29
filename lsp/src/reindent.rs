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
//!
//! § 3.2: LF, CR and CRLF are equivalent line terminators — splitting
//! goes through [`crate::lines`] so a CR-only or CRLF document is
//! reindented line-for-line just like its LF equivalent. Output is
//! always LF-canonical (a multi-line string's parsed value already
//! normalises every terminator to `\n` — see `ktav`'s own multi-line
//! handling — so re-emitting with `\n` never changes what the document
//! means, only its on-disk bytes).

use crate::tokens::{find_key_separator, is_ktav_ws, DocumentContext, LineKind};

const INDENT: &str = "    ";

/// Re-emit `src` with canonical indentation. Blank lines, comments,
/// and multi-line string contents are preserved.
///
/// A leading BOM is excluded from parser-like line processing and
/// re-emitted verbatim at the start of the output.
pub fn reindent(src: &str) -> String {
    let bom = &src[..crate::lines::leading_bom_len(src)];
    let mut out = String::with_capacity(src.len() + 32);
    out.push_str(bom);
    let mut context = DocumentContext::default();

    // `split_lines("a\n")` yields `["a", ""]` — the trailing empty entry
    // represents "no characters after the final terminator", not a
    // blank line. Drop it so we don't emit an extra `\n`. (If the user
    // actually wrote `a\n\n`, that's `["a", "", ""]` — we still drop
    // only the last, preserving the explicit blank.) The splitter
    // already strips whichever terminator (LF/CR/CRLF) each line had, so
    // there's no separate CR-stripping step here.
    let lines: Vec<&str> = {
        let mut v = crate::lines::content_lines(src);
        if v.last().map(|s| s.is_empty()).unwrap_or(false) {
            v.pop();
        }
        v
    };
    let trailing_newline = crate::lines::ends_with_terminator(src);

    for line in lines {
        let trimmed = line.trim_matches(is_ktav_ws);
        let classified = context.next_line(line);

        if classified.multiline_content {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        // ---- Blank line: keep as visual separator ----
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // A colon-bearing Array String is not a pair (§ 5.1 rule 7).
        let line_to_emit = if matches!(classified.kind, LineKind::Pair { .. }) {
            canonicalise_paren_scalar(trimmed)
        } else {
            std::borrow::Cow::Borrowed(trimmed)
        };
        // Do not expose a content BOM as document metadata (§ 3.1).
        if out.is_empty() && trimmed.starts_with('\u{FEFF}') {
            out.push_str(&line[..line.len() - line.trim_start_matches(is_ktav_ws).len()]);
        } else {
            push_indent(&mut out, classified.depth);
        }
        out.push_str(&line_to_emit);
        out.push('\n');
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
    // Find the key/value separator the same way the parser does: the
    // first UNESCAPED `:`, opaque to `<quoted-segment>` content (§ 5.3.3)
    // and to an escaped `\:` inside the key (§ 3.7). A naive first-byte
    // scan (as a prior version of this did) would treat a `:` inside a
    // quoted key (`"a:b": (x)`) as the separator, splicing `::` into the
    // middle of the quoted text instead of right after it. We accept
    // `:` and `::` as markers; only the plain `:` is the case we
    // rewrite (`::` already means "raw" and isn't ambiguous). Typed
    // markers `:i` / `:f` were removed in spec 0.5.0 and no longer exist.
    let bytes = trimmed.as_bytes();
    let colon = match find_key_separator(trimmed) {
        Some(p) => p,
        None => return std::borrow::Cow::Borrowed(trimmed),
    };

    // Reject `::` — it's already explicit (raw marker).
    if colon + 1 < bytes.len() && bytes[colon + 1] == b':' {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // The `:` must be followed by at least one whitespace character to
    // be a valid pair separator (Ktav § 6.10).
    if !trimmed[colon + 1..].starts_with(is_ktav_ws) {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // The value starts after the leading whitespace.
    let value = trimmed[colon + 1..].trim_start_matches(is_ktav_ws);

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

    // `()` / `(())` are the one-line empty-stripped / empty-verbatim
    // forms — under a Plain marker they mean the empty STRING (same as
    // `{}` means the empty Object), not the literal 2-/4-byte text.
    // Rewriting either to `::` would change the parsed value: `key:: ()`
    // is the literal string "()", not "". Distinct from every other
    // paren-wrapped value, where Plain and Raw already agree byte-for-byte
    // (`key: (value)` and `key:: (value)` both yield `String("(value)")`).
    if value == "()" || value == "(())" {
        return std::borrow::Cow::Borrowed(trimmed);
    }

    // Inline `(`-starting value: rewrite `:` → `::`. Keep the same
    // whitespace shape: replace exactly the single `:` token.
    let key_part = &trimmed[..colon];
    let after_colon = &trimmed[colon + 1..];
    std::borrow::Cow::Owned(format!("{}::{}", key_part, after_colon))
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
        // Spec 0.5.0: a comment is a leading `##`.
        let src = "## top\nname: a\n## inline\nother: b\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn single_hash_key_is_not_a_comment() {
        // A lone `#` is an ordinary character — `#child` is a valid key,
        // not a comment, and `#child: {` opens a compound like any other
        // pair. Before the fix, `trimmed.starts_with('#')` swallowed this
        // line whole: it never bumped `depth`, so `a: 1` below rendered
        // at depth 0 instead of 1, and the closing `}` never dedented.
        let src = "#child: {\n    a: 1\n}\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn single_hash_scalar_item_is_not_a_comment() {
        // Top-level Array of bare-scalar items (spec § 5.0.1) — a lone
        // `#` is ordinary content, kept as-is, not dropped as a comment.
        let src = "#not-a-comment\nother-item\n";
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
    fn empty_paren_forms_are_not_rewritten_to_raw() {
        // valid/inline/paren_empty_string_matches_raw_body.ktav: `()` and
        // `(())` under a Plain marker mean the empty STRING — rewriting
        // either to `::` would turn it into the literal 2-/4-byte text
        // "()" / "(())" instead, changing the parsed value.
        assert_eq!(reindent("a: ()\n"), "a: ()\n");
        assert_eq!(reindent("b: (())\n"), "b: (())\n");
    }

    #[test]
    fn quoted_key_with_colon_paren_scalar_gets_raw_marker_after_the_key() {
        // `"a:b": (x)` — the quoted key's OWN `:` must not be mistaken
        // for the key/value separator (§ 5.3.3). Before the fix (a naive
        // first-byte `:` scan) this spliced `::` into the middle of the
        // quoted key instead of right after it.
        let src = "\"a:b\": (x)\n";
        let want = "\"a:b\":: (x)\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn colon_not_followed_by_whitespace_unchanged() {
        // `x:(5)` has no space after `:`, so it isn't a valid pair
        // separator (§ 6.10) — not our job to rewrite malformed input.
        let src = "x:(5)\n";
        let want = "x:(5)\n";
        assert_eq!(reindent(src), want);
    }

    // ---- Structural opener detection (§ 4 / § 4.1) ----

    #[test]
    fn raw_double_paren_value_is_not_an_opener() {
        // § 4: `::` is a raw-scalar production — `key:: ((` is the
        // literal string "((", not a multi-line opener. Before the fix
        // (`trimmed.ends_with("((")`), this line switched the reindenter
        // into verbatim multi-line mode, which then swallowed `sibling`
        // and the closing `}` as literal content instead of reindenting
        // them.
        let src = "outer: {\n    key:: ((\n    sibling: 1\n}\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn raw_single_paren_value_is_not_an_opener() {
        let src = "outer: {\n    key:: (\n    sibling: 1\n}\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn raw_brace_value_is_not_an_opener() {
        let src = "outer: {\n    key:: {\n    sibling: 1\n}\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn escaped_colon_key_ending_in_brace_is_not_an_opener() {
        // `a\: {` has no unescaped `:` at all — it's a bare scalar item,
        // not a pair with a trailing `{` opener. A trailing-token
        // heuristic (`trimmed.ends_with(": {")`) would false-positive
        // here; the structural classifier correctly sees no separator.
        let src = "a\\: {\nsibling\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn value_that_merely_ends_in_brace_is_not_an_opener() {
        // The value is the whole string "text ending with a brace {" —
        // not exactly "{" — so it does not open a compound, even though
        // the line's tail looks like one.
        let src = "note: text ending with a brace {\nsibling: 1\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn quoted_key_containing_brace_still_opens_correctly() {
        // A quoted key may itself contain `: {` as literal text; the
        // classifier is opaque to quoted segments (§ 5.3.3), so the real
        // separator and the real (structural) opener after it are still
        // found correctly.
        let src = "\"x: {\": {\n    a: 1\n}\n";
        assert_eq!(reindent(src), src);
    }

    // ---- § 3.2: LF / CR / CRLF are equivalent line terminators ----

    #[test]
    fn cr_only_document_formats_like_lf() {
        let lf = "outer: {\n    inner: 1\n}\n";
        let cr = "outer: {\rinner: 1\r}\r";
        assert_eq!(reindent(cr), lf);
    }

    #[test]
    fn crlf_document_formats_like_lf() {
        let lf = "outer: {\n    inner: 1\n}\n";
        let crlf = "outer: {\r\ninner: 1\r\n}\r\n";
        assert_eq!(reindent(crlf), lf);
    }

    #[test]
    fn cr_only_no_trailing_terminator_preserved() {
        let src = "a: 1\rb: 2";
        let want = "a: 1\nb: 2";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn cr_only_comment_and_multiline_block_formatted_like_lf() {
        let lf = "## note\nkey: (\n    line1\n    line2\n)\ndone: 1\n";
        let cr = "## note\rkey: (\r    line1\r    line2\r)\rdone: 1\r";
        assert_eq!(reindent(cr), lf);
    }

    // ---- § 3.1: leading BOM is metadata, preserved not lost ----

    #[test]
    fn leading_bom_is_preserved_and_not_reindented_as_content() {
        let src = "\u{FEFF}host: value\n";
        assert_eq!(reindent(src), src);
    }

    #[test]
    fn leading_bom_document_still_reindents_its_content() {
        let src = "\u{FEFF}outer: {\n  inner: 1\n}\n";
        let want = "\u{FEFF}outer: {\n    inner: 1\n}\n";
        assert_eq!(reindent(src), want);
    }

    #[test]
    fn no_bom_input_yields_no_bom_output() {
        let src = "host: value\n";
        assert!(!reindent(src).starts_with('\u{FEFF}'));
    }
}
