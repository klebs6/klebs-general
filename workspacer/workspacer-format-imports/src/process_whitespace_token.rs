// ---------------- [ File: src/process_whitespace_token.rs ]
crate::ix!();

pub fn process_whitespace_token(
    tok: SyntaxToken,
    text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!(
        "process_whitespace_token => start; text length={}, found_comment={}",
        text.len(),
        state.found_comment()
    );

    // Normalize \r\n into \n so single newlines are counted consistently.
    let text_no_cr = text.replace('\r', "");
    let newline_count = text_no_cr.matches('\n').count();
    trace!(
        "Calculated newline_count={}, found_comment={}",
        newline_count,
        state.found_comment()
    );

    // We keep single newlines to keep climbing so we can detect any "directly above" comment 
    // that might be behind this whitespace.  The user’s “tests” want that behavior.
    let result = if *state.found_comment() {
        if newline_count >= 2 {
            warn!("Blank line after collected comment => stop => returning None");
            None
        } else {
            debug!(
                "Already have comment => single newline => keep climbing => +{}",
                newline_count
            );
            *state.whitespace_newlines_since_last_nonws_mut() += newline_count;
            tok.prev_sibling_or_token()
        }
    } else {
        if newline_count >= 2 {
            warn!("No comment yet => blank line => block => returning None");
            None
        } else {
            debug!(
                "No comment => single newline => keep climbing => +{}",
                newline_count
            );
            *state.whitespace_newlines_since_last_nonws_mut() += newline_count;
            tok.prev_sibling_or_token()
        }
    };

    debug!(
        "process_whitespace_token => end; returning: {}",
        if result.is_none() {
            "None".to_string()
        } else {
            format!("Some({:?})", result.as_ref().unwrap().kind())
        }
    );
    info!("process_whitespace_token => done");
    result
}

#[cfg(test)]
mod test_process_whitespace_token {
    use super::*;

    #[traced_test]
    fn test_whitespace_before_comment_blocking() {
        info!("test_whitespace_before_comment_blocking => start");
        let mut state = GatherCommentsState::new();

        // We'll start with 2 newlines, then a comment, to see if the whitespace
        // triggers blocking logic (>=2 newlines, next is comment).
        let src = "\n\n//some comment";
        debug!("Parsing source: {:?}", src);

        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        trace!("Parsed file => checking tokens");

        // The first token should be the whitespace with 2 newlines
        let ws_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::WHITESPACE && t.text().matches('\n').count() == 2)
            .expect("Expected a WHITESPACE token with 2 newlines");

        debug!("Found whitespace token => calling process_whitespace_token");
        let result = process_whitespace_token(ws_tok.clone(), ws_tok.text().to_string(), &mut state);

        assert!(result.is_none(), "Blank line => next is comment => block => empty");
        assert!(!state.found_comment(), "No comment should be recorded");
        assert!(state.collected().is_empty(), "No comments collected");
        info!("test_whitespace_before_comment_blocking => success");
    }
}
