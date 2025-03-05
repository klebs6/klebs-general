// ---------------- [ File: workspacer-format-imports/src/collect_comment_token.rs ]
crate::ix!();

pub fn collect_comment_token(
    tok: SyntaxToken,
    text: String,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!("collect_comment_token => start");
    let line = if text.ends_with('\n') {
        text
    } else {
        format!("{}\n", text)
    };
    debug!("Adding comment line => {:?}", line);
    state.collected_mut().insert(0, line);
    state.set_found_comment(true);
    state.set_whitespace_newlines_since_last_nonws(0);

    let ret = tok.prev_sibling_or_token();
    trace!("collect_comment_token => returning {:?}", ret.as_ref().map(|x| x.kind()));
    info!("collect_comment_token => done");
    ret
}

#[cfg(test)]
mod test_collect_comment_token {
    use super::*;

    #[traced_test]
    fn test_collect_comment_basic() {
        info!("test_collect_comment_basic => start");
        let mut state = GatherCommentsState::new();

        let src = "// hi";
        debug!("Parsing => {:?}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        trace!("Looking for COMMENT token");

        let cmt_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::COMMENT)
            .expect("Expected a COMMENT token in '// hi'");

        let ret = collect_comment_token(cmt_tok.clone(), cmt_tok.text().to_string(), &mut state);

        assert!(state.found_comment(), "Should mark found_comment as true");
        assert_eq!(state.collected().len(), 1, "We collected one comment line");
        assert_eq!(state.collected()[0], "// hi\n", "Should have forced a newline at the end");
        assert_eq!(
            *state.whitespace_newlines_since_last_nonws(),
            0,
            "Should have reset newlines count"
        );
        assert_eq!(
            ret,
            cmt_tok.prev_sibling_or_token(),
            "Should return the previous sibling or token as the climb point"
        );
        info!("test_collect_comment_basic => success");
    }
}
