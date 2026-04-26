use tower_lsp::{LspService, Server};
use tracing_subscriber::EnvFilter;

use ktav_lsp::Backend;

#[tokio::main]
async fn main() {
    // stdout is reserved for LSP messages — log to stderr only.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_env("KTAV_LSP_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
