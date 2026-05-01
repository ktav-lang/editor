//! Conformance test: walk the language-agnostic Ktav test suite under
//! `<repo>/spec/versions/0.1/tests/` (a git submodule of `ktav-lang/spec`)
//! and exercise the `ktav::parse` reference parser plus the LSP's
//! `parse_for_diagnostics` wrapper against every fixture.
//!
//! Two assertions per side:
//!
//! 1. **Reference parser (`ktav::parse`)**
//!    * `valid/**.ktav` — must parse `Ok(_)`.
//!    * `invalid/**.ktav` — must return `Err(Error::Syntax(...))`. The
//!      free-form message is matched against the expected error
//!      category from the fixture's `.json` oracle (`{"error":"<cat>"}`)
//!      via substring lookup. `ktav 0.1.4` includes the category
//!      verbatim in the message for the categories we currently
//!      support; a missing category in the message is reported but
//!      not fatal — see `MISSING_CATEGORY_IS_FATAL` below.
//!
//! 2. **LSP diagnostic wrapper (`parse_for_diagnostics`)**
//!    * `valid/**.ktav` — diagnostic count must be `0`.
//!    * `invalid/**.ktav` — diagnostic count must be `>= 1` (the
//!      LSP currently surfaces a single diagnostic per parse failure;
//!      we assert "at least one" to stay forward-compatible with a
//!      future multi-error parser).
//!
//! If the spec submodule is not initialised (fresh clone without
//! `--recurse-submodules`), every test in this file logs and returns —
//! local `cargo test` stays green, and CI (with `submodules: recursive`)
//! exercises the full suite.
//!
//! NOTE: this file does NOT compare the parsed `Value` tree against
//! the JSON oracle for `valid/**` — that's the reference Rust crate's
//! responsibility, and the LSP only consumes `ktav::parse` results.
//! Pinning the exact text of every error message is handled separately
//! by `tests/error_format_pinning.rs`. This file is the "did the parser
//! accept/reject the right files" floor.

use std::fs;
use std::path::{Path, PathBuf};

use ktav::Error;

/// If `true`, an `invalid/**` fixture whose error category is NOT
/// found in the parser's message becomes a hard failure. Set to
/// `false` to keep the test green when `ktav` adds a new internal
/// rename without updating the spec — but lose visibility. Today the
/// suite is small enough that we keep this strict.
const MISSING_CATEGORY_IS_FATAL: bool = true;

fn spec_tests_dir() -> Option<PathBuf> {
    // `CARGO_MANIFEST_DIR` for this crate is `<repo>/lsp`, so the
    // submodule lives one level up.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(manifest).join("../spec/versions/0.1/tests");
    if p.join("valid").is_dir() && p.join("invalid").is_dir() {
        Some(p)
    } else {
        None
    }
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

/// Read the sibling `<name>.json` for an invalid fixture and pull out
/// the `"error"` value. The file format is `{"error":"<category>"}`
/// (see `spec/versions/0.1/tests/README.md`). We do a tiny manual scan
/// rather than pulling in `serde_json` — the format is one-line and
/// stable.
fn expected_error_category(ktav_path: &Path) -> Option<String> {
    let json_path = ktav_path.with_extension("json");
    let text = fs::read_to_string(&json_path).ok()?;
    // Look for `"error"` key.
    let after_key = text.split("\"error\"").nth(1)?;
    let after_colon = after_key.split(':').nth(1)?;
    // First quoted string after the colon is the value.
    let q1 = after_colon.find('"')?;
    let rest = &after_colon[q1 + 1..];
    let q2 = rest.find('"')?;
    Some(rest[..q2].to_string())
}

#[test]
fn conformance_reference_parser_valid() {
    let Some(tests_dir) = spec_tests_dir() else {
        eprintln!(
            "skipping: spec submodule not initialised (run \
             `git submodule update --init --recursive`)"
        );
        return;
    };

    let files = collect_ktav_files(&tests_dir.join("valid"));
    assert!(!files.is_empty(), "no valid fixtures found");

    let mut failures = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).expect("read");
        match ktav::parse(&text) {
            Ok(_) => {}
            Err(e) => failures.push(format!("{}: {:?}", path.display(), e)),
        }
    }
    if !failures.is_empty() {
        panic!(
            "{} of {} valid fixtures failed to parse:\n{}",
            failures.len(),
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "conformance/ktav::parse: {} valid fixtures parsed Ok",
        files.len()
    );
}

#[test]
fn conformance_reference_parser_invalid() {
    let Some(tests_dir) = spec_tests_dir() else {
        eprintln!("skipping: spec submodule not initialised");
        return;
    };

    let files = collect_ktav_files(&tests_dir.join("invalid"));
    assert!(!files.is_empty(), "no invalid fixtures found");

    let mut accepted = Vec::new();
    let mut wrong_category = Vec::new();
    let mut missing_category_in_msg = Vec::new();

    for path in &files {
        let text = fs::read_to_string(path).expect("read");
        let expected = expected_error_category(path);
        match ktav::parse(&text) {
            Ok(_) => {
                accepted.push(format!(
                    "{} (expected error '{}')",
                    path.display(),
                    expected.as_deref().unwrap_or("<unknown>")
                ));
            }
            Err(Error::Syntax(msg)) => {
                if let Some(cat) = expected.as_deref() {
                    if !msg.contains(cat) && !category_is_message_aliased(cat, &msg) {
                        // The current `ktav 0.1.4` message format
                        // does not name every category verbatim — but
                        // it does for the ones in the suite as of
                        // this writing. Still, log + collect.
                        missing_category_in_msg.push(format!(
                            "{}: expected '{}' in message, got: {:?}",
                            path.display(),
                            cat,
                            msg
                        ));
                    }
                }
                let _ = wrong_category; // keep slot for future shape mismatches
            }
            Err(other) => {
                wrong_category.push(format!(
                    "{}: expected Syntax error, got {:?}",
                    path.display(),
                    other
                ));
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
    if !wrong_category.is_empty() {
        report.push(format!(
            "{} fixtures returned non-Syntax errors:\n{}",
            wrong_category.len(),
            wrong_category.join("\n")
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

/// Some categories are surfaced under historic / human-friendly names
/// in `ktav 0.1.4`'s error messages rather than the spec's CamelCase
/// constant. Keep this list small and tightly justified — it exists
/// solely to avoid coupling the conformance test to message wording
/// already pinned in `error_format_pinning.rs`.
fn category_is_message_aliased(cat: &str, msg: &str) -> bool {
    // Empirical mapping: spec category → substring(s) actually present
    // in `ktav 0.1.4` `Error::Syntax` messages. Any change in wording on
    // the parser side will surface here AND in `error_format_pinning.rs`
    // (which pins the exact strings) — update both together.
    match cat {
        "DuplicateName" => msg.contains("duplicate key") || msg.contains("Duplicate key"),
        "PathConflict" => msg.contains("conflict at "),
        "EmptyKey" => msg.contains("Empty key"),
        "InvalidKey" => msg.contains("Invalid key"),
        "UnbalancedBracket" => {
            msg.contains("Unclosed")
                || msg.contains("without matching")
                || msg.contains("Unexpected")
        }
        "MismatchedBracket" => {
            msg.contains("does not match the open") || msg.contains("Mismatched")
        }
        "OrphanLine" => msg.contains("no ':'") || msg.contains("Orphan") || msg.contains("orphan"),
        "InlineNonEmptyCompound" => msg.contains("inline ") || msg.contains("Inline "),
        _ => false,
    }
}

#[test]
fn conformance_lsp_diagnostics_valid() {
    let Some(tests_dir) = spec_tests_dir() else {
        eprintln!("skipping: spec submodule not initialised");
        return;
    };

    let files = collect_ktav_files(&tests_dir.join("valid"));
    let mut failures = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).expect("read");
        let diags = ktav_lsp::diagnostics::parse_for_diagnostics(&text);
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
            files.len(),
            failures.join("\n")
        );
    }
    eprintln!(
        "conformance/parse_for_diagnostics: {} valid fixtures produced no diagnostics",
        files.len()
    );
}

#[test]
fn conformance_lsp_diagnostics_invalid() {
    let Some(tests_dir) = spec_tests_dir() else {
        eprintln!("skipping: spec submodule not initialised");
        return;
    };

    let files = collect_ktav_files(&tests_dir.join("invalid"));
    let mut failures = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).expect("read");
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
