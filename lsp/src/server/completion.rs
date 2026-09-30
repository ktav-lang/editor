//! Value-shape completion items offered after a `key:` separator.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub(super) fn value_item(
    label: &str,
    insert: &str,
    detail: &str,
    whitespace: &str,
) -> CompletionItem {
    let mut insert_text = String::with_capacity(insert.len() + whitespace.len());
    insert_text.push_str(whitespace);
    insert_text.push_str(insert);
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::VALUE),
        detail: Some(detail.to_string()),
        insert_text: Some(insert_text),
        ..Default::default()
    }
}
