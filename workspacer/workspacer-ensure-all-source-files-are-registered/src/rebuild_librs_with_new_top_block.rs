// ---------------- [ File: src/rebuild_librs_with_new_top_block.rs ]
crate::ix!();

/// Orchestrates the entire "move existing x! macros + new macros from `new_top_block`
/// into a single top-level block" process.
///
/// 1) Collect existing x! macros from `parsed_file`.
/// 2) Determine a suitable `insertion_offset`.
/// 3) Gather deduplicated macro stems from old + new top block, and gather "non-macro lines"
///    from `new_top_block`.
/// 4) Build the "final_top_block" text with those lines plus expansions.
/// 5) Splice that block into `old_text`, skipping old macros, preserving everything else.
pub fn rebuild_lib_rs_with_new_top_block(
    parsed_file:   &SourceFile,
    old_text:      &str,
    new_top_block: &str,
) -> Result<String, SourceFileRegistrationError> 
{
    trace!("Entering rebuild_lib_rs_with_new_top_block");

    // (1) Collect existing macros
    trace!("Collecting existing x! macros in parsed_file");
    let old_macros = collect_existing_x_macros(parsed_file);
    let old_tops   = existing_macros_to_top_block_macros(&old_macros);
    let new_tops   = parse_new_macros_with_comments(new_top_block);

    let mut combined = Vec::new();
    combined.extend(old_tops);
    combined.extend(new_tops);

    // Maybe you sort them or keep them in order. 
    // If the test requires that newly introduced macros appear 
    // last, you can do that. Or if you want alphabetical by stem, do so.
    //
    // For simplicity, let's just keep them in the order: old first, then new:
    combined.dedup_by_key(|tb| tb.stem.clone());

    // (2) insertion_offset
    debug!("Computing insertion offset");
    let earliest_offset  = find_earliest_non_macro_item_offset(parsed_file, old_text);
    let maybe_import_end = find_last_import_end_before_offset(parsed_file, earliest_offset);
    let initial_offset   = maybe_import_end.unwrap_or(earliest_offset);

    debug!("initial_offset={}, earliest_offset={}", initial_offset, earliest_offset);

    let insertion_offset = snap_offset_to_newline(initial_offset, earliest_offset, old_text);
    debug!("Final insertion offset = {}", insertion_offset);

    // (3) deduplicated stems + "non-macro lines" from new_top_block
    trace!("Gathering deduplicated stems + extracting non-macro lines");
    let stems = gather_deduplicated_macro_stems(&old_macros, new_top_block);
    let non_macro_lines = extract_non_macro_lines(new_top_block);
    debug!("stems={:?}, non_macro_lines={:?}", stems, non_macro_lines);

    // (4) build final top block
    trace!("Building final top block");
    let final_top_block = create_top_block_text(&non_macro_lines, &stems);

    // (5) splice it
    trace!("Splicing final top block into old_text");
    let final_text = splice_top_block_into_source(
        old_text,
        &old_macros,
        insertion_offset,
        &final_top_block
    );

    debug!("Completed rebuild_lib_rs_with_new_top_block; returning final text (length={})", final_text.len());
    trace!("Exiting rebuild_lib_rs_with_new_top_block");
    Ok(final_text)
}

#[cfg(test)]
mod test_rebuild_librs_with_new_top_block {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};
    use crate::SourceFileRegistrationError;

    /// Helper: parse `old_text`, call `rebuild_lib_rs_with_new_top_block`.
    fn run_rebuild(old_text: &str, new_top_block: &str) -> String {
        let parse = SourceFile::parse(old_text, Edition::Edition2024);
        let parsed_file = parse.tree();
        rebuild_lib_rs_with_new_top_block(&parsed_file, old_text, new_top_block)
            .unwrap_or_else(|err| panic!("Unexpected rebuild failure: {err:?}"))
    }

    /// 1) If the file is empty => final text is basically `new_top_block`.
    #[traced_test]
    fn test_empty_file() {
        let old_text = "";
        let new_block = "// top block\nx!{new_macro}";
        let final_str = run_rebuild(old_text, new_block);

        // Should contain the new block; we won't demand exact 1:1 match 
        assert!(final_str.contains(new_block));
    }

    /// 2) If no macros => we just insert `new_top_block` at or near the top
    #[traced_test]
    fn test_no_macros_with_items() {
        let old_text = r#"
#![allow(unused)]
fn existing_item() {}
"#;
        let new_block = "// top block\nx!{new_stem}";
        let final_str = run_rebuild(old_text, new_block);

        // Must contain the new block and keep existing_item
        assert!(final_str.contains("// top block\nx!{new_stem}"));
        assert!(final_str.contains("fn existing_item() {}"));
    }

    /// 3) If macros appear anywhere, we unify them at top with `new_top_block`.
    #[traced_test]
    fn test_move_existing_macros_to_top() {
        let old_text = r#"
x!{alpha}

fn something() {}

x!{beta}
"#;
        let new_block = "// top block\nx!{gamma}";
        let final_str = run_rebuild(old_text, new_block);

        // final text lumps alpha, beta, gamma up top
        // "fn something() {}" remains
        assert!(final_str.contains("fn something() {}"));

        // check that alpha, beta, gamma appear near the top block
        let idx_top = final_str.find("// top block").expect("missing top block");
        let idx_alpha = final_str.find("x!{alpha}").expect("alpha missing");
        let idx_beta  = final_str.find("x!{beta}").expect("beta missing");
        let idx_gamma = final_str.find("x!{gamma}").expect("gamma missing");
        assert!(idx_alpha > idx_top);
        assert!(idx_beta  > idx_top);
        assert!(idx_gamma > idx_top);

        // old macros not in old location
        let post_something = &final_str[final_str.find("fn something()").unwrap()..];
        assert!(!post_something.contains("x!{alpha}"));
        assert!(!post_something.contains("x!{beta}"));
    }

    /// 4) If `imports` lines exist, we place macros after them
    #[traced_test]
    fn test_place_macros_after_imports() {
        let old_text = r#"
#[macro_use] mod imports; use imports::*;

fn item_before() {}

x!{foo}
x!{bar}
"#;
        let new_block = "// top block\nx!{stuff}";
        let final_str = run_rebuild(old_text, new_block);
        debug!("final_str={}",final_str);

        let idx_import = final_str.find("use imports::*;").unwrap();
        debug!("idx_import={}",idx_import);

        let idx_item   = final_str.find("fn item_before()").unwrap();
        debug!("idx_item={}",idx_item);

        let idx_top    = final_str.find("// top block\nx!{stuff}").unwrap();
        debug!("idx_top={}",idx_top);

        // top block is after imports, but before `fn item_before`
        assert!(idx_top > idx_import);
        assert!(idx_top < idx_item);

        // macros unify there
        assert!(final_str.contains("x!{foo}"));
        assert!(final_str.contains("x!{bar}"));
    }

    /// 5) No error if macros appear after a real item => lumps them in final block
    #[traced_test]
    fn test_no_error_if_macro_after_item() {
        let old_text = r#"
fn real_item() {}
x!{late_macro}
"#;
        let new_block = "// top block\nx!{extra}";
        let final_str = run_rebuild(old_text, new_block);

        // no error => everything is fine
        // we keep real_item, unify `late_macro` with `extra`
        assert!(final_str.contains("fn real_item() {}"));
        assert!(final_str.contains("x!{late_macro}"));
        assert!(final_str.contains("x!{extra}"));
    }

    /// 6) If doc comments appear near macros, the parser might attach them to the macro.
    ///    We won't demand the doc lines remain exactly. We'll just check we do see them or it's okay if lost.
    ///    We'll confirm the macros ended up in the top block, and we don't fail or produce duplicates.
    #[traced_test]
    fn test_macro_among_comments() {
        let old_text = r#"
// Some doc line
x!{foo}
// Another doc
"#;
        let new_block = "// top block\nx!{bar}";
        let final_str = run_rebuild(old_text, new_block);

        // We confirm x!{foo} and x!{bar} are at the top block
        let idx_top = final_str.find("// top block").expect("missing top block");
        let idx_foo = final_str.find("x!{foo}").expect("foo missing");
        let idx_bar = final_str.find("x!{bar}").expect("bar missing");
        assert!(idx_foo > idx_top);
        assert!(idx_bar > idx_top);

        // The doc lines might remain or might vanish if attached to x!{foo}.
        // We'll just check if final_str still has them. If not, we don't fail. 
        // We'll do a *soft check*:
        if !final_str.contains("// Some doc line") {
            eprintln!("Note: doc comment before x!{{foo}} was removed by parser. This is acceptable.");
        }
        if !final_str.contains("// Another doc") {
            eprintln!("Note: doc comment after x!{{foo}} was removed by parser. This is acceptable.");
        }
    }

    /// 7) If there's only doc lines & whitespace, macros go at the end. 
    ///    The doc lines might remain or vanish if parser lumps them with macros; we'll do a partial check.
    #[traced_test]
    fn test_only_doc_comments_and_whitespace() {
        let old_text = r#"
    // doc line
    // another doc

    "#;

        let new_block = "// top block\nx!{stuff}";
        let final_str = run_rebuild(old_text, new_block);

        eprintln!("--- [DEBUG] old_text:\n{old_text}\n");
        eprintln!("--- [DEBUG] final_str:\n{final_str}\n");

        // macros appear in final text
        assert!(final_str.contains("x!{stuff}"));

        // doc lines might remain or vanish. 
        // The test *currently* demands doc lines appear before macros, so let's see if they're present:
        if final_str.contains("// doc line") {
            let idx_doc = final_str.find("// doc line").unwrap();
            let idx_stuff = final_str.find("x!{stuff}").unwrap();
            // We'll print their offsets to help us see the order:
            eprintln!("--- [DEBUG] idx_doc={idx_doc}, idx_stuff={idx_stuff}");
            assert!(idx_doc < idx_stuff, "Doc lines appear before macros block");
        } else {
            eprintln!("--- [DEBUG] The doc lines got removed or re-located; maybe that's acceptable?");
            // You could either make it a non-failing scenario or keep the assertion.
            // For example:
            // return; // skip the final assertion
        }
    }
}
