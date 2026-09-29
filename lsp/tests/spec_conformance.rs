//! Conformance test: walk the language-agnostic Ktav test suite under
//! `<repo>/spec/versions/0.8/tests/` (a git submodule of `ktav-lang/spec`)
//! and exercise the `ktav` reference crate (`ktav::parse` /
//! `ktav::parse_strict`) plus the LSP's `parse_for_diagnostics` wrapper
//! against every fixture. Pinned to spec 0.8.0 / `ktav = "0.8"`.
//!
//! Per-category contract:
//!
//! * **`valid/**.ktav`** (including `*.canonical.ktav`) — must parse
//!   `Ok(_)` via `ktav::parse` and yield zero LSP diagnostics.
//! * **`invalid/**.ktav`** — must return `Err` via `ktav::parse` whose
//!   `ErrorEnvelope` category matches the fixture's `.json` oracle
//!   (`{"expected_error": "<category>"}`), and yield `>= 1` LSP
//!   diagnostic. A non-UTF-8 fixture is read as raw bytes and must
//!   itself expect `InvalidUtf8` (§ 6.15).
//! * **`parseable-unrepresentable/`** — syntactically valid but not
//!   writer-representable (e.g. a CR byte decoded from an escape):
//!   must parse `Ok(_)` via `ktav::parse` with zero diagnostics.
//! * **`strict-lossy/`** — must parse `Ok(_)` in the lax entry point
//!   (`ktav::parse`) but `Err` in `ktav::parse_strict`, whose
//!   `ErrorEnvelope` category matches the oracle's `expected_error`
//!   (always `LossyScalar` — a lexical form, like `5e3`, whose
//!   canonical spelling differs from what it lexically reads as).
//! * **`unrepresentable/`** — writer-side JSON-only fixtures (no
//!   `.ktav` companion, e.g. a scalar or `NaN` document root); there is
//!   nothing for a parser to run, so only the oracle's own schema is
//!   validated.
//!
//! The corpus manifest (`manifest.json`) is validated before any
//! fixture runs: its `schema_version`, the exact category-name set, and
//! each category's pinned fixture count must match what this file
//! expects — see `validate_manifest`. A missing spec submodule (a
//! fresh clone without `--recurse-submodules`) is a hard failure, not a
//! silent skip: CI always checks out submodules recursively, so a
//! green run that quietly skipped the corpus would prove nothing.
//!
//! NOTE: this file does NOT compare the parsed `Value` tree against the
//! JSON oracle for `valid/**` — that's the reference Rust crate's own
//! test suite's job; the LSP only ever consumes `ktav::parse`'s result.
//! Pinning the exact text of every error message is handled separately
//! by `tests/error_format_pinning.rs`. This file is the "did the parser
//! accept/reject the right files, in every corpus category" floor.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// If `true`, an `invalid/**` fixture whose error category is NOT
/// found in the parser's message becomes a hard failure. Set to
/// `false` to keep the test green when `ktav` adds a new internal
/// rename without updating the spec — but lose visibility. Today the
/// suite is small enough that we keep this strict.
const MISSING_CATEGORY_IS_FATAL: bool = true;

/// Corpus manifest schema version this file understands (see
/// `manifest.json`'s own `$comment`). An unrecognised schema is a hard
/// failure rather than a guess at an unfamiliar shape.
const SUPPORTED_MANIFEST_SCHEMA: u64 = 1;

/// Pinned fixture count per category, kept in lockstep with
/// `manifest.json`'s `categories.<name>.count`. Growing the corpus
/// must update the count here in the same change that adds the
/// fixture — deliberate friction so a runner can never silently run
/// fewer cases than the corpus defines (or more, from a category that
/// grew without anyone noticing).
const EXPECTED_CATEGORIES: [(&str, u64); 5] = [
    ("valid", 223),
    ("invalid", 74),
    ("unrepresentable", 5),
    ("parseable-unrepresentable", 4),
    ("strict-lossy", 13),
];

/// `valid/**` (including `*.canonical.ktav`) must always be UTF-8: an
/// invalid byte sequence is itself one of the `invalid` category's own
/// error conditions (§ 6.15 `InvalidUtf8`), so it can never
/// simultaneously be a `valid` fixture. This is asserted rather than
/// assumed — see `read_valid_fixture_texts`.
const EXPECTED_NON_UTF8_VALID_FIXTURES: usize = 0;

/// Locate the pinned spec 0.8.0 conformance corpus and validate its
/// manifest. Panics with a clear message if the `spec` submodule was
/// not checked out, instead of the old behaviour of every test quietly
/// `return`ing early (green locally on a shallow clone, and — had CI
/// ever lost `submodules: recursive` — silently green there too).
fn spec_tests_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // Spec 0.8.0 conformance corpus, matching the `ktav = "0.8"` floor.
    let root = Path::new(manifest_dir).join("../spec/versions/0.8/tests");
    assert!(
        root.join("valid").is_dir() && root.join("invalid").is_dir(),
        "spec submodule not initialised at {} — run \
         `git submodule update --init --recursive`. CI always checks out \
         submodules recursively; this test suite fails rather than skips \
         so a missing corpus can never pass as green.",
        root.display()
    );
    validate_manifest(&root);
    root
}

/// Validate `manifest.json`'s schema version, category-name set, and
/// per-category fixture counts against `EXPECTED_CATEGORIES`, and cross
/// -check the category directories actually on disk. A category
/// rename/add/remove, or fixture-count drift, fails here — loudly and
/// specifically — before any per-fixture test even runs.
fn validate_manifest(root: &Path) {
    let manifest_path = root.join("manifest.json");
    let bytes = fs::read(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let manifest: serde_json::Value =
        serde_json::from_slice(&bytes).expect("manifest.json is not valid JSON");

    let schema_version = manifest["schema_version"].as_u64();
    assert_eq!(
        schema_version,
        Some(SUPPORTED_MANIFEST_SCHEMA),
        "unsupported corpus manifest schema_version {:?} — this test suite \
         understands schema {}",
        schema_version,
        SUPPORTED_MANIFEST_SCHEMA
    );

    let categories = manifest["categories"]
        .as_object()
        .expect("manifest.json missing 'categories' object");
    let expected_names: HashSet<&str> = EXPECTED_CATEGORIES.iter().map(|(n, _)| *n).collect();
    let actual_names: HashSet<&str> = categories.keys().map(String::as_str).collect();
    assert_eq!(
        actual_names, expected_names,
        "corpus category set changed (added/removed/renamed) — a runner \
         must handle the new set before this can pass"
    );

    for (name, expected_count) in EXPECTED_CATEGORIES {
        let manifest_count = categories[name]["count"].as_u64();
        assert_eq!(
            manifest_count,
            Some(expected_count),
            "manifest.json count for '{name}' is {manifest_count:?}, \
             expected {expected_count} — update EXPECTED_CATEGORIES here \
             in the same change that grows the corpus"
        );
    }

    let actual_dirs: HashSet<String> = fs::read_dir(root)
        .expect("read corpus root")
        .map(|e| e.expect("read corpus root entry").path())
        .filter(|p| p.is_dir())
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let expected_dirs: HashSet<String> = expected_names.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        actual_dirs, expected_dirs,
        "corpus category directories on disk do not match manifest.json"
    );
}

fn expected_count(category: &str) -> u64 {
    EXPECTED_CATEGORIES
        .iter()
        .find(|(name, _)| *name == category)
        .unwrap_or_else(|| panic!("no expected count registered for category '{category}'"))
        .1
}

fn collect_ktav_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(it) => it,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("ktav") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Read every `valid/**.ktav` fixture (`collect_ktav_files` already
/// includes `*.canonical.ktav` — `Path::extension` reports `ktav` for
/// both) as text, asserting the number that are not valid UTF-8
/// matches `EXPECTED_NON_UTF8_VALID_FIXTURES` instead of either
/// silently dropping such a file or aborting the whole test on the
/// first `fs::read_to_string` failure with an unrelated-looking panic.
fn read_valid_fixture_texts(files: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut out = Vec::with_capacity(files.len());
    let mut non_utf8 = 0usize;
    for path in files {
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        match String::from_utf8(bytes) {
            Ok(text) => out.push((path.clone(), text)),
            Err(_) => non_utf8 += 1,
        }
    }
    assert_eq!(
        non_utf8, EXPECTED_NON_UTF8_VALID_FIXTURES,
        "found {non_utf8} non-UTF-8 file(s) under valid/** — a valid Ktav \
         document can never be invalid UTF-8 (§ 6.15 InvalidUtf8 is an \
         `invalid`-category error), so this is either corpus corruption \
         or a spec change this test needs updating for"
    );
    out
}

/// Read the sibling `<name>.json` for an invalid fixture and pull out
/// the expected category. The 0.8 format is `{"expected_error":"<category>", ...}`
/// (see `spec/versions/0.8/tests/README.md`). We do a tiny manual scan
/// rather than pulling in full JSON parsing for this one call site —
/// the format is one-line and stable.
fn expected_error_category(ktav_path: &Path) -> Option<String> {
    let json_path = ktav_path.with_extension("json");
    let text = fs::read_to_string(&json_path).ok()?;
    // 0.8 oracles use `expected_error`; older corpora used `error`.
    let after_key = text
        .split("\"expected_error\"")
        .nth(1)
        .or_else(|| text.split("\"error\"").nth(1))?;
    let after_colon = after_key.split(':').nth(1)?;
    let q1 = after_colon.find('"')?;
    let rest = &after_colon[q1 + 1..];
    let q2 = rest.find('"')?;
    Some(rest[..q2].to_string())
}

fn read_json_oracle(ktav_or_json_path: &Path) -> serde_json::Value {
    let json_path = ktav_or_json_path.with_extension("json");
    let text = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", json_path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", json_path.display()))
}

/// An invalid fixture whose raw bytes are not UTF-8 (§ 6.15) is rejected
/// at the decoding boundary: an LSP client only ever sends UTF-8 text.
fn read_invalid_fixture(path: &Path) -> Option<String> {
    let bytes = fs::read(path).expect("read");
    match String::from_utf8(bytes) {
        Ok(text) => Some(text),
        Err(_) => {
            assert_eq!(
                expected_error_category(path).as_deref(),
                Some("InvalidUtf8"),
                "{}: a non-UTF-8 fixture must expect InvalidUtf8",
                path.display()
            );
            None
        }
    }
}

#[test]
fn conformance_reference_parser_valid() {
    let tests_dir = spec_tests_dir();

    let files = collect_ktav_files(&tests_dir.join("valid"));
    assert!(!files.is_empty(), "no valid fixtures found");
    let texts = read_valid_fixture_texts(&files);

    let mut failures = Vec::new();
    for (path, text) in &texts {
        match ktav::parse(text) {
            Ok(_) => {}
            Err(e) => failures.push(format!("{}: {:?}", path.display(), e)),
        }
    }
    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixtures failed to parse:\n{}",
            failures.len(),
            texts.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "conformance/ktav::parse: {} valid fixtures parsed Ok",
        texts.len()
    );
}

#[test]
fn conformance_reference_parser_invalid() {
    let tests_dir = spec_tests_dir();

    let files = collect_ktav_files(&tests_dir.join("invalid"));
    assert!(!files.is_empty(), "no invalid fixtures found");
    assert_eq!(
        files.len() as u64,
        expected_count("invalid"),
        "wrong invalid fixture count"
    );

    let mut accepted = Vec::new();
    let mut missing_category_in_msg = Vec::new();

    for path in &files {
        let Some(text) = read_invalid_fixture(path) else {
            continue;
        };
        let expected = expected_error_category(path);
        match ktav::parse(&text) {
            Ok(_) => {
                accepted.push(format!(
                    "{} (expected error '{}')",
                    path.display(),
                    expected.as_deref().unwrap_or("<unknown>")
                ));
            }
            Err(e) => {
                // The structured envelope names the category exactly
                // (`ErrorEnvelope::error`); no message-substring guessing.
                let envelope = ktav::ErrorEnvelope::from_error(&e, &text);
                if let Some(cat) = expected.as_deref() {
                    if envelope.error != cat {
                        missing_category_in_msg.push(format!(
                            "{}: expected '{}', envelope reports '{}' ({:?})",
                            path.display(),
                            cat,
                            envelope.error,
                            e.to_string()
                        ));
                    }
                }
            }
        }
    }

    let mut report = Vec::new();
    if !accepted.is_empty() {
        report.push(format!(
            "{} invalid fixtures were accepted by the parser:\n{}",
            accepted.len(),
            accepted.join("\n")
        ));
    }
    if MISSING_CATEGORY_IS_FATAL && !missing_category_in_msg.is_empty() {
        report.push(format!(
            "{} fixtures did not include the expected category in the \
             error message:\n{}",
            missing_category_in_msg.len(),
            missing_category_in_msg.join("\n")
        ));
    }
    if !report.is_empty() {
        panic!("{}", report.join("\n---\n"));
    }
    eprintln!(
        "conformance/ktav::parse: {} invalid fixtures rejected with the \
         expected error category",
        files.len()
    );
}

#[test]
fn conformance_lsp_diagnostics_valid() {
    let tests_dir = spec_tests_dir();

    let files = collect_ktav_files(&tests_dir.join("valid"));
    let texts = read_valid_fixture_texts(&files);
    let mut failures = Vec::new();
    for (path, text) in &texts {
        let diags = ktav_lsp::diagnostics::parse_for_diagnostics(text);
        if !diags.is_empty() {
            failures.push(format!(
                "{}: expected 0 diagnostics, got {}: {:?}",
                path.display(),
                diags.len(),
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            ));
        }
    }
    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixtures produced diagnostics:\n{}",
            failures.len(),
            texts.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "conformance/parse_for_diagnostics: {} valid fixtures produced no diagnostics",
        texts.len()
    );
}

#[test]
fn conformance_lsp_diagnostics_invalid() {
    let tests_dir = spec_tests_dir();

    let files = collect_ktav_files(&tests_dir.join("invalid"));
    let mut failures = Vec::new();
    for path in &files {
        let Some(text) = read_invalid_fixture(path) else {
            continue;
        };
        let diags = ktav_lsp::diagnostics::parse_for_diagnostics(&text);
        if diags.is_empty() {
            failures.push(format!(
                "{}: expected >= 1 diagnostic, got 0",
                path.display()
            ));
        }
    }
    if !failures.is_empty() {
        panic!(
            "{} of {} invalid fixtures produced no diagnostics:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "conformance/parse_for_diagnostics: {} invalid fixtures produced \
         >= 1 diagnostic each",
        files.len()
    );
}

/// `parseable-unrepresentable/` fixtures are syntactically valid Ktav
/// that a conforming writer could never re-emit (e.g. a String
/// containing a raw CR byte, decoded from an inline-compound escape).
/// They must still parse cleanly through both the reference parser and
/// the LSP's diagnostic wrapper — "unrepresentable" is a writer-side
/// concern only.
#[test]
fn conformance_parseable_unrepresentable_parses_cleanly() {
    let tests_dir = spec_tests_dir();
    let dir = tests_dir.join("parseable-unrepresentable");
    let files = collect_ktav_files(&dir);
    assert_eq!(
        files.len() as u64,
        expected_count("parseable-unrepresentable"),
        "wrong parseable-unrepresentable fixture count"
    );

    let mut failures = Vec::new();
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        match ktav::parse(&text) {
            Ok(_) => {}
            Err(e) => {
                failures.push(format!("{}: ktav::parse failed: {:?}", path.display(), e));
                continue;
            }
        }
        let diags = ktav_lsp::diagnostics::parse_for_diagnostics(&text);
        if !diags.is_empty() {
            failures.push(format!(
                "{}: expected 0 diagnostics, got {:?}",
                path.display(),
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} parseable-unrepresentable fixtures failed:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!(
        "conformance: {} parseable-unrepresentable fixtures parsed cleanly",
        files.len()
    );
}

/// `strict-lossy/` fixtures (e.g. `build: 5e3`, canonical `5000.0`) are
/// accepted by the lax entry point (`ktav::parse`, which infers the
/// canonical number and discards the lossy spelling) but must be
/// rejected by `ktav::parse_strict`, which refuses to silently rewrite
/// a lexical form that differs from its own canonical spelling. The
/// oracle's `expected_error` names the exact category `parse_strict`
/// must report (always `LossyScalar` for this category today, but this
/// test reads it from the oracle rather than hard-coding it).
#[test]
fn conformance_strict_lossy_rejected_by_parse_strict() {
    let tests_dir = spec_tests_dir();
    let dir = tests_dir.join("strict-lossy");
    let files = collect_ktav_files(&dir);
    assert_eq!(
        files.len() as u64,
        expected_count("strict-lossy"),
        "wrong strict-lossy fixture count"
    );

    let mut failures = Vec::new();
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if let Err(e) = ktav::parse(&text) {
            failures.push(format!(
                "{}: ktav::parse (lax) unexpectedly failed: {:?}",
                path.display(),
                e
            ));
            continue;
        }
        let oracle = read_json_oracle(path);
        let expected = oracle["expected_error"]
            .as_str()
            .unwrap_or_else(|| panic!("{}: oracle missing expected_error", path.display()));
        match ktav::parse_strict(&text) {
            Ok(_) => failures.push(format!(
                "{}: ktav::parse_strict unexpectedly accepted a strict-lossy fixture \
                 (expected '{}')",
                path.display(),
                expected
            )),
            Err(e) => {
                let envelope = ktav::ErrorEnvelope::from_error(&e, &text);
                if envelope.error != expected {
                    failures.push(format!(
                        "{}: parse_strict expected '{}', got '{}' ({:?})",
                        path.display(),
                        expected,
                        envelope.error,
                        e.to_string()
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} strict-lossy fixtures failed:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
    eprintln!(
        "conformance: {} strict-lossy fixtures accepted by ktav::parse and \
         rejected by ktav::parse_strict with the expected category",
        files.len()
    );
}

/// `unrepresentable/` holds writer-side JSON-only fixtures (no `.ktav`
/// companion — there is no document a parser could run on a value like
/// a bare scalar root or `NaN`). Only the oracle's own schema is
/// validated: `value` present, `unrepresentable_reason` one of the
/// three reasons available to this category, `note` a non-empty
/// string.
#[test]
fn conformance_unrepresentable_oracle_schema() {
    let tests_dir = spec_tests_dir();
    let dir = tests_dir.join("unrepresentable");
    let files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.expect("read dir entry").path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    assert_eq!(
        files.len() as u64,
        expected_count("unrepresentable"),
        "wrong unrepresentable fixture count"
    );

    const VALID_REASONS: [&str; 3] = ["ScalarRoot", "EmptyKeyName", "NonFiniteFloat"];
    for path in &files {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let oracle: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: invalid JSON: {e}", path.display()));
        let object = oracle
            .as_object()
            .unwrap_or_else(|| panic!("{}: oracle must be a JSON object", path.display()));
        let expected_fields: HashSet<&str> = ["value", "unrepresentable_reason", "note"]
            .into_iter()
            .collect();
        assert_eq!(
            object.keys().map(String::as_str).collect::<HashSet<_>>(),
            expected_fields,
            "{}: oracle fields changed",
            path.display()
        );
        assert!(
            object.contains_key("value"),
            "{}: oracle missing 'value'",
            path.display()
        );
        let reason = object
            .get("unrepresentable_reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{}: oracle missing unrepresentable_reason", path.display()));
        assert!(
            VALID_REASONS.contains(&reason),
            "{}: unrepresentable_reason '{}' is not one of {:?}",
            path.display(),
            reason,
            VALID_REASONS
        );
        let note = object.get("note").and_then(serde_json::Value::as_str);
        assert!(
            matches!(note, Some(n) if !n.is_empty()),
            "{}: oracle missing non-empty 'note'",
            path.display()
        );
    }
    eprintln!(
        "conformance: {} unrepresentable oracle(s) schema-validated",
        files.len()
    );
}

/// A `valid/**.ktav` fixture (excluding `*.canonical.ktav` re-renders,
/// which duplicate the same `Value` tree) whose lexical form overflows
/// i64/f64 range: `ktav::parse` demotes these to `Value::String` (no
/// `::` marker involved — see § 5.2), but `semantic_tokens` colours the
/// surface spelling as a Number regardless of magnitude (highlighting is
/// lexical, not semantic — the doc comment on
/// `ktav_lsp::tokens::looks_numeric` spells this out explicitly). That
/// is a deliberate, pinned divergence between "how many Number/Bool/Null
/// leaves the parsed tree has" and "how many Number/Keyword/Null tokens
/// the highlighter emits" — the only one — so these five fixtures are
/// excluded from `conformance_semantic_tokens_leaf_counts_match_parse`
/// rather than silently tolerated by a fuzzy count.
const SEMANTIC_TOKEN_LEAF_COUNT_EXCEPTIONS: [&str; 5] = [
    "numbers/float/just_above_max_finite_to_string.ktav",
    "numbers/float/negative_overflow_to_string.ktav",
    "numbers/float/positive_overflow_to_string.ktav",
    "numbers/integer/big_overflow_to_string.ktav",
    "numbers/integer/i64_overflow_to_string.ktav",
];

/// Count of Number-shaped (`Integer`/`Float`), Bool, and Null leaves in a
/// parsed `Value` tree — the semantic-tokens counterpart to
/// `count_semantic_leaf_tokens`.
fn count_value_leaves(v: &ktav::Value, counts: &mut (usize, usize, usize)) {
    match v {
        ktav::Value::Integer(_) | ktav::Value::Float(_) => counts.0 += 1,
        ktav::Value::Bool(_) => counts.1 += 1,
        ktav::Value::Null => counts.2 += 1,
        ktav::Value::String(_) => {}
        ktav::Value::Array(items) => {
            for item in items {
                count_value_leaves(item, counts);
            }
        }
        ktav::Value::Object(map) => {
            for (_, v) in map {
                count_value_leaves(v, counts);
            }
        }
    }
}

/// Count Number / Keyword (bool) / "null" tokens `semantic_tokens`
/// emits for `text`. Token-type indices are resolved from
/// `token_types()` itself (not hard-coded) so this test cannot silently
/// drift from `analysis::semantic`'s own index assignment.
fn count_semantic_leaf_tokens(text: &str) -> (usize, usize, usize) {
    use tower_lsp::lsp_types::SemanticTokenType;

    let types = ktav_lsp::semantic::token_types();
    let index_of = |want: &SemanticTokenType| {
        types
            .iter()
            .position(|t| t == want)
            .expect("token_types() must contain this type") as u32
    };
    let number_idx = index_of(&SemanticTokenType::NUMBER);
    let keyword_idx = index_of(&SemanticTokenType::KEYWORD);
    let null_idx = index_of(&SemanticTokenType::new("null"));

    let mut counts = (0usize, 0usize, 0usize);
    for tok in ktav_lsp::semantic::semantic_tokens(text) {
        if tok.token_type == number_idx {
            counts.0 += 1;
        } else if tok.token_type == keyword_idx {
            counts.1 += 1;
        } else if tok.token_type == null_idx {
            counts.2 += 1;
        }
    }
    counts
}

/// Regression guard for the multi-line-block and escaped-inline-key
/// semantic-token defects: on every `valid/**.ktav` fixture (skipping
/// `*.canonical.ktav` re-renders and the pinned overflow exceptions
/// above), the number of Number/Bool/Null LEAVES in the parsed `Value`
/// tree must equal the number of Number/Keyword/"null" TOKENS
/// `semantic_tokens` emits. Before the fix this failed on, at least,
/// `multiline/stripped_basic.ktav` (a `"qwe": 1` content line inside a
/// `(` block was mis-tokenized as a real pair, adding a spurious Number
/// token) and `key_escaping/escaped_open_bracket_in_inline_pair_key_after_comma.ktav`
/// (an escaped `\[` in an inline key desynced the inline scanner enough
/// to swallow a real Number token into a String run).
#[test]
fn conformance_semantic_tokens_leaf_counts_match_parse() {
    let tests_dir = spec_tests_dir();
    let valid_dir = tests_dir.join("valid");
    let files: Vec<PathBuf> = collect_ktav_files(&valid_dir)
        .into_iter()
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("ktav"))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| !n.ends_with(".canonical.ktav"))
                .unwrap_or(false)
        })
        .collect();
    assert!(!files.is_empty(), "no valid fixtures found");

    let exceptions: HashSet<PathBuf> = SEMANTIC_TOKEN_LEAF_COUNT_EXCEPTIONS
        .iter()
        .map(|rel| valid_dir.join(rel))
        .collect();
    for rel in &exceptions {
        assert!(
            rel.is_file(),
            "pinned exception {} no longer exists in the corpus — update \
             SEMANTIC_TOKEN_LEAF_COUNT_EXCEPTIONS",
            rel.display()
        );
    }

    let mut checked = 0usize;
    let mut mismatches = Vec::new();
    for path in &files {
        if exceptions.contains(path) {
            continue;
        }
        let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let Ok(value) = ktav::parse(&text) else {
            continue;
        };

        let mut leaf_counts = (0usize, 0usize, 0usize);
        count_value_leaves(&value, &mut leaf_counts);
        let token_counts = count_semantic_leaf_tokens(&text);

        if leaf_counts != token_counts {
            mismatches.push(format!(
                "{}: parsed leaves (number={}, keyword={}, null={}) != \
                 semantic tokens (number={}, keyword={}, null={})",
                path.display(),
                leaf_counts.0,
                leaf_counts.1,
                leaf_counts.2,
                token_counts.0,
                token_counts.1,
                token_counts.2
            ));
        }
        checked += 1;
    }

    assert!(
        mismatches.is_empty(),
        "{} of {} valid fixtures disagree on Number/Bool/Null leaf vs. \
         token counts:\n{}",
        mismatches.len(),
        checked,
        mismatches.join("\n")
    );
    eprintln!(
        "conformance/semantic_tokens: {} valid fixtures agree on \
         Number/Bool/Null leaf vs. token counts ({} exceptions skipped)",
        checked,
        exceptions.len()
    );
}
