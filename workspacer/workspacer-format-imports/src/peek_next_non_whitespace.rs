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
                // Skip whitespace, keep going up
                cur = tok.prev_sibling_or_token();
            }
            NodeOrToken::Token(tok) if tok.kind() == SyntaxKind::COMMENT => {
                trace!("peek_next_non_whitespace => found COMMENT => returning Some(true)");
                return Some(true);
            }
            // Anything else (punct, keyword, node, etc.) => "false"
            _ => {
                trace!("peek_next_non_whitespace => found non-whitespace, non-comment => Some(false)");
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

    /// Helper: parse a snippet, find the first `FN` node, return
    /// the token "above" it in syntax order. If there's no node or
    /// token above, returns `None`.
    fn find_token_above_first_fn(src: &str) -> Option<SyntaxElement> {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();

        // Find the first FN node
        let fn_node = file.syntax().descendants().find(|n| n.kind() == SyntaxKind::FN)?;
        // We'll look for the first token of that node
        let fn_first_token = fn_node.first_token()?;
        // Then see if there's a previous sibling or token (the thing "above" it)
        fn_first_token.prev_sibling_or_token()
    }

    #[test]
    fn test_single_comment_is_found() {
        let src = r#"
// a comment
fn main() {}
"#;
        let above = find_token_above_first_fn(src);
        assert!(
            above.is_some(),
            "Expected some preceding token above the fn in the syntax"
        );
        let result = peek_next_non_whitespace(&above);
        // There's a comment right above. So eventually we expect Some(true).
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_skip_whitespace_then_comment() {
        let src = r#"
   // hello
fn foo(){}
"#;
        let above = find_token_above_first_fn(src);
        assert!(above.is_some());
        let result = peek_next_non_whitespace(&above);
        // There's whitespace, then a comment => Some(true).
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_skip_whitespace_then_node() {
        // We'll place no comments, just whitespace above fn
        let src = r#"

fn bar(){}
"#;
        let above = find_token_above_first_fn(src);
        assert!(above.is_some());
        let result = peek_next_non_whitespace(&above);
        // There's no comment => we should get Some(false) or None, but typically Some(false).
        assert_eq!(result, Some(false));
    }
}
