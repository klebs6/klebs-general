// ---------------- [ File: src/create_top_block_text.rs ]
crate::ix!();

pub fn create_top_block_text(tops: &[TopBlockMacro]) -> String {
    trace!("Entering create_top_block_text");
    debug!("tops.len()={}", tops.len());

    let mut buffer = String::new();

    for top in tops {
        // (1) Insert the macro’s leading comments, but only insert a
        //     separating newline if the buffer is *non-empty*.
        if !top.leading_comments().is_empty() {
            // => *only* if buffer is already non-empty do we push a newline:
            if !buffer.is_empty() && !buffer.ends_with('\n') {
                buffer.push('\n');
            }
            buffer.push_str(top.leading_comments());
        }

        // (2) Then add the macro line, e.g. `x!{stem}`
        //     always add a trailing newline, which we’ll trim at the end.
        buffer.push_str(&format!("x!{{{}}}\n", top.stem()));
    }

    // (3) Trim trailing newlines
    while buffer.ends_with('\n') {
        buffer.pop();
    }

    debug!("Exiting create_top_block_text =>\n{}", buffer);
    buffer
}

#[cfg(test)]
mod test_create_top_block_text {
    use super::*;
    use tracing::{trace, debug};

    /// Helper for building a `TopBlockMacro`.
    fn tb_macro(stem: &str, comments: &str) -> TopBlockMacro {
        TopBlockMacroBuilder::default()
            .stem(stem.to_string())
            .leading_comments(comments.to_string())
            .build()
            .unwrap()
    }

    /// 1) If the list is empty => returns ""
    #[traced_test]
    fn test_empty_list() {
        trace!("Starting test_empty_list for create_top_block_text");
        let result = create_top_block_text(&[]);
        debug!("Result:\n{}", result);
        assert!(result.is_empty(), "No macros => empty string");
    }

    /// 2) Single macro with no leading comment => just "x!{stem}"
    #[traced_test]
    fn test_single_macro_no_comment() {
        trace!("Starting test_single_macro_no_comment for create_top_block_text");
        let macros = [tb_macro("alpha", "")];
        let result = create_top_block_text(&macros);
        debug!("Result:\n{}", result);

        // Should be "x!{alpha}" with no trailing newline
        // Actually the code ends with while buffer.ends_with('\n') => pop.
        // But we DO push a newline after each macro, so let's see.
        // We do `buffer.push_str(&format!("x!{{{}}}\n", top.stem()));`
        // Then we strip trailing newlines. => final is "x!{alpha}" with no newline
        assert_eq!(result, "x!{alpha}");
    }

    /// 3) Single macro with leading comment => comment + macro
    #[traced_test]
    fn test_single_macro_with_comment() {
        trace!("Starting test_single_macro_with_comment for create_top_block_text");
        let macros = [tb_macro("beta", "// doc about beta\n")];
        let result = create_top_block_text(&macros);
        debug!("Result:\n{}", result);

        // We push a newline if !buffer.ends_with('\n')
        // Then "comment"
        // Then "x!{beta}\n"
        // Then we strip trailing newlines => final => 
        //   // doc about beta
        //   x!{beta}
        let expected = r#"// doc about beta
x!{beta}"#;
        assert_eq!(result, expected);
    }

    /// 4) Multiple macros => each with optional leading comment
    #[traced_test]
    fn test_multiple_macros() {
        trace!("Starting test_multiple_macros for create_top_block_text");
        let macros = [
            tb_macro("one", "// doc1\n"),
            tb_macro("two", ""), 
            tb_macro("three", "// doc3\n"),
        ];
        let result = create_top_block_text(&macros);
        debug!("Result:\n{}", result);

        // We'll see something like:
        //   // doc1
        //   x!{one}
        //   x!{two}
        //   // doc3
        //   x!{three}
        //
        // check final has no trailing newline
        assert!(result.contains("// doc1\nx!{one}"));
        assert!(result.contains("x!{two}"));
        assert!(result.contains("// doc3\nx!{three}"));
        assert!(!result.ends_with('\n'), "No trailing newline");
    }

    /// 5) Already starts with a leading comment => no extra blank line at top
    #[traced_test]
    fn test_no_unwanted_blank_line_at_start() {
        trace!("Starting test_no_unwanted_blank_line_at_start for create_top_block_text");
        let macros = [tb_macro("zeta", "// docz\n")];
        let result = create_top_block_text(&macros);
        debug!("Result:\n{}", result);

        // Should not start with an extra newline. Should start with `// docz`
        assert!(result.starts_with("// docz"), "Should not have an extra blank line at top");
    }
}
