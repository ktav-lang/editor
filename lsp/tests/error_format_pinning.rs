//! Pinning tests for the `ktav` error message *Display* contract.
//!
//! `ktav 0.1.5+` returns `Error::Structured(ErrorKind)` with a tight
//! source span instead of a free-form `Error::Syntax(String)`, but the
//! Display impl on `ErrorKind` reproduces byte-for-byte what
//! `Error::Syntax(format!(…))` produced in `0.1.4`. These tests pin
//! that Display contract — when `cargo test` fails here, the parser's
//! error wording has drifted and any downstream consumer that
//! string-matches (legacy bindings, log scrapers) needs to be checked.
//!
//! The contract is the rendered string, not the enum variant: this
//! file deliberately accepts either `Error::Syntax(s)` or
//! `Error::Structured(k)` and asserts on `s` / `k.to_string()` alike.
//!
//! All fixtures use the real parser; no mocks (per project memory rules).

use ktav::Error;

fn err(text: &str) -> String {
    match ktav::parse(text) {
        Err(Error::Syntax(m)) => m,
        Err(Error::Structured(k)) => k.to_string(),
        Err(other) => panic!("expected Syntax/Structured error, got {:?}", other),
        Ok(_) => panic!("expected parse to fail for: {:?}", text),
    }
}

#[test]
fn pin_missing_separator_space() {
    // First line forces Object root (per spec 0.1.1 a bare
    // `key:value` at the document start parses as a top-level Array
    // string item — by spec design — so we anchor with a real pair).
    let msg = err("anchor: 1\nkey:value\n");
    assert_eq!(
        msg,
        "Line 2: MissingSeparatorSpace: separator must be followed by whitespace or end of line"
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
    // First line forces Object root (per spec 0.1.1 a bare `: value`
    // at the document start parses as a top-level Array string item).
    let msg = err("anchor: 1\n: value\n");
    assert_eq!(msg, "Empty key at line 2");
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
