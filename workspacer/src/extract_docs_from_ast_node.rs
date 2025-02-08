// ---------------- [ File: src/extract_docs_from_ast_node.rs ]
crate::ix!();

pub fn extract_docs(node: &SyntaxNode) -> Option<String> {
    let doc_comments = node
        .children_with_tokens()
        .filter_map(|child| child.into_token().and_then(|token| {
            if token.kind() == ra_ap_syntax::SyntaxKind::COMMENT {
                // Only collect doc comments (/// or /** */)
                let text = token.text().to_string();
                if text.starts_with("///") || text.starts_with("/**") {
                    Some(text)
                } else {
                    None
                }
            } else {
                None
            }
        }))
        .collect::<Vec<_>>();

    if doc_comments.is_empty() {
        None
    } else {
        Some(doc_comments.join("\n"))
    }
}
