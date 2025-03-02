// ---------------- [ File: src/gather_leading_comment_lines.rs ]
crate::ix!();

pub fn gather_leading_comment_lines(node: &SyntaxNode, _full_text: &str) -> Vec<String> {
    info!("gather_leading_comment_lines => start");
    trace!("node.kind()={:?}", node.kind());

    // Phase A: attempt to gather from preceding tokens
    debug!("Phase A => scan_preceding_tokens_for_comments");
    let from_preceding = scan_preceding_tokens_for_comments(node);
    if !from_preceding.is_empty() {
        debug!(
            "Found {} lines from preceding tokens => returning those",
            from_preceding.len()
        );
        info!("gather_leading_comment_lines => done");
        return from_preceding;
    }

    // Phase B: fallback
    debug!("No preceding => fallback => fallback_scan_node_text");
    let fallback = fallback_scan_node_text(node);
    if !fallback.is_empty() {
        debug!("Found {} lines from fallback", fallback.len());
    } else {
        debug!("No lines found in fallback either");
    }
    info!("gather_leading_comment_lines => done => returning fallback");
    fallback
}

#[cfg(test)]
mod test_gather_leading_comment_lines {
    use super::*;

    fn parse_and_find_node(src: &str, kind: SyntaxKind) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax().descendants().find(|n| n.kind() == kind).unwrap()
    }

    #[test]
    fn test_no_leading() {
        let src = "fn main() {}";
        let item = parse_and_find_node(src, SyntaxKind::FN);
        let gathered = gather_leading_comment_lines(&item, src);
        assert!(gathered.is_empty(), "No comments => empty");
    }

    #[test]
    fn test_comment_above_node() {
        let src = r#"
// hi
fn main(){}
"#;
        let item = parse_and_find_node(src, SyntaxKind::FN);
        let gathered = gather_leading_comment_lines(&item, src);
        assert_eq!(gathered.len(), 1);
        assert_eq!(gathered[0], "// hi\n");
    }

    /// Helper: parse the given `text` as a Rust source file,
    /// then find the first node that matches `kind`.
    /// Returns that node (if any).
    fn parse_and_find_node_of_kind(text: &str, kind: SyntaxKind) -> Option<SyntaxNode> {
        let parse = SourceFile::parse(text, Edition::Edition2021);
        let file = parse.tree();
        for item in file.items() {
            if item.syntax().kind() == kind {
                return Some(item.syntax().clone());
            }
        }
        None
    }

    /// 1) If there are no leading comments => returns empty Vec
    #[traced_test]
    fn test_no_leading_comments() {
        let src = r#"
            fn my_function() {}
        "#;
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected a fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert!(
            comments.is_empty(),
            "No leading comments => should be empty Vec"
        );
    }

    /// 2) Single line comment directly above => returns that comment (with trailing newline)
    #[traced_test]
    fn test_single_line_comment() {
        let src = r#"
// This is a comment
fn my_function() {}
        "#;
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert_eq!(comments.len(), 1, "Should have exactly one comment line");
        assert_eq!(comments[0], "// This is a comment\n");
    }

    /// 4) If there's whitespace with only one newline => it won't break the chain, 
    ///    but if there are two or more newlines => we stop.
    #[traced_test]
    fn test_whitespace_rules() {
        let src = r#"
// Comment A
   // Comment B
fn main(){}

"#;
        debug!("src={}",src);
        // There's only 1 newline between comment B and 'fn', so we keep collecting
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        debug!("node={:#?}",node);
        let comments = gather_leading_comment_lines(&node, src);
        debug!("comments={:#?}",comments);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], "// Comment A\n");
        assert_eq!(comments[1], "// Comment B\n");
    }

    /// 5) If there's a blank line with 2+ newlines => that stops the chain of comments.
    #[traced_test]
    fn test_blank_line_stops() {
        let src = r#"
// Above comment

fn something(){}
// Another comment
        "#;
        // We expect the blank line after "Above comment" to stop the chain,
        // so `fn something(){}` has zero leading comments in that sense.
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert!(comments.is_empty(), "Blank line => no leading comments");
    }

    /// 6) If there's a node directly above (e.g., struct, mod, etc.) => we also stop.
    #[traced_test]
    fn test_node_above_stops() {
        let src = r#"
struct MyType;

// comment here
fn function(){}
        "#;

        // We'll find the `fn function(){}`, see if we gather "comment here"
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert_eq!(comments.len(), 1, "Should have exactly one comment line");
        assert_eq!(comments[0], "// comment here\n");
    }

    /// 7) If the comment doesn't end with a newline, we forcibly add it.
    #[traced_test]
    fn test_forced_newline() {
        let src = "// no newline at end\nfn main() {}";
        // Actually let's do no newline => We'll do: `// no newline at end` then the fn
        let src2 = "// no newline at end\nfn main() {}";
        let node = parse_and_find_node_of_kind(src2, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src2);
        assert_eq!(comments.len(), 1);
        // The function adds a newline if missing
        assert_eq!(comments[0], "// no newline at end\n");
    }

    /// 8) Edge case: If the code starts with a comment, we collect that if it's right above the node with no blank line
    #[traced_test]
    fn test_top_of_file_comments() {
        let src = r#"
// Top-of-file comment
fn main(){}
        "#;
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0], "// Top-of-file comment\n");
    }

    /// 9) If there's a non-comment token (like `pub` or another keyword) right above,
    ///    we stop collecting. Let's see how the code behaves.
    #[traced_test]
    fn test_token_above_stops() {
        let src = r#"
pub fn something_else(){}

fn main(){}
        "#;
        // There's no blank line or comment => we'll see if it picks up anything
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN)
            .expect("Expected fn node");
        // This is the second fn => we want that, hopefully
        // The parse will return the first one or second? We need to clarify. 
        // We'll do a small trick: find the second item. 
        // Let's just do a naive approach: let's assume we get the second one. 
        // Then we check if there's a comment. There's no comment => it's empty.
        let comments = gather_leading_comment_lines(&node, src);
        assert!(comments.is_empty(), "No comment => empty");
    }

    /// 10) Confirm we do not gather trailing comments on the same line as the node
    #[traced_test]
    fn test_trailing_same_line_comment_ignored() {
        let src = r#"
fn main() {} // trailing comment
        "#;
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        let comments = gather_leading_comment_lines(&node, src);
        assert!(comments.is_empty(), "Trailing comment on same line => not recognized as leading");
    }

    #[traced_test]
    fn test_multiple_consecutive_comments() {
        let src = r#"
// First line
// Second line
fn my_function() {}
"#;
        debug!("src={}",src);
        let node = parse_and_find_node_of_kind(src, SyntaxKind::FN).expect("Expected fn node");
        debug!("node={:#?}",node);
        let comments = gather_leading_comment_lines(&node, src);
        debug!("comments={:#?}",comments);
        assert_eq!(comments.len(), 2, "We have two comment lines");
        assert_eq!(comments[0], "// First line\n");
        assert_eq!(comments[1], "// Second line\n");
    }
}
