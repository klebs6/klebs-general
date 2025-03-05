// ---------------- [ File: workspacer-format-imports/src/gather_sibling_comments_above.rs ]
crate::ix!();

pub fn gather_sibling_comments_above(node: &SyntaxNode) -> Vec<String> {
    info!(
        "gather_sibling_comments_above => start; node.kind()={:?}",
        node.kind()
    );

    let Some(parent) = node.parent() else {
        debug!("No parent => returning empty Vec");
        info!("gather_sibling_comments_above => done => returning empty");
        return Vec::new();
    };

    // Collect the parent's children, in order
    let siblings: Vec<SyntaxElement> = parent.children_with_tokens().collect();

    // Find the index of our node among siblings
    let node_elem = SyntaxElement::Node(node.clone());
    let Some(idx) = siblings.iter().position(|elem| *elem == node_elem) else {
        debug!("Node not found among parent's children => returning empty Vec");
        info!("gather_sibling_comments_above => done => returning empty");
        return Vec::new();
    };

    // If idx == 0 => no preceding siblings => fallback to the node’s own leading-token approach
    if idx == 0 {
        trace!("No preceding siblings => fallback to gather_leading_token_comments");
        let fallback = gather_leading_token_comments(node);
        debug!(
            "gather_sibling_comments_above => returning fallback with {} lines",
            fallback.len()
        );
        info!("gather_sibling_comments_above => done");
        return fallback;
    }

    let mut collected = Vec::new();
    let mut found_comment = false;
    let mut whitespace_newlines_since_last_nonws = 0;

    // Iterate backwards from the sibling *just before* this node, up to 0
    for elem in siblings[..idx].iter().rev() {
        match elem.kind() {
            SyntaxKind::COMMENT => {
                // Insert the comment at the front
                let txt = elem.to_string();
                let line = if txt.ends_with('\n') {
                    txt
                } else {
                    format!("{}\n", txt)
                };
                trace!("Found COMMENT => line={:?}", line);
                collected.insert(0, line);
                found_comment = true;
                whitespace_newlines_since_last_nonws = 0;
            }
            SyntaxKind::WHITESPACE => {
                let txt_no_cr = elem.to_string().replace('\r', "");
                // Check if there are embedded `//` lines in the whitespace
                if let Some(comment_lines) = try_extract_embedded_comment_in_whitespace(&txt_no_cr) {
                    trace!("Found {} embedded comments inside whitespace", comment_lines.len());
                    for line in comment_lines {
                        let forced_newline = if line.ends_with('\n') {
                            line
                        } else {
                            format!("{}\n", line)
                        };
                        collected.insert(0, forced_newline);
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
                        // If we've already got comments, 2+ newlines => blank line => stop scanning
                        if newline_count >= 2 {
                            warn!("Blank line after comment => stopping");
                            break;
                        }
                        whitespace_newlines_since_last_nonws += newline_count;
                    } else {
                        // If we haven't found any comment yet and we see 2+ newlines => block
                        if newline_count >= 2 {
                            warn!("Blank line => no comment yet => returning empty Vec");
                            info!("gather_sibling_comments_above => done => returning empty");
                            return Vec::new();
                        } else {
                            whitespace_newlines_since_last_nonws += newline_count;
                        }
                    }
                }
            }
            other => {
                // Non-comment sibling: if we haven't accumulated 2+ newlines, that means we stop.
                trace!(
                    "Found non-comment sibling => kind={:?}, ws_newlines={}",
                    other,
                    whitespace_newlines_since_last_nonws
                );
                if whitespace_newlines_since_last_nonws < 2 {
                    debug!("Non-comment sibling with <2 newlines => stop");
                    break;
                } else {
                    debug!("Non-comment sibling with >=2 newlines => skip & continue");
                    whitespace_newlines_since_last_nonws = 0;
                }
            }
        }
    }

    // If we found nothing among siblings, let's see if there's a comment in the node’s own leading tokens
    if collected.is_empty() {
        debug!("No sibling comments found => fallback to gather_leading_token_comments");
        let fallback = gather_leading_token_comments(node);
        if !fallback.is_empty() {
            info!(
                "gather_sibling_comments_above => done => returning fallback with {} lines",
                fallback.len()
            );
            return fallback;
        }
    }

    debug!(
        "gather_sibling_comments_above => returning {} collected lines",
        collected.len()
    );
    info!("gather_sibling_comments_above => done");
    collected
}

#[cfg(test)]
mod test_gather_sibling_comments_above {
    use super::*;

    #[traced_test]
    fn test_gather_sibling_no_comments() {
        info!("test_gather_sibling_no_comments => start");
        let src = r#"
fn main() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        debug!("collected={:?}", gathered);
        assert!(gathered.is_empty(), "Expected no sibling comments above");
        info!("test_gather_sibling_no_comments => success");
    }

    #[traced_test]
    fn test_gather_sibling_blank_line_blocks_comment() {
        info!("test_gather_sibling_blank_line_blocks_comment => start");
        let src = r#"
// above1
// above2

fn foo() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        debug!("collected={:?}", gathered);
        assert!(
            gathered.is_empty(),
            "Blank line should block the comment above"
        );
        info!("test_gather_sibling_blank_line_blocks_comment => success");
    }

    fn first_fn_node(src: &str) -> SyntaxNode {
        trace!("first_fn_node => parsing:\n{}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        let node = file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .expect("Expected FN node");
        node
    }

    #[traced_test]
    fn test_gather_sibling_comment_directly_above() {
        info!("test_gather_sibling_comment_directly_above => start");
        let src = r#"
// comment above
fn bar() {}
"#;
        let fn_node = first_fn_node(src);
        let gathered = gather_sibling_comments_above(&fn_node);
        debug!("collected={:?}", gathered);
        assert_eq!(
            gathered.len(),
            1,
            "Expected exactly one comment line directly above"
        );
        assert_eq!(gathered[0], "// comment above\n");
        info!("test_gather_sibling_comment_directly_above => success");
    }
}
