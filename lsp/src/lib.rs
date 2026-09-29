//! LSP server for the Ktav configuration format.
//!
//! Thin wrapper over the [`ktav`](https://crates.io/crates/ktav) crate:
//! parses on every `did_open`/`did_change`, publishes diagnostics, and
//! answers `hover` / `completion` / `documentSymbol` / `semanticTokens`
//! requests by walking the parsed [`ktav::Value`] tree.

pub mod analysis;
pub mod diagnostics;
pub mod reindent;
pub mod server;
/// Shared line-tokenizer.
///
/// **Unstable internal API.** Exposed `pub` only because integration tests
/// (`tests/*.rs`) treat the LSP crate as an external consumer. External
/// users should not depend on this module — its surface mirrors
/// `ktav::parser` line-shape rules and may change without notice as the
/// parser evolves.
pub mod tokens;

// Keeps `ktav_lsp::semantic` / `ktav_lsp::symbols` working for external
// consumers (integration tests and benches import these paths directly).
pub use analysis::{semantic, symbols};

/// Shared line-splitting model (§ 3.2: LF / CR / CRLF are equivalent).
/// Lives under [`tokens`] (it's the same "single source of truth for
/// line shape" concern) but re-exported at the crate root, both for
/// `crate::lines::...` callers throughout this crate and so integration
/// tests can reach it as `ktav_lsp::lines`.
pub use tokens::lines;

pub use server::Backend;
