// ---------------- [ File: src/peek_next_non_whitespace.rs ]
crate::ix!();


/// Looks upward (previous siblings/tokens) from `start`,
/// skipping any whitespace tokens, and returns:
///   - `Some(true)`  if the next non-whitespace item is a comment token
///   - `Some(false)` if it's a node or any other non-comment token
///   - `None`        if there is no non-whitespace item above
pub fn peek_next_non_whitespace(start: &Option<SyntaxElement>) -> Option<bool> {
    trace!("peek_next_non_whitespace => start={:?}", start);

    let mut cur = start.clone();
    while let Some(e) = cur {
        match e {
            NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::WHITESPACE => {
                trace!(
                    "  skipping WHITESPACE => prev_sibling_or_token from {:?}",
                    tok.text()
                );
                cur = tok.prev_sibling_or_token();
            }
            NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::COMMENT => {
                trace!("peek_next_non_whitespace => found COMMENT => returning Some(true)");
                return Some(true);
            }
            NodeOrToken::Token(tok) => {
                // some non-comment token
                trace!(
                    "peek_next_non_whitespace => found token kind={:?}, text={:?} => Some(false)",
                    tok.kind(),
                    tok.text()
                );
                return Some(false);
            }
            NodeOrToken::Node(n) => {
                trace!(
                    "peek_next_non_whitespace => found a node kind={:?} => Some(false)",
                    n.kind()
                );
                return Some(false);
            }
        }
    }
    trace!("peek_next_non_whitespace => found nothing => returning None");
    None
}

#[cfg(test)]
mod test_peek_next_non_whitespace {
    use super::*;
    use tracing::{trace};
    use ra_ap_syntax::{SyntaxToken};

    /// Gathers all tokens in ascending text-range order,
    /// finds the first `FN_KW` token, and returns the one immediately before it.
    fn token_before_first_fn(src: &str) -> Option<SyntaxElement> {
        trace!("token_before_first_fn => src:\n{}", src);

        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        if !parse.errors().is_empty() {
            trace!("Parse errors => {:#?}", parse.errors());
        }

        // gather all tokens
        let mut tokens: Vec<_> = file
            .syntax()
            .descendants_with_tokens()
            .filter_map(|x| x.into_token())
            .collect();

        // sort them by their text_range().start()
        tokens.sort_by_key(|t| t.text_range().start());

        trace!("=== Full token list (in textual order) ===");
        for (i, tok) in tokens.iter().enumerate() {
            trace!(
                "Index={} kind={:?}, range={:?}, text={:?}",
                i,
                tok.kind(),
                tok.text_range(),
                tok.text()
            );
        }

        // find the first FN_KW token
        let fn_idx = tokens.iter().position(|t| t.kind() == SyntaxKind::FN_KW)?;
        trace!("Found `FN_KW` token at index {}", fn_idx);

        if fn_idx == 0 {
            trace!("fn_idx==0 => no token above => None");
            return None;
        }
        let above = tokens[fn_idx - 1].clone();
        trace!(
            "above => index={}, kind={:?}, text={:?}",
            fn_idx - 1,
            above.kind(),
            above.text()
        );
        Some(NodeOrToken::Token(above))
    }

    /// 1) single block comment line above `fn main()`.
    ///    We make sure the comment is separated by a valid item (a struct).
    ///    So the parser sees:
    ///        struct Dummy1 {}
    ///        /* single comment */
    ///        fn main() {}
    #[traced_test]
    fn test_single_comment_is_found() {
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
    }

    /// 2) whitespace, then a block comment, then `fn foo()`.
    ///    We add a struct above plus some newlines => the block comment => `fn foo()`.
    ///    This ensures the comment is not leading trivia for the fn.
    #[traced_test]
    fn test_skip_whitespace_then_comment() {
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
    }

    /// 3) whitespace, then a node (not a comment) => we want `Some(false)`.
    ///    We define two struct items at top level. Then we define `fn bar()`. 
    ///    The token before `fn bar()` is presumably the closing brace or whitespace near the second struct.
    ///    => that is not a comment => Some(false).
    #[traced_test]
    fn test_skip_whitespace_then_node() {
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
    }
}
