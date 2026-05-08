//! LSP server for the Ktav configuration format.
//!
//! Thin wrapper over the [`ktav`](https://crates.io/crates/ktav) crate:
//! parses on every `did_open`/`did_change`, publishes diagnostics, and
//! answers `hover` / `completion` / `documentSymbol` / `semanticTokens`
//! requests by walking the parsed [`ktav::Value`] tree.

pub mod diagnostics;
pub mod reindent;
pub mod semantic;
pub mod server;
pub mod symbols;
/// Shared line-tokenizer.
///
/// **Unstable internal API.** Exposed `pub` only because integration tests
/// (`tests/*.rs`) treat the LSP crate as an external consumer. External
/// users should not depend on this module — its surface mirrors
/// `ktav::parser` line-shape rules and may change without notice as the
/// parser evolves.
pub mod tokens;

pub use server::Backend;
