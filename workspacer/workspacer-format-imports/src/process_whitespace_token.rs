// ---------------- [ File: src/process_whitespace_token.rs ]
crate::ix!();

/// If it's whitespace, we check how many newlines.
/// Then we decide if it triggers a blank-line blocking scenario or if we
/// accumulate the newlines, or if we skip a node, etc.
pub fn process_whitespace_token(
    tok: SyntaxToken,
    text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    let newline_count = text.matches('\n').count();
    trace!(
        "process_whitespace_token => newline_count={}, found_comment={}",
        newline_count,
        state.found_comment()
    );

    if *state.found_comment() {
        // If we've started collecting, a blank line (>=2 newlines) stops.
        if newline_count >= 2 {
            debug!("Blank line after collected comment => stop");
            return None;
        }
        *state.whitespace_newlines_since_last_nonws_mut() += newline_count;
        tok.prev_sibling_or_token()
    } else {
        // No comment found yet
        if newline_count < 2 {
            // fewer than 2 newlines => accumulate & move on
            *state.whitespace_newlines_since_last_nonws_mut() += newline_count;
            tok.prev_sibling_or_token()
        } else {
            // 2+ newlines => a blank line => peek next non-whitespace
            debug!("No comment yet => blank line => peek next non-whitespace");
            let above = tok.prev_sibling_or_token();
            if let Some(is_comment) = super::peek_next_non_whitespace(&above) {
                if is_comment {
                    // The next actual token is a comment => block
                    debug!("Blank line => next is comment => block => empty");
                    None
                } else {
                    // Next is a node => skip node plus trailing whitespace
                    debug!("Blank line => next is node => skip => keep going");
                    state.set_whitespace_newlines_since_last_nonws(0);
                    skip_upward_node_with_whitespace(above)
                }
            } else {
                debug!("Blank line => nothing above => empty");
                None
            }
        }
    }
}

#[cfg(test)]
mod test_process_whitespace_token {
    use super::*;
    use ra_ap_syntax::SyntaxKind;

    #[test]
    fn test_whitespace_before_comment_blocking() {
        let mut state = GatherCommentsState::new();
        let tok = SyntaxToken::new_root(SyntaxKind::WHITESPACE, "\n\n".into());
        let ret = process_whitespace_token(tok, "\n\n".into(), &mut state);
        assert!(ret.is_none(), "We expect block => empty, or no further climbing");
        assert!(!state.found_comment());
        assert!(state.collected().is_empty());
    }
}
