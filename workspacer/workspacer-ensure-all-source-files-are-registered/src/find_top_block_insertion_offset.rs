// ---------------- [ File: src/find_top_block_insertion_offset.rs ]
crate::ix!();

pub fn find_top_block_insertion_offset(
    parsed_file: &SourceFile,
    edited_text: &str,
) -> Result<usize, SourceFileRegistrationError> {
    trace!("Entering find_top_block_insertion_offset");
    let mut earliest_offset: Option<usize> = None;

    for item in parsed_file.items() {
        let start = item.syntax().text_range().start();
        let start_usize = usize::from(start);
        trace!("Examining item with start={}", start_usize);

        if earliest_offset.map_or(true, |prev| start_usize < prev) {
            debug!("Updating earliest_offset from {:?} to {}", earliest_offset, start_usize);
            earliest_offset = Some(start_usize);
        }
    }

    let final_offset = earliest_offset.unwrap_or_else(|| {
        debug!("No items => default offset is end of edited_text={}", edited_text.len());
        edited_text.len()
    });

    debug!("Final insertion offset is {}", final_offset);
    trace!("Exiting find_top_block_insertion_offset");
    Ok(final_offset)
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

    #[traced_test]
    fn test_empty_source() {
        let src = "";
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset = end => 0
        assert_eq!(offset, 0);
    }

    #[traced_test]
    fn test_only_whitespace() {
        let src = "   \n   \n";
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset = end => src.len()
        assert_eq!(offset, src.len());
    }

    #[traced_test]
    fn test_only_comments() {
        let src = r#"
// comment
// more comment
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, src.len(), "No items => offset is end");
    }

    #[traced_test]
    fn test_only_file_level_attr() {
        let src = r#"
#![allow(dead_code)]
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, src.len());
    }

    #[traced_test]
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

    #[traced_test]
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

    #[traced_test]
    fn test_immediate_real_item() {
        let src = "fn start() {}";
        let offset = run_insertion_offset(src, None).unwrap();
        assert_eq!(offset, 0, "Immediately at start => offset=0");
    }

    #[traced_test]
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

    #[traced_test]
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

    #[traced_test]
    fn test_no_real_item_just_random_tokens() {
        let src = r#"
;; ?? !?!?!?
"#;
        let offset = run_insertion_offset(src, None).unwrap();
        // No items => offset is end => src.len()
        assert_eq!(offset, src.len());
    }

    #[traced_test]
    fn test_edited_text_override() {
        let src = "fn placeholder() {}";
        let override_edited = "fn placeholder()"; // missing braces
        let offset = run_insertion_offset(src, Some(override_edited)).unwrap();
        // We have 1 item => offset=0 anyway
        assert_eq!(offset, 0);
    }
}
