crate::ix!();

// ---------------- [ File: src/sort_and_format_imports.rs ]

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

/// The top-level function, calling each subroutine.
pub fn sort_and_format_imports_in_text(old_text: &str) -> Result<String, SortAndFormatImportsError> {
    // 1) Parse & validate
    let file = parse_and_validate_syntax(old_text)?;

    // 2) Gather use items (leading comments, path, etc.)
    let uses_data = gather_use_items(&file, old_text);

    // 3) Group them by (vis, prefix) => produce a map + comment map
    let (grouped_map, comment_map) = group_and_sort_uses(&uses_data);

    // 4) Build new use lines from those groups
    let new_uses_block = build_new_use_lines(&grouped_map, &comment_map);

    // 5) Remove old use statements from `old_text` to preserve non-use lines
    let remainder = remove_old_use_statements(&uses_data, old_text);

    // 6) Place the new uses block at the top, then combine
    let final_text = combine_new_uses_with_remainder(&new_uses_block, &remainder);

    Ok(final_text)
}

#[cfg(test)]
mod test_sort_and_format_imports_in_text {
    use super::*;

    #[traced_test]
    fn test_no_uses() {
        let old_text = "fn main() {}";
        let out = sort_and_format_imports_in_text(old_text).unwrap();
        // Instead of `assert_eq!(out, old_text)` do:
        assert_eq!(out.trim(), old_text.trim(), "No uses => text should remain basically the same");
    }

    #[traced_test]
    fn test_single_use() {
        let old_text = r#"fn main(){} pub(crate) use std::io; "#; 
        let out = sort_and_format_imports_in_text(old_text).unwrap(); 
        // The new uses block should appear at top, so let's do a "trim" check 
        let trimmed_out = out.trim(); 
        assert!( trimmed_out.contains("pub(crate) use std::io;"), "Expect to see the sorted use statement" ); 
    }
}
