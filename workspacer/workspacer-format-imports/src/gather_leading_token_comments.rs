// ---------------- [ File: src/gather_leading_token_comments.rs ]
crate::ix!();

/// Because some parse configurations might put a line comment *very* close to `fn`,
/// we attempt a token-based fallback: starting from the node's *first_token()*,
/// then scanning upward with `prev_token()`. This is optional in some parse trees,
/// but if your parse never places the comment as the "leading trivia" of `fn`,
/// you may not need this. If you do, here's a fallback approach:
pub fn gather_leading_token_comments(start_node: &SyntaxNode) -> Vec<String> {
    trace!(
        "gather_leading_token_comments => node.kind()={:?}",
        start_node.kind()
    );

    let Some(first_tok) = start_node.first_token() else {
        debug!("No first_token => returning empty");
        return Vec::new();
    };

    let mut collected = Vec::new();
    let mut found_comment = false;
    let mut whitespace_newlines_since_last_nonws = 0;

    let mut cur = first_tok.prev_token();
    while let Some(tok) = cur {
        let txt = tok.text().to_string();
        match tok.kind() {
            SyntaxKind::COMMENT => {
                let line = if txt.ends_with('\n') {
                    txt
                } else {
                    format!("{}\n", txt)
                };
                trace!("Found comment above token => pushing line={:?}", line);
                collected.insert(0, line);
                found_comment = true;
                whitespace_newlines_since_last_nonws = 0;
            }
            SyntaxKind::WHITESPACE => {
                let newline_count = txt.matches('\n').count();
                trace!(
                    "Found whitespace => newline_count={}, found_comment={}",
                    newline_count,
                    found_comment
                );
                if found_comment {
                    if newline_count >= 2 {
                        debug!("Blank line after comment => stopping");
                        break;
                    }
                    whitespace_newlines_since_last_nonws += newline_count;
                } else {
                    if newline_count >= 2 {
                        debug!("Blank line => no comment found yet => returning empty");
                        return Vec::new();
                    } else {
                        whitespace_newlines_since_last_nonws += newline_count;
                    }
                }
            }
            // Non-comment => check blank-line skipping logic
            _ => {
                trace!(
                    "Non-comment token kind={:?}, ws_newlines={}",
                    tok.kind(),
                    whitespace_newlines_since_last_nonws
                );
                if whitespace_newlines_since_last_nonws < 2 {
                    debug!("Non-comment token with <2 newlines => stop");
                    break;
                } else {
                    debug!("Non-comment token with >=2 newlines => skip & continue");
                    whitespace_newlines_since_last_nonws = 0;
                }
            }
        }
        cur = tok.prev_token();
    }

    collected
}

#[cfg(test)]
mod test_gather_leading_token_comments {
    use super::*;
    use ra_ap_syntax::{SourceFile, SyntaxKind};

    fn first_fn_node(src: &str) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[traced_test]
    fn test_leading_token_no_comment() {
        let src = r#"
fn main() {}
"#;
        let fn_node = first_fn_node(src);
        let collected = gather_leading_token_comments(&fn_node);
        assert!(collected.is_empty());
    }

    #[traced_test]
    fn test_leading_token_comment_right_above() {
        let src = r#"
// hi
fn main() {}
"#;
        let fn_node = first_fn_node(src);
        let collected = gather_leading_token_comments(&fn_node);
        assert_eq!(collected.len(), 1, "Should find the // hi comment");
        assert_eq!(collected[0], "// hi\n");
    }

    #[traced_test]
    fn test_gather_sibling_comment_directly_above() {
        let src = r#"
struct Dummy {} 
// comment above 
fn bar() {} "#; 
        let fn_node = first_fn_node(src); 
        let gathered = gather_sibling_comments_above(&fn_node); 
        assert_eq!(gathered.len(), 1, "Expected exactly one comment line directly above"); 
        assert_eq!(gathered[0], "// comment above\n"); 
    }
}
