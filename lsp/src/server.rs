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
use crate::tokens::{
    byte_to_utf16, classify_line, cursor_is_after_separator, prefix_by_encoding, LineKind,
};

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
        // Format = parse → render. If parse fails (file has syntax errors),
        // we have no canonical Value to render — return None and let the
        // editor leave the buffer untouched. Diagnostics will still flag
        // the underlying error.
        let Some((text, parsed)) = self
            .docs
            .get(&params.text_document.uri)
            .map(|e| (e.text.clone(), e.parsed.clone()))
        else {
            return Ok(None);
        };
        let Some(value) = parsed else {
            // Parsing failed — refuse to format malformed content.
            return Ok(None);
        };

        let formatted = match ktav::render::render(&value) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        if formatted == text {
            // Already canonical — no edit needed.
            return Ok(Some(Vec::new()));
        }

        // Replace whole document with formatted text. Range covers the entire
        // current text — line/column count derived from the original.
        let last_line = text.split('\n').count().saturating_sub(1) as u32;
        let last_col = text
            .split('\n')
            .next_back()
            .map(|s| s.chars().count() as u32)
            .unwrap_or(0);
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

// ---- byte→UTF-16 column conversion (only used when the negotiated
// position encoding is UTF-16) ----

fn convert_diagnostics_to_utf16(diags: &mut [Diagnostic], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    for d in diags {
        convert_position_to_utf16(&mut d.range.start, &lines);
        convert_position_to_utf16(&mut d.range.end, &lines);
    }
}

fn convert_symbols_to_utf16(symbols: &mut [DocumentSymbol], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    fn walk(syms: &mut [DocumentSymbol], lines: &[&str]) {
        for s in syms {
            convert_position_to_utf16(&mut s.range.start, lines);
            convert_position_to_utf16(&mut s.range.end, lines);
            convert_position_to_utf16(&mut s.selection_range.start, lines);
            convert_position_to_utf16(&mut s.selection_range.end, lines);
            if let Some(kids) = s.children.as_mut() {
                walk(kids, lines);
            }
        }
    }
    walk(symbols, &lines);
}

fn convert_position_to_utf16(pos: &mut Position, lines: &[&str]) {
    let line = lines.get(pos.line as usize).copied().unwrap_or("");
    pos.character = byte_to_utf16(line, pos.character as usize);
}

/// Re-encode the absolute (line, start, length) implied by a delta-encoded
/// `SemanticToken` stream from byte offsets to UTF-16 code units.
fn convert_semantic_tokens_to_utf16(toks: &mut [SemanticToken], text: &str) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut abs_line: u32 = 0;
    let mut abs_start_bytes: u32 = 0;
    let mut prev_emit_line: u32 = 0;
    let mut prev_emit_start_u16: u32 = 0;

    for t in toks.iter_mut() {
        // Reconstruct absolute byte position on this token.
        abs_line += t.delta_line;
        if t.delta_line == 0 {
            abs_start_bytes += t.delta_start;
        } else {
            abs_start_bytes = t.delta_start;
        }

        let line = lines.get(abs_line as usize).copied().unwrap_or("");
        let start_bytes = abs_start_bytes as usize;
        let end_bytes = start_bytes + t.length as usize;
        let start_u16 = byte_to_utf16(line, start_bytes);
        let end_u16 = byte_to_utf16(line, end_bytes);

        let new_delta_line = abs_line - prev_emit_line;
        let new_delta_start = if new_delta_line == 0 {
            start_u16 - prev_emit_start_u16
        } else {
            start_u16
        };

        t.delta_line = new_delta_line;
        t.delta_start = new_delta_start;
        t.length = end_u16 - start_u16;

        prev_emit_line = abs_line;
        prev_emit_start_u16 = start_u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::parse_for_diagnostics;
    use crate::semantic::semantic_tokens;

    /// Decode a delta-encoded token stream into absolute
    /// `(line, start_col, length)` triples — easier to assert against.
    fn absolutize(toks: &[SemanticToken]) -> Vec<(u32, u32, u32)> {
        let mut out = Vec::with_capacity(toks.len());
        let mut line: u32 = 0;
        let mut col: u32 = 0;
        for t in toks {
            line += t.delta_line;
            if t.delta_line == 0 {
                col += t.delta_start;
            } else {
                col = t.delta_start;
            }
            out.push((line, col, t.length));
        }
        out
    }

    #[test]
    fn convert_semantic_tokens_to_utf16_multibyte_multiline() {
        // Line 0: ASCII pair so we have a baseline.
        // Line 1: Cyrillic key (each cyrillic letter = 2 bytes UTF-8 / 1 UTF-16 unit).
        // Line 2: emoji in the value (4 bytes UTF-8 / 2 UTF-16 units).
        let text = "name: alice\nимя: bob\ngreeting: 😀\n";

        let mut byte_toks = semantic_tokens(text);
        // Sanity: byte-encoded stream should reflect UTF-8 byte columns.
        let abs_bytes = absolutize(&byte_toks);
        // Expect at least one token per line.
        assert!(abs_bytes.iter().any(|(l, _, _)| *l == 0));
        assert!(abs_bytes.iter().any(|(l, _, _)| *l == 1));
        assert!(abs_bytes.iter().any(|(l, _, _)| *l == 2));

        // Pick the key-token on line 1 (the Cyrillic "имя"). In bytes it
        // is 6 long; in UTF-16 it is 3 long.
        let line1_key_bytes = abs_bytes
            .iter()
            .find(|(l, c, _)| *l == 1 && *c == 0)
            .copied()
            .expect("key token on line 1");
        assert_eq!(line1_key_bytes.2, 6, "cyrillic key in bytes");

        convert_semantic_tokens_to_utf16(&mut byte_toks, text);
        let abs_u16 = absolutize(&byte_toks);

        // Same token, now in UTF-16: column still 0, length 3.
        let line1_key_u16 = abs_u16
            .iter()
            .find(|(l, c, _)| *l == 1 && *c == 0)
            .copied()
            .expect("key token on line 1 (utf16)");
        assert_eq!(line1_key_u16.2, 3, "cyrillic key in utf-16 units");

        // Line 2: the value "😀" is at byte column 10 ("greeting: " = 10
        // bytes). In UTF-16 the value column is also 10 (ASCII prefix),
        // but the *length* is 2 (surrogate pair), not 4 bytes.
        let line2_val_u16 = abs_u16
            .iter()
            .find(|(l, c, len)| *l == 2 && *c == 10 && *len == 2)
            .copied();
        assert!(
            line2_val_u16.is_some(),
            "expected line-2 value token of utf-16 length 2 (emoji surrogate pair); got {:?}",
            abs_u16
        );

        // Cross-line delta invariant: when delta_line > 0, delta_start is
        // an absolute column. This re-deltaing bug used to surface as
        // negative/overshoot deltas — pin it.
        let mut prev_line: u32 = 0;
        let mut prev_col: u32 = 0;
        for t in &byte_toks {
            let cur_line = prev_line + t.delta_line;
            let cur_col = if t.delta_line == 0 {
                prev_col + t.delta_start
            } else {
                t.delta_start
            };
            // No underflow asserts implicitly happened above. Also assert
            // monotonic-by-line column on same-line tokens.
            if t.delta_line == 0 {
                assert!(
                    cur_col >= prev_col,
                    "non-monotonic same-line column: {} < {}",
                    cur_col,
                    prev_col,
                );
            }
            prev_line = cur_line;
            prev_col = cur_col;
        }
    }

    #[test]
    fn diagnostic_range_utf16_cyrillic_key() {
        // Duplicate-key error on a line whose key is Cyrillic. Without
        // UTF-16 conversion the column would be in bytes (each cyrillic
        // letter = 2 bytes); after conversion it must reflect UTF-16 code
        // units (1 unit per BMP cyrillic letter).
        let text = "имя: a\nимя: b\n";
        let mut diags = parse_for_diagnostics(text);
        assert!(!diags.is_empty(), "expected a duplicate-key diagnostic");

        // Find the diagnostic on line 1 (the second occurrence). Some
        // upstream parser versions report on line 0 — accept either, but
        // pin the byte→utf16 conversion below regardless.
        let d = diags
            .iter()
            .find(|d| d.range.start.line == 1)
            .or_else(|| diags.first())
            .cloned()
            .unwrap();

        let byte_start_col = d.range.start.character;
        let byte_end_col = d.range.end.character;

        convert_diagnostics_to_utf16(&mut diags, text);
        let d2 = diags
            .iter()
            .find(|d| d.range.start.line == 1)
            .or_else(|| diags.first())
            .cloned()
            .unwrap();

        // The line text on whichever line the diagnostic points to.
        let line_text: &str = text.lines().nth(d2.range.start.line as usize).unwrap_or("");
        let expected_start = byte_to_utf16(line_text, byte_start_col as usize);
        let expected_end = byte_to_utf16(line_text, byte_end_col as usize);
        assert_eq!(d2.range.start.character, expected_start);
        assert_eq!(d2.range.end.character, expected_end);

        // And: for a Cyrillic-only key the UTF-16 column must be strictly
        // less than the byte column (whenever the byte column is > 0).
        if byte_start_col > 0 {
            assert!(
                d2.range.start.character < byte_start_col,
                "utf-16 col {} should be < byte col {} for cyrillic input",
                d2.range.start.character,
                byte_start_col,
            );
        }
    }

    #[test]
    fn diagnostic_range_utf16_emoji_value() {
        // Value containing an emoji (surrogate pair) on a syntactically
        // invalid line. Use an unclosed brace — guaranteed to produce a
        // diagnostic in current parser versions.
        let text = "greeting: 😀\nbroken: {\n";
        let mut diags = parse_for_diagnostics(text);
        assert!(!diags.is_empty(), "expected a diagnostic");

        // Snapshot pre-conversion byte columns.
        let pre: Vec<_> = diags
            .iter()
            .map(|d| {
                (
                    d.range.start.line,
                    d.range.start.character,
                    d.range.end.character,
                )
            })
            .collect();

        convert_diagnostics_to_utf16(&mut diags, text);

        for (i, d) in diags.iter().enumerate() {
            let (line_no, byte_start, byte_end) = pre[i];
            let line_text = text.lines().nth(line_no as usize).unwrap_or("");
            assert_eq!(
                d.range.start.character,
                byte_to_utf16(line_text, byte_start as usize),
            );
            assert_eq!(
                d.range.end.character,
                byte_to_utf16(line_text, byte_end as usize),
            );
        }
    }
}
