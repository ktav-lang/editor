//! Value-shape completion items offered after a `key:` separator.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub(super) fn value_item(label: &str, insert: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::VALUE),
        detail: Some(detail.to_string()),
        insert_text: Some(insert.to_string()),
        ..Default::default()
    }
}
