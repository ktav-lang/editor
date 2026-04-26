//! `tower_lsp::LanguageServer` impl. Holds an in-memory document store
//! (one entry per open `Url`) and re-parses on every change.

use std::sync::Arc;

use dashmap::DashMap;
use ktav::Value;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics::parse_for_diagnostics;
use crate::semantic::{semantic_tokens, token_types};
use crate::symbols::build_symbols;

/// Backend state — one per running server process.
pub struct Backend {
    client: Client,
    docs: Arc<DashMap<Url, String>>,
}

impl Backend {
    /// Wire up a fresh backend bound to an LSP `Client`.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(DashMap::new()),
        }
    }

    async fn refresh_diagnostics(&self, uri: Url, text: &str, version: Option<i32>) {
        let diags = parse_for_diagnostics(text);
        self.client.publish_diagnostics(uri, diags, version).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> RpcResult<InitializeResult> {
        let semantic_legend = SemanticTokensLegend {
            token_types: token_types(),
            token_modifiers: vec![],
        };

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ktav-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_legend,
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("ktav-lsp ready");
    }

    async fn shutdown(&self) -> RpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        self.docs.insert(uri.clone(), text.clone());
        self.refresh_diagnostics(uri, &text, Some(params.text_document.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        // We advertised FULL sync, so the first change carries the whole file.
        if let Some(change) = params.content_changes.into_iter().next() {
            self.docs.insert(uri.clone(), change.text.clone());
            self.refresh_diagnostics(uri, &change.text, Some(params.text_document.version))
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.docs
                .insert(params.text_document.uri.clone(), text.clone());
            self.refresh_diagnostics(params.text_document.uri, &text, None)
                .await;
        } else if let Some(entry) = self.docs.get(&params.text_document.uri) {
            let text = entry.clone();
            drop(entry);
            self.refresh_diagnostics(params.text_document.uri, &text, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.docs.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> RpcResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(text) = self.docs.get(uri).map(|t| t.clone()) else {
            return Ok(None);
        };

        let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
        let trimmed = line.trim_start();

        // Comment line — no hover.
        if trimmed.starts_with('#') {
            return Ok(None);
        }

        let Some(colon) = trimmed.find(':') else {
            return Ok(None);
        };
        let key = trimmed[..colon].trim();
        if key.is_empty() {
            return Ok(None);
        }

        let parsed = ktav::parse(&text).ok();
        let value_info = parsed
            .as_ref()
            .and_then(|v| lookup_dotted(v, key))
            .map(describe_value)
            .unwrap_or_else(|| "value".to_string());

        let md = format!("**{}**\n\n_{}_", key, value_info);
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> RpcResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(text) = self.docs.get(uri).map(|t| t.clone()) else {
            return Ok(None);
        };

        let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
        let upto: String = line.chars().take(pos.character as usize).collect();
        let trimmed = upto.trim_start();

        // After "key: " or "key:" — offer value-shape completions.
        let after_separator = trimmed
            .find(':')
            .map(|i| {
                let after = &trimmed[i + 1..];
                // raw marker uses `::` — treat the second `:` itself as still in-key territory.
                if let Some(rest) = after.strip_prefix(':') {
                    rest.trim_start().is_empty() || rest.chars().all(char::is_whitespace)
                } else {
                    after.trim_start().is_empty()
                        || after.chars().all(|c| c == ' ' || c == '\t' || c == ':')
                }
            })
            .unwrap_or(false);

        if !after_separator {
            return Ok(None);
        }

        let items = vec![
            value_item("null", "null", "the null keyword"),
            value_item("true", "true", "boolean true"),
            value_item("false", "false", "boolean false"),
            value_item("{", "{", "open multi-line object"),
            value_item("}", "}", "close multi-line object"),
            value_item("[", "[", "open multi-line array"),
            value_item("]", "]", "close multi-line array"),
            value_item("{}", "{}", "empty inline object"),
            value_item("[]", "[]", "empty inline array"),
            value_item("(", "(", "open multi-line raw block"),
            value_item("((", "((", "open verbatim raw block"),
            value_item("()", "()", "empty raw value"),
            value_item(":", ":", "raw-marker — second `:`, value is literal string"),
            value_item("i", "i", "typed integer marker (use as `:i value`)"),
            value_item("f", "f", "typed float marker (use as `:f value`)"),
        ];

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> RpcResult<Option<DocumentSymbolResponse>> {
        let Some(text) = self.docs.get(&params.text_document.uri).map(|t| t.clone()) else {
            return Ok(None);
        };
        let Ok(value) = ktav::parse(&text) else {
            return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
        };
        let symbols = build_symbols(&value, &text);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> RpcResult<Option<SemanticTokensResult>> {
        let Some(text) = self.docs.get(&params.text_document.uri).map(|t| t.clone()) else {
            return Ok(None);
        };
        let data = semantic_tokens(&text);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }
}

fn value_item(label: &str, insert: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::VALUE),
        detail: Some(detail.to_string()),
        insert_text: Some(insert.to_string()),
        ..Default::default()
    }
}

/// Walk a dotted path through a [`Value::Object`] tree.
fn lookup_dotted<'a>(root: &'a Value, dotted: &str) -> Option<&'a Value> {
    let mut cur = root;
    for seg in dotted.split('.') {
        let Value::Object(map) = cur else { return None };
        cur = map.get(seg)?;
    }
    Some(cur)
}

fn describe_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => format!("bool: `{}`", b),
        Value::Integer(s) => format!("integer (typed): `{}`", s.as_str()),
        Value::Float(s) => format!("float (typed): `{}`", s.as_str()),
        Value::String(s) => {
            let shown = if s.len() > 80 {
                format!("{}…", &s[..80])
            } else {
                s.to_string()
            };
            format!("string: `{}`", shown)
        }
        Value::Array(a) => format!("array of {} items", a.len()),
        Value::Object(o) => format!("object with {} keys", o.len()),
    }
}
