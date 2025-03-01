crate::ix!();

/// Climb upward (siblings/tokens) from `start_node`, collecting comment lines.
/// Returns them in top-to-bottom order if found, else empty.
///
/// The rules implemented:
///  - If we see a comment token, collect it (with forced trailing `\n`).
///  - If we see whitespace, we count newlines:
///    * If we already collected a comment, >=2 newlines => stop (blank line).
///    * If we haven't found a comment yet and see >=2 newlines, we peek what's above:
///      - if next non-whitespace is comment => block it => return empty
///      - else skip the node (for "node_above_stops" scenario).
///  - If we see a non-comment node with <2 newlines, we stop. If >=2 newlines, we skip it.
pub fn scan_preceding_tokens_for_comments(start_node: &SyntaxNode) -> Vec<String> {
    trace!(
        "scan_preceding_tokens_for_comments => node.kind()={:?}",
        start_node.kind()
    );

    let mut collected = Vec::new();
    let mut found_comment = false;
    let mut whitespace_newlines_since_last_nonws = 0;
    let mut cur = start_node.prev_sibling_or_token();

    while let Some(n_or_t) = cur {
        match n_or_t {
            NodeOrToken::Token(tok) => {
                let kind = tok.kind();
                let txt = tok.text().to_string();

                match kind {
                    SyntaxKind::COMMENT => {
                        let line = if txt.ends_with('\n') {
                            txt
                        } else {
                            format!("{}\n", txt)
                        };
                        collected.insert(0, line);
                        found_comment = true;
                        whitespace_newlines_since_last_nonws = 0;
                        cur = tok.prev_sibling_or_token();
                    }
                    SyntaxKind::WHITESPACE => {
                        let newline_count = txt.matches('\n').count();
                        if found_comment {
                            // if already have comments, 2+ newlines => stop
                            if newline_count >= 2 {
                                debug!("PrecedingTokens: Found blank line => stop");
                                break;
                            }
                            whitespace_newlines_since_last_nonws += newline_count;
                            cur = tok.prev_sibling_or_token();
                        } else {
                            // no comment found yet
                            if newline_count >= 2 {
                                // peek above
                                let above = tok.prev_sibling_or_token();
                                if let Some(is_comment) = super::peek_next_non_whitespace(&above) {
                                    if is_comment {
                                        debug!("Blank line blocks => returning empty");
                                        return Vec::new();
                                    } else {
                                        debug!("Blank line, next is node => skip node => keep climbing");
                                        whitespace_newlines_since_last_nonws = 0;
                                        cur = above.and_then(|elt| elt.prev_sibling_or_token());
                                    }
                                } else {
                                    debug!("Blank line => nothing above => empty");
                                    return Vec::new();
                                }
                            } else {
                                whitespace_newlines_since_last_nonws += newline_count;
                                cur = tok.prev_sibling_or_token();
                            }
                        }
                    }
                    // Non-comment token => treat like a node
                    _ => {
                        if whitespace_newlines_since_last_nonws < 2 {
                            debug!("Non-comment token with <2 newlines => stop");
                            break;
                        } else {
                            debug!("Non-comment token with >=2 newlines => skip & continue");
                            whitespace_newlines_since_last_nonws = 0;
                            cur = tok.prev_sibling_or_token();
                        }
                    }
                }
            }
            NodeOrToken::Node(_) => {
                if whitespace_newlines_since_last_nonws < 2 {
                    debug!("Encountered node with <2 newlines => stop");
                    break;
                } else {
                    debug!("Encountered node with >=2 => skip & continue");
                    whitespace_newlines_since_last_nonws = 0;
                    cur = n_or_t.prev_sibling_or_token();
                }
            }
        }
    }

    collected
}

#[cfg(test)]
mod test_scan_preceding_tokens_for_comments {
    use super::*;
    use ra_ap_syntax::{SourceFile, SyntaxKind};

    fn parse_and_find_fn(src: &str) -> SyntaxNode {
        let parse = SourceFile::parse(src, ra_ap_syntax::Edition::Edition2021);
        let file = parse.tree();
        // We'll find the first "fn"
        file.syntax()
            .descendants()
            .find(|n| n.kind() == SyntaxKind::FN)
            .unwrap()
    }

    #[traced_test]
    fn test_no_comments_above() {
        // Just blank line or no line => empty
        let src = r#"
fn main(){}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        assert!(gathered.is_empty());
    }

    #[traced_test]
    fn test_blank_line_then_comment_is_blocked() {
        // If there's a blank line, and then a comment above that, it should return empty
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

    // etc. Expand as needed
    #[traced_test]
    fn test_comment_directly_above() {
        let src = r#"
// My comment
fn main() {}
"#;
        let fn_node = parse_and_find_fn(src);
        let gathered = scan_preceding_tokens_for_comments(&fn_node);
        assert_eq!(gathered.len(), 1, "We want exactly one comment directly above");
        assert_eq!(gathered[0], "// My comment\n");
    }
}
