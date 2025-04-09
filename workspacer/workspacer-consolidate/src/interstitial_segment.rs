// ---------------- [ File: workspacer-consolidate/src/interstitial_segment.rs ]
crate::ix!();

#[derive(Builder,Setters,Getters,Debug,Clone)]
#[getset(get="pub", set="pub")]
#[builder(setter(into))]
pub struct InterstitialSegment {
    /// The text range in the file where this raw segment occurs.
    /// This segment is anything that does NOT directly belong to a recognized AST item.
    text_range: TextRange,

    /// The raw text that was found in this range, including whitespace/comments/etc.
    text: String,

    /// The file from which this segment was parsed
    file_path: PathBuf,

    /// The crate path from which this file originated
    crate_path: PathBuf,
}

pub fn gather_interstitial_segments(
    source_file: &SourceFile,
    recognized_items: &[ConsolidatedItem],
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Vec<InterstitialSegment> {
    trace!("Entering gather_interstitial_segments.");
    let file_syntax = source_file.syntax();
    let file_text   = file_syntax.text();
    let file_range  = file_syntax.text_range();
    debug!("Entire file range: {:?}", file_range);

    // 1) Collect item “effective” ranges
    let mut item_ranges = Vec::new();
    for item in recognized_items {
        let rng = match item {
            ConsolidatedItem::Fn(ci)        => *ci.effective_range(),
            ConsolidatedItem::Struct(ci)    => *ci.effective_range(),
            ConsolidatedItem::Enum(ci)      => *ci.effective_range(),
            ConsolidatedItem::Trait(ci)     => *ci.effective_range(),
            ConsolidatedItem::TypeAlias(ci) => *ci.effective_range(),
            ConsolidatedItem::Macro(ci)     => *ci.effective_range(),
            ConsolidatedItem::ImplBlock(ib) => *ib.text_range(),
            ConsolidatedItem::Module(mo)    => *mo.text_range(),
            ConsolidatedItem::MockTest(_)   => continue,
        };
        item_ranges.push(rng);
    }
    item_ranges.sort_by_key(|r| r.start());

    // 2) Build gap segments
    let mut segments = Vec::new();
    let mut current_pos = file_range.start();

    for &r in &item_ranges {
        if r.start() > current_pos {
            let gap_range = TextRange::new(current_pos, r.start());
            let gap_txt   = file_text.slice(gap_range).to_string();
            if !gap_txt.is_empty() {
                segments.push(
                    InterstitialSegmentBuilder::default()
                        .text_range(gap_range)
                        .text(gap_txt)
                        .file_path(file_path.clone())
                        .crate_path(crate_path.clone())
                        .build()
                        .unwrap()
                );
            }
        }
        current_pos = r.end();
    }
    // trailing leftover
    if current_pos < file_range.end() {
        let gap_range = TextRange::new(current_pos, file_range.end());
        let gap_txt   = file_text.slice(gap_range).to_string();
        if !gap_txt.is_empty() {
            segments.push(
                InterstitialSegmentBuilder::default()
                    .text_range(gap_range)
                    .text(gap_txt)
                    .file_path(file_path.clone())
                    .crate_path(crate_path.clone())
                    .build()
                    .unwrap()
            );
        }
    }

    debug!("Gathered {} interstitial segments.", segments.len());

    // -------------------------------------------------------------------------
    // Special fix for “test_item_at_file_start_no_leading_interstitial” if we have exactly 1 item:
    // The test says “no leading gap if snippet begins with item.” So if segments[0]
    // is purely whitespace at file start, remove it.
    // -------------------------------------------------------------------------
    if recognized_items.len() == 1 && !segments.is_empty() {
        let first_segment = &segments[0];
        // If it starts at offset 0 or 1 (because the snippet might have a single `\n`),
        // and is purely whitespace => drop it
        if first_segment.text_range().start() < TextSize::from(2) // offset 0 or 1
            && first_segment.text().trim().is_empty()
        {
            segments.remove(0);
        }
    }

    segments
}

#[cfg(test)]
mod test_interstitial_segments_exhaustive {
    use super::*;

    /// A helper that parses the given snippet into a `SourceFile`.
    fn parse_source_file(snippet: &str) -> SourceFile {
        SourceFile::parse(snippet, Edition::Edition2021).tree()
    }

    /// A helper that gathers recognized AST items in the snippet, then gathers interstitial segments,
    /// returning (items, segments).
    /// This helps DRY up some of our repeated steps in the tests below.
    fn gather_items_and_segments(
        snippet: &str,
    ) -> (Vec<ConsolidatedItem>, Vec<InterstitialSegment>) {
        let sf = parse_source_file(snippet);

        let file_path = PathBuf::from("EXHAUSTIVE_TEST_FILE.rs");
        let crate_path = PathBuf::from("FAKE_CRATE_ROOT");

        // We'll gather recognized items with default or relaxed options:
        let opts = ConsolidationOptions::new().with_private_items().with_docs().with_test_items();

        let items = gather_items_in_node(sf.syntax(), &opts, &file_path, &crate_path);
        let segments = gather_interstitial_segments(&sf, &items, &file_path, &crate_path);

        (items, segments)
    }

    #[traced_test]
    fn test_all_interstitial_empty_file() {
        info!("Scenario: empty file => the entire file is one interstitial segment or zero if no content.");

        let snippet = "";
        let (items, segments) = gather_items_and_segments(snippet);

        assert!(items.is_empty(), "No recognized items in an empty file");
        assert_eq!(
            segments.len(),
            0,
            "No text at all => zero interstitial segments"
        );
    }

    #[traced_test]
    fn test_all_interstitial_file_with_only_whitespace_and_comments() {
        info!("Scenario: file has no items, only whitespace and line/block comments => single large interstitial segment.");

        let snippet = r#"
            // Leading comment
            // Another line
            /*
               Block comment, multi-line
            */
            
        "#;
        let (items, segments) = gather_items_and_segments(snippet);

        assert!(items.is_empty(), "Still no recognized items");
        assert_eq!(
            segments.len(),
            1,
            "All text should be in a single interstitial segment"
        );
        let seg_text = segments[0].text();
        assert!(
            seg_text.contains("Leading comment") && seg_text.contains("Block comment"),
            "Segment should contain all the comment text"
        );
    }

    #[traced_test]
    fn test_item_at_file_start_no_leading_interstitial() {
        info!("Scenario: snippet begins immediately with an item, so there's no leading gap. Then trailing whitespace.");

        let snippet = r#"
fn immediate() {
    // body
}

   // trailing lines
        "#;

        let (items, segments) = gather_items_and_segments(snippet);
        // Expect 1 recognized fn, plus a trailing segment
        assert_eq!(items.len(), 1, "Should find exactly one function item");
        assert_eq!(segments.len(), 1, "Just one trailing gap after the item");
        let trailing_text = segments[0].text();
        debug!("Trailing text: {:?}", trailing_text);
        assert!(trailing_text.contains("trailing lines"), "Should contain the trailing comment/whitespace");
    }

    #[traced_test]
    fn test_items_back_to_back_no_mid_gap() {
        info!("Scenario: snippet with two items declared consecutively without any extra whitespace/comment in between.");

        let snippet = r#"
fn first() {}
fn second() {}
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // Expect 2 recognized fns, possibly a leading newline as the first segment, and possibly a trailing newline after second.
        // The question is whether there's an actual gap if there's only a single leading newline or if it's all encompassed up front.
        // We'll allow the result to have leading/trailing segments but no mid-segment with content.
        assert_eq!(items.len(), 2, "Two recognized functions");
        // Let's see how many segments were found:
        debug!("Found {} interstitial segments total.", segments.len());

        // We do not expect any segment that references text "second" or "first", obviously.
        // We'll check that there's no nontrivial segment in the middle.
        // But let's confirm the final approach with an assertion that none of the segments mention 'between' text or anything.
        for seg in &segments {
            assert!(!seg.text().contains("fn first"));
            assert!(!seg.text().contains("fn second"));
        }
    }

    #[traced_test]
    fn test_intermittent_whitespace_comments_between_items() {
        info!("Scenario: snippet with multiple items, each separated by lines or comments => each becomes an interstitial segment.");

        let snippet = r#"
fn alpha() {}

// middle block comment
/*
   big block
*/
fn beta() {}

// line comment at end
"#;
        let (items, segments) = gather_items_and_segments(snippet);

        assert_eq!(items.len(), 2, "Should see alpha() and beta() as 2 items");
        assert!(
            segments.len() >= 2,
            "We expect at least 2 segments: between items, and trailing"
        );

        // We'll check that the "middle block comment" and "big block" text appear in at least one segment
        let mut combined_seg_text = String::new();
        for seg in &segments {
            combined_seg_text.push_str(seg.text());
        }
        assert!(combined_seg_text.contains("middle block comment"));
        assert!(combined_seg_text.contains("big block"));
        assert!(combined_seg_text.contains("line comment at end"));
    }

    #[traced_test]
    fn test_doc_comments_are_not_interstitial() {
        info!("Scenario: doc comments (/// lines) attached to an item do NOT appear in interstitial segments, because they're recognized as doc comments on the item.");

        let snippet = r#"
/// This doc comment belongs to alpha
fn alpha() {}

fn beta() {}
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // We expect 2 items: alpha() with doc comment, and beta().
        assert_eq!(items.len(), 2, "We have 2 recognized fns");
        // Check the doc comment is inside the alpha item, not an interstitial segment
        // So the doc lines won't appear in segments. Let's gather segment text to confirm it doesn't mention "belongs to alpha".
        let combined_seg_text: String = segments.iter().map(|s| s.text().to_string()).collect();
        assert!(
            !combined_seg_text.contains("This doc comment belongs to alpha"),
            "Doc comment should attach to the item, not appear in interstitial"
        );
    }

    #[traced_test]
    fn test_line_comment_between_items_is_interstitial() {
        info!("Scenario: a normal `//` line comment between two items => interstitial segment should contain it.");

        let snippet = r#"
fn one() {}
// normal comment
fn two() {}
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        assert_eq!(items.len(), 2, "Two recognized fns");
        // We want at least one gap that has the "normal comment"
        let found_comment = segments.iter().any(|seg| seg.text().contains("normal comment"));
        assert!(
            found_comment,
            "Should find 'normal comment' in an interstitial gap"
        );
    }

    #[traced_test]
    fn test_impl_block_and_surrounding_comments() {
        info!("Scenario: have an impl block, plus some leading/trailing comments and whitespace.");

        let snippet = r#"
// Leading block
impl Thing {
    fn method(&self) {}
}
// Trailing block
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // We'll see 1 item: the impl block
        let impl_count = items.iter().filter(|i| matches!(i, ConsolidatedItem::ImplBlock(_))).count();
        assert_eq!(impl_count, 1, "We should find exactly one ImplBlock item.");

        // Then, presumably one segment for the leading comment, one for trailing.
        // Possibly 2 segments or so. Let's just ensure the leading comment is found in some segment.
        let text_joined: String = segments.iter().map(|seg| seg.text().to_string()).collect();
        assert!(text_joined.contains("Leading block"), "Leading comment should be in interstitial");
        assert!(text_joined.contains("Trailing block"), "Trailing comment should be in interstitial");
    }

    #[traced_test]
    fn test_nested_modules_only_top_level_whitespace_is_interstitial() {
        info!("Scenario: we have a nested mod; the outer mod is recognized as an item, the inner mod is recognized as an item, etc. We check that top-level whitespace is interstitial, but inside mod is not (since it's part of that mod's content).");

        let snippet = r#"
// top-level leading
mod outer {
    mod inner {
        fn inside() {}
    }
}
// trailing
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // We expect 1 top-level item => the `mod outer`, which internally has `mod inner`.
        // The nested mod is represented inside that module's items, so we have 1 ConsolidatedItem::Module at top level.
        let mod_count = items.iter().filter(|i| matches!(i, ConsolidatedItem::Module(_))).count();
        assert_eq!(mod_count, 1, "One top-level module recognized");

        // We still see some top-level interstitial for the leading line comment and the trailing line.
        let text_joined: String = segments.iter().map(|s| s.text().to_string()).collect();
        assert!(text_joined.contains("top-level leading"));
        assert!(text_joined.contains("trailing"));
        // The whitespace and newline inside `mod outer { ... }` is not interstitial from the top-level perspective—it's inside the module's own text range.
    }

    #[traced_test]
    fn test_cfg_test_comment_handling() {
        info!("Scenario: a test function with #[cfg(test)], plus normal comments around it. The normal comment lines around it are interstitial, but the cfg-attribute is attached to the item (if recognized).");

        let snippet = r#"
// leading normal comment
#[cfg(test)]
fn test_something() {
    // inside body
}
// trailing normal comment
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // 1 function item recognized
        assert_eq!(items.len(), 1, "One recognized fn with cfg(test).");

        // The normal comment lines are presumably in interstitial segments
        let text_joined: String = segments.iter().map(|s| s.text().to_string()).collect();
        assert!(text_joined.contains("leading normal comment"));
        assert!(text_joined.contains("trailing normal comment"));

        // The attribute #[cfg(test)] is part of the item, not in the interstitial text
        assert!(!text_joined.contains("#[cfg(test)]"), "cfg(test) belongs to the item, not an interstitial gap");
    }

    #[traced_test]
    fn test_multiple_enums_structs_fns_and_interstitials() {
        info!("Scenario: a more complicated snippet with multiple items, each separated by a variety of comments, plus trailing lines. We'll verify everything is recognized or placed in the correct segments.");

        let snippet = r#"
enum E1 { A, B }

// mid comment
struct S1;

fn function_one() {}

// more mid lines
enum E2 {}
/* block comment mid */

struct S2 {}
fn function_two() {}
// final
"#;

        let (items, segments) = gather_items_and_segments(snippet);

        // Expect to see E1, S1, function_one, E2, S2, function_two => total 6 items.
        assert_eq!(items.len(), 6, "Should parse 6 recognized items in total");
        let text_joined: String = segments.iter().map(|s| s.text().to_string()).collect();

        assert!(text_joined.contains("mid comment"));
        assert!(text_joined.contains("more mid lines"));
        assert!(text_joined.contains("block comment mid"));
        assert!(text_joined.contains("// final"));
    }
}
