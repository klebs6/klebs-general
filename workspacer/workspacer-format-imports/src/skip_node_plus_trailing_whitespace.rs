// ---------------- [ File: workspacer-format-imports/src/skip_node_plus_trailing_whitespace.rs ]
crate::ix!();

/// Skip exactly one node plus any *preceding* whitespace tokens, returning
/// the next `NodeOrToken` above. This is used when:
/// - We see a blank line
/// - The next non-whitespace is a node (not a comment)
/// - We want to “skip” that node and keep climbing upward.
///
/// We also skip any whitespace tokens directly above that node.
pub fn skip_node_plus_trailing_whitespace(mut cur: Option<NodeOrToken<SyntaxNode, SyntaxToken>>)
    -> Option<NodeOrToken<SyntaxNode, SyntaxToken>>
{
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
