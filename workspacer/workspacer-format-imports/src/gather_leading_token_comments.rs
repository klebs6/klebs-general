// ---------------- [ File: workspacer-format-imports/src/gather_leading_token_comments.rs ]
crate::ix!();

pub fn gather_leading_token_comments(node: &SyntaxNode) -> Vec<String> {
    info!(
        "gather_leading_token_comments => start; node.kind()={:?}",
        node.kind()
    );

    let tokens: Vec<SyntaxElement> = node.children_with_tokens().collect();
    if tokens.is_empty() {
        debug!("No tokens => returning empty Vec");
        info!("gather_leading_token_comments => done => returning empty");
        return Vec::new();
    }

    let mut collected = Vec::new();
    let mut found_comment = false;
    let mut whitespace_newlines_since_last_nonws = 0;

    for elem in &tokens {
        match elem.kind() {
            SyntaxKind::COMMENT => {
                // Found a comment => store it
                let txt = elem.to_string();
                let line = if txt.ends_with('\n') {
                    txt
                } else {
                    format!("{}\n", txt)
                };
                trace!("Found COMMENT => line={:?}", line);
                collected.push(line);
                found_comment = true;
                whitespace_newlines_since_last_nonws = 0;
            }
            SyntaxKind::WHITESPACE => {
                let txt_no_cr = elem.to_string().replace('\r', "");
                // Check for embedded `//` lines
                if let Some(comment_lines) = try_extract_embedded_comment_in_whitespace(&txt_no_cr) {
                    trace!("Found {} embedded comments in whitespace", comment_lines.len());
                    for cmt in comment_lines {
                        let forced_newline = if cmt.ends_with('\n') {
                            cmt
                        } else {
                            format!("{}\n", cmt)
                        };
                        collected.push(forced_newline);
                    }
                    found_comment = true;
                    whitespace_newlines_since_last_nonws = 0;
                } else {
                    let newline_count = txt_no_cr.matches('\n').count();
                    debug!(
                        "WHITESPACE => newline_count={}, found_comment={}",
                        newline_count, found_comment
                    );
                    if found_comment {
                        // Once we've started collecting comments, 2+ newlines => blank line => stop
                        if newline_count >= 2 {
                            warn!("Blank line after comment => stopping");
                            break;
                        }
                        whitespace_newlines_since_last_nonws += newline_count;
                    } else {
                        // If no comment yet => 2+ newlines => block => return empty
                        if newline_count >= 2 {
                            warn!("Blank line => no comment yet => returning empty");
                            info!("gather_leading_token_comments => done => returning empty");
                            return Vec::new();
                        }
                        whitespace_newlines_since_last_nonws += newline_count;
                    }
                }
            }
            other => {
                trace!(
                    "Found non-comment token => kind={:?}, ws_newlines={}",
                    other,
                    whitespace_newlines_since_last_nonws
                );
                // If we've found comments already, then <2 newlines => stop collecting
                // If we haven't found any comment, <2 newlines => also stop
                if whitespace_newlines_since_last_nonws < 2 {
                    debug!("Non-comment token with <2 newlines => stop collecting");
                    break;
                } else {
                    debug!("Non-comment token with >=2 newlines => skip & continue");
                    whitespace_newlines_since_last_nonws = 0;
                }
            }
        }
    }

    debug!(
        "gather_leading_token_comments => returning {} collected lines",
        collected.len()
    );
    info!("gather_leading_token_comments => done");
    collected
}


pub fn try_extract_embedded_comment_in_whitespace(ws_text: &str) -> Option<Vec<String>> {
    // If the text doesn't contain "//", no need to parse further
    if !ws_text.contains("//") {
        return None;
    }
    let mut lines = Vec::new();
    for raw_line in ws_text.lines() {
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("//") {
            lines.push(trimmed.to_owned());
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

#[cfg(test)]
mod test_gather_leading_token_comments {
    use super::*;
    use ra_ap_syntax::{SourceFile, SyntaxKind};
    use tracing::{trace, info, debug};

    fn first_fn_node(src: &str) -> SyntaxNode {
        info!("first_fn_node => start; parse src len={}", src.len());
        trace!("Source:\n{}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        debug!("Parsed => searching for FN node");
        let fn_node = file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .expect("Expected an FN node");
        info!("first_fn_node => done");
        fn_node
    }

    #[traced_test]
    fn test_leading_token_no_comment() {
        info!("test_leading_token_no_comment => start");
        let src = "fn main() {}";
        debug!("src={}",src);
        let fn_node = first_fn_node(src);
        debug!("fn_node={:#?}",fn_node);
        let collected = gather_leading_token_comments(&fn_node);
        debug!("collected={:?}", collected);
        assert!(collected.is_empty(), "No comment => should be empty");
        info!("test_leading_token_no_comment => success");
    }

    #[traced_test]
    fn test_leading_token_comment_right_above() {
        info!("test_leading_token_comment_right_above => start");
        let src = r#"// hi
fn main() {}
"#;
        debug!("src={}",src);
        let fn_node = first_fn_node(src);
        debug!("fn_node={:#?}",fn_node);
        let collected = gather_leading_token_comments(&fn_node);
        debug!("collected={:?}", collected);

        assert_eq!(collected.len(), 1, "Should find the // hi comment");
        assert_eq!(collected[0], "// hi\n");
        info!("test_leading_token_comment_right_above => success");
    }

    #[traced_test]
    fn test_gather_sibling_comment_directly_above() {
        info!("test_gather_sibling_comment_directly_above => start");
        let src = r#"// comment above
fn bar() {}
"#;
        debug!("src={}",src);
        let fn_node = first_fn_node(src); 
        debug!("fn_node={:#?}",fn_node);
        let gathered = gather_sibling_comments_above(&fn_node); 
        debug!("collected={:?}", gathered);
        assert_eq!(gathered.len(), 1, "Expected exactly one comment line directly above");
        assert_eq!(gathered[0], "// comment above\n");
        info!("test_gather_sibling_comment_directly_above => success");
    }
}
