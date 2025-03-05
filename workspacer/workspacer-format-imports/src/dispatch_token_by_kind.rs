// ---------------- [ File: workspacer-format-imports/src/dispatch_token_by_kind.rs ]
crate::ix!();

pub fn dispatch_token_by_kind(
    tok: SyntaxToken,
    state: &mut GatherCommentsState,
) -> Option<NodeOrToken<SyntaxNode, SyntaxToken>> {
    info!("dispatch_token_by_kind => start");
    use SyntaxKind::*;
    let text = tok.text().to_string();
    debug!("Token kind={:?}, text.len()={}", tok.kind(), text.len());

    let result = match tok.kind() {
        COMMENT => {
            debug!("COMMENT => collect_comment_token");
            collect_comment_token(tok, text, state)
        }
        WHITESPACE => {
            debug!("WHITESPACE => process_whitespace_token");
            process_whitespace_token(tok, text, state)
        }
        _ => {
            debug!("OTHER => process_other_token");
            process_other_token(tok, text, state)
        }
    };

    debug!(
        "dispatch_token_by_kind => done; returning {}",
        if result.is_none() {
            "None".to_string()
        } else {
            format!("Some({:?})", result.as_ref().unwrap().kind())
        }
    );
    info!("dispatch_token_by_kind => done");
    result
}

#[cfg(test)]
mod test_dispatch_token_by_kind {
    use super::*;
    use tracing::{trace, info, debug};

    #[traced_test]
    fn test_dispatch_comment() {
        info!("test_dispatch_comment => start");
        let mut state = GatherCommentsState::new();

        let src = "// hey";
        debug!("Parsing src => {:?}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        trace!("Parsed => searching for COMMENT token");

        let cmt_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::COMMENT)
            .expect("Expected a COMMENT token");

        let returned = dispatch_token_by_kind(cmt_tok.clone(), &mut state);

        assert!(state.found_comment(), "Should detect a comment");
        assert_eq!(state.collected().len(), 1, "One comment collected");
        assert_eq!(
            returned,
            cmt_tok.prev_sibling_or_token(),
            "Should climb from previous sibling/token"
        );
        info!("test_dispatch_comment => success");
    }

    #[traced_test]
    fn test_dispatch_whitespace() {
        trace!("test_dispatch_whitespace => start");
        let mut state = GatherCommentsState::new();

        let src = "   ";
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        let ws_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::WHITESPACE)
            .expect("Expected a WHITESPACE token in '   '");

        let returned = dispatch_token_by_kind(ws_tok.clone(), &mut state);

        assert!(!state.found_comment(), "No comment found in mere whitespace");
        assert_eq!(
            returned,
            ws_tok.prev_sibling_or_token(),
            "Should climb from the whitespace token's previous sibling/token"
        );
    }

    #[traced_test]
    fn test_dispatch_other() {
        trace!("test_dispatch_other => start");
        let mut state = GatherCommentsState::new();

        let src = "fn foo() {}";
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        let ident_tok = file.syntax()
            .descendants_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::IDENT)
            .expect("Expected an IDENT token in 'fn foo() {}'");

        let returned = dispatch_token_by_kind(ident_tok.clone(), &mut state);

        assert!(!state.found_comment(), "Should not have found any comment");
        assert_eq!(
            returned,
            ident_tok.prev_sibling_or_token(),
            "Should climb from the ident token's previous sibling/token"
        );
    }
}
