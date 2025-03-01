// ---------------- [ File: src/gather_token_comments_above.rs ]
crate::ix!();

/// 2) Sometimes a comment “directly above” a node is *not* stored as a sibling,
///    but rather as a token’s leading/adjacent content in the parse tree.
///    For RA/AP or rowan-based trees, you often have to `prev_token()` from
///    the node’s first token to find line comments that appear physically above.
///
///    The logic is essentially the same as `gather_sibling_comments_above`,
///    but we climb token-by-token from `start_node.first_token().prev_token()`.
///
///    (No `.leading_trivia()` is used here—just the `prev_token()` approach.)
pub fn gather_token_comments_above(start_node: &SyntaxNode) -> Vec<String> {
    trace!(
        "gather_token_comments_above => node.kind()={:?}",
        start_node.kind()
    );

    let Some(first_tok) = start_node.first_token() else {
        debug!("No tokens in node => no comment");
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
                trace!("Found comment => pushing line={:?}", line);
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
                    // If a comment was found, 2+ newlines => blank line => stop
                    if newline_count >= 2 {
                        debug!("Blank line after comment => stopping");
                        break;
                    }
                    whitespace_newlines_since_last_nonws += newline_count;
                } else {
                    // No comment yet
                    if newline_count >= 2 {
                        debug!("Blank line blocks => returning empty");
                        return Vec::new();
                    } else {
                        whitespace_newlines_since_last_nonws += newline_count;
                    }
                }
            }
            _ => {
                trace!(
                    "Non-comment token => ws_newlines={}",
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
mod test_gather_token_comments_above {
    use super::*;
    use ra_ap_syntax::{SourceFile, SyntaxKind};
    use tracing::trace;

    fn first_fn_node(src: &str) -> SyntaxNode {
        trace!("Parsing src:\n{src}");
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[traced_test]
    fn test_token_comments_no_comment() {
        let src = "fn main() {}";
        let fn_node = first_fn_node(src);
        let collected = gather_token_comments_above(&fn_node);
        assert!(collected.is_empty(), "Expected no comment above");
    }

    #[traced_test]
    fn test_token_comments_comment_directly_above() {
        let src = r#"
// This is above main
fn main() {}
"#;
        let fn_node = first_fn_node(src);
        let collected = gather_token_comments_above(&fn_node);
        assert_eq!(
            collected.len(),
            1,
            "Expected exactly one comment line directly above"
        );
        assert_eq!(collected[0], "// This is above main\n");
    }

    #[traced_test]
    fn test_token_comments_blank_line_blocks() {
        let src = r#"
// Some comment

fn foo() {}
"#;
        let fn_node = first_fn_node(src);
        let collected = gather_token_comments_above(&fn_node);
        assert!(
            collected.is_empty(),
            "Blank line should block the comment above"
        );
    }
}

