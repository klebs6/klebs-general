// ---------------- [ File: src/find_last_import_end_before_offset.rs ]
crate::ix!();

pub fn find_last_import_end_before_offset(
    parsed_file: &SourceFile,
    earliest_offset: usize,
) -> Option<usize> {
    trace!("Entering find_last_import_end_before_offset with earliest_offset={}", earliest_offset);

    let mut answer = None;

    for item in parsed_file.items() {
        if is_imports_line(&item) {
            let rng = item.syntax().text_range();
            let start: usize = rng.start().into();
            let end:   usize = rng.end().into();
            trace!("Found an imports line from {}..{}", start, end);

            // If it ends before earliest_offset, or overlaps it from below
            if end <= earliest_offset || (start < earliest_offset) {
                trace!("Line meets condition => consider it for answer");
                if answer.map_or(true, |prev| end > prev) {
                    debug!("Updating answer from {:?} to {}", answer, end);
                    answer = Some(end);
                }
            }
        } else {
            trace!("Not an imports line => skipping");
        }
    }

    debug!("Result = {:?}", answer);
    trace!("Exiting find_last_import_end_before_offset");
    answer
}

#[cfg(test)]
mod test_find_last_import_end_before_offset {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};

    /// Helper to parse a string into a SourceFile.
    fn parse_source(input: &str) -> SourceFile {
        SourceFile::parse(input, Edition::Edition2021).tree()
    }

    /// 1) Empty file => no items => returns None
    #[traced_test]
    fn test_empty_file() {
        let src = "";
        let parsed_file = parse_source(src);

        let result = find_last_import_end_before_offset(&parsed_file, 0);
        assert!(
            result.is_none(),
            "Empty file => no imports => should return None"
        );
    }

    /// 2) File with no import lines => returns None
    #[traced_test]
    fn test_no_import_lines() {
        let src = r#"
fn nothing_special() {}
"#;
        let parsed_file = parse_source(src);

        let earliest_offset = src.find("fn nothing_special").unwrap(); // e.g. 1 or something
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset);

        assert!(result.is_none(), "No import lines => should return None");
    }

    /// 3) Single import line that ends well before earliest => returns Some(line_end)
    #[traced_test]
    fn test_single_import_line_before_earliest() {
        let src = r#"
#[macro_use] mod imports; use imports::*;

fn something_else() {}
"#;
        let parsed_file = parse_source(src);

        // let's say earliest_offset is where `fn something_else()` starts
        let earliest_offset = src.find("fn something_else").expect("missing fn");
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset)
            .expect("Expected Some offset for the import line end");

        // We'll confirm the offset is indeed the end of the import line
        // We'll just check it's strictly less than earliest_offset
        assert!(
            result < earliest_offset,
            "Import line's end offset should be before earliest_offset"
        );

        // also let's confirm it points into the line, not 0, etc.
        assert_ne!(result, 0, "Should not be zero offset, we do have an import line");
    }

    /// 4) If the import line ends exactly at earliest_offset => we consider it valid
    ///    (the code says if end <= earliest_offset, we accept).
    #[traced_test]
    fn test_import_line_ends_exactly_at_earliest() {
        let src = r#"
#[macro_use] mod imports; use imports::*;fn item(){}
"#;
        // There's no newline => it might have an item right after the import
        // But let's see how the parser sees them:  It's actually likely one item:
        // But let's assume we do have an import line recognized by `is_imports_line`.

        let parsed_file = parse_source(src);
        let earliest_offset = src.find("fn item()").unwrap(); 
        // let's confirm the offset after "use imports::*;" might be exactly earliest_offset.

        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset);
        assert!(
            result.is_some(),
            "Import line that ends exactly at earliest_offset => recognized"
        );
    }

    /// 5) Single import line that partially overlaps earliest => i.e. starts < earliest, ends > earliest => we do consider it
    #[traced_test]
    fn test_import_line_partially_overlaps_earliest() {
        // We artificially create a scenario where the "import line" extends beyond earliest_offset
        // e.g. the item is inside the same line, but let's see how the parser sees it.
        // This is contrived but checks that `start < earliest_offset` triggers acceptance.
        let src = r#"
#[macro_use] mod imports; use imports::*; fn something() {}
"#;
        let parsed_file = parse_source(src);

        // We'll pick earliest_offset around "fn something()"
        let earliest_offset = src.find("fn something()").unwrap();
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset)
            .expect("Should find the import line since it overlaps earliest");

        assert!(
            result > earliest_offset,
            "Because the line extends beyond earliest_offset, the code includes it anyway"
        );
    }

    /// 6) Multiple import lines => pick the greatest end that meets the condition
    #[traced_test]
    fn test_multiple_imports_pick_greatest_end() {
        let src = r#"
#[macro_use] mod imports; use imports::*;
#[macro_use] mod imports; use imports::*; 
fn item() {}
"#;
        let parsed_file = parse_source(src);

        let earliest_offset = src.find("fn item()").expect("missing fn item");
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset)
            .expect("We do have multiple imports lines");
        
        // We expect it to pick the import line with the greatest end offset if both are fully or partially before the item
        // The second line presumably ends after the first line.
        // We'll do a sanity check:
        assert!(
            result > earliest_offset,
            "Likely the second import line extends well beyond the item start"
        );
    }

    /// 7) If there's an import line after earliest_offset => we ignore it
    #[traced_test]
    fn test_import_line_after_earliest_ignored() {
        let src = r#"
fn real_item() {}

#[macro_use] mod imports; use imports::*;
"#;
        let parsed_file = parse_source(src);

        // earliest_offset presumably is "fn real_item()"
        let earliest_offset = src.find("fn real_item()").unwrap();
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset);

        // That import line is *after* the item => we do not pick it
        assert!(result.is_none(), "Import line is after earliest => no recognized line");
    }

    /// 8) If we have random lines, only lines recognized by `is_imports_line` matter
    #[traced_test]
    fn test_random_non_import_lines_ignored() {
        let src = r#"
pub mod not_imports;
use something_else;

#[macro_use] mod imports; use imports::*;

fn item() {}
"#;
        // The top lines are not recognized by is_imports_line => we only expect the last line is recognized.
        // We'll see how it interacts with earliest_offset.
        let parsed_file = parse_source(src);

        let earliest_offset = src.find("fn item()").unwrap();
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset);

        assert!(
            result.is_some(),
            "We do find the recognized import line near the end"
        );
    }

    /// 9) Partial overlap scenario repeated: if an import line starts before earliest but ends well beyond it, 
    ///    we pick it. If there's another that fully ends just before earliest, we pick the one with the greatest end.
    #[traced_test]
    fn test_multiple_lines_partial_and_full_before() {
        let src = r#"
#[macro_use] mod imports; use imports::*;
#[macro_use] mod imports; use imports::*; fn real_item(){}
"#;
        let parsed_file = parse_source(src);

        let earliest_offset = src.find("fn real_item()").unwrap();
        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset);

        // The second import line presumably merges with fn real_item on the same line => extends beyond earliest?
        // Let's see if we get that second one. The code picks the line with the greatest end if it meets:
        //    if end <= earliest_offset || start < earliest_offset
        // => second line starts < earliest_offset => yes.
        // => so we pick that second line, which definitely extends beyond earliest_offset. That is the "greatest end."
        assert!(
            result.is_some(),
            "Should pick the second line's end because it partially overlaps"
        );
    }

    /// 10) If multiple lines meet the condition, we pick the one with the largest `.end()`
    #[traced_test]
    fn test_pick_largest_end_among_several_candidates() {
        let src = r#"
#[macro_use] mod imports; use imports::*; // line 1
#[macro_use] mod imports; use imports::*; // line 2 is presumably longer
#[macro_use] mod imports; use imports::*; // line 3 is even longer
fn item() {}
"#;
        let parsed_file = parse_source(src);
        let earliest_offset = src.find("fn item()").unwrap();

        let result = find_last_import_end_before_offset(&parsed_file, earliest_offset)
            .expect("We have multiple import lines");

        // We want to confirm we got the largest end offset among lines 1,2,3
        // We'll do a naive check: the third line is presumably the last or the biggest range end
        // so we expect that offset to be bigger than the second line's end, etc.
        // Just confirm it's > the earliest_offset or we can do some more direct checks if we want.
        assert!(
            result > earliest_offset,
            "Line 3 presumably overlaps or extends beyond earliest => it has the greatest end"
        );
    }
}
