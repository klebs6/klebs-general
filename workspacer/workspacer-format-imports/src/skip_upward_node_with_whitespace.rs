// ---------------- [ File: src/skip_upward_node_with_whitespace.rs ]
crate::ix!();

/// Skip exactly one node plus any *preceding* whitespace tokens, returning
/// the next `NodeOrToken` above. Used when we see a blank line => next is a node => skip.
pub fn skip_upward_node_with_whitespace(
    mut cur: Option<NodeOrToken<SyntaxNode, SyntaxToken>>,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    // First skip the node itself
    if let Some(NodeOrToken::Node(_)) = cur {
        cur = cur.and_then(|x| x.prev_sibling_or_token());
    }
    // Then skip any whitespace tokens above that node
    while let Some(n_or_t) = &cur {
        if let NodeOrToken::Token(tok) = n_or_t {
            if tok.kind() == SyntaxKind::WHITESPACE {
                cur = tok.prev_sibling_or_token();
                continue;
            }
        }
        // If it's not a whitespace token, break
        break;
    }
    cur
}

#[cfg(test)]
mod test_skip_upward_node_with_whitespace {
    use super::*;
    use ra_ap_syntax::{SyntaxNode, SyntaxKind, NodeOrToken};

    #[test]
    fn test_skip_node() {
        // Synthetic scenario: node => ws => (end).
        // We want skip_upward_node_with_whitespace => None.
        let node = SyntaxNode::new_root(SyntaxKind::STRUCT, []);
        let ws = SyntaxToken::new_root(SyntaxKind::WHITESPACE, "\n\n".into());
        node.insert_slots(0, &[NodeOrToken::Token(ws.clone())]);

        let cur = Some(NodeOrToken::Node(node));
        let result = skip_upward_node_with_whitespace(cur);
        assert!(result.is_none());
    }
}
