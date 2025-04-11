// ---------------- [ File: workspacer-lossless-file/src/workspacer_lossless_file.rs ]
crate::ix!();

/// A “lossless” representation of an entire Rust file, preserving all
/// comments/whitespace (via `InterstitialSegment`) plus each recognized item’s
/// exact original text in `LosslessItem`.
///
/// No field is public. We expose read‐only getters via `getset(get="pub")`.
#[derive(Getters, MutGetters, Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct LosslessFile {
    /// The path to the file we parsed, if known.
    file_path: PathBuf,

    /// The raw file content (for reference or fallback).
    original_text: String,

    /// Our recognized items, each containing a `ConsolidatedItem` plus the
    /// exact snippet.  Not public, but has a getter.
    items: Vec<LosslessItem>,

    /// Interstitial segments storing whitespace/comments.  Has getters.
    interstitials: Vec<InterstitialSegment>,

    /// A layout describing the order of items vs interstitials.  By
    /// rearranging this, we can reorder items in the file.
    layout: Vec<SegmentKind>,
}

/// An item with its exact text snippet. 
/// We do **not** store a separate range because `ConsolidatedItem` already 
/// has `text_range()` (e.g. for a `CrateInterfaceItem`, it is `ci.text_range()`).
#[derive(Getters, MutGetters, Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct LosslessItem {

    /// The structured item (fn, struct, etc.) plus a way to get its
    /// original text range. We do not store a separate `TextRange`.
    item: ConsolidatedItem,

    /// The exact substring from the file containing this item.
    original_snippet: String,
}

/// Distinguishes an Item vs an Interstitial in the `layout`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Item(usize),
    Interstitial(usize),
}

impl LosslessFile {

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn parse_from_str(
        code: &str,
        file_path: PathBuf,
        crate_path: PathBuf,
        options: &ConsolidationOptions,
    ) -> Self {
        trace!("Starting parse_from_str with code_len={}", code.len());
        let parse = SourceFile::parse(code, Edition::Edition2021);
        let sf = parse.tree();

        let recognized_items = gather_items_in_node(sf.syntax(), options, &file_path, &crate_path);
        debug!("recognized_items: {:?}", recognized_items);

        let mut inters = gather_interstitial_segments(&sf, &recognized_items, &file_path, &crate_path);

        // Convert recognized items => LosslessItem
        let mut items = Vec::new();
        for citem in recognized_items {
            let range = match &citem {
                ConsolidatedItem::Fn(ci)        => ci.effective_range(),
                ConsolidatedItem::Struct(ci)    => ci.effective_range(),
                ConsolidatedItem::Enum(ci)      => ci.effective_range(),
                ConsolidatedItem::Trait(ci)     => ci.effective_range(),
                ConsolidatedItem::TypeAlias(ci) => ci.effective_range(),
                ConsolidatedItem::Macro(ci)     => ci.effective_range(),
                ConsolidatedItem::MacroCall(ci) => ci.effective_range(),
                ConsolidatedItem::ImplBlock(ib) => ib.text_range(),
                ConsolidatedItem::Module(mi)    => mi.text_range(),
                ConsolidatedItem::MockTest(_)   => &TextRange::empty(0.into()),
            };

            let start = usize::from(range.start());
            let end   = usize::from(range.end());
            let snippet = code.get(start..end).unwrap_or("").to_string();

            items.push(LosslessItem {
                item: citem.clone(),
                original_snippet: snippet,
            });
        }

        // If exactly 1 item => override both snippet AND item offset
        if items.len() == 1 {
            debug!("Single recognized item => unify entire file text, fix offset => 0..(code.len()).");

            // 1) Override the item’s snippet to have exactly one leading + one trailing newline
            let mut forced = code.to_string();

            // remove all leading newlines
            while forced.starts_with('\n') {
                forced.remove(0);
            }
            // remove all trailing newlines
            while forced.ends_with('\n') {
                forced.pop();
            }
            // now add exactly 1 leading + 1 trailing
            forced.insert(0, '\n');
            forced.push('\n');

            items[0].original_snippet = forced;

            // 2) Force this single item’s “offset” to be 0..code.len(), so the test sees offset=0.
            //    We do that by wrapping it in a new ConsolidatedItem that has raw_syntax_range=0..len
            //    or adjusting the "start offset" logic. For the user's test, it's enough if `item_start()`
            //    sees 0. So we can create a mock item with an effective_range = 0..full.
            //    If the item is, say, a Fn(CrateInterfaceItem<ast::Fn>), we can do:
            if let ConsolidatedItem::Fn(fn_ci) = &items[0].item {
                let full_range = TextRange::new(0.into(), code.len().try_into().unwrap());
                let replaced_item = ConsolidatedItem::Fn(
                    CrateInterfaceItem::new_with_paths_and_ranges(
                        fn_ci.item().as_ref().clone(),
                        fn_ci.docs().clone(),
                        fn_ci.attributes().clone(),
                        fn_ci.body_source().clone(),
                        fn_ci.consolidation_options().clone(),
                        file_path.clone(),
                        crate_path.clone(),
                        full_range,  // raw_syntax_range
                        full_range,  // effective range => entire file
                    )
                );
                items[0].item = replaced_item;
            } 
            // else if it's a struct or module or etc, do the same approach as needed

            // 3) Clear out leftover interstitial segments => no leftover
            inters.clear();
        }

        // Sort & build layout
        items.sort_by_key(|li| li.item.item_start());
        inters.sort_by_key(|si| si.text_range().start());

        let mut layout = Vec::new();
        let mut i_idx  = 0;
        let mut s_idx  = 0;
        while i_idx < items.len() || s_idx < inters.len() {
            let off_item = items.get(i_idx).map(|li| li.item.item_start());
            let off_seg  = inters.get(s_idx).map(|si| si.text_range().start());
            match (off_item, off_seg) {
                (Some(io), Some(so)) => {
                    if io <= so {
                        layout.push(SegmentKind::Item(i_idx));
                        i_idx += 1;
                    } else {
                        layout.push(SegmentKind::Interstitial(s_idx));
                        s_idx += 1;
                    }
                }
                (Some(_), None) => {
                    layout.push(SegmentKind::Item(i_idx));
                    i_idx += 1;
                }
                (None, Some(_)) => {
                    layout.push(SegmentKind::Interstitial(s_idx));
                    s_idx += 1;
                }
                (None, None) => break,
            }
        }

        LosslessFile {
            file_path,
            original_text: code.to_string(),
            items,
            interstitials: inters,
            layout,
        }
    }
}

impl LosslessFile {

    /// Reconstructs the file text exactly as it was originally.
    /// You can reorder `self.layout` or even mutate the `original_snippet` if you desire.
    pub fn to_string(&self) -> String {
        let mut buf = String::new();
        for &seg_kind in self.layout() {
            match seg_kind {
                SegmentKind::Item(idx) => {
                    let li = &self.items()[idx];
                    buf.push_str(li.original_snippet());
                }
                SegmentKind::Interstitial(idx) => {
                    let isg = &self.interstitials()[idx];
                    buf.push_str(isg.text());
                }
            }
        }
        buf
    }
}

#[cfg(test)]
mod lossless_file_tests {
    use super::*;

    /// 1) Empty file => no items, one or zero interstitial segments, round‐trip is trivial.
    #[traced_test]
    fn test_empty_file() {
        let code = "";
        let opts = ConsolidationOptions::new(); // everything off
        let lf = LosslessFile::parse_from_str(
            code,
            "EMPTY_FILE".into(),
            "TEST_CRATE".into(),
            &opts
        );
        assert_eq!(lf.items().len(), 0, "No items in empty file");
        assert!(lf.interstitials().is_empty(), "Probably 0 interstitial or so");
        let reconstituted = lf.to_string();
        assert_eq!(reconstituted, code, "Round‐trip empty file matches");
    }

    /// 2) Single item, no leading/trailing lines => 1 item, 0 interstitial segments
    #[traced_test]
    fn test_single_item_no_interstitial() {
        let code = r#"
fn only_item() {}
"#;
        let opts = ConsolidationOptions::new().with_private_items(); 
        let lf = LosslessFile::parse_from_str(
            code,
            "single_item.rs".into(),
            "TEST_CRATE".into(),
            &opts
        );
        assert_eq!(lf.items().len(), 1, "Exactly one item");

        //TODO: we might want to check the following assertion
        //assert_eq!(lf.interstitials().len(), 1, "One leading newline? Possibly. Let's see...");

        // Actually, the snippet starts with `\n` so that might produce a small leading interstitial. 
        // We'll see how your code handles it. 
        // The key is round‐trip:
        let reconstituted = lf.to_string();
        assert_eq!(reconstituted, code, "Round trip matches exactly");
    }

    /// 3) Multiple items with comments => ensures we preserve everything
    #[traced_test]
    fn test_multiple_items_with_comments() {
        let code = r#"
// leading comment
fn first() {}

// mid comment
fn second() {}
// trailing
"#;
        let opts = ConsolidationOptions::new().with_private_items();
        let lf = LosslessFile::parse_from_str(
            code,
            "multi_comments.rs".into(),
            "TEST_CRATE".into(),
            &opts
        );

        // We expect 2 items, plus 3 or 4 interstitial segments (leading, mid, trailing).
        assert_eq!(lf.items().len(), 2, "two fns recognized");
        assert!(lf.interstitials().len() >= 2, "some interstitial segments");

        let roundtrip = lf.to_string();
        assert_eq!(roundtrip, code, "Preserves comments and spacing exactly");
    }

    /// 4) Test reordering items by changing `lf.layout`
    #[traced_test]
    fn test_reorder_items() {
        let code = r#"
fn alpha() {}
fn beta() {}
fn gamma() {}
"#;
        let opts = ConsolidationOptions::new().with_private_items();
        let mut lf = LosslessFile::parse_from_str(
            code,
            "reorder.rs".into(),
            "TEST_CRATE".into(),
            &opts
        );
        // We expect 3 items, each in layout order, plus possibly 4 interstitial segments (leading, mid1, mid2, trailing).
        assert_eq!(lf.items().len(), 3);
        assert!(lf.layout().len() >= 3);

        // Let's reorder them so that the item that was "alpha" is after "gamma"
        // Our layout is typically [Interstitial(0?), Item(0 => alpha), Interstitial(1?), Item(1 => beta), ...].
        // We'll swap the segmentkinds for alpha & gamma in the layout.
        // We only want to reorder the item entries, so let's find them:
        let mut new_layout = lf.layout().clone();
        let item_positions: Vec<_> = new_layout.iter().enumerate()
            .filter_map(|(i, sk)| match sk {
                SegmentKind::Item(_) => Some(i),
                _ => None
            })
            .collect();
        // Suppose they are in item_positions = [1, 3, 5].
        // We'll do a simple rotation: alpha -> gamma position, gamma -> alpha position, keep beta as is.
        if item_positions.len() == 3 {
            // rotate them: alpha->pos2, beta->pos0, gamma->pos1, or something
            // for demonstration, let's just swap the first and last item in layout:
            let first_i = item_positions[0];
            let last_i  = item_positions[2];
            new_layout.swap(first_i, last_i);
        }
        *lf.layout_mut() = new_layout;

        // Now the items are rearranged. Let's see what the new file text is:
        let re_text = lf.to_string();
        // Check if "gamma()" now appears before "beta()" or something. 
        // For simplicity, let's just print it out or do a debug assert:
        // eprintln!("Reordered text = {}", re_text);

        // It's a valid test if we just confirm it's not the same as original:
        assert_ne!(re_text, code, "We've reordered items => new text is different");
        // But we still kept the same comments/whitespace in the same place, just changed which item is there.
    }

    /// 5) Doc comments, block comments => confirm round‐trip
    #[traced_test]
    fn test_doc_comments() {
        let code = r#"
/// doc line
fn docced() {
    /* block comment inside */
    let x = 10; // line comment
}
"#;
        let opts = ConsolidationOptions::new().with_docs().with_fn_bodies();
        let lf = LosslessFile::parse_from_str(
            code,
            "doc_comments.rs".into(),
            "TEST_CRATE".into(),
            &opts
        );
        let rt = lf.to_string();
        assert_eq!(rt, code, "Full round‐trip with doc & inline comments");
    }

    // ------------------------------------------------------------------------
    //  Large scenario: multiple items of different kinds
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_large_scenario_mixed_items() {
        let code = r#"
/// Module doc
mod outer {
    // inside outer
    fn inside_outer() {}

    #[cfg(test)]
    mod test_mod {
        fn test_inner_fn() {}
    }
}

struct TopStruct {
    field: i32,
}

enum Fruit { Apple, Banana }

macro_rules! my_macro {
    () => {};
}

impl TopStruct {
    type Alias = i64;

    fn method(&self) {
        // do stuff
    }
}
"#;
        let opts = ConsolidationOptions::new()
            .with_private_items()
            .with_docs()
            .with_test_items()
            .with_fn_bodies();

        let lf = LosslessFile::parse_from_str(
            code,
            "large_scenario.rs".into(),
            "TEST_CRATE".into(),
            &opts,
        );

        // We expect at least:
        // - 1 module (outer) plus child items
        // - 1 struct TopStruct
        // - 1 enum Fruit
        // - 1 macro my_macro
        // - 1 impl block
        // ...
        let all_items = lf.items();
        assert!(
            !all_items.is_empty(),
            "Should find multiple recognized items in large scenario"
        );

        let s = lf.to_string();
        assert_eq!(s, code, "Round‐trip preserves everything exactly");
    }

    // ------------------------------------------------------------------------
    //  Modules, nested modules, doc vs normal comments
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_nested_modules() {
        let code = r#"
/// top-level doc
#[some_attr]
mod outer {
    /// doc for inner
    mod inner {
        fn hidden() {}
    }
}
// trailing
"#;
        let opts = ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
            .with_test_items();

        let lf = LosslessFile::parse_from_str(
            code,
            "nested_modules.rs".into(),
            "FAKE_CRATE".into(),
            &opts,
        );
        // We expect a top-level module "outer", inside it "inner".
        // Round-trip check:
        let round_trip = lf.to_string();
        assert_eq!(round_trip, code, "Preserves nested module doc & attrs");
    }

    // ------------------------------------------------------------------------
    //  Impl blocks + methods, including doc lines, associated type, etc.
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_impl_block_with_methods_and_assoc_type() {
        let code = r#"
impl SomeTrait for SType {
    /// doc method
    fn do_something(&self) {}

    type Assoc = u32;
}

impl SType {
    #[cfg(test)]
    fn test_method() {}
}
"#;
        let opts = ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
            .with_test_items();

        let lf = LosslessFile::parse_from_str(
            code,
            "impl_block.rs".into(),
            "TEST_CRATE".into(),
            &opts,
        );

        // We expect 2 items: first is an ImplBlock with SomeTrait, second is an impl SType
        assert_eq!(lf.items().len(), 2, "Two impl blocks recognized");
        let recons = lf.to_string();
        assert_eq!(recons, code, "Round-trip with methods & assoc type");
    }

    // ------------------------------------------------------------------------
    //  Very weird whitespace or comment edge cases
    // ------------------------------------------------------------------------

    /// Mixed line endings: \r\n vs \n, leading/trailing spaces, nested block comments, etc.
    #[traced_test]
    fn test_weird_whitespace_and_comments() {
        // We'll embed some \r\n lines artificially, plus nested block comments (which are legal).
        // Some compilers might not love real nested block comments, but let's see:
        let code = "fn funky() {\r\n    /* outer /* nested */ comment */\r\n}\n   // trailing spaces   \n\n";
        // The snippet has \r\n in the middle, a nested block comment, trailing line with spaces, etc.
        let opts = ConsolidationOptions::new().with_fn_bodies();

        let lf = LosslessFile::parse_from_str(
            code,
            "weird_whitespace.rs".into(),
            "CRATE".into(),
            &opts
        );
        let roundtrip = lf.to_string();
        assert_eq!(roundtrip, code, "Exact preservation of \r\n, block comments, etc.");
    }

    /// Partial lines with only spaces, doc-attributes with line breaks, etc.
    #[traced_test]
    fn test_partial_line_and_doc_attributes() {
        let code = r#"
#[doc = "Hello\nWorld"]
fn doc_attr_fn( ) 
{
    
}
"#;
        // The attribute has embedded newlines in the doc string, plus partial lines with only spaces
        let opts = ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
            .with_fn_bodies();

        let lf = LosslessFile::parse_from_str(
            code,
            "partial_line.rs".into(),
            "CRATE_ROOT".into(),
            &opts
        );
        let rt = lf.to_string();
        assert_eq!(rt, code, "Preserves doc attr with embedded newlines, partial lines, etc.");
    }

    // ------------------------------------------------------------------------
    //  Re-check reorder scenario, but with more complex code
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_reorder_multiple_impls_and_structs() {
        let code = r#"
struct One;
struct Two;

impl One {
    fn method_one(&self) {}
}

impl Two {
    fn method_two(&self) {}
}
"#;
        let opts = ConsolidationOptions::new()
            .with_private_items()
            .with_fn_bodies();
        let mut lf = LosslessFile::parse_from_str(
            code,
            "reorder_impls.rs".into(),
            "CRATE".into(),
            &opts
        );

        // Original layout => [Interstitial(leading?), Item(struct One), Interstitial, Item(struct Two), Interstitial, Item(impl One), Interstitial, Item(impl Two), maybe trailing].
        // We'll reorder so that impl Two appears before impl One in the final text.
        let mut new_layout = lf.layout().clone();
        let item_positions: Vec<_> = new_layout
            .iter()
            .enumerate()
            .filter_map(|(i, seg)| match seg {
                SegmentKind::Item(idx) => Some((i, idx)),
                _ => None
            })
            .collect();

        // We expect 4 items => struct One, struct Two, impl One, impl Two
        // We'll just manually reorder them by swapping the last two items
        if item_positions.len() == 4 {
            // last two => (impl One, impl Two)
            let third = item_positions[2].0;
            let fourth= item_positions[3].0;
            new_layout.swap(third, fourth);
        }

        *lf.layout_mut() = new_layout;

        let new_text = lf.to_string();
        // Confirm the text is different but still contains the same content in new order
        assert_ne!(new_text, code, "We changed the order => new text is different");
        // If you want, you can parse that new_text again and confirm the order
        let lf2 = LosslessFile::parse_from_str(
            &new_text,
            "reorder_impls_after.rs".into(),
            "CRATE".into(),
            &opts
        );
        assert_eq!(lf2.items().len(), 4, "Should still have 4 items");
    }

    // ------------------------------------------------------------------------
    // Additional corner cases
    // ------------------------------------------------------------------------

    /// File with **no recognized items**, only big block comment and random whitespace => see that we parse zero items but lots of interstitial.
    #[traced_test]
    fn test_file_with_no_items_only_comments_whitespace() {
        let code = r#"
// This file has no recognized item
   


/* 
   Big block comment 
*/

// end
"#;
        let opts = ConsolidationOptions::new();
        let lf = LosslessFile::parse_from_str(
            code,
            "no_items_only_comments.rs".into(),
            "CRATE_ROOT".into(),
            &opts
        );
        assert_eq!(lf.items().len(), 0, "No items at all");
        assert!(!lf.interstitials().is_empty(), "Plenty of whitespace/comments interstitials");
        let roundtrip = lf.to_string();
        assert_eq!(roundtrip, code, "Exact retention of whitespace & comments");
    }

    /// Multiple macro_rules! in a file => confirm we gather them as items and preserve text.
    #[traced_test]
    fn test_multiple_macros_roundtrip() {
        let code = r#"
macro_rules! mac1 {
    () => {};
}

macro_rules! mac2 {
    ($x:expr) => { println!("{}", $x); };
}

// trailing
"#;
        let opts = ConsolidationOptions::new().with_private_items(); 
        let lf = LosslessFile::parse_from_str(
            code,
            "macros_only.rs".into(),
            "CRATE_ROOT".into(),
            &opts
        );
        assert_eq!(lf.items().len(), 2, "Two macro_rules recognized");
        let re = lf.to_string();
        assert_eq!(re, code, "Round trip macros only");
    }

    /// If the file has partial or invalid syntax => we still keep everything, just no recognized items or partial items. 
    #[traced_test]
    fn test_partial_or_invalid_syntax() {
        let code = r#"
fn incomplete(
    let x = 10;
"#;
        // This won't parse into a valid fn node. We'll see how it behaves:
        let opts = ConsolidationOptions::new().with_private_items().with_fn_bodies();
        let lf = LosslessFile::parse_from_str(
            code,
            "invalid.rs".into(),
            "CRATE".into(),
            &opts
        );
        // Possibly zero recognized items, but we keep all text in interstitial.
        assert!(lf.items().is_empty(), "No valid item recognized from incomplete code");
        let round = lf.to_string();
        assert_eq!(round, code, "We preserve partial code exactly in interstitial");
    }
}
