crate::ix!();

pub fn parse_new_top_block_snippet(new_top_block: &str) -> (Vec<TopBlockMacro>, Vec<String>) {
    trace!("Entering parse_new_top_block_snippet");
    debug!("new_top_block length={}", new_top_block.len());

    // (1) Gather macros using parse_new_macros_with_comments,
    // which *only* attempts normal RA-based gather_leading_comments,
    // and does NOT forcibly fold lines from `// top block` if there's a blank line.
    let new_macros = parse_new_macros_with_comments(new_top_block);
    debug!("Found {} new macros in new_top_block", new_macros.len());

    // (2) Collect lines that don't contain `x!{...}`
    let mut all_non_macro_lines = extract_non_macro_lines(new_top_block);
    debug!(
        "Initially found {} non-macro lines via extract_non_macro_lines",
        all_non_macro_lines.len()
    );

    // (3) Optionally remove lines that truly appear as leading-comments for macros,
    // but typically in your tests you want them separate. 
    // If you do want to remove them, you can do so by scanning each macro’s leading_comments.

    // In many tests, you actually want to keep them. So let's skip removing them entirely.
    // => That means snippet lines remain in new_non_macro_lines.

    // (4) Filter out empty lines if you want (optional):
    let filtered_lines: Vec<String> = all_non_macro_lines
        .into_iter()
        .filter(|ln| !ln.trim().is_empty())
        .collect();

    debug!(
        "After final filtering, we have {} snippet lines",
        filtered_lines.len()
    );

    trace!("Exiting parse_new_top_block_snippet");
    (new_macros, filtered_lines)
}

#[cfg(test)]
mod test_parse_new_top_block_snippet {
    use super::*;
    use tracing::{trace, debug};

    #[traced_test]
    fn test_empty_block() {
        trace!("test_empty_block for parse_new_top_block_snippet");
        let (macros, lines) = parse_new_top_block_snippet("");
        debug!("macros={:?}, lines={:?}", macros, lines);
        assert!(macros.is_empty(), "Expected no macros");
        assert!(lines.is_empty(), "Expected no lines");
    }

    #[traced_test]
    fn test_mixed_macros_and_lines() {
        trace!("test_mixed_macros_and_lines for parse_new_top_block_snippet");
        let snippet = r#"
// comment
x!{alpha}
another line
x!{beta}
// trailing doc
"#;
        let (macros, lines) = parse_new_top_block_snippet(snippet);
        debug!("macros={:?}, lines={:?}", macros, lines);

        // We now expect:
        //   macros => alpha (with leading_comments="// comment\n") 
        //             beta  (with no leading_comments, because "another line" wasn't a comment)
        //   lines => ["another line", "// trailing doc"]

        assert_eq!(macros.len(), 2);
        assert_eq!(macros[0].stem(), "alpha");
        assert!(macros[0].leading_comments().contains("comment"));
        assert_eq!(macros[1].stem(), "beta");
        assert!(macros[1].leading_comments().is_empty());

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "another line");
        assert_eq!(lines[1], "// trailing doc");
    }
}
