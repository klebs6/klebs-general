// ---------------- [ File: src/process_other_token.rs ]
crate::ix!();

pub fn process_other_token(
    tok: SyntaxToken,
    text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!("process_other_token => start");
    trace!(
        "kind={:?}, newlines={}, text length={}",
        tok.kind(),
        state.whitespace_newlines_since_last_nonws(),
        text.len()
    );

    let newlines = *state.whitespace_newlines_since_last_nonws();
    let result = if *state.found_comment() {
        // Already collecting => <2 newlines => stop, else skip
        if newlines < 2 {
            debug!("Already found comment => <2 newlines => stop collecting => None");
            None
        } else {
            debug!("Already found comment => >=2 newlines => skip & continue => resetting to 0");
            state.set_whitespace_newlines_since_last_nonws(0);
            tok.prev_sibling_or_token()
        }
    } else {
        // No comment yet => skip if >=2 newlines, else stop
        if newlines >= 2 {
            debug!("No comment yet => >=2 newlines => skip token & continue => resetting to 0");
            state.set_whitespace_newlines_since_last_nonws(0);
            tok.prev_sibling_or_token()
        } else {
            debug!("No comment yet => <2 newlines => stop => None");
            None
        }
    };

    debug!(
        "process_other_token => end; returning {}",
        if result.is_none() {
            "None".to_string()
        } else {
            format!("Some({:?})", result.as_ref().unwrap().kind())
        }
    );
    info!("process_other_token => done");
    result
}

#[cfg(test)]
mod test_process_other_token {
    use super::*;

    #[traced_test]
    fn test_other_token_no_comment_stop() {
        info!("test_other_token_no_comment_stop => start");
        let mut state = GatherCommentsState::new();
        state.set_whitespace_newlines_since_last_nonws(1);

        let src = "fn x() {}";
        debug!("Parsing src => {:?}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        let ident_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .expect("Expected an IDENT token in 'fn x() {}'");

        trace!("Found ident token => calling process_other_token");
        let result = process_other_token(ident_tok.clone(), ident_tok.text().to_string(), &mut state);

        assert!(result.is_none(), "Should stop if <2 newlines and no prior comment found");
        assert!(!state.found_comment(), "Should not mark found_comment");
        assert!(state.collected().is_empty(), "No comments were collected");
        info!("test_other_token_no_comment_stop => success");
    }
}
