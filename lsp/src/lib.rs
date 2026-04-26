//! LSP server for the Ktav configuration format.
//!
//! Thin wrapper over the [`ktav`](https://crates.io/crates/ktav) crate:
//! parses on every `did_open`/`did_change`, publishes diagnostics, and
//! answers `hover` / `completion` / `documentSymbol` / `semanticTokens`
//! requests by walking the parsed [`ktav::Value`] tree.

pub mod diagnostics;
pub mod semantic;
pub mod server;
pub mod symbols;

pub use server::Backend;
