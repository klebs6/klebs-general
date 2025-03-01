// ---------------- [ File: src/gather_leading_comments.rs ]
crate::ix!();

/// A helper that walks backward from `item.syntax()` to pick up line-comments,
/// similarly to what we do for old macros. If no separate tokens are found,
/// but the item’s own text starts with `//`, we parse those lines directly from 
/// `full_text` as a fallback.
pub fn gather_leading_comments(item: &ast::Item) -> String {
    let mut lines = Vec::new();
    let mut cur = item.syntax().prev_sibling_or_token();
    let mut encountered_blank = false;

    // 1) Normal RA-based walk upward, collecting comments from preceding tokens
    while let Some(n_or_t) = cur {
        match n_or_t {
            NodeOrToken::Token(tok) => {
                let kind = tok.kind();
                let txt  = tok.text().to_string();

                match kind {
                    SyntaxKind::COMMENT => {
                        if encountered_blank {
                            break;
                        }
                        // Prepend
                        lines.push(txt);
                    }
                    SyntaxKind::WHITESPACE => {
                        // If there's a blank line => stop
                        if txt.matches('\n').count() >= 2 {
                            encountered_blank = true;
                            break;
                        }
                    }
                    _ => {
                        // Some non-comment => stop
                        break;
                    }
                }
                // Move upward
                cur = tok.prev_sibling_or_token();
            }
            NodeOrToken::Node(_) => {
                // Another node => stop
                break;
            }
        }
    }

    lines.reverse();
    let mut joined = if !lines.is_empty() {
        // We found at least one preceding comment token => join them
        let tmp = lines.join("\n");
        // Ensure exactly one trailing newline
        if tmp.ends_with('\n') { tmp } else { format!("{tmp}\n") }
    } else {
        String::new()
    };

    // 2) Fallback: If joined is empty but the item text itself starts with `//`, 
    // RA likely merged top-of-file comment lines into the same syntax node. 
    // We'll parse them out from the front of the item’s full text, stopping 
    // at the first blank line or any line that doesn't start with `//`.
    if joined.is_empty() {
        let full_text = item.syntax().text().to_string();
        let trimmed = full_text.trim_start();
        if trimmed.starts_with("//") {
            // We'll gather consecutive lines of the form `// ...`, 
            // stopping at the first blank line or a line that doesn't start with `//`.
            let mut out = Vec::new();
            for line in full_text.lines() {
                let tline = line.trim_start();
                if tline.starts_with("//") {
                    out.push(line);
                } else if tline.is_empty() {
                    // a blank line => stop
                    break;
                } else {
                    // a non-comment line => stop
                    break;
                }
            }
            if !out.is_empty() {
                let tmp = out.join("\n");
                // ensure exactly one trailing newline
                joined = if tmp.ends_with('\n') {
                    tmp
                } else {
                    format!("{tmp}\n")
                };
            }
        }
    }

    joined
}

#[cfg(test)]
mod test_gather_leading_comments {
    use super::*;
    use ra_ap_syntax::{SourceFile, Edition, ast, SyntaxKind, SyntaxNode, SyntaxToken, NodeOrToken};
    use tracing::{trace, debug};

    /// Helper to parse snippet & pick the first top-level item, for which we'll gather comments.
    fn parse_and_pick_item(text: &str) -> Option<ast::Item> {
        let parse = SourceFile::parse(text, Edition::Edition2021);
        let file = parse.tree();
        file.items().next()
    }

    /// 1) If there's no comment => result is ""
    #[traced_test]
    fn test_no_comments() {
        trace!("Starting test_no_comments for gather_leading_comments");
        let text = r#"fn something() {}"#;
        let maybe_item = parse_and_pick_item(text);
        assert!(maybe_item.is_some());

        let item = maybe_item.unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);
        assert_eq!(result, "", "No comments => no result");
    }

    /// 2) Single line comment directly above => recognized
    #[traced_test]
    fn test_single_line_comment() {
        trace!("Starting test_single_line_comment for gather_leading_comments");
        let text = r#"
// This is a doc
fn something() {}
"#;
        let maybe_item = parse_and_pick_item(text).expect("Should parse one item");
        let result = gather_leading_comments(&maybe_item);
        debug!("Result = {:?}", result);
        assert!(result.contains("This is a doc"), "Expected to see the doc line");
        assert!(result.ends_with('\n'), "Should end with exactly one newline");
    }

    /// 3) Multiple consecutive comments => gather them all
    #[traced_test]
    fn test_multiple_comments() {
        trace!("Starting test_multiple_comments for gather_leading_comments");
        let text = r#"
// First line
// Second line
fn foo() {}
"#;
        let item = parse_and_pick_item(text).unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);

        assert!(result.contains("First line"), "Should have first line");
        assert!(result.contains("Second line"), "Should have second line");
        assert_eq!(result.matches('\n').count(), 2, "We have 2 lines + final newline => total 2 newlines");
    }

    /// 4) If there's a blank line => we stop collecting
    #[traced_test]
    fn test_blank_line_stops_collecting() {
        trace!("Starting test_blank_line_stops_collecting for gather_leading_comments");
        let text = r#"
// good line
// second doc

fn something() {}
"#;
        let item = parse_and_pick_item(text).unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);

        // The blank line means we do not attach any lines above that blank line.
        // So we expect "good line" and "second doc" are *NOT* attached, because
        // there's a blank line before the item. 
        // Or, if we are reversing logic, it might skip them. 
        // Let's confirm your gather_leading_comments does that:
        assert_eq!(result, "", "Blank line => no attached comments after it");
    }

    /// 5) If we place whitespace with only one newline => we keep collecting
    #[traced_test]
    fn test_single_newline_between_comments_still_collects() {
        trace!("Starting test_single_newline_between_comments_still_collects for gather_leading_comments");
        let text = r#"
// doc line
// doc line2
fn something() {}
"#;
        let item = parse_and_pick_item(text).unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);

        // We didn't see 2 consecutive newlines => so we keep them
        assert!(result.contains("doc line\n"), "Should keep doc line");
        assert!(result.contains("doc line2"), "Should keep doc line2");
    }

    /// 6) Comments after the item => not recognized
    #[traced_test]
    fn test_comments_after_item_are_ignored() {
        trace!("Starting test_comments_after_item_are_ignored for gather_leading_comments");
        let text = r#"
fn something() {}
// trailing comment
"#;
        let item = parse_and_pick_item(text).unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);
        assert!(result.is_empty(), "No leading comments => empty string");
    }

    /// 7) If there's already a newline in the last comment => we won't add an extra one.
    ///    But typically each line comment ends with `\n` from the parser. 
    ///    We'll do a small test verifying the final string ends with exactly one newline.
    #[traced_test]
    fn test_ensure_exactly_one_trailing_newline() {
        trace!("Starting test_ensure_exactly_one_trailing_newline for gather_leading_comments");
        let text = r#"
// line one
// line two
fn main() {}
"#;
        let item = parse_and_pick_item(text).unwrap();
        let result = gather_leading_comments(&item);
        debug!("Result = {:?}", result);

        assert!(result.contains("line one"));
        assert!(result.contains("line two"));
        // Should end with exactly one newline
        let last_char = result.chars().last().unwrap_or('\0');
        assert_eq!(last_char, '\n', "Should end with a newline");
        // The total line count is 2 comment lines + final newline => 2 newlines.
        // But the function ensures there's exactly 1 trailing newline at the end of all comments.
    }
}
