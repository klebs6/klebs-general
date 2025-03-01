// ---------------- [ File: src/collect_comment_token.rs ]
crate::ix!();

/// If it's a comment token, collect it, reset newlines, and climb upward.
pub fn collect_comment_token(
    tok: SyntaxToken,
    text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    let line = if text.ends_with('\n') {
        text
    } else {
        format!("{}\n", text)
    };
    trace!("collect_comment_token => line={:?}", line);
    state.collected_mut().insert(0, line);
    state.set_found_comment(true);
    state.set_whitespace_newlines_since_last_nonws(0);

    tok.prev_sibling_or_token()
}

#[cfg(test)]
mod test_collect_comment_token {
    use super::*;

    #[test]
    fn test_collect_comment_basic() {
        let mut state = GatherCommentsState::new();
        let tok = SyntaxToken::new_root(SyntaxKind::COMMENT, "// hi".into());
        let ret = collect_comment_token(tok.clone(), "// hi".into(), &mut state);
        assert!(state.found_comment());
        assert_eq!(state.collected().len(), 1);
        assert_eq!(state.collected()[0], "// hi\n");
        assert_eq!(*state.whitespace_newlines_since_last_nonws(), 0);
        assert_eq!(ret, tok.prev_sibling_or_token());
    }
}

