// ---------------- [ File: src/assemble_final_top_block_snippet.rs ]
crate::ix!();

/// Builds the final top-block snippet by combining:
///   - old_top_macros (possibly with leading comments),
///   - user-provided non-macro lines (like snippet lines),
///   - new_top_macros (possibly with leading comments),
/// in either "imports line" order or "no imports" order.
/// 
/// This is a pure function: it does *not* parse or modify old_text, 
/// it simply assembles the final snippet into a `String`.
pub fn assemble_final_top_block_snippet(
    has_imports_line: bool,
    old_top_macros: &[TopBlockMacro],
    new_top_macros: &[TopBlockMacro],
    non_macro_lines: &[String],
) -> String {
    trace!("Entering assemble_final_top_block_snippet");
    debug!("has_imports_line={}", has_imports_line);
    debug!(
        "old_top_macros.len()={}, new_top_macros.len()={}, non_macro_lines.len()={}",
        old_top_macros.len(),
        new_top_macros.len(),
        non_macro_lines.len()
    );

    // We'll produce the snippet in a buffer; then remove trailing newlines.
    let mut buffer = if has_imports_line {
        trace!("has_imports_line=true => old macros first, snippet lines, then new macros");
        build_top_block_for_imports_line(old_top_macros, non_macro_lines, new_top_macros)
    } else {
        trace!("has_imports_line=false => snippet lines first, old macros, then new macros");
        build_top_block_for_no_imports_line(old_top_macros, non_macro_lines, new_top_macros)
    };

    remove_trailing_newlines(&mut buffer);

    debug!(
        "assemble_final_top_block_snippet => final snippet length={}",
        buffer.len()
    );
    trace!("Exiting assemble_final_top_block_snippet");
    buffer
}

#[cfg(test)]
mod test_assemble_final_top_block_snippet {
    use super::*;
    use tracing::{trace, debug};

    /// Helper: quickly build a `TopBlockMacro` with given stem & optional leading comments
    fn tb_macro(stem: &str, comments: &str) -> TopBlockMacro {
        TopBlockMacroBuilder::default()
            .stem(stem.to_string())
            .leading_comments(comments.to_string())
            .build()
            .unwrap()
    }

    /// 1) If `has_imports_line = true` => old macros first, snippet lines, then new macros
    #[traced_test]
    fn test_with_imports_line() {
        trace!("Starting test_with_imports_line for assemble_final_top_block_snippet");
        let old = vec![
            tb_macro("alpha", "// doc for alpha\n"),
            tb_macro("beta", ""),
        ];
        let new_lines = vec!["// user snippet".to_string()];
        let new_macros = vec![
            tb_macro("gamma", "// doc for gamma\n"),
        ];

        let result = assemble_final_top_block_snippet(true, &old, &new_macros, &new_lines);
        debug!("Result:\n{}", result);

        // We expect old macros first:
        //   x!{alpha}, x!{beta}, then snippet line, then x!{gamma}
        // Also alpha has a leading comment.
        let expected_substring = r#"// doc for alpha
x!{alpha}
x!{beta}
// user snippet
// doc for gamma
x!{gamma}"#;
        assert!(result.contains(expected_substring), "must contain that ordering");
    }

    /// 2) If `has_imports_line = false` => snippet lines first, old macros next, new macros last
    #[traced_test]
    fn test_no_imports_line() {
        trace!("Starting test_no_imports_line for assemble_final_top_block_snippet");
        let old = vec![
            tb_macro("foo", "// doc-foo\n"),
        ];
        let new_lines = vec!["// snippet top".to_string(), "// snippet next".to_string()];
        let new_macros = vec![
            tb_macro("bar", ""),
        ];

        let result = assemble_final_top_block_snippet(false, &old, &new_macros, &new_lines);
        debug!("Result:\n{}", result);

        // snippet lines first => 
        //   // snippet top
        //   // snippet next
        // then old macros => 
        //   // doc-foo
        //   x!{foo}
        // then new macros => x!{bar}
        let expected_substring = r#"// snippet top
// snippet next
// doc-foo
x!{foo}
x!{bar}"#;
        assert!(result.contains(expected_substring), "must contain that ordering");
    }

    /// 3) Trailing newlines are trimmed
    #[traced_test]
    fn test_trailing_newline_trimmed() {
        trace!("Starting test_trailing_newline_trimmed for assemble_final_top_block_snippet");
        let old = vec![tb_macro("alpha", "")];
        let new_lines = vec![];
        let new_macros = vec![tb_macro("beta", "")];

        let result = assemble_final_top_block_snippet(true, &old, &new_macros, &new_lines);
        debug!("Result:\n{}", result);

        // The function removes trailing newlines. Let's ensure it doesn't end with \n
        assert!(!result.ends_with('\n'), "Should have trailing newlines trimmed");
    }

    /// 4) If there's nothing in old macros, snippet lines, or new macros => returns empty
    #[traced_test]
    fn test_all_empty() {
        trace!("Starting test_all_empty for assemble_final_top_block_snippet");
        let result = assemble_final_top_block_snippet(false, &[], &[], &[]);
        debug!("Result:\n{}", result);
        assert!(result.is_empty(), "No macros, no lines => empty output");
    }

    /// 5) Leading comments should appear just above each macro with exactly one newline separation
    #[traced_test]
    fn test_leading_comments_on_macros() {
        trace!("Starting test_leading_comments_on_macros for assemble_final_top_block_snippet");
        let old = vec![
            tb_macro("one", "// first doc\n"),
            tb_macro("two", "// second doc\n// more doc\n"),
        ];
        let result = assemble_final_top_block_snippet(true, &old, &[], &[]);
        debug!("Result:\n{}", result);

        // Should see
        //   // first doc
        //   x!{one}
        //   // second doc
        //   // more doc
        //   x!{two}
        assert!(result.contains("// first doc\nx!{one}"), "Doc for macro 'one'");
        assert!(result.contains("// second doc\n// more doc\nx!{two}"), "Doc for macro 'two'");
    }
}
