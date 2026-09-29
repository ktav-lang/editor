//! Cross-cutting conformance checks for the editor-facing LSP features —
//! document symbols, hover, semantic tokens, the formatter — over every
//! `valid/**/*.ktav` fixture in the pinned spec 0.8 corpus (`*.canonical.ktav`
//! siblings are excluded: same `Value` tree as the `.ktav` they re-render, so
//! they would only double-count). `spec_conformance.rs` already proves the
//! corpus parses and diagnoses correctly; this file is the "did the editor
//! features render it correctly" floor built on top of that.
//!
//! * **Document symbols** (`ktav_lsp::symbols::build_symbols`, a pure
//!   function): the resulting tree is compared against the parsed `Value`
//!   tree structurally — same key/index names, same nesting, same
//!   `SymbolKind` per leaf/compound — independently of `build_symbols`'s own
//!   line-position bookkeeping (which the module's own doc comment already
//!   calls "approximate").
//! * **Hover**: exercised through the real `Backend` (the only public
//!   surface — `server::hover`'s helpers are `pub(super)`, unreachable from
//!   an external integration test). Every line of every fixture is hovered
//!   to prove the handler never panics; additionally, every scalar leaf
//!   reached via an *object key* (the only line shape `hover()` ever
//!   resolves — array items have no `key:` line) is checked to report the
//!   expected type word, but ONLY when `key_matches_this_line` independently
//!   confirms `build_symbols`'s line number actually names this key on ITS
//!   OWN — see that function's doc comment: hover only ever resolves a
//!   line's own top-level `key:` pair, never a key reached through an inline
//!   compound sharing that line with another key (`cfg: {a: 1, b: 2}`'s `a`
//!   line is `cfg`'s own line, whose top-level key is `cfg`, not `a`) —
//!   that is a real, standing limitation of the per-line hover dispatch, not
//!   a position-bookkeeping defect, so skipping it is legitimate rather than
//!   papering over a bug.
//! * **Semantic tokens** (`ktav_lsp::semantic::semantic_tokens`, a pure
//!   function): every token decodes to a span within its own line's byte
//!   length, and tokens are strictly ordered/non-overlapping.
//! * **Formatting** (`ktav_lsp::reindent::reindent`, a pure function):
//!   idempotent (`format(format(x)) == format(x)`) and value-preserving
//!   (`ktav::parse(format(x)) == ktav::parse(x)`) over every fixture.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::Value;
use ktav_lsp::semantic::semantic_tokens;
use ktav_lsp::symbols::build_symbols;
use ktav_lsp::tokens::{classify_line, split_dotted, LineKind};
use ktav_lsp::Backend;
use tower_lsp::lsp_types::*;
use tower_lsp::{LanguageServer, LspService};

fn valid_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).join("../spec/versions/0.8/tests/valid")
}

/// Every `valid/**/*.ktav` fixture, excluding `*.canonical.ktav` re-renders
/// (same parsed `Value` as their `.ktav` sibling stem).
fn collect_valid_ktav_files() -> Vec<PathBuf> {
    let root = valid_dir();
    assert!(
        root.is_dir(),
        "spec submodule not initialised at {} — see spec_conformance.rs's \
         spec_tests_dir()",
        root.display()
    );
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("ktav")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".canonical.ktav"))
                    .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn kind_for(v: &Value) -> SymbolKind {
    match v {
        Value::Null => SymbolKind::NULL,
        Value::Bool(_) => SymbolKind::BOOLEAN,
        Value::Integer(_) | Value::Float(_) => SymbolKind::NUMBER,
        Value::String(_) => SymbolKind::STRING,
        Value::Array(_) => SymbolKind::ARRAY,
        Value::Object(_) => SymbolKind::MODULE,
    }
}

fn is_leaf(v: &Value) -> bool {
    !matches!(v, Value::Object(_) | Value::Array(_))
}

// ---------------------------------------------------------------------------
// Document symbols
// ---------------------------------------------------------------------------

/// Compare a `Value` compound (Object or Array) against the `DocumentSymbol`
/// slice `build_symbols`/`build_children` produced for it: same length,
/// same names in the same order, same `SymbolKind`, same leaf/compound
/// shape (recursing into compounds). Positions are intentionally never
/// compared — see the module doc comment.
fn check_children(value: &Value, symbols: &[DocumentSymbol]) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            if map.len() != symbols.len() {
                return Err(format!(
                    "object has {} key(s) but {} symbol(s)",
                    map.len(),
                    symbols.len()
                ));
            }
            for ((k, v), sym) in map.iter().zip(symbols) {
                if sym.name.as_str() != k.as_str() {
                    return Err(format!(
                        "expected symbol name '{}', got '{}'",
                        k.as_str(),
                        sym.name
                    ));
                }
                check_kind_and_recurse(k.as_str(), v, sym)?;
            }
        }
        Value::Array(items) => {
            if items.len() != symbols.len() {
                return Err(format!(
                    "array has {} item(s) but {} symbol(s)",
                    items.len(),
                    symbols.len()
                ));
            }
            for (i, (v, sym)) in items.iter().zip(symbols).enumerate() {
                let expected_name = format!("[{i}]");
                if sym.name != expected_name {
                    return Err(format!(
                        "expected item name '{}', got '{}'",
                        expected_name, sym.name
                    ));
                }
                check_kind_and_recurse(&expected_name, v, sym)?;
            }
        }
        _ => return Err("top-level value must be Object or Array (spec § 5.0.1)".into()),
    }
    Ok(())
}

fn check_kind_and_recurse(label: &str, v: &Value, sym: &DocumentSymbol) -> Result<(), String> {
    if sym.kind != kind_for(v) {
        return Err(format!(
            "'{label}': expected SymbolKind {:?}, got {:?}",
            kind_for(v),
            sym.kind
        ));
    }
    if is_leaf(v) {
        if sym.children.is_some() {
            return Err(format!("leaf '{label}' unexpectedly has children"));
        }
    } else {
        let children = sym
            .children
            .as_ref()
            .ok_or_else(|| format!("compound '{label}' is missing its children"))?;
        check_children(v, children)?;
    }
    Ok(())
}

#[test]
fn document_symbols_match_parsed_value_tree() {
    let files = collect_valid_ktav_files();
    assert!(!files.is_empty(), "no valid fixtures found");

    let mut failures = Vec::new();
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("{}: ktav::parse failed: {e:?}", path.display()));
                continue;
            }
        };
        let symbols = build_symbols(&value, &text);
        if let Err(msg) = check_children(&value, &symbols) {
            failures.push(format!("{}: {msg}", path.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} valid fixtures: document symbol tree diverges from the \
         parsed Value tree:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!(
        "conformance/symbols: {} valid fixtures' symbol trees match their \
         parsed Value tree",
        files.len()
    );
}

// ---------------------------------------------------------------------------
// Hover
// ---------------------------------------------------------------------------

fn expected_type_phrase(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool:",
        Value::Integer(_) => "integer:",
        Value::Float(_) => "float:",
        Value::String(_) => "string:",
        Value::Array(_) => "array of",
        Value::Object(_) => "object with",
    }
}

/// Independently confirm that `text`'s line `line` is a `key: value` line
/// whose key really is `expected_name`, before trusting `build_symbols`'s
/// line number for a hover check.
///
/// `hover()` only ever resolves a line's OWN top-level `key:` pair (it
/// reclassifies the whole line via `classify_line` and reads off that
/// line's `key`) — it has no column awareness, so it can never answer a
/// key reached through an inline compound that shares its line with
/// another key (`cfg: {a: 1, b: 2}`'s line is `cfg`'s own line; hovering
/// anywhere on it can only ever resolve `cfg`, never `a`/`b`, even though
/// `build_symbols` now reports `a`/`b`'s own precise sub-range on that
/// same line). Re-deriving the line's own key (via the same public
/// `classify_line` / `split_dotted` helpers `hover()` and `build_symbols`
/// both use) and comparing it to `expected_name` catches exactly that
/// case: on a mismatch, the caller skips the type check instead of
/// asserting against a key `hover()` structurally cannot answer.
fn key_matches_this_line(text: &str, line: u32, expected_name: &str) -> bool {
    let Some(line_text) = text.split('\n').nth(line as usize) else {
        return false;
    };
    let LineKind::Pair {
        key_start,
        key_length,
        ..
    } = classify_line(line_text)
    else {
        return false;
    };
    let lo = key_start as usize;
    let hi = lo + key_length as usize;
    let Some(raw_key) = line_text.get(lo..hi) else {
        return false;
    };
    // `expected_name` is one already-decoded, already-merged level of the
    // parsed key tree — take the LAST dotted segment of the line's own key
    // text (undecoded) and strip a matching quote pair, best-effort. A
    // decode mismatch (escapes, `\u` sequences) just fails the comparison,
    // which is the safe direction: the caller skips rather than asserts.
    let Some((_, last_seg)) = split_dotted(0, raw_key).last() else {
        return false;
    };
    let seg = last_seg.trim();
    let unquoted = match seg.as_bytes().first() {
        Some(b'"') | Some(b'\'') | Some(b'`') if seg.len() >= 2 => &seg[1..seg.len() - 1],
        _ => seg,
    };
    unquoted == expected_name
}

/// Collect `(key, expected type phrase, symbol line)` for every scalar leaf
/// reached through an object key whose line `key_matches_this_line`
/// confirms — see that function's doc comment for why some leaves are
/// skipped rather than checked. Array items are skipped entirely regardless
/// of position: they have no `key:` line, and `hover()`'s `classify_line`
/// dispatch only ever answers `LineKind::Pair` — a bare array item line
/// always returns `None`, so there is nothing for hover to resolve there.
fn collect_object_leaves(
    value: &Value,
    symbols: &[DocumentSymbol],
    text: &str,
    out: &mut Vec<(String, &'static str, u32)>,
    skipped: &mut usize,
) {
    match value {
        Value::Object(map) => {
            for ((k, v), sym) in map.iter().zip(symbols) {
                if is_leaf(v) {
                    let line = sym.range.start.line;
                    if key_matches_this_line(text, line, k.as_str()) {
                        out.push((k.to_string(), expected_type_phrase(v), line));
                    } else {
                        *skipped += 1;
                    }
                } else if let Some(children) = &sym.children {
                    collect_object_leaves(v, children, text, out, skipped);
                }
            }
        }
        Value::Array(items) => {
            for (v, sym) in items.iter().zip(symbols) {
                if !is_leaf(v) {
                    if let Some(children) = &sym.children {
                        collect_object_leaves(v, children, text, out, skipped);
                    }
                }
            }
        }
        _ => {}
    }
}

fn hover_params(uri: &Url, line: u32) -> HoverParams {
    HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position { line, character: 0 },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

#[test]
fn hover_resolves_object_leaves_and_never_panics() {
    let files = collect_valid_ktav_files();
    assert!(!files.is_empty(), "no valid fixtures found");

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let (service, _socket) = LspService::new(Backend::new);
    let backend = service.inner();
    let uri = Url::parse("file:///editor-features-corpus-fixture.ktav").unwrap();

    let mut panics = Vec::new();
    let mut type_mismatches = Vec::new();
    let mut checked_leaves = 0usize;
    let mut skipped_leaves = 0usize;

    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                panics.push(format!(
                    "{}: ktav::parse failed (should already be caught by \
                     spec_conformance.rs): {e:?}",
                    path.display()
                ));
                continue;
            }
        };

        rt.block_on(backend.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "ktav".to_string(),
                version: 1,
                text: text.clone(),
            },
        }));

        // Full-line no-panic sweep. `hover()` only ever reads
        // `position.line` (never `.character` — the whole line is
        // reclassified via `classify_line`), so one column per line
        // already exercises every reachable branch; one line past EOF
        // exercises the defensive `.nth(..).unwrap_or("")` fallback.
        let num_lines = text.split('\n').count() as u32;
        for line in 0..=num_lines {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                rt.block_on(backend.hover(hover_params(&uri, line)))
            }));
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => panics.push(format!(
                    "{}: hover returned Err at line {line}: {e:?}",
                    path.display()
                )),
                Err(_) => panics.push(format!("{}: hover panicked at line {line}", path.display())),
            }
        }

        // Targeted leaf-type check on object-key leaves only.
        let symbols = build_symbols(&value, &text);
        let mut leaves = Vec::new();
        collect_object_leaves(&value, &symbols, &text, &mut leaves, &mut skipped_leaves);
        for (key, expected, line) in leaves {
            checked_leaves += 1;
            match rt.block_on(backend.hover(hover_params(&uri, line))) {
                Ok(Some(h)) => {
                    let md = match h.contents {
                        HoverContents::Markup(m) => m.value,
                        other => format!("{other:?}"),
                    };
                    if !md.contains(expected) {
                        type_mismatches.push(format!(
                            "{}: key '{key}' at line {line}: expected hover to \
                             mention '{expected}', got {md:?}",
                            path.display()
                        ));
                    }
                }
                Ok(None) => type_mismatches.push(format!(
                    "{}: key '{key}' at line {line}: hover returned None, \
                     expected a value description",
                    path.display()
                )),
                Err(e) => panics.push(format!(
                    "{}: hover returned Err for key '{key}' at line {line}: {e:?}",
                    path.display()
                )),
            }
        }
    }

    assert!(
        panics.is_empty(),
        "hover panicked or errored on {} case(s):\n{}",
        panics.len(),
        panics.join("\n")
    );
    assert!(
        type_mismatches.is_empty(),
        "{} of {} checked object-key leaves: hover did not report the \
         expected type:\n{}",
        type_mismatches.len(),
        checked_leaves,
        type_mismatches.join("\n")
    );
    eprintln!(
        "conformance/hover: {} valid fixtures swept line-by-line for \
         panics, {} object-key leaves type-checked, {} skipped \
         (key_matches_this_line could not confirm the symbol's line — see \
         module doc comment)",
        files.len(),
        checked_leaves,
        skipped_leaves
    );
}

// ---------------------------------------------------------------------------
// Semantic tokens
// ---------------------------------------------------------------------------

#[test]
fn semantic_tokens_are_well_formed_across_valid_corpus() {
    let files = collect_valid_ktav_files();
    assert!(!files.is_empty(), "no valid fixtures found");

    let mut failures = Vec::new();
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let lines: Vec<&str> = text.split('\n').collect();
        let toks = semantic_tokens(&text);

        let mut line = 0u32;
        let mut col = 0u32;
        // (line, end_col) of the previously decoded token.
        let mut prev: Option<(u32, u32)> = None;
        for (i, t) in toks.iter().enumerate() {
            line += t.delta_line;
            col = if t.delta_line == 0 {
                col + t.delta_start
            } else {
                t.delta_start
            };
            let end = col + t.length;

            let Some(line_text) = lines.get(line as usize) else {
                failures.push(format!(
                    "{}: token {i} at line {line} is out of range ({} lines)",
                    path.display(),
                    lines.len()
                ));
                continue;
            };
            if end as usize > line_text.len() {
                failures.push(format!(
                    "{}: token {i} on line {line} spans byte [{col}, {end}) \
                     past the line's own length {}",
                    path.display(),
                    line_text.len()
                ));
            }
            if let Some((prev_line, prev_end)) = prev {
                let ordered = line > prev_line || (line == prev_line && col >= prev_end);
                if !ordered {
                    failures.push(format!(
                        "{}: token {i} at (line {line}, col {col}) is out of \
                         order or overlaps the previous token ending at \
                         (line {prev_line}, col {prev_end})",
                        path.display()
                    ));
                }
            }
            prev = Some((line, end));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} valid fixtures: malformed semantic tokens:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!(
        "conformance/semantic_tokens: {} valid fixtures' tokens are within \
         their line's bounds, ordered, and non-overlapping",
        files.len()
    );
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// `ktav_lsp::reindent::reindent` over every `valid/**/*.ktav` fixture must
/// be idempotent and value-preserving. `*.canonical.ktav` siblings are
/// covered too (`collect_valid_ktav_files` only excludes them from the
/// symbols/hover/semantic-tokens sweeps above, which compare against a
/// fixture's OWN parsed value — irrelevant here, since this test parses
/// each file's own text before and after formatting it).
#[test]
fn format_is_idempotent_and_value_preserving_across_valid_corpus() {
    let root = valid_dir();
    assert!(
        root.is_dir(),
        "spec submodule not initialised at {}",
        root.display()
    );
    let mut files = collect_valid_ktav_files();
    // Re-include `.canonical.ktav` siblings for this sweep (see doc comment).
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("ktav")
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".canonical.ktav"))
                    .unwrap_or(false)
            {
                files.push(path);
            }
        }
    }
    files.sort();
    assert!(!files.is_empty(), "no valid fixtures found");

    let mut failures = Vec::new();
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let formatted = ktav_lsp::reindent::reindent(&text);
        let formatted_twice = ktav_lsp::reindent::reindent(&formatted);
        if formatted != formatted_twice {
            failures.push(format!(
                "{}: format is not idempotent — format(format(x)) != format(x)",
                path.display()
            ));
            continue;
        }

        let orig_value = match ktav::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "{}: ktav::parse(x) failed (should already be caught by \
                     spec_conformance.rs): {e:?}",
                    path.display()
                ));
                continue;
            }
        };
        let formatted_value = match ktav::parse(&formatted) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "{}: format(x) no longer parses: {e:?}",
                    path.display()
                ));
                continue;
            }
        };
        if orig_value != formatted_value {
            failures.push(format!(
                "{}: ktav::parse(format(x)) != ktav::parse(x) — formatting \
                 changed the document's value",
                path.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} valid fixtures: formatter defect:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!(
        "conformance/format: {} valid fixtures format idempotently and \
         value-preservingly",
        files.len()
    );
}
