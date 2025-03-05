// ---------------- [ File: workspacer-format-imports/src/skip_upward_node_with_whitespace.rs ]
crate::ix!();

pub fn skip_upward_node_with_whitespace(
    mut cur: Option<NodeOrToken<SyntaxNode, SyntaxToken>>,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!("skip_upward_node_with_whitespace => entered");
    if let Some(ref e) = cur {
        debug!("Starting from element kind={:?}", e.kind());
    } else {
        debug!("Starting from None");
    }

    // First skip the node itself
    if let Some(NodeOrToken::Node(_)) = cur {
        trace!("Skipping the node");
        cur = cur.and_then(|x| x.prev_sibling_or_token());
    }

    // Then skip any whitespace tokens above that node
    while let Some(n_or_t) = &cur {
        trace!("Examining preceding sibling/token => kind={:?}", n_or_t.kind());
        if let NodeOrToken::Token(tok) = n_or_t {
            if tok.kind() == SyntaxKind::WHITESPACE {
                debug!("Skipping whitespace token");
                cur = tok.prev_sibling_or_token();
                continue;
            }
        }
        break;
    }

    debug!("Final element after skipping => {:?}", cur.as_ref().map(|x| x.kind()));
    info!("skip_upward_node_with_whitespace => done");
    cur
}

#[cfg(test)]
mod test_skip_upward_node_with_whitespace {
    use super::*;

    #[traced_test]
    fn test_skip_node() {
        info!("test_skip_node => start");
        let src = "struct Foo {}\n\n";
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        let struct_node = file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::STRUCT)
            .expect("Expected a struct node in 'struct Foo {}'");

        let cur = Some(NodeOrToken::Node(struct_node));
        debug!("Calling skip_upward_node_with_whitespace");
        let result = skip_upward_node_with_whitespace(cur);

        assert!(
            result.is_none(),
            "Expected None after skipping node + preceding whitespace"
        );
        info!("test_skip_node => success");
    }
}
