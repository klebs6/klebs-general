// ---------------- [ File: src/scan_preceding_tokens_for_comments.rs ]
crate::ix!();

/// 3) **Public**: tries `gather_sibling_comments_above` first. If that yields nothing,
///    it tries `gather_token_comments_above`.
///
/// This covers both parse scenarios—whether the comment is a sibling,
/// or is stored with the node’s first token in the syntax tree.
pub fn scan_preceding_tokens_for_comments(start_node: &SyntaxNode) -> Vec<String> {
    trace!(
        "scan_preceding_tokens_for_comments => node.kind()={:?}",
        start_node.kind()
    );

    // 1) Try sibling approach
    let sibling_comments = gather_sibling_comments_above(start_node);
    if !sibling_comments.is_empty() {
        return sibling_comments;
    }

    // 2) If empty, try token-based approach
    gather_token_comments_above(start_node)
}

#[cfg(test)]
mod test_scan_preceding_tokens_for_comments {
    use super::*;
    use ra_ap_syntax::{SourceFile, SyntaxKind};

    fn parse_and_find_fn(src: &str) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[traced_test]
    fn test_no_comments_above() {
        let src = r#"fn main(){}"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        assert!(gathered.is_empty());
    }

    #[traced_test]
    fn test_blank_line_then_comment_is_blocked() {
        let src = r#"
// Some comment
// Another comment

fn main() {}
"#;
        // The blank line is right above fn => that means these comments are "blocked"
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        assert!(gathered.is_empty());
    }

    #[traced_test]
    fn test_comment_directly_above() {
        let src = r#"
// My comment
fn main() {}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        assert_eq!(
            gathered.len(),
            1,
            "We want exactly one comment directly above"
        );
        assert_eq!(gathered[0], "// My comment\n");
    }
}
