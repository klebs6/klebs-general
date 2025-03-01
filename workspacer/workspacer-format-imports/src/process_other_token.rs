// ---------------- [ File: src/process_other_token.rs ]
crate::ix!();

/// If it's neither COMMENT nor WHITESPACE, we handle it here.
/// We check whether we've started collecting or not, and skip/stop accordingly.
pub fn process_other_token(
    tok: SyntaxToken,
    _text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    trace!(
        "process_other_token => kind={:?}, newlines={}",
        tok.kind(),
        state.whitespace_newlines_since_last_nonws()
    );

    if *state.found_comment() {
        // Already collecting => <2 newlines => stop, else skip
        if *state.whitespace_newlines_since_last_nonws() < 2 {
            debug!("Non-comment token with <2 newlines => stop collecting");
            None
        } else {
            debug!("Non-comment token with >=2 newlines => skip & continue");
            state.set_whitespace_newlines_since_last_nonws(0);
            tok.prev_sibling_or_token()
        }
    } else {
        // No comment yet => skip if >=2 newlines, else stop
        if *state.whitespace_newlines_since_last_nonws() >= 2 {
            debug!("No comment yet, 2+ newlines => skip token & continue upward");
            state.set_whitespace_newlines_since_last_nonws(0);
            tok.prev_sibling_or_token()
        } else {
            debug!("No comment yet, <2 newlines => stop");
            None
        }
    }
}

#[cfg(test)]
mod test_process_other_token {
    use super::*;
    use ra_ap_syntax::SyntaxKind;

    #[test]
    fn test_other_token_no_comment_stop() {
        let mut state = GatherCommentsState::new();
        state.set_whitespace_newlines_since_last_nonws(1); // <2 => should stop
        let pretend_tok = SyntaxToken::new_root(SyntaxKind::IDENT, "foo".into());
        let result = process_other_token(pretend_tok.clone(), "foo".into(), &mut state);
        assert!(result.is_none(), "Should stop if <2 newlines, no comment yet");
        assert!(!state.found_comment());
        assert_eq!(state.collected().len(), 0);
    }
}
