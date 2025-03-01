// ---------------- [ File: src/process_upward_node.rs ]
crate::ix!();

/// Handles a `NodeOrToken::Node(...)` by deciding whether to stop or skip upward,
/// based on how many newlines we've seen and whether we've collected any comment yet.
///
/// Returns the *new* element to continue climbing from (the updated `cur`).
pub fn process_upward_node(
    element: NodeOrToken<SyntaxNode, SyntaxToken>,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    trace!(
        "process_upward_node => kind={:?}, newlines={}",
        element.kind(),
        state.whitespace_newlines_since_last_nonws()
    );

    let node = match element {
        NodeOrToken::Node(n) => n,
        NodeOrToken::Token(_) => unreachable!("process_upward_node called with Token"),
    };

    if *state.found_comment() {
        // If we've started collecting, <2 newlines => stop, else skip
        if *state.whitespace_newlines_since_last_nonws() < 2 {
            debug!("Node with <2 newlines => stop collecting");
            None
        } else {
            debug!("Node with >=2 newlines => skip & continue");
            state.set_whitespace_newlines_since_last_nonws(0);
            node.prev_sibling_or_token()
        }
    } else {
        // No comment yet => skip if >=2 newlines, else stop
        if *state.whitespace_newlines_since_last_nonws() >= 2 {
            debug!("No comment yet, 2+ newlines => skip node & continue upward");
            state.set_whitespace_newlines_since_last_nonws(0);
            skip_upward_node_with_whitespace(Some(NodeOrToken::Node(node)))
        } else {
            debug!("Node with <2 newlines => stop => no comments found yet");
            None
        }
    }
}

#[cfg(test)]
mod test_process_upward_node {
    use super::*;
    use ra_ap_syntax::SourceFile;

    #[test]
    fn test_node_simple_stop_no_comment_yet() {
        let mut state = GatherCommentsState::new();
        state.set_whitespace_newlines_since_last_nonws(0); // <2 => we expect a stop
        let parse = SourceFile::parse("struct Foo{}", ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        let node = file.syntax();
        let element = NodeOrToken::Node(node);

        let next = process_upward_node(element, &mut state);
        assert!(next.is_none(), "Expected to stop because <2 newlines, no comment found");
        assert!(state.collected().is_empty(), "No comments were found");
    }
}
