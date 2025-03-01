// ---------------- [ File: src/dispatch_token_by_kind.rs ]
crate::ix!();

/// Given a `SyntaxToken`, dispatch to the correct subroutine based on `tok.kind()`.
pub fn dispatch_token_by_kind(
    tok: SyntaxToken,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    use SyntaxKind::*;
    let text = tok.text().to_string();

    match tok.kind() {
        COMMENT => collect_comment_token(tok, text, state),
        WHITESPACE => process_whitespace_token(tok, text, state),
        _ => process_other_token(tok, text, state),
    }
}

#[cfg(test)]
mod test_dispatch_token_by_kind {
    use super::*;
    use ra_ap_syntax::SyntaxKind;

    #[test]
    fn test_dispatch_comment() {
        let mut state = GatherCommentsState::new();
        let tok = SyntaxToken::new_root(SyntaxKind::COMMENT, "// hey".into());
        let returned = dispatch_token_by_kind(tok.clone(), &mut state);
        assert!(state.found_comment(), "Should detect a comment");
        assert_eq!(state.collected().len(), 1);
        assert_eq!(returned, tok.prev_sibling_or_token(), "Should return climbing point");
    }

    #[test]
    fn test_dispatch_whitespace() {
        let mut state = GatherCommentsState::new();
        let tok = SyntaxToken::new_root(SyntaxKind::WHITESPACE, "\n".into());
        let returned = dispatch_token_by_kind(tok.clone(), &mut state);
        assert!(!state.found_comment());
        assert_eq!(returned, tok.prev_sibling_or_token());
    }

    #[test]
    fn test_dispatch_other() {
        let mut state = GatherCommentsState::new();
        let tok = SyntaxToken::new_root(SyntaxKind::IDENT, "foo".into());
        let returned = dispatch_token_by_kind(tok.clone(), &mut state);
        assert!(!state.found_comment());
        assert_eq!(returned, tok.prev_sibling_or_token());
    }
}
