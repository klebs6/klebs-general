// ---------------- [ File: workspacer-register/src/find_earliest_non_macro_item_offset.rs ]
crate::ix!();

pub fn find_earliest_non_macro_item_offset(parsed_file: &SourceFile, old_text: &str) -> usize {

    trace!("Entering find_earliest_non_macro_item_offset");

    let mut earliest = old_text.len();
    debug!("Initial earliest offset = old_text.len()={}", earliest);

    for item in parsed_file.items() {
        // Skip if this is an x! macro:
        if let Some(full_text) = is_x_macro(&item) {
            trace!("Skipping x! macro => full_text='{}'", full_text);
            continue;
        }

        // RA occasionally parses stray tokens like `;random tokens`
        // as a MacroCall but with no bang token. We only want to
        // treat actual macros (with a '!' present) or real items
        // as "non-macro items":
        if let Some(mc) = ast::MacroCall::cast(item.syntax().clone()) {
            if mc.excl_token().is_none() {
                trace!("Skipping partial macro call => no bang token");
                continue;
            }
        }

        let start: usize = item.syntax().text_range().start().into();
        trace!("Found a non-macro item at offset={}", start);
        if start < earliest {
            debug!("Updating earliest offset from {} to {}", earliest, start);
            earliest = start;
        }
    }

    debug!("Final earliest offset is {}", earliest);
    trace!("Exiting find_earliest_non_macro_item_offset");
    earliest
}

#[cfg(test)]
mod test_find_earliest_non_macro_item_offset {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// Helper to parse a string into a SourceFile.
    fn parse_source(input: &str) -> SourceFile {
        SourceFile::parse(input, Edition::Edition2021).tree()
    }

    /// 1) If the file is empty, we have no items => offset should be old_text.len()
    #[traced_test]
    fn test_empty_file() {
        let src = "";
        let parsed_file = parse_source(src);

        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);
        assert_eq!(
            offset,
            src.len(),
            "Empty file => earliest offset is the file's length"
        );
    }

    /// 2) If the file has only macros => earliest non-macro item is again old_text.len()
    #[traced_test]
    fn test_only_macros() {
        let src = r#"
x!{alpha}
x!{beta}
"#;
        let parsed_file = parse_source(src);

        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);
        assert_eq!(
            offset,
            src.len(),
            "File has only x! macros => earliest offset should be src.len()"
        );
    }

    /// 3) If there's a single non-macro item at the start => that is offset=0 or close to it.
    #[traced_test]
    fn test_item_at_start() {
        let src = r#"
fn my_item() {}
x!{macro_after}
"#;
        let parsed_file = parse_source(src);

        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // We'll find the offset of "fn my_item()". 
        // The first line is a blank line, so the item might start at offset 1 or 2.
        // Let's find that actual substring in `src`.
        let idx_fn = src.find("fn my_item()").expect("item not found in src");
        assert_eq!(
            offset, 
            idx_fn,
            "Earliest non-macro item should match the offset where 'fn my_item()' appears."
        );
    }

    /// 4) If there's an item near the end, but macros near the start => we find the offset of that item near the end
    #[traced_test]
    fn test_item_at_end() {
        let src = r#"
x!{something}
x!{another}
struct MyData {
    val: i32,
}

"#;
        let parsed_file = parse_source(src);

        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // The non-macro item "struct MyData" is presumably recognized as earliest. 
        // Let's find it:
        let idx_struct = src.find("struct MyData").unwrap();
        assert_eq!(
            offset,
            idx_struct,
            "Earliest non-macro item offset should match 'struct MyData' start offset."
        );
    }

    /// 5) Multiple items => we want the earliest one
    #[traced_test]
    fn test_multiple_items() {
        let src = r#"
fn first_item() {}
x!{macro_in_between}
fn second_item() {}
fn third_item() {}
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // We'll see if "fn first_item()" is earliest
        let idx_first_item = src.find("fn first_item()").expect("Should find first_item");
        assert_eq!(
            offset,
            idx_first_item,
            "Earliest item is the first one => offset matches 'fn first_item()' start."
        );
    }

    /// 6) If there's random tokens, doc comments, etc. => as soon as we see a real item, that's the offset.
    #[traced_test]
    fn test_random_tokens_and_doc_comments_before_real_item() {
        let src = r#"
// doc line
;;; ??? #
// Another doc

x!{macro_1}
fn real_item() {}
x!{macro_2}
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // The earliest non-macro item is "fn real_item()", let's find it:
        let idx_fn = src.find("fn real_item()").unwrap();
        assert_eq!(
            offset, 
            idx_fn,
            "Earliest non-macro item offset is where 'fn real_item()' appears"
        );
    }

    /// 7) If there's some indentation or leading spaces, we still should see the correct offset
    #[traced_test]
    fn test_indented_item() {
        let src = r#"
    x!{macro_1}

        fn spaced_item() {
            // body
        }
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // let's locate "fn spaced_item()"
        let idx_fn = src.find("fn spaced_item()").unwrap();
        assert_eq!(offset, idx_fn);
    }

    /// 8) Confirm that we skip *only* x! macros. If there's some other macros (foo!{}),
    ///    that is presumably a "non-macro item" for the logic that checks is_x_macro.
    #[traced_test]
    fn test_other_macros_count_as_items() {
        let src = r#"
foo!{stuff}
fn real_fn() {}
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // "foo!{stuff}" is not recognized by is_x_macro => it's a non-macro item for our logic
        // so the earliest offset might actually be "foo!{stuff}". Let's confirm:
        let idx_foo = src.find("foo!{stuff}").unwrap();

        assert_eq!(
            offset, 
            idx_foo,
            "We expect to treat 'foo!{{stuff}}' as a non-x! macro => so earliest offset is that line"
        );
    }

    /// 9) If there's a "raw mod" line or something that isn't x! => it's also a non-macro item
    #[traced_test]
    fn test_raw_mod_lines_are_items() {
        let src = r#"
mod raw_mod;
x!{macro_there}
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // "mod raw_mod;" is presumably the earliest item
        let idx_mod = src.find("mod raw_mod;").unwrap();
        assert_eq!(offset, idx_mod);
    }

    /// 10) Integration scenario: multiple macros, multiple non-macros, some doc lines, random tokens => pick earliest non-macro
    #[traced_test]
    fn test_integration_mixed() {
        let src = r#"
// doc
x!{macro_1}
;random tokens
fn first_fn() {}
x!{macro_2}
struct SomeStruct;
"#;
        let parsed_file = parse_source(src);
        let offset = find_earliest_non_macro_item_offset(&parsed_file, src);

        // The earliest non-macro is "fn first_fn() {}"
        let idx_fn = src.find("fn first_fn()").unwrap();
        assert_eq!(offset, idx_fn);
    }
}
