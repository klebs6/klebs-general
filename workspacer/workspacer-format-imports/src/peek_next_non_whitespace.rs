// ---------------- [ File: workspacer-format-imports/src/peek_next_non_whitespace.rs ]
crate::ix!();


pub fn peek_next_non_whitespace(start: &Option<SyntaxElement>) -> Option<bool> {
    info!("peek_next_non_whitespace => start");
    trace!("Initial start={:?}", start);

    let mut cur = start.clone();
    while let Some(e) = cur {
        match e {
            NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::WHITESPACE => {
                debug!("Skipping whitespace token => text len={}", tok.text().len());
                cur = tok.prev_sibling_or_token();
            }
            NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::COMMENT => {
                trace!("Found comment token => returning Some(true)");
                info!("peek_next_non_whitespace => done => Some(true)");
                return Some(true);
            }
            NodeOrToken::Token(tok) => {
                debug!(
                    "Found non-comment token => kind={:?}, text={:?} => returning Some(false)",
                    tok.kind(),
                    tok.text()
                );
                info!("peek_next_non_whitespace => done => Some(false)");
                return Some(false);
            }
            NodeOrToken::Node(n) => {
                debug!(
                    "Found node => kind={:?} => returning Some(false)",
                    n.kind()
                );
                info!("peek_next_non_whitespace => done => Some(false)");
                return Some(false);
            }
        }
    }
    trace!("No more elements => returning None");
    info!("peek_next_non_whitespace => done => None");
    None
}

#[cfg(test)]
mod test_peek_next_non_whitespace {
    use super::*;

    /// Gathers all tokens in ascending text-range order,
    /// finds the first `FN_KW` token, and returns the one immediately before it.
    fn token_before_first_fn(src: &str) -> Option<SyntaxElement> {
        info!("token_before_first_fn => start => src length={}", src.len());
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        if !parse.errors().is_empty() {
            warn!("Parse errors => {:#?}", parse.errors());
        }

        let mut tokens: Vec<_> = file
            .syntax()
            .descendants_with_tokens()
            .filter_map(|x| x.into_token())
            .collect();
        tokens.sort_by_key(|t| t.text_range().start());

        trace!("Listing tokens in textual order:");
        for (i, tok) in tokens.iter().enumerate() {
            trace!(
                "index={}, kind={:?}, range={:?}, text={:?}",
                i,
                tok.kind(),
                tok.text_range(),
                tok.text()
            );
        }

        let fn_idx = tokens.iter().position(|t| t.kind() == SyntaxKind::FN_KW)?;
        debug!("Found FN_KW token at index {}", fn_idx);

        if fn_idx == 0 {
            debug!("fn_idx==0 => no token above => None");
            return None;
        }
        let above = tokens[fn_idx - 1].clone();
        debug!(
            "Above => index={}, kind={:?}, text={:?}",
            fn_idx - 1,
            above.kind(),
            above.text()
        );
        info!("token_before_first_fn => done");
        Some(NodeOrToken::Token(above))
    }

    #[traced_test]
    fn test_single_comment_is_found() {
        info!("test_single_comment_is_found => start");
        let src = r#"
struct Dummy1 {}

/* single comment */
fn main() {}
"#;
        let above = token_before_first_fn(src);
        assert!(above.is_some(), "Expected a token above 'fn main()'");

        let result = super::peek_next_non_whitespace(&above);
        assert_eq!(
            result,
            Some(true),
            "Expected to find a COMMENT token above fn main()"
        );
        info!("test_single_comment_is_found => success");
    }

    #[traced_test]
    fn test_skip_whitespace_then_comment() {
        info!("test_skip_whitespace_then_comment => start");
        let src = r#"
struct Dummy2 {}

    /* hello */
fn foo(){}
"#;
        let above = token_before_first_fn(src);
        assert!(above.is_some(), "Should find some token above 'fn foo()'");

        let result = super::peek_next_non_whitespace(&above);
        assert_eq!(
            result,
            Some(true),
            "Expected to see block-comment above fn foo()"
        );
        info!("test_skip_whitespace_then_comment => success");
    }

    #[traced_test]
    fn test_skip_whitespace_then_node() {
        info!("test_skip_whitespace_then_node => start");
        let src = r#"
struct Something {}

struct Another {}

fn bar(){}
"#;
        let above = token_before_first_fn(src);
        assert!(above.is_some(), "Should find a token above 'fn bar()'");

        let result = super::peek_next_non_whitespace(&above);
        assert_eq!(
            result,
            Some(false),
            "Above is a node or non-comment token => Some(false)"
        );
        info!("test_skip_whitespace_then_node => success");
    }
}
