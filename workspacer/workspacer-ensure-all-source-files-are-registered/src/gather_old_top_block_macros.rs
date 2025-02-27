crate::ix!();

pub fn gather_old_top_block_macros(parsed_file: &SourceFile) -> Vec<TopBlockMacro> {
    trace!("Entering gather_old_top_block_macros");

    debug!("Collecting existing x! macros from the parsed file");
    let old_macros = collect_existing_x_macros(parsed_file);
    debug!("Found {} old macros in the file", old_macros.len());

    trace!("Converting old macros into TopBlockMacro structs");
    let result = existing_macros_to_top_block_macros(&old_macros);
    debug!("Converted old macros => {} TopBlockMacro entries", result.len());

    trace!("Exiting gather_old_top_block_macros");
    result
}

#[cfg(test)]
mod test_gather_old_top_block_macros {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};
    use tracing::{trace, debug};

    /// Helper to parse source text into a `SourceFile`.
    fn parse_source(text: &str) -> SourceFile {
        SourceFile::parse(text, Edition::Edition2021).tree()
    }

    /// 1) Empty file => returns an empty Vec
    #[traced_test]
    fn test_empty_file() {
        trace!("Starting test_empty_file for gather_old_top_block_macros");
        let src = "";
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);
        assert!(result.is_empty(), "Expected no macros in empty file");
    }

    /// 2) File with no x! macros => returns an empty Vec
    #[traced_test]
    fn test_no_macros() {
        trace!("Starting test_no_macros for gather_old_top_block_macros");
        let src = r#"
            fn something() {}
            pub mod another {}
        "#;
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);
        assert!(result.is_empty(), "Expected no macros if file has no x! macros");
    }

    /// 3) Single x! macro, with no leading comment
    #[traced_test]
    fn test_single_macro_no_leading_comment() {
        trace!("Starting test_single_macro_no_leading_comment for gather_old_top_block_macros");
        let src = r#"
x!{alpha}
fn main() {}
"#;
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1, "Should find exactly one macro");
        assert_eq!(result[0].stem(), "alpha", "Expected macro stem to be 'alpha'");
        assert!(result[0].leading_comments().is_empty(), "Expected no leading comments");
    }

    /// 4) Multiple x! macros, some with leading comments
    #[traced_test]
    fn test_multiple_macros_with_leading_comments() {
        trace!("Starting test_multiple_macros_with_leading_comments for gather_old_top_block_macros");
        let src = r#"
// Hello alpha
x!{alpha}

x!{beta}
// a doc for gamma
// second doc line
x!{gamma}
"#;

        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 3, "Expected to find alpha, beta, gamma macros");

        // alpha:
        assert_eq!(result[0].stem(), "alpha");
        // leading comment might be: "// Hello alpha\n"
        assert!(result[0].leading_comments().contains("Hello alpha"), "Should contain alpha doc line");

        // beta:
        assert_eq!(result[1].stem(), "beta");
        assert!(result[1].leading_comments().is_empty(), "beta has no leading comment");

        // gamma:
        assert_eq!(result[2].stem(), "gamma");
        let gamma_comments = result[2].leading_comments();
        assert!(gamma_comments.contains("a doc for gamma"), "Expected first doc line");
        assert!(gamma_comments.contains("second doc line"), "Expected second doc line");
    }

    /// 5) Macros that have whitespace or blank lines before them
    ///    ensures we stop collecting if there's a blank line
    #[traced_test]
    fn test_macro_with_blank_line_before_comments() {
        trace!("Starting test_macro_with_blank_line_before_comments for gather_old_top_block_macros");
        let src = r#"
// Some doc
// Another doc

x!{delta}
"#;
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);

        // The blank line should stop us from associating those lines with the macro
        // => leading comments should be empty.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "delta");
        assert!(result[0].leading_comments().is_empty(), "Blank line => no attached comments");
    }

    /// 6) Comments that appear after the macro => not recognized as leading
    #[traced_test]
    fn test_comments_after_macro() {
        trace!("Starting test_comments_after_macro for gather_old_top_block_macros");
        let src = r#"
x!{zeta}
// This is after
"#;
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "zeta");
        assert!(result[0].leading_comments().is_empty(), "Comments after macro do not count as leading");
    }

    /// 7) If a macro has multiple lines, or weird spacing, we can still parse it.
    #[traced_test]
    fn test_macro_with_weird_spacing() {
        trace!("Starting test_macro_with_weird_spacing for gather_old_top_block_macros");
        let src = "x!   {   spaced    }\n";
        let parsed_file = parse_source(src);
        let result = gather_old_top_block_macros(&parsed_file);
        debug!("Result = {:?}", result);

        // In your pipeline, if is_x_macro recognized "x!{spaced}", it might trim whitespace.
        // This test ensures we handle that. But if your `is_x_macro` rejects whitespace,
        // this might fail. Adjust if needed.
        assert_eq!(result.len(), 1, "We might or might not parse a weirdly spaced macro, depending on is_x_macro");
        if !result.is_empty() {
            assert_eq!(result[0].stem(), "spaced", "Should trim whitespace inside braces");
        }
    }
}
