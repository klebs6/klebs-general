// ---------------- [ File: workspacer-format-imports/src/process_upward_node.rs ]
crate::ix!();

pub fn process_upward_node(
    element: NodeOrToken<SyntaxNode, SyntaxToken>,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!("process_upward_node => start");
    trace!(
        "element.kind()={:?}, whitespace_newlines_since_last_nonws={}",
        element.kind(),
        state.whitespace_newlines_since_last_nonws()
    );

    let node = match element {
        NodeOrToken::Node(n) => {
            debug!("Got a Node => continuing");
            n
        }
        NodeOrToken::Token(_) => {
            error!("process_upward_node called with a Token => unreachable");
            return None;
        }
    };

    // If the node is the top-level file node, we skip it so that a single newline
    // between file start and our `fn` or `use` doesn't block comment collection.
    if node.kind() == SyntaxKind::SOURCE_FILE {
        info!("Skipping SOURCE_FILE node => keep climbing");
        state.set_whitespace_newlines_since_last_nonws(0);
        let ret = node.prev_sibling_or_token();
        debug!("process_upward_node => returning after skipping SOURCE_FILE => {:?}", ret.as_ref().map(|x| x.kind()));
        return ret;
    }

    let newlines = *state.whitespace_newlines_since_last_nonws();
    let result = if *state.found_comment() {
        // If we've started collecting, <2 newlines => stop; else skip
        if newlines < 2 {
            debug!("Found comment, node with <2 newlines => stop collecting => return None");
            None
        } else {
            debug!("Found comment, node with >=2 newlines => skip node & continue upward");
            state.set_whitespace_newlines_since_last_nonws(0);
            node.prev_sibling_or_token()
        }
    } else {
        // No comment yet => skip if >=2 newlines; else stop
        if newlines >= 2 {
            debug!("No comment yet, 2+ newlines => skip node & continue upward");
            state.set_whitespace_newlines_since_last_nonws(0);
            skip_upward_node_with_whitespace(Some(NodeOrToken::Node(node)))
        } else {
            debug!("Node with <2 newlines => stop => no comments found yet => return None");
            None
        }
    };

    debug!(
        "process_upward_node => end; returning: {}",
        if result.is_none() {
            "None".to_string()
        } else {
            format!("Some({:?})", result.as_ref().unwrap().kind())
        }
    );
    info!("process_upward_node => done");
    result
}

#[cfg(test)]
mod test_process_upward_node {
    use super::*;

    #[traced_test]
    fn test_node_simple_stop_no_comment_yet() {
        info!("test_node_simple_stop_no_comment_yet => start");
        let mut state = GatherCommentsState::new();
        state.set_whitespace_newlines_since_last_nonws(0);

        let parse = SourceFile::parse("struct Foo{}", ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        trace!("Parsed a file containing 'struct Foo{{}}'");

        let node = file.syntax().clone();
        debug!("Using the entire file.syntax() as our node");

        let element = NodeOrToken::Node(node);
        let next = process_upward_node(element, &mut state);

        assert!(next.is_none(), "Expected to stop because <2 newlines, no comment found");
        assert!(state.collected().is_empty(), "No comments should have been found");
        info!("test_node_simple_stop_no_comment_yet => success");
    }
}
