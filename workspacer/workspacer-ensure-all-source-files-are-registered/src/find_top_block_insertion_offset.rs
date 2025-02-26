// ---------------- [ File: src/find_top_block_insertion_offset.rs ]
crate::ix!();

/// Finds a suitable insertion offset to place the new top block by scanning
/// for the earliest AST item (fn, struct, etc.), ignoring doc comments,
/// whitespace, random tokens, etc.
/// If no items exist, we place at the end of `edited_text`.
pub fn find_top_block_insertion_offset(
    parsed_file: &SourceFile,
    edited_text: &str,
) -> Result<usize, SourceFileRegistrationError> {
    let mut earliest_offset: Option<usize> = None;

    // 1) For each top-level AST item, track the earliest start offset
    for item in parsed_file.items() {
        let start = item.syntax().text_range().start();
        let start_usize = usize::from(start);

        if earliest_offset.map_or(true, |prev| start_usize < prev) {
            earliest_offset = Some(start_usize);
        }
    }

    // 2) If we found an item, return that offset. Otherwise, place at the end.
    Ok(earliest_offset.unwrap_or_else(|| edited_text.len()))
}

#[cfg(test)]
mod test_find_top_block_insertion_offset {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// Helper that calls `find_top_block_insertion_offset` with an optional override of `edited_text`.
    fn run_insertion_offset(src: &str, override_edited: Option<&str>) -> Result<usize, SourceFileRegistrationError> {
        let parse = SourceFile::parse(src, Edition::Edition2021);
        let parsed_file = parse.tree();

        let final_text = override_edited.unwrap_or(src);
        find_top_block_insertion_offset(&parsed_file, final_text)
    }

    #[test]
    fn test_empty_source() {
        let src = "";
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset = end => 0
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_only_whitespace() {
        let src = "   \n   \n";
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset = end => src.len()
        assert_eq!(offset, src.len());
    }

    #[test]
    fn test_only_comments() {
        let src = r#"
// comment
// more comment
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, src.len(), "No items => offset is end");
    }

    #[test]
    fn test_only_file_level_attr() {
        let src = r#"
#![allow(dead_code)]
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, src.len());
    }

    #[test]
    fn test_attr_followed_by_real_item() {
        let src = r#"
#![allow(unused)]
fn real_function() {}
"#;
        let parse = SourceFile::parse(src, Edition::Edition2021);
        let parsed_file = parse.tree();

        let first_item = parsed_file.items().next().expect("No items found!");
        let ast_expected_offset = usize::from(first_item.syntax().text_range().start());

        let offset = find_top_block_insertion_offset(&parsed_file, src).unwrap();
        assert_eq!(offset, ast_expected_offset, "Matches AST's earliest item offset");
    }

    #[test]
    fn test_doc_comments_followed_by_item() {
        let src = r#"
// doc
// another doc
fn main() {}
"#;
        let parse = SourceFile::parse(src, Edition::Edition2021);
        let parsed_file = parse.tree();

        let first_item = parsed_file.items().next().expect("Expected a fn item");
        let ast_expected_offset = usize::from(first_item.syntax().text_range().start());

        let offset = find_top_block_insertion_offset(&parsed_file, src).unwrap();
        assert_eq!(offset, ast_expected_offset, "Should match the AST item offset");
    }

    #[test]
    fn test_immediate_real_item() {
        let src = "fn start() {}";
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, 0, "Immediately at start => offset=0");
    }

    #[test]
    fn test_complex_mixture() {
        let src = r#"
// doc
     
#![allow(something)]

fn do_something() {}
"#;
        let parse = SourceFile::parse(src, Edition::Edition2021);
        let parsed_file = parse.tree();

        let first_item = parsed_file.items().next().expect("Expected a fn item");
        let ast_expected_offset = usize::from(first_item.syntax().text_range().start());

        let offset = find_top_block_insertion_offset(&parsed_file, src).unwrap();
        assert_eq!(offset, ast_expected_offset);
    }

    #[test]
    fn test_random_tokens_until_real_item() {
        let src = r#"
;; ;; // random tokens
// doc
fn real() {}
"#;
        let parse = SourceFile::parse(src, Edition::Edition2021);
        let parsed_file = parse.tree();

        let first_item = parsed_file.items().next().expect("Expected a fn item");
        let ast_expected_offset = usize::from(first_item.syntax().text_range().start());

        let offset = find_top_block_insertion_offset(&parsed_file, src).unwrap();
        assert_eq!(offset, ast_expected_offset, "Matches earliest AST item offset");
    }

    #[test]
    fn test_no_real_item_just_random_tokens() {
        let src = r#"
;; ?? !?!?!?
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset is end => src.len()
        assert_eq!(offset, src.len());
    }

    #[test]
    fn test_edited_text_override() {
        let src = "fn placeholder() {}";
        let override_edited = "fn placeholder()"; // missing braces
        let offset = run_insertion_offset(src, Some(override_edited)).unwrap();
        // We have 1 item => offset=0 anyway
        assert_eq!(offset, 0);
    }
}
