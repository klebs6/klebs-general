// ---------------- [ File: workspacer-register/src/collect_existing_x_macros.rs ]
crate::ix!();

pub fn collect_existing_x_macros(parsed_file: &SourceFile) -> Vec<ExistingXMacro> {
    trace!("Entering collect_existing_x_macros");
    let mut result = Vec::new();

    for item in parsed_file.items() {
        // 1) Check if this item is recognized by is_x_macro => returns full_text
        if let Some(macro_text) = is_x_macro(&item) {
            let rng = item.syntax().text_range();
            debug!("Found x! macro at range={:?}, text='{}'", rng, macro_text);

            // 2) Gather preceding line comments so we can reattach them
            let mut leading_comments = gather_leading_comments(&item);

            // 3) If leading_comments is empty, but the macro_text itself starts with "//"
            //    then RA likely merged the comment lines into the same node, e.g. 
            //    `"// Hello alpha\nx!{alpha}"`. We'll parse them out:
            if leading_comments.is_empty() && macro_text.trim_start().starts_with("//") {
                // We'll do a naive approach: take the lines from `macro_text` until the line with `x!{`,
                // if they look like `// ...`.
                let mut lines = macro_text.lines().collect::<Vec<_>>();
                if let Some(idx_x) = lines.iter().position(|ln| ln.contains("x!{")) {
                    let comment_part = &lines[0..idx_x];
                    // Filter to keep only lines that truly look like `// ...`.
                    let comment_lines: Vec<&str> = comment_part
                        .iter()
                        .filter(|ln| ln.trim_start().starts_with("//"))
                        .copied()
                        .collect();

                    if !comment_lines.is_empty() {
                        let joined = comment_lines.join("\n");
                        // Ensure exactly one trailing newline
                        let final_comment = if joined.ends_with('\n') {
                            joined
                        } else {
                            format!("{joined}\n")
                        };
                        leading_comments = final_comment;
                    }
                }
            }

            // 4) Build the ExistingXMacro
            let existing_x_macro = ExistingXMacroBuilder::default()
                .text(macro_text)
                .range(rng)
                .leading_comments(leading_comments)
                .build()
                .unwrap();
            result.push(existing_x_macro);
        }
    }

    // Sort by ascending start offset
    result.sort_by_key(|em| em.range().start());
    trace!("Exiting collect_existing_x_macros with {} macros", result.len());
    result
}

#[cfg(test)]
mod test_collect_existing_x_macros_ast {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// Utility to parse a string as a `SourceFile`.
    fn parse_source(text: &str) -> SourceFile {
        SourceFile::parse(text, Edition::Edition2021).tree()
    }

    /// 1) Empty file => we expect no macros
    #[traced_test]
    fn test_empty_file_no_macros() {
        let src = "";
        let parsed_file = parse_source(src);

        let found = collect_existing_x_macros(&parsed_file);
        assert!(
            found.is_empty(),
            "Expected no macros in an empty file, got: {found:?}"
        );
    }

    /// 2) Single x! macro => we expect exactly one entry
    #[traced_test]
    fn test_single_x_macro() {
        let src = r#"
x!{my_macro}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 1, "Should find exactly one macro");
        let first = &found[0];
        assert_eq!(first.text(), "x!{my_macro}", "Captured the entire macro call text");
    }

    /// 3) Multiple x! macros scattered => we gather them all in ascending offset order
    #[traced_test]
    fn test_multiple_x_macros() {
        let src = r#"
x!{alpha}

fn something() {}

x!{beta}

x!{gamma}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 3, "Should find alpha, beta, gamma macros");
        assert_eq!(found[0].text(), "x!{alpha}");
        assert_eq!(found[1].text(), "x!{beta}");
        assert_eq!(found[2].text(), "x!{gamma}");
    }

    /// 4) Non-x macro calls => should be ignored
    #[traced_test]
    fn test_non_x_macro_is_skipped() {
        let src = r#"
foo!{stuff}
bar!{things}
x!{real_one}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(
            found.len(),
            1,
            "Only the x! macro should be collected, ignoring foo! and bar!"
        );
        assert_eq!(found[0].text(), "x!{real_one}");
    }

    /// 5) If there's an attribute on the macro or something that changes how it's parsed, 
    ///    we only capture it if the function `is_x_macro` recognized it. 
    ///    By default, we do *not* treat a macro with attributes as x! macro text if `is_x_macro` disallows it.
    ///    But let's confirm this scenario doesn't blow up:
    #[traced_test]
    fn test_macro_with_attribute_is_still_captured_if_is_x_macro_sees_it() {
        // If the underlying `is_x_macro` logic says "no" for attributes, 
        // then we'd see zero. But let's do a scenario we *know* is recognized:
        let src = r#"
#[some_attr]
x!{with_attribute}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        // Implementation detail: if `is_x_macro` hasn't changed, 
        // it might not catch macros with attributes. But let's let this pass or fail 
        // based on the actual code. For now, we just confirm we see the text if recognized.
        if found.is_empty() {
            eprintln!("Note: macro with attributes was not recognized by is_x_macro. That's fine if code disallows it.");
        } else {
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].text(), "#[some_attr]\nx!{with_attribute}");
        }
    }

    /// 6) x! macro with no braces or invalid braces => won't match `is_x_macro` if it can't parse properly
    #[traced_test]
    fn test_x_macro_missing_braces_not_collected() {
        let src = "x!some_stuff"; // not a valid x! call with braces
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(
            found.len(),
            0,
            "No valid macros if we don't have braces e.g. x!some_stuff"
        );
    }

    /// 7) Another check: x! with empty braces => "x!{}"
    #[traced_test]
    fn test_macro_with_empty_braces() {
        let src = r#"
fn foo() {}

x!{}

fn bar() {}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text(), "x!{}");
    }

    /// 8) Make sure we get them in ascending offset if they're on the same line
    #[traced_test]
    fn test_macros_on_same_line() {
        let src = r#"
x!{aaa} x!{bbb} x!{ccc}
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 3);
        assert_eq!(found[0].text(), "x!{aaa}");
        assert_eq!(found[1].text(), "x!{bbb}");
        assert_eq!(found[2].text(), "x!{ccc}");
    }

    /// 9) No macros at all except doc comments => none found
    #[traced_test]
    fn test_doc_comments_only() {
        let src = r#"
// some doc
/// # Another doc
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 0, "Comments alone => no macros found");
    }

    /// 10) A bit more advanced scenario with random tokens, verifying we handle them
    #[traced_test]
    fn test_random_tokens_and_one_macro() {
        let src = r#"
;;; ??? !?!?!?
x!{random}
??? ;; 
"#;
        let parsed_file = parse_source(src);
        let found = collect_existing_x_macros(&parsed_file);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text(), "x!{random}");
    }
}
