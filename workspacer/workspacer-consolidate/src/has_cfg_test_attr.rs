crate::ix!();

/// Detects if the given `SyntaxNode` has an attribute of the form `#[cfg(test)]`.
pub fn has_cfg_test_attr(node: &SyntaxNode) -> bool {
    use ra_ap_syntax::ast::{Attr, HasAttrs};
    for child_attr in node.children().filter_map(Attr::cast) {
        if let Some(meta) = child_attr.meta() {
            if let Some(path_node) = meta.path() {
                if path_node.syntax().text().to_string().contains("cfg") {
                    let tokens = meta.token_tree().map(|tt| tt.to_string()).unwrap_or_default();
                    if tokens.contains("test") {
                        return true;
                    }
                }
            }
        }
    }
    false
}
