// ---------------- [ File: src/gather_sibling_comments_above.rs ]
crate::ix!();

/// 1) Climb upward *among sibling-or-token predecessors* of `start_node`,
///    collecting any comment lines in top-to-bottom order.
///
///    This function implements the user’s “two tricky tests” semantics:
///      - If a blank line (2+ newlines) appears **directly** above `fn` and
///        the very next non-whitespace above that blank line is a comment,
///        we “block” that comment (return empty).
///      - Otherwise, if that next item is a node, we skip it (and its trailing
///        whitespace), then keep climbing. Eventually, we may discover a comment above.
///
///    *Production-quality naming* means each function is named for **what it does**.
///    The subroutines have names like `process_upward_node`, `collect_comment_token`,
///    etc., to clearly reflect their roles.
pub fn gather_sibling_comments_above(start_node: &SyntaxNode) -> Vec<String> {
    trace!(
        "gather_sibling_comments_above => node.kind()={:?}",
        start_node.kind()
    );

    let mut state = GatherCommentsState::new();

    // We'll climb from the previous sibling-or-token of `start_node`
    let mut cur = start_node.prev_sibling_or_token();

    while let Some(element) = cur {
        match element {
            NodeOrToken::Node(_) => {
                // If it's a Node, handle accordingly
                cur = process_upward_node(element, &mut state);
            }
            NodeOrToken::Token(tok) => {
                // If it's a Token, we break down by token kind
                cur = dispatch_token_by_kind(tok, &mut state);
            }
        }

        // If a subroutine returned None => we stop climbing
        if cur.is_none() {
            break;
        }
    }

    state.collected().to_vec()
}

#[cfg(test)]
mod test_gather_sibling_comments_above {
    use super::*;

    fn first_fn_node(src: &str) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[test]
    fn test_gather_sibling_no_comments() {
        let src = r#"
fn main() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        assert!(gathered.is_empty(), "Expected no sibling comments above");
    }

    #[test]
    fn test_gather_sibling_blank_line_blocks_comment() {
        let src = r#"
// above1
// above2

fn foo() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        assert!(
            gathered.is_empty(),
            "Blank line should block the comment above"
        );
    }

    #[test]
    fn test_gather_sibling_comment_directly_above() {
        let src = r#"
struct Dummy {}

// comment above
fn bar() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        assert_eq!(
            gathered.len(),
            1,
            "Expected exactly one comment line directly above"
        );
        assert_eq!(gathered[0], "// comment above\n");
    }
}
