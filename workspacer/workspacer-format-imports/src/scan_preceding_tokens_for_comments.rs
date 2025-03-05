// ---------------- [ File: workspacer-format-imports/src/scan_preceding_tokens_for_comments.rs ]
crate::ix!();

pub fn scan_preceding_tokens_for_comments(start_node: &SyntaxNode) -> Vec<String> {
    info!(
        "scan_preceding_tokens_for_comments => node.kind()={:?}",
        start_node.kind()
    );

    trace!("Trying gather_sibling_comments_above first");
    let sibling_comments = gather_sibling_comments_above(start_node);
    if !sibling_comments.is_empty() {
        debug!(
            "Found {} sibling comment lines => returning early",
            sibling_comments.len()
        );
        return sibling_comments;
    }

    trace!("No sibling comments => calling gather_token_comments_above");
    let token_comments = gather_token_comments_above(start_node);
    debug!("Found {} token-based comment lines", token_comments.len());
    token_comments
}


#[cfg(test)]
mod test_scan_preceding_tokens_for_comments {
    use super::*;

    #[traced_test]
    fn test_no_comments_above() {
        info!("test_no_comments_above => start");
        let src = r#"fn main(){}"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        debug!("Result => {:?}", gathered);
        assert!(gathered.is_empty());
        info!("test_no_comments_above => success");
    }

    #[traced_test]
    fn test_blank_line_then_comment_is_blocked() {
        info!("test_blank_line_then_comment_is_blocked => start");
        let src = r#"
// Some comment
// Another comment

fn main() {}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        debug!("Result => {:?}", gathered);
        assert!(gathered.is_empty());
        info!("test_blank_line_then_comment_is_blocked => success");
    }

    fn parse_and_find_fn(src: &str) -> SyntaxNode {
        trace!("parse_and_find_fn => parsing src:\n{src}");
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[traced_test]
    fn test_comment_directly_above() {
        info!("test_comment_directly_above => start");
        let src = r#"
// My comment
fn main() {}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        debug!("Result => {:?}", gathered);
        assert_eq!(
            gathered.len(),
            1,
            "We want exactly one comment directly above"
        );
        assert_eq!(gathered[0], "// My comment\n");
        info!("test_comment_directly_above => success");
    }
}
