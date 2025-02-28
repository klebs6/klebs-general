crate::ix!();

/// A version of `parse_new_macros_with_comments` that does NOT do fallback
/// merging snippet lines into the macro. 
pub fn parse_new_macros_with_comments(new_top_block: &str) -> Vec<TopBlockMacro> {
    use ra_ap_syntax::{Edition, SourceFile};

    let parse = SourceFile::parse(new_top_block, Edition::Edition2021);
    let file = parse.tree();
    let mut results = vec![];

    for item in file.items() {
        if let Some(full_text) = is_x_macro(&item) {
            // Extract macro’s stem
            if let Some(stem) = extract_stem(&full_text) {
                // Normal gather of doc lines:
                let leading = gather_leading_comments(&item);
                let mac = TopBlockMacroBuilder::default()
                    .stem(stem)
                    .leading_comments(leading)
                    .build()
                    .unwrap();
                results.push(mac);
            }
        }
    }
    results
}

#[cfg(test)]
mod test_parse_new_macros_with_comments {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};
    use tracing::{trace, debug};

    /// Helper: parse text via parse_new_macros_with_comments
    fn parse_macros(text: &str) -> Vec<TopBlockMacro> {
        parse_new_macros_with_comments(text)
    }

    /// 1) Empty snippet => no macros
    #[traced_test]
    fn test_empty_snippet() {
        trace!("test_empty_snippet for parse_new_macros_with_comments");
        let result = parse_macros("");
        debug!("result={:?}", result);
        assert!(result.is_empty(), "No macros => empty vector");
    }

    /// 2) Single x! macro => parse the stem
    #[traced_test]
    fn test_single_macro() {
        trace!("test_single_macro for parse_new_macros_with_comments");
        let text = "x!{alpha}";
        let result = parse_macros(text);
        debug!("result={:?}", result);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "alpha");
        assert!(result[0].leading_comments().is_none());
    }

    /// 3) Macro with leading comments
    #[traced_test]
    fn test_macro_with_leading_comments() {
        trace!("test_macro_with_leading_comments for parse_new_macros_with_comments");
        let text = r#"
// Some doc
x!{beta}
"#;
        let result = parse_macros(text);
        debug!("result={:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "beta");
        let c = result[0].leading_comments().as_ref().unwrap();
        assert!(c.contains("Some doc"), "Should preserve doc line");
    }

    /// 4) Multiple macros in snippet => parse all
    #[traced_test]
    fn test_multiple_macros() {
        trace!("test_multiple_macros for parse_new_macros_with_comments");
        let text = r#"
x!{foo}
// doc line
x!{bar}
"#;
        let result = parse_macros(text);
        debug!("result={:?}", result);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].stem(), "foo");
        assert!(result[0].leading_comments().is_none());
        assert_eq!(result[1].stem(), "bar");
        assert!(result[1].leading_comments().as_ref().unwrap().contains("doc line"), "Should attach doc above bar");
    }

    /// 5) Macros with invalid format => skip them
    #[traced_test]
    fn test_invalid_macros_skipped() {
        trace!("test_invalid_macros_skipped for parse_new_macros_with_comments");
        let text = r#"
x! alpha
x!{}
x!{valid}
"#;
        // `x! alpha` => missing braces => skip
        // `x!{}` => empty => valid => stem=""
        // `x!{valid}` => stem=valid
        let result = parse_macros(text);
        debug!("result={:?}", result);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].stem(), "", "Empty braces => empty stem");
        assert_eq!(result[1].stem(), "valid");
    }
}
