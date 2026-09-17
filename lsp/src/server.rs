//! `tower_lsp::LanguageServer` impl. Holds an in-memory document store
//! (one entry per open `Url`) and re-parses on every change.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use ktav::Value;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics::parse_for_diagnostics;
use crate::semantic::{semantic_tokens, token_types};
use crate::symbols::build_symbols;
use crate::tokens::{classify_line, cursor_is_after_separator, prefix_by_encoding, LineKind};
mod completion;
mod formatting;
mod hover;
mod utf16;

use completion::value_item;
use formatting::end_of_document;
use hover::{describe_value, lookup_dotted};
use utf16::{
    convert_diagnostics_to_utf16, convert_semantic_tokens_to_utf16, convert_symbols_to_utf16,
};

#[cfg(test)]
mod tests;

/// Negotiated position-encoding. Stored as `AtomicU8` on [`Backend`] so the
/// async handlers can read it without locking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionEncoding {
    /// Byte offsets — emitted directly, no conversion.
    Utf8,
    /// LSP default — bytes must be converted to UTF-16 code units.
    Utf16,
}

impl PositionEncoding {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => PositionEncoding::Utf8,
            _ => PositionEncoding::Utf16,
        }
    }
    fn as_u8(self) -> u8 {
        match self {
            PositionEncoding::Utf8 => 0,
            PositionEncoding::Utf16 => 1,
        }
    }
}

/// Document store entry: the latest known text and its `did_change`
/// version, used to drop stale diagnostic publishes (race in fix #4).
///
/// `parsed` is the eagerly-cached `ktav::parse(&text)` result, populated
/// by [`DocEntry::new`] on `did_open` / `did_change` / `did_save`. Hover
/// and `documentSymbol` consume the cache instead of re-parsing the full
/// document on every keystroke / hover. The parse cost was already paid
/// for diagnostics on the same edit, so caching is essentially free.
///
/// `Arc<Value>` keeps clones cheap when the entry is read out of the
/// `DashMap`. `None` means the document failed to parse — handlers
/// degrade gracefully (hover returns a generic label, `documentSymbol`
/// returns an empty list).
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub version: i32,
    pub text: String,
    pub parsed: Option<Arc<Value>>,
}

impl DocEntry {
    /// Build a fresh entry, eagerly parsing once. The result is shared
    /// across all readers via `Arc`.
    pub fn new(version: i32, text: String) -> Self {
        let parsed = ktav::parse(&text).ok().map(Arc::new);
        Self {
            version,
            text,
            parsed,
        }
    }
}

/// Backend state — one per running server process.
pub struct Backend {
    client: Client,
    /// Latest text + version per open document. Wrapped in `Arc` so the
    /// async refresh task can read without holding a `DashMap` shard lock.
    docs: Arc<DashMap<Url, DocEntry>>,
    /// Negotiated position encoding (UTF-8 if the client advertised it,
    /// otherwise UTF-16). Encoded as `u8` for `AtomicU8` storage.
    encoding: Arc<AtomicU8>,
}

impl Backend {
    /// Wire up a fresh backend bound to an LSP `Client`.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(DashMap::new()),
            // Default until `initialize` negotiates.
            encoding: Arc::new(AtomicU8::new(PositionEncoding::Utf16.as_u8())),
        }
    }

    /// Read the negotiated position encoding.
    pub fn encoding(&self) -> PositionEncoding {
        PositionEncoding::from_u8(self.encoding.load(Ordering::Relaxed))
    }

    /// Re-parse `text` and publish diagnostics, but only if `version`
    /// is still the latest for `uri`. If a newer `did_change` raced ahead
    /// while we were parsing, the publish is dropped — the next refresh
    /// (triggered by that newer change) will overwrite.
    async fn refresh_diagnostics(&self, uri: Url, text: String, version: Option<i32>) {
        let mut diags = parse_for_diagnostics(&text);
        if self.encoding() == PositionEncoding::Utf16 {
            convert_diagnostics_to_utf16(&mut diags, &text);
        }

        // Stale-publish guard: if the doc has been updated to a newer
        // version since this refresh started, skip publishing.
        if let Some(v) = version {
            if let Some(entry) = self.docs.get(&uri) {
                if entry.version > v {
                    return;
                }
            }
        }
        self.client.publish_diagnostics(uri, diags, version).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> RpcResult<InitializeResult> {
        // Negotiate position encoding: prefer UTF-8 if the client
        // advertised it (modern clients do — vscode-languageclient ≥ 9,
        // helix, neovim builtin lsp). Otherwise fall back to UTF-16,
        // which is the LSP default and what older clients expect.
        let advertised = params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_ref());
        let chosen = match advertised {
            Some(list) if list.contains(&PositionEncodingKind::UTF8) => PositionEncoding::Utf8,
            _ => PositionEncoding::Utf16,
        };
        self.encoding.store(chosen.as_u8(), Ordering::Relaxed);

        let semantic_legend = SemanticTokensLegend {
            token_types: token_types(),
            token_modifiers: vec![],
        };

        let position_encoding = match chosen {
            PositionEncoding::Utf8 => Some(PositionEncodingKind::UTF8),
            PositionEncoding::Utf16 => Some(PositionEncodingKind::UTF16),
        };

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ktav-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                position_encoding,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
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
        // Returning Ok immediately. Client-side hangs around `client.stop()`
        // (notably VS Code's languageclient) are the client's timeout to
        // enforce — there is nothing for the server to do here.
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        let version = params.text_document.version;
        self.docs
            .insert(uri.clone(), DocEntry::new(version, text.clone()));
        self.refresh_diagnostics(uri, text, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        // We advertised FULL sync, so the first change carries the whole file.
        if let Some(change) = params.content_changes.into_iter().next() {
            self.docs
                .insert(uri.clone(), DocEntry::new(version, change.text.clone()));
            self.refresh_diagnostics(uri, change.text, Some(version))
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            let prior_version = self
                .docs
                .get(&params.text_document.uri)
                .map(|e| e.version)
                .unwrap_or(0);
            let entry = DocEntry::new(prior_version, text.clone());
            let v = entry.version;
            self.docs.insert(params.text_document.uri.clone(), entry);
            self.refresh_diagnostics(params.text_document.uri, text, Some(v))
                .await;
        } else if let Some(entry) = self.docs.get(&params.text_document.uri) {
            let text = entry.text.clone();
            let v = entry.version;
            drop(entry);
            self.refresh_diagnostics(params.text_document.uri, text, Some(v))
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
        let Some((text, parsed)) = self
            .docs
            .get(uri)
            .map(|e| (e.text.clone(), e.parsed.clone()))
        else {
            return Ok(None);
        };

        let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
        // Route through the shared classifier so dotted keys, `:: literal`
        // array-items, comments and brace-only lines behave consistently
        // with semantic tokens / diagnostics.
        let key: &str = match classify_line(line) {
            LineKind::Pair {
                key_start,
                key_length,
                ..
            } => {
                let s = key_start as usize;
                let e = s + key_length as usize;
                line.get(s..e).unwrap_or("").trim()
            }
            LineKind::Comment { .. }
            | LineKind::Blank
            | LineKind::CloseBrace { .. }
            | LineKind::RawArrayItem { .. }
            | LineKind::ArrayItem { .. } => return Ok(None),
        };
        if key.is_empty() {
            return Ok(None);
        }

        // Read from the cached parse populated on `did_open`/`did_change`
        // — avoids re-running the full parser on every hover request.
        let value_info = parsed
            .as_deref()
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
        let Some(text) = self.docs.get(uri).map(|e| e.text.clone()) else {
            return Ok(None);
        };

        let line = text.split('\n').nth(pos.line as usize).unwrap_or("");
        // `pos.character` is in the negotiated encoding (UTF-8 bytes or
        // UTF-16 code units) — slice accordingly so non-ASCII lines work.
        let upto = prefix_by_encoding(line, pos.character, self.encoding());

        // After "key: " or "key:" — offer value-shape completions.
        if !cursor_is_after_separator(upto) {
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
            value_item(
                ":",
                ":",
                "literal-string marker — second `:`, value is a literal string",
            ),
        ];

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> RpcResult<Option<DocumentSymbolResponse>> {
        let Some((text, parsed)) = self
            .docs
            .get(&params.text_document.uri)
            .map(|e| (e.text.clone(), e.parsed.clone()))
        else {
            return Ok(None);
        };
        // Cached parse — same `Arc<Value>` populated on `did_open`/
        // `did_change`. If parsing failed, return an empty outline.
        let Some(value) = parsed else {
            return Ok(Some(DocumentSymbolResponse::Nested(Vec::new())));
        };
        let mut symbols = build_symbols(&value, &text);
        if self.encoding() == PositionEncoding::Utf16 {
            convert_symbols_to_utf16(&mut symbols, &text);
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> RpcResult<Option<Vec<TextEdit>>> {
        // Format = line-based canonical re-indent. We DO NOT round-trip
        // through `parse → render(value)` because that would discard
        // user-controlled formatting that the parser doesn't store in
        // the `Value` tree:
        //   * blank lines (visual section separators)
        //   * `#` comments
        //   * choice of multi-line form (`( ... )` vs `(( ... ))`)
        //
        // Instead [`crate::reindent::reindent`] walks the source line by
        // line, tracks nesting depth from structural tokens, and emits
        // each line at canonical depth. Inside `( ... )` / `(( ... ))`
        // blocks lines are copied verbatim.
        //
        // Format runs the text-level reindent unconditionally — it
        // operates on lines (no parse needed) and one of its jobs is
        // exactly to auto-fix forms that the parser rejects, e.g.
        // `name: (value)` → `name:: (value)`. After that, the result
        // is canonical Ktav and parses cleanly. If the document has
        // unrelated syntax errors (unclosed compound, bad typed
        // literal, etc.) reindent still produces a sensible re-indent
        // and the underlying error stays in diagnostics.
        let Some(text) = self
            .docs
            .get(&params.text_document.uri)
            .map(|e| e.text.clone())
        else {
            return Ok(None);
        };

        let formatted = crate::reindent::reindent(&text);

        if formatted == text {
            // Already canonical — no edit needed.
            return Ok(Some(Vec::new()));
        }

        // Replace whole document with formatted text. Range covers the
        // entire current text — line/column derived from the original, in
        // the negotiated position encoding (see `end_of_document`).
        let (last_line, last_col) = end_of_document(&text, self.encoding());
        let edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: last_line,
                    character: last_col,
                },
            },
            new_text: formatted,
        };
        Ok(Some(vec![edit]))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> RpcResult<Option<SemanticTokensResult>> {
        let Some(text) = self
            .docs
            .get(&params.text_document.uri)
            .map(|e| e.text.clone())
        else {
            return Ok(None);
        };
        let mut data = semantic_tokens(&text);
        if self.encoding() == PositionEncoding::Utf16 {
            convert_semantic_tokens_to_utf16(&mut data, &text);
        }
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }
}
