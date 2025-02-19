crate::ix!();

/// Returns true if any ancestor is a `mod` that has `#[cfg(test)]`.
pub fn is_in_test_module(mut node: SyntaxNode) -> bool {
    use ra_ap_syntax::SyntaxKind::{MODULE, SOURCE_FILE};
    while node.kind() != SOURCE_FILE {
        if node.kind() == MODULE {
            if has_cfg_test_attr(&node) {
                return true;
            }
        }
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            break;
        }
    }
    false
}
