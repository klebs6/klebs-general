crate::ix!();

pub fn parse_new_top_block_snippet(new_top_block: &str) -> (Vec<TopBlockMacro>, Vec<String>) {
    trace!("Entering parse_new_top_block_snippet");
    debug!("new_top_block length={}", new_top_block.len());

    // (1) Gather macros using RA-based parsing so we pick up leading comments:
    let new_macros = parse_new_macros_with_comments(new_top_block);
    debug!("Found {} new macros in new_top_block", new_macros.len());

    // (2) Collect lines that do NOT contain `x!{...}` (so, likely user snippet lines).
    let mut candidate_lines = extract_non_macro_lines(new_top_block);
    debug!(
        "Initially found {} non-macro lines via extract_non_macro_lines",
        candidate_lines.len()
    );

    // (3) Remove from `candidate_lines` any lines that appear in a macro’s leading_comments.
    //     Because we want those comment lines to stay "attached" to the macro, not counted
    //     as separate snippet lines.
    for mac in &new_macros {
        for comment_line in mac.leading_comments().lines() {
            // We remove exact matches to that line, ignoring only the trailing newline from the macro text itself.
            let comment_line = comment_line; // direct, no trimming by default
            candidate_lines.retain(|cl| cl != comment_line);
        }
    }

    // (4) Filter out any totally empty lines after that.
    let snippet_lines: Vec<String> = candidate_lines
        .into_iter()
        .filter(|ln| !ln.trim().is_empty())
        .collect();

    debug!(
        "After removing macro-leading lines and filtering empties, we have {} snippet lines",
        snippet_lines.len()
    );
    trace!("Exiting parse_new_top_block_snippet");

    (new_macros, snippet_lines)
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
