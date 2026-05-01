//! Pinning tests for `ktav::Error::Syntax(String)` message shapes.
//!
//! `diagnostics.rs` recovers structured information (line number, key
//! segment, error category) by **substring-matching** the free-form
//! error message returned by `ktav 0.1.4`. The moment a future `ktav`
//! release renames `MissingSeparatorSpace` → `MissingSep` or reformats
//! `"Line N: duplicate key '<k>'"`, the LSP's range tightening will
//! silently regress to whole-line ranges with no compile-time error.
//!
//! These tests assert the EXACT formatted string produced by a real
//! `ktav::parse` call on a known-bad input — one fixture per category
//! `diagnostics.rs` looks at. When `cargo test` fails here, update both
//! the assertions AND the corresponding `msg.contains(...)` branches in
//! `diagnostics.rs` together.
//!
//! All fixtures use the real parser; no mocks (per project memory rules).

use ktav::Error;

fn err(text: &str) -> String {
    match ktav::parse(text) {
        Err(Error::Syntax(m)) => m,
        Err(other) => panic!("expected Syntax error, got {:?}", other),
        Ok(_) => panic!("expected parse to fail for: {:?}", text),
    }
}

#[test]
fn pin_missing_separator_space() {
    let msg = err("key:value\n");
    assert_eq!(
        msg,
        "Line 1: MissingSeparatorSpace: separator must be followed by whitespace or end of line"
    );
}

#[test]
fn pin_invalid_typed_scalar() {
    let msg = err("port:i abc\n");
    assert!(
        msg.starts_with("Line 1: InvalidTypedScalar: "),
        "got: {msg}"
    );
}

#[test]
fn pin_duplicate_key() {
    let msg = err("port: 80\nport: 443\n");
    assert_eq!(msg, "Line 2: duplicate key 'port'");
}

#[test]
fn pin_key_path_conflict() {
    // Top-level `db` is a string, then `db.x` tries to descend → conflict.
    let msg = err("db: 1\ndb.x: 2\n");
    assert!(
        msg.starts_with("Line 2: conflict at 'db.x'")
            && msg.contains("an existing value blocks the path"),
        "got: {msg}"
    );
}

#[test]
fn pin_empty_key() {
    let msg = err(": value\n");
    assert_eq!(msg, "Empty key at line 1");
}

#[test]
fn pin_invalid_key() {
    // Trailing dot is an invalid key.
    let msg = err("a.: 1\n");
    assert!(msg.starts_with("Invalid key at line 1: '"), "got: {msg}");
}

#[test]
fn pin_unclosed() {
    let msg = err("obj: {\n  a: 1\n");
    assert_eq!(msg, "Unclosed object at end of input");
}
