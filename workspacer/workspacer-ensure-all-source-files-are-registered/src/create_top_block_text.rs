// ---------------- [ File: src/create_top_block_text.rs ]
crate::ix!();

pub fn create_top_block_text(tops: &[TopBlockMacro]) -> String {
    // You might have some initial comment lines or text you want at top:
    let mut buffer = String::new();

    for top in tops {
        // Insert the user’s leading comments (which might include doc lines or `// ...`)
        if !top.leading_comments.is_empty() {
            // Ensure we separate them from prior macro with a newline if needed
            if !buffer.ends_with('\n') {
                buffer.push('\n');
            }
            buffer.push_str(&top.leading_comments);
        }
        // Then add the macro line
        buffer.push_str(&format!("x!{{{}}}\n", top.stem));
    }

    // Trim trailing newlines
    while buffer.ends_with('\n') {
        buffer.pop();
    }

    buffer
}

#[cfg(test)]
mod test_create_top_block_text {
    use super::*;

    /// 1) If both `non_macro_lines` and `stems` are empty => result is empty
    #[traced_test]
    fn test_empty_lines_and_stems() {
        let lines: Vec<String> = vec![];
        let stems: Vec<String> = vec![];
        let result = create_top_block_text(&lines, &stems);

        assert!(
            result.is_empty(),
            "Expected an empty result if we have no lines and no stems."
        );
    }

    /// 2) If we have non-macro lines but no stems => we just return those lines (with no trailing newline).
    #[traced_test]
    fn test_lines_no_stems() {
        let lines = vec![
            "line one".to_string(),
            "line two".to_string(),
            "line three".to_string(),
        ];
        let stems: Vec<String> = vec![];

        let result = create_top_block_text(&lines, &stems);

        // We expect:
        // line one
        // line two
        // line three
        // (no trailing newline)
        let expected = "line one\nline two\nline three";
        assert_eq!(result, expected);
    }

    /// 3) If we have no lines but some stems => the block is just one line per stem, no trailing newline.
    #[traced_test]
    fn test_no_lines_some_stems() {
        let lines: Vec<String> = vec![];
        let stems = vec!["alpha".to_string(), "beta".to_string()];

        let result = create_top_block_text(&lines, &stems);

        // We expect:
        // x!{alpha}
        // x!{beta}
        // (with no extra trailing newline)
        let expected = "x!{alpha}\nx!{beta}";
        assert_eq!(result, expected);
    }

    /// 4) If we have both lines and stems, we first list the lines, then the stems, no trailing newline.
    #[traced_test]
    fn test_both_lines_and_stems() {
        let lines = vec![
            "// top block".to_string(),
            "// some other line".to_string(),
        ];
        let stems = vec!["thing_a".to_string(), "thing_b".to_string()];

        let result = create_top_block_text(&lines, &stems);

        let expected = r#"// top block
// some other line
x!{thing_a}
x!{thing_b}"#;

        assert_eq!(result, expected);
    }

    /// 5) Verify that if lines or stems contain trailing newlines in them, 
    ///    we still unify them as single lines and remove trailing newlines from the final result.
    ///    This is partially tested by the function's logic, but let's ensure it doesn't break.
    #[traced_test]
    fn test_trailing_newlines_removed() {
        let lines = vec![
            "something ".to_string(),
            "another line ".to_string(),
            "".to_string(), // blank line
        ];
        let stems = vec!["stem_a".to_string()];

        let result = create_top_block_text(&lines, &stems);
        // The function always appends '\n' after each line or stem, 
        // then strips any trailing '\n' from the entire output.
        // So the final text won't end with a newline.
        let expected = r#"something 
another line 

x!{stem_a}"#;

        assert_eq!(result, expected);
        assert!(
            !result.ends_with('\n'),
            "Should not end with a newline in final text"
        );
    }

    /// 6) Multiple stems => each on its own line, in the order given.
    #[traced_test]
    fn test_multiple_stems_order() {
        let lines: Vec<String> = vec!["header".to_string()];
        let stems = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()];

        let result = create_top_block_text(&lines, &stems);
        let expected = r#"header
x!{aaa}
x!{bbb}
x!{ccc}"#;
        assert_eq!(result, expected);
    }

    /// 7) Non-macro lines might include blank lines, 
    ///    we keep them as-is, each followed by a newline, except for the very last one is trimmed.
    #[traced_test]
    fn test_blank_lines_among_non_macro() {
        let lines = vec![
            "// line 1".to_string(),
            "".to_string(), // blank line
            "// line 2".to_string(),
            "  ".to_string(), // line with just spaces
        ];
        let stems: Vec<String> = vec![];

        let result = create_top_block_text(&lines, &stems);

        // We do keep these lines, each ends with '\n' in the buffer, 
        // but we strip the trailing one. So the final won't end with newline:
        let expected = r#"// line 1

// line 2
  "#;
        assert_eq!(result, expected);
    }

    /// 8) Stems with special characters => we literally do x!{special_char_stem}.
    #[traced_test]
    fn test_stems_with_special_characters() {
        let lines: Vec<String> = vec!["// special block".to_string()];
        let stems = vec![
            "foo/bar".to_string(),
            "hello-world".to_string(),
            "some_λ_symbol".to_string(),
        ];

        let result = create_top_block_text(&lines, &stems);

        let expected = r#"// special block
x!{foo/bar}
x!{hello-world}
x!{some_λ_symbol}"#;
        assert_eq!(result, expected);
    }

    /// 9) If some lines are duplicates or repeated, we keep them as is — 
    ///    this function doesn't do any dedup of lines.
    #[traced_test]
    fn test_repeated_non_macro_lines() {
        let lines = vec![
            "repeat".to_string(),
            "repeat".to_string(),
            "unique".to_string(),
            "repeat".to_string(),
        ];
        let stems: Vec<String> = vec![];

        let result = create_top_block_text(&lines, &stems);

        let expected = r#"repeat
repeat
unique
repeat"#;
        assert_eq!(result, expected);
    }

    /// 10) A small integration scenario: non-macro lines + multiple stems + blank lines.
    #[traced_test]
    fn test_integration_mixed() {
        let lines = vec![
            "// commentary".to_string(),
            "".to_string(),
            "another line".to_string(),
        ];
        let stems = vec!["foo".to_string(), "bar".to_string()];

        let result = create_top_block_text(&lines, &stems);

        let expected = r#"// commentary

another line
x!{foo}
x!{bar}"#;
        assert_eq!(result, expected);
    }
}
