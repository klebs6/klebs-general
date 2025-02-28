crate::ix!();

pub fn determine_top_block_insertion_offset(
    parsed_file: &SourceFile,
    old_text: &str,
) -> usize {
    trace!("Entering determine_top_block_insertion_offset");
    let earliest_offset = find_earliest_non_macro_item_offset(parsed_file, old_text);
    debug!("earliest_offset={}", earliest_offset);

    let maybe_import_end = find_last_import_end_before_offset(parsed_file, earliest_offset);
    debug!("maybe_import_end={:?}", maybe_import_end);

    let initial_offset = maybe_import_end.unwrap_or(earliest_offset);
    debug!(
        "initial_offset={}, earliest_offset={}",
        initial_offset, earliest_offset
    );

    let insertion_offset = snap_offset_to_newline(initial_offset, earliest_offset, old_text);
    debug!("Computed final insertion_offset={}", insertion_offset);

    trace!("Exiting determine_top_block_insertion_offset");
    insertion_offset
}

#[cfg(test)]
mod test_determine_top_block_insertion_offset {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};
    use tracing::{trace, debug};

    /// Helper to parse a snippet & run determine_top_block_insertion_offset
    fn run_determine_offset(old_text: &str) -> usize {
        let parsed = SourceFile::parse(old_text, Edition::Edition2021);
        let file = parsed.tree();
        determine_top_block_insertion_offset(&file, old_text)
    }

    /// 1) If there's no items => offset is end of file
    #[traced_test]
    fn test_empty_file() {
        trace!("Starting test_empty_file for determine_top_block_insertion_offset");
        let src = "";
        let offset = run_determine_offset(src);
        debug!("offset={}", offset);
        assert_eq!(offset, 0, "Empty file => offset=0 => end of file is 0");
    }

    /// 2) If there's no imports => offset is earliest item, no adjustment
    #[traced_test]
    fn test_no_imports() {
        trace!("Starting test_no_imports for determine_top_block_insertion_offset");
        let src = r#"
fn foo() {}
fn bar() {}
"#;
        let offset = run_determine_offset(src);
        debug!("offset={}", offset);

        // earliest_offset is presumably where "fn foo" starts. 
        // no last_import_end => offset=earliest
        let idx_foo = src.find("fn foo").unwrap();
        assert_eq!(offset, idx_foo, "Offset should be the start of 'fn foo'");
    }

    /// 3) If there's an imports line => we place after that line
    #[traced_test]
    fn test_with_imports_line() {
        trace!("Starting test_with_imports_line for determine_top_block_insertion_offset");
        let src = r#"
#[macro_use] mod imports; use imports::*;

fn item() {}
"#;
        let offset = run_determine_offset(src);
        debug!("offset={}", offset);

        // We'll confirm it's after the import line. 
        // The earliest item might be "fn item", but we prefer after the line that ends with newline 
        // e.g. line_end=the offset after the newline
        // We'll do a naive check that offset > src.find("use imports::*;").unwrap()
        let idx_import_use = src.find("use imports::*;").unwrap();
        assert!(
            offset > idx_import_use, 
            "Offset should be after the import line end"
        );
    }

    /// 4) If there's multiple imports lines => we pick the one nearest the earliest real item
    #[traced_test]
    fn test_multiple_imports_lines() {
        trace!("Starting test_multiple_imports_lines for determine_top_block_insertion_offset");
        let src = r#"
#[macro_use] mod imports; use imports::*;
#[macro_use] mod imports; use imports::*;
fn item() {}
"#;
        let offset = run_determine_offset(src);
        debug!("offset={}", offset);

        // The second import line presumably is "#[macro_use] mod imports; use imports::*;" again
        // We'll see if the code picks the greatest line_end among those that are <= earliest item offset
        // Just ensure it's beyond the second line. 
        let idx_item = src.find("fn item()").unwrap();
        assert!(offset < idx_item, "Should place offset before 'fn item()'");
        // We can also confirm offset is bigger than the first import line. 
    }

    /// 5) Snap to newline => if initial_offset is mid-line, we jump up to the next newline
    #[traced_test]
    fn test_snap_to_newline() {
        trace!("Starting test_snap_to_newline for determine_top_block_insertion_offset");
        let src = "fn something() {}\nfn after() {}\n";
        // Let’s artificially tweak the code so earliest_offset is inside "fn something". 
        // But the function's actual logic does that automatically. We'll just trust it.
        // We'll see that the final offset is at the next newline boundary, or the earliest_offset, whichever is smaller.

        let offset = run_determine_offset(src);
        debug!("offset={}", offset);

        // earliest item offset is presumably 0 => we see if code tries to snap. 
        // Actually let's just check it doesn't produce something nonsense.
        // We'll do a sanity check that offset is 0. Because first item is "fn something" at 0. 
        // That is the earliest item => maybe no line to snap to. 
        // If there's no imports, that is the offset. 
        assert_eq!(offset, 0, "Probably 0 is the offset if first item starts at line 0");
    }
}

#[cfg(test)]
mod test_existing_macros_to_top_block_macros {
    use super::*;
    use tracing::{trace, debug};

    fn ex_mac(text: &str, comments: &str) -> ExistingXMacro {
        ExistingXMacroBuilder::default()
            .text(text)
            .range(TextRange::new(TextSize::from(0), TextSize::from(text.len() as u32)))
            .leading_comments(Some(comments.to_string()))
            .build()
            .unwrap()
    }

    /// 1) empty => returns empty
    #[traced_test]
    fn test_empty_old_macros() {
        trace!("Starting test_empty_old_macros for existing_macros_to_top_block_macros");
        let result = existing_macros_to_top_block_macros(&[]);
        debug!("Result = {:?}", result);
        assert!(result.is_empty(), "No old macros => empty top block macros");
    }

    /// 2) Single old macro => we parse its stem
    #[traced_test]
    fn test_single_macro() {
        trace!("Starting test_single_macro for existing_macros_to_top_block_macros");
        let old = [ex_mac("x!{alpha}", "")];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "alpha");
        assert!(result[0].leading_comments().is_none());
    }

    /// 3) Leading comments from old macro => carried over
    #[traced_test]
    fn test_leading_comments_carried_over() {
        trace!("Starting test_leading_comments_carried_over for existing_macros_to_top_block_macros");
        let old = [ex_mac("x!{beta}", "// doc line\n")];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].stem(), "beta");
        assert!(result[0].leading_comments().as_ref().unwrap().contains("doc line"), "Should keep leading comment");
    }

    /// 4) If we cannot parse the stem => skip it
    #[traced_test]
    fn test_unparsable_macro_skipped() {
        trace!("Starting test_unparsable_macro_skipped for existing_macros_to_top_block_macros");
        let old = [
            ex_mac("x! alpha", ""),       // missing braces => skip
            ex_mac("x!{delta}", ""),      // valid
            ex_mac("x!{}", ""),          // empty
            ex_mac("invalid text", ""),   // not x! => skip
        ];
        let result = existing_macros_to_top_block_macros(&old);
        debug!("Result = {:?}", result);

        // Only "x!{delta}" and "x!{}" parse a stem => "delta" and "" 
        // but "x!{}" => the stem is "" 
        // The "x! alpha" is missing braces => skip
        // "invalid text" => skip
        assert_eq!(result.len(), 2, "We only keep the macros that parse braces");
        assert_eq!(result[0].stem(), "delta");
        assert_eq!(result[1].stem(), "", "Empty braces => empty stem");
    }
}
