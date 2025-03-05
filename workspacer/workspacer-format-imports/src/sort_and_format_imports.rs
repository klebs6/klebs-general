// ---------------- [ File: workspacer-format-imports/src/sort_and_format_imports.rs ]
crate::ix!();


/// Our trait for sorting and formatting imports in a crate or workspace.
/// The goal:
///  - If `src/imports.rs` exists, parse it with RA-AP-syntax
///  - Gather all `use` statements
///  - Group them by prefix, so e.g. 
///       pub(crate) use std::collections::HashMap;
///       pub(crate) use std::collections::HashSet;
///    becomes 
///       pub(crate) use std::collections::{HashMap, HashSet};
///  - Sort them in alphabetical order
///  - Preserve comments
/// Then rewrite `imports.rs` with the new grouping.
#[async_trait]
pub trait SortAndFormatImports {
    type Error;

    /// Sorts & formats the imports for either a single crate or an entire workspace.
    async fn sort_and_format_imports(&self) -> Result<(), Self::Error>;
}

pub fn sort_and_format_imports_in_text(old_text: &str) -> Result<String, SortAndFormatImportsError> {
    info!("Entering sort_and_format_imports_in_text");
    debug!("Input text length is {}", old_text.len());

    // 1) Parse
    trace!("About to parse/validate syntax");
    let file = match parse_and_validate_syntax(old_text) {
        Ok(f) => {
            debug!("parse_and_validate_syntax succeeded");
            f
        }
        Err(e) => {
            error!("parse_and_validate_syntax failed => {:?}", e);
            return Err(e);
        }
    };

    // 2) Gather use items
    debug!("Gathering use items from file & old_text");
    let uses_data = gather_use_items(&file, old_text);
    trace!("Collected {} use items", uses_data.len());

    // 3) Group them => now returns (grouped_map, comment_map, trailing_comments)
    debug!("Grouping and sorting use items now");
    let (grouped_map, comment_map, trailing_comments) = group_and_sort_uses(&uses_data);
    trace!(
        "grouped_map.len()={}, comment_map.len()={}, trailing_comments.len()={}",
        grouped_map.len(),
        comment_map.len(),
        trailing_comments.len()
    );

    // 4) Build new use lines
    debug!("Building new use lines...");
    let new_uses_block = build_new_use_lines(&grouped_map, &comment_map, &trailing_comments);
    trace!("New uses block:\n{}", new_uses_block);

    // 5) Remove old uses
    debug!("Removing old use statements from the original text");
    let remainder = remove_old_use_statements(&uses_data, old_text);

    // 6) Combine
    debug!("Combining new uses block with remainder");
    let final_text = combine_new_uses_with_remainder(&new_uses_block, &remainder);

    info!(
        "Finished sort_and_format_imports_in_text => final_text length is {}",
        final_text.len()
    );
    Ok(final_text)
}

#[cfg(test)]
mod test_sort_and_format_imports_in_text {
    use super::*;

    #[traced_test]
    fn test_no_uses() {
        info!("test_no_uses => start");
        let old_text = "fn main() {}";
        trace!("Calling sort_and_format_imports_in_text");
        let out = sort_and_format_imports_in_text(old_text)
            .expect("Expected no error for basic code with no uses");
        debug!("Got result => length={}, content=\n{}", out.len(), out);

        assert_eq!(
            out.trim(),
            old_text.trim(),
            "No uses => text should remain basically the same"
        );
        info!("test_no_uses => success");
    }

    /// Previously, the snippet had both `fn main(){}` and the `use` on the same line,
    /// which could parse unpredictably. Now we separate them into distinct lines so that
    /// the `use` is recognized cleanly and placed at the top. 
    #[traced_test]
    fn test_single_use() {
        info!("test_single_use => start");
        let old_text = r#"
fn main(){}
pub(crate) use std::io;
"#;
        trace!("Calling sort_and_format_imports_in_text for single_use scenario");
        let out = sort_and_format_imports_in_text(old_text)
            .expect("Expected no error for single-use scenario");
        debug!("Got result => length={}, content=\n{}", out.len(), out);
        let trimmed_out = out.trim();

        assert!(
            trimmed_out.contains("pub(crate) use std::io;"),
            "Expect to see the sorted use statement"
        );
        info!("test_single_use => success");
    }
}
