// ---------------- [ File: workspacer-format-imports/src/gather_token_comments_above.rs ]
crate::ix!();

pub fn gather_token_comments_above(start_node: &SyntaxNode) -> Vec<String> {
    info!(
        "gather_token_comments_above => start; node.kind()={:?}",
        start_node.kind()
    );

    let tokens: Vec<SyntaxElement> = start_node.children_with_tokens().collect();
    if tokens.is_empty() {
        debug!("No tokens => returning empty");
        info!("gather_token_comments_above => done => returning empty");
        return Vec::new();
    }

    let mut collected = Vec::new();
    let mut found_comment = false;
    let mut whitespace_newlines_since_last_nonws = 0;

    for elem in &tokens {
        match elem.kind() {
            SyntaxKind::COMMENT => {
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
                if let Some(embedded) = try_extract_embedded_comment_in_whitespace(&txt_no_cr) {
                    trace!("Found {} embedded comments in whitespace", embedded.len());
                    for line in embedded {
                        let forced = if line.ends_with('\n') {
                            line
                        } else {
                            format!("{}\n", line)
                        };
                        collected.push(forced);
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
                        if newline_count >= 2 {
                            warn!("Blank line after comment => stopping collection");
                            break;
                        }
                        whitespace_newlines_since_last_nonws += newline_count;
                    } else {
                        if newline_count >= 2 {
                            warn!("Blank line blocks => returning empty immediately");
                            info!("gather_token_comments_above => done => returning empty");
                            return Vec::new();
                        } else {
                            whitespace_newlines_since_last_nonws += newline_count;
                        }
                    }
                }
            }
            other_kind => {
                trace!(
                    "Encountered non-comment token => kind={:?}, ws_newlines={}",
                    other_kind,
                    whitespace_newlines_since_last_nonws
                );
                if whitespace_newlines_since_last_nonws < 2 {
                    debug!("Non-comment token with <2 newlines => stopping");
                    break;
                } else {
                    debug!("Non-comment token with >=2 newlines => skip & continue");
                    whitespace_newlines_since_last_nonws = 0;
                }
            }
        }
    }

    debug!(
        "gather_token_comments_above => collected {} lines => returning",
        collected.len()
    );
    info!("gather_token_comments_above => done");
    collected
}

#[cfg(test)]
mod test_gather_token_comments_above {
    use super::*;

    fn first_fn_node(src: &str) -> SyntaxNode {
        info!("first_fn_node => start");
        trace!("Parsing src:\n{}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        debug!("Parsed SourceFile => searching for FN node");
        let node = file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .expect("Expected an FN node");
        info!("first_fn_node => done");
        node
    }

    #[traced_test]
    fn test_token_comments_no_comment() {
        info!("test_token_comments_no_comment => start");
        let src = "fn main() {}";
        let fn_node = first_fn_node(src);
        let collected = gather_token_comments_above(&fn_node);
        debug!("collected={:?}", collected);
        assert!(collected.is_empty(), "Expected no comment above");
        info!("test_token_comments_no_comment => success");
    }

    #[traced_test]
    fn test_token_comments_blank_line_blocks() {
        info!("test_token_comments_blank_line_blocks => start");
        let src = r#"
// Some comment

fn foo() {}
"#;
        let fn_node = first_fn_node(src);
        let collected = gather_token_comments_above(&fn_node);
        debug!("collected={:?}", collected);
        assert!(
            collected.is_empty(),
            "Blank line should block the comment above"
        );
        info!("test_token_comments_blank_line_blocks => success");
    }

    fn parse_and_find_fn(src: &str) -> SyntaxNode {
        info!("parse_and_find_fn => start");
        trace!("Parsing src:\n{}", src);
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        debug!("Parsed file => searching for FN node");
        let node = file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap();
        info!("parse_and_find_fn => done");
        node
    }

    #[traced_test]
    fn test_token_comments_comment_directly_above() {
        info!("test_token_comments_comment_directly_above => start");
        let src = r#"
// This is above main
fn main() {}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = gather_token_comments_above(&fn_node);
        debug!("collected={:?}", gathered);
        assert_eq!(
            gathered.len(),
            1,
            "Expected exactly one comment line directly above"
        );
        assert_eq!(gathered[0], "// This is above main\n");
        info!("test_token_comments_comment_directly_above => success");
    }
}
