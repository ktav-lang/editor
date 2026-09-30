//! One test binary for the in-process end-to-end suites: text goes through
//! the public modules (diagnostics, tokens, symbols, formatter) with the
//! real `ktav::parse` — no mocks.

mod integration;
mod lsp_context;
mod structured_diagnostics;
