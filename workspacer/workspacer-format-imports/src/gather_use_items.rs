// ---------------- [ File: src/gather_use_items.rs ]
crate::ix!();

pub fn gather_use_items(file: &SourceFile, old_text: &str) -> Vec<UseItemInfo> {
    info!("Entering gather_use_items => file has {} items", file.items().count());
    let mut uses_data = Vec::new();

    for item in file.items() {
        // Only if it's a `use` item
        if let Some(u) = ast::Use::cast(item.syntax().clone()) {
            let rng = u.syntax().text_range();
            let start: usize = rng.start().into();
            let end:   usize = rng.end().into();

            let entire_range_text = &old_text[start..end];
            debug!("Found a `use` item at range {}..{}, raw_text:\n{}", start, end, entire_range_text);

            // Gather leading comments
            let leading_comments = gather_leading_comment_lines(u.syntax(), old_text);
            debug!("Leading comments => {:#?}", leading_comments);

            // We may detect a trailing same-line comment after the semicolon
            // (or after the block if there is no semicolon for some reason).
            let mut trailing_comment = None;
            if let Some((tc_text, tc_len)) = detect_trailing_comment_same_line(old_text, end) {
                debug!("Found trailing comment => {:?}, length={}", tc_text, tc_len);

                // Expand the range_end to cover the trailing comment
                // so we remove it from the old file in remove_old_use_statements
                let new_end = (end + tc_len).min(old_text.len());
                debug!("Expanded range_end from {} to {}", end, new_end);

                // We'll store that comment text in the struct
                trailing_comment = Some(tc_text);
                // Then we actually set `end` to the new_end for the final UseItemInfo
                // so that we don't leave the trailing comment behind in the old file
            }

            // Some AST fragments include leading comments in the same node text,
            // so remove exactly those leading comment bytes from `entire_range_text`.
            let leading_comment_bytes: usize = leading_comments.iter().map(|c| c.len()).sum();
            debug!("Truncating {} bytes of leading comment text from raw_text", leading_comment_bytes);

            let truncated_text = if leading_comment_bytes < entire_range_text.len() {
                &entire_range_text[leading_comment_bytes..]
            } else {
                ""
            };
            let truncated_text = truncated_text.trim_start();
            debug!("Truncated text for dissect:\n{}", truncated_text);

            // Attempt to dissect
            if let Some((vis, _use_kw, path_list)) = dissect_use_statement(truncated_text) {
                debug!("Dissect => visibility='{}', path_list='{}'", vis, path_list);

                let info = UseItemInfoBuilder::default()
                    .leading_comments(leading_comments)
                    .raw_text(entire_range_text.to_string())
                    .range_start(start)
                    // IMPORTANT: if we found a trailing comment, use that expanded new_end
                    .range_end(trailing_comment
                               .as_ref()
                               .map(|tc| end + tc.len())
                               .unwrap_or(end)
                               .min(old_text.len()))
                    .visibility(vis)
                    .path_list(path_list)
                    .trailing_comment(trailing_comment)
                    .build()
                    .unwrap();

                uses_data.push(info);
            } else {
                warn!("Failed to dissect => not recognized as a 'use' statement after truncation:\n{}", truncated_text);
            }
        } else {
            trace!("Item is not a `use` => ignoring");
        }
    }

    debug!("Exiting gather_use_items => returning {} items", uses_data.len());
    uses_data
}

#[cfg(test)]
mod test_gather_use_items {
    use super::*;

    /// 1) If there are no `use` statements => returns an empty vec.
    #[traced_test]
    fn test_no_use_statements() {
        info!("Testing no_use_statements");
        let src = "fn main() {}";
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);
        assert!(result.is_empty(), "No uses => empty result");
    }

    /// 2) Single `use` with a direct single-line comment right above => 
    ///    should produce exactly one leading comment.
    #[traced_test]
    fn test_single_use_with_one_leading_comment() {
        info!("Testing single_use_with_one_leading_comment");
        // NOTE: no extra blank lines, 
        // the comment is *immediately* above the use statement.
        let src = r#"// leading comment
pub(crate) use std::collections::HashMap;"#;

        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1, "One use statement => one UseItemInfo");

        let info = &result[0];
        assert_eq!(
            info.leading_comments().len(), 
            1, 
            "Should capture the single leading comment line"
        );
        assert_eq!(
            info.leading_comments()[0], 
            "// leading comment\n",
            "Should store the comment text with trailing newline"
        );
        assert_eq!(info.visibility(), "pub(crate)");
        assert_eq!(info.path_list(), "std::collections::HashMap");
    }

    /// 3) Single use with multiple consecutive comments. 
    #[traced_test]
    fn test_single_use_with_multiple_consecutive_comments() {
        info!("Testing single_use_with_multiple_consecutive_comments");
        let src = r#"
// First comment
// Second comment
pub use crate::foo;
"#;

        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert_eq!(info.leading_comments().len(), 2, "two consecutive comment lines");
        assert_eq!(info.leading_comments()[0], "// First comment\n");
        assert_eq!(info.leading_comments()[1], "// Second comment\n");
        assert_eq!(info.visibility(), "pub");
        assert_eq!(info.path_list(), "crate::foo");
    }

    /// 4) If there's a blank line (2+ newlines) => no leading comments.
    #[traced_test]
    fn test_blank_line_prevents_leading_comment() {
        info!("Testing blank_line_prevents_leading_comment");
        let src = r#"
// This comment is above

pub use crate::bar;
"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert!(info.leading_comments().is_empty(), "Blank line => no leading comments");
        assert_eq!(info.visibility(), "pub");
        assert_eq!(info.path_list(), "crate::bar");
    }

    /// 5) Single newline with indent => check if it's recognized as a single line or if it has 2 newlines.
    #[traced_test]
    fn test_single_newline_with_indent_keeps_comment() {
        info!("Testing single_newline_with_indent_keeps_comment");
        let src = r#"
// top comment
    pub(in crate) use std::io;
"#;
        // If the whitespace token between `// top comment` and `pub(in crate)...`
        // has only 1 newline, we keep it. 
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1, "We expect one use statement");
        let info = &result[0];
        assert_eq!(info.leading_comments().len(), 1, "Should keep single top comment");
        assert_eq!(info.leading_comments()[0], "// top comment\n");
        assert_eq!(info.visibility(), "pub(in crate)");
        assert_eq!(info.path_list(), "std::io");
    }

    /// 6) Multiple use statements => check each leading comment.
    #[traced_test]
    fn test_multiple_use_statements() {
        info!("Testing multiple_use_statements");
        let src = r#"
use a::A;

// comment for second
pub use b::B;

// comment for third
pub(crate) use c::C;
"#;

        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 3, "Three use statements => three UseItemInfo");

        // #1
        assert_eq!(result[0].leading_comments().len(), 0, "No comment above 'use a::A'");
        assert_eq!(result[0].visibility(), "");
        assert_eq!(result[0].path_list(), "a::A");

        // #2
        assert_eq!(result[1].leading_comments().len(), 1, "One comment above second use");
        assert_eq!(result[1].leading_comments()[0], "// comment for second\n");
        assert_eq!(result[1].visibility(), "pub");
        assert_eq!(result[1].path_list(), "b::B");

        // #3
        assert_eq!(result[2].leading_comments().len(), 1, "One comment above third use");
        assert_eq!(result[2].leading_comments()[0], "// comment for third\n");
        assert_eq!(result[2].visibility(), "pub(crate)");
        assert_eq!(result[2].path_list(), "c::C");
    }

    /// 7) Trailing comment => not leading
    #[traced_test]
    fn test_trailing_same_line_comment_not_leading() {
        info!("Testing trailing_same_line_comment_not_leading");
        let src = r#"
pub use alpha::Beta; // trailing
"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert!(info.leading_comments().is_empty(), "No leading for trailing same-line");
        assert_eq!(info.visibility(), "pub");
        assert_eq!(info.path_list(), "alpha::Beta");
    }

    /// 8) Doc comments
    #[traced_test]
    fn test_doc_comments_are_gathered() {
        info!("Testing doc_comments_are_gathered");
        let src = r#"
/// This is doc comment
/// next line
use something::S;
"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert_eq!(info.leading_comments().len(), 2, "Two doc comments");
        assert_eq!(info.leading_comments()[0], "/// This is doc comment\n");
        assert_eq!(info.leading_comments()[1], "/// next line\n");
        assert_eq!(info.visibility(), "");
        assert_eq!(info.path_list(), "something::S");
    }

    /// 9) use at top-of-file => no preceding lines => no leading comments
    #[traced_test]
    fn test_use_at_top_of_file() {
        info!("Testing use_at_top_of_file");
        let src = r#"use top::Level;"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert!(info.leading_comments().is_empty());
        assert_eq!(info.visibility(), "");
        assert_eq!(info.path_list(), "top::Level");
    }

    /// 10) Indented comment with one newline => should gather
    #[traced_test]
    fn test_indented_comment_followed_by_use() {
        info!("Testing indented_comment_followed_by_use");
        let src = r#"
// hello
    use abc::XYZ;
"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert_eq!(info.leading_comments().len(), 1, "Should gather the single comment line");
        assert_eq!(info.leading_comments()[0], "// hello\n");
        assert_eq!(info.visibility(), "");
        assert_eq!(info.path_list(), "abc::XYZ");
    }

    /// 11) Extra blank lines => no leading comments if there's 2+ newlines
    #[traced_test]
    fn test_comment_with_too_many_newlines() {
        info!("Testing comment_with_too_many_newlines");
        let src = r#"
// comment

use ab::CD;
"#;
        let file = parse_and_validate_syntax(src).unwrap();
        let result = gather_use_items(&file, src);
        debug!("Result => {:?}", result);

        assert_eq!(result.len(), 1);
        let info = &result[0];
        assert!(info.leading_comments().is_empty(), "No leading comment because blank line");
        assert_eq!(info.path_list(), "ab::CD");
    }
}
