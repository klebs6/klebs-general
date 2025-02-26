// ---------------- [ File: src/rebuild_librs_with_new_top_block.rs ]
crate::ix!();

#[derive(Debug)]
struct ExistingMacro {
    pub text: String,     // the full macro call text, e.g. "x!{command_runner}"
    pub range: TextRange, // so we can remove it from the file
}

fn is_imports_line(item: &ast::Item) -> bool {
    if let Some(module_item) = ast::Module::cast(item.syntax().clone()) {
        if let Some(name_ident) = module_item.name() {
            return name_ident.text() == "imports";
        }
    } else if let Some(use_item) = ast::Use::cast(item.syntax().clone()) {
        let text = use_item.syntax().text().to_string();
        if text.contains("imports::*") {
            return true;
        }
    }
    false
}

/// Returns Some("x!{foo}") if it's an `x!{...}` macro call, else None.
fn is_x_macro(item: &ast::Item) -> Option<String> {
    let mac_call = ast::MacroCall::cast(item.syntax().clone())?;
    let path = mac_call.path()?;
    if path.syntax().text().to_string().trim() == "x" {
        let macro_text = item.syntax().text().to_string();
        Some(macro_text)
    } else {
        None
    }
}

/// Extract the stem from a macro call like `x!{some_name}` => `"some_name"`.
fn parse_x_macro_stem(macro_text: &str) -> Option<String> {
    let start = macro_text.find('{')?;
    let end   = macro_text.rfind('}')?;
    if start+1 < end {
        Some(macro_text[start+1..end].trim().to_string())
    } else {
        None
    }
}

/// Rebuilds `lib.rs` so that:
///  1) The existing `#[macro_use] mod imports; use imports::*;` lines remain at the top
///  2) We remove all x! calls from anywhere in the file, unify them with new stems, sort them,
///     and insert them as a single block **right after** the last import line
///  3) Everything else remains exactly as-is
///
/// `existing_new_stems`: e.g. ["my_new_file"]
pub fn rebuild_lib_rs_with_new_top_block(
    parsed_file: &SourceFile,
    old_text: &str,
    existing_new_stems: &[String],
    file_path: &Path,
) -> Result<String, SourceFileRegistrationError> 
{
    let file_syntax: SyntaxNode = parsed_file.syntax().clone();

    // 1) Identify lines that are `imports`, plus any existing x! macros
    let mut imports_ranges = vec![];
    let mut existing_macros = vec![];

    for item in parsed_file.items() {
        if is_imports_line(&item) {
            imports_ranges.push(item.syntax().text_range());
        } else if let Some(mac_text) = is_x_macro(&item) {
            existing_macros.push(ExistingMacro {
                text: mac_text,
                range: item.syntax().text_range(),
            });
        }
    }

    // 2) Sort macros by ascending start offset, then remove them from the text in descending order
    //    so we don't disrupt the offsets for earlier macros.
    existing_macros.sort_by_key(|m| m.range.start());
    // remove them in reverse
    let mut edited_text = old_text.to_string();
    for mac in existing_macros.iter().rev() {
        let start = usize::from(mac.range.start()).min(edited_text.len());
        let end   = usize::from(mac.range.end()).min(edited_text.len());
        if start < end {
            edited_text.replace_range(start..end, "");
        }
    }

    // 3) Parse out the stems from existing macros, unify them with `existing_new_stems`.
    //    We'll then sort them so final x! calls are alphabetical.
    let mut stems = Vec::new();
    // The macros are reversed, so if we want top-down order, let's re-reverse the iteration:
    for mac in existing_macros.iter() {
        if let Some(stem) = parse_x_macro_stem(&mac.text) {
            stems.push(stem);
        }
    }
    // stems is in bottom-up order, so let's reverse to get top-down order if you prefer:
    stems.reverse();

    // unify with new stems
    for new_stem in existing_new_stems {
        if !stems.contains(new_stem) {
            stems.push(new_stem.clone());
        }
    }
    // **Sort them** for alphabetical order
    stems.sort();

    // 4) Build a single block of x! calls:
    //  // ---------------- [ File: src/lib.rs ]
    //  x!{some_stem}
    //  x!{another_stem}
    let mut lines = vec!["// ---------------- [ File: src/lib.rs ]".to_string()];
    for stem in stems {
        lines.push(format!("x!{{{}}}", stem));
    }
    let new_block = lines.join("\n");

    // 5) Insert that block right after the last import line
    let mut max_import_end: Option<usize> = None;
    for rng in &imports_ranges {
        let end_usize = usize::from(rng.end());
        if max_import_end.map_or(true, |old| end_usize > old) {
            max_import_end = Some(end_usize);
        }
    }
    let insertion_offset = max_import_end.unwrap_or(0);

    // 6) Splice the new block into `edited_text`
    let mut final_text = String::new();
    if insertion_offset > edited_text.len() {
        // If it’s out of range for some reason, clamp
        let clamped = edited_text.len();
        final_text.push_str(&edited_text[..clamped]);
        final_text.push('\n');
        final_text.push_str(&new_block);
        final_text.push('\n');
        // no remainder
    } else {
        final_text.push_str(&edited_text[..insertion_offset]);
        final_text.push('\n');
        final_text.push_str(&new_block);
        final_text.push('\n');
        final_text.push_str(&edited_text[insertion_offset..]);
    }

    Ok(final_text)
}

#[cfg(test)]
mod test_rebuild_librs_with_new_top_block {
    use super::*;
    use ra_ap_syntax::{SourceFile, Edition};
    use crate::SourceFileRegistrationError;

    /// Helper function to parse the file and call rebuild.
    fn run_rebuild(old_text: &str, new_top_block: &str) -> Result<String, SourceFileRegistrationError> {
        let parsed_file = SourceFile::parse(old_text, Edition::Edition2021).tree();
        rebuild_lib_rs_with_new_top_block(&parsed_file, old_text, new_top_block)
    }

    /// 1) If there are no x! macros, we simply insert the new block at the top of the file
    ///    (or at the end if no real items).
    #[test]
    fn test_no_macros_entirely_empty_file() {
        let old_text = "";
        let new_block = "// top block\nx!{something}";
        let final_str = run_rebuild(old_text, new_block).expect("Should succeed");

        // We'll do a partial check:
        //  - it should contain the new block
        //  - we expect at least one trailing newline
        assert!(final_str.contains("// top block\nx!{something}"));
        // There's no macros or items => the entire text is basically our new block
        // If you want an exact check that it's "block\n" with no leading whitespace, do that:
        assert_eq!(final_str.trim(), new_block.trim());
    }

    /// 2) If there's no macros, but we do have some real items, we place the top block above them.
    #[test]
    fn test_no_macros_with_real_items() {
        let old_text = r#"
#![allow(unused)]
fn existing_function() {}

"#;
        let new_block = "// top block\nx!{stuff}";
        let final_str = run_rebuild(old_text, new_block).unwrap();

        // Partial checks:
        //  - We didn't remove the existing function or the attr
        assert!(final_str.contains("fn existing_function() {}"));
        assert!(final_str.contains("#![allow(unused)]"));
        //  - The new block is inserted somewhere near the top, presumably after the attr
        // We'll check that the final text has the new block *before* "fn existing_function()".
        let idx_new_block = final_str.find(new_block).expect("new block not found!");
        let idx_existing_fn = final_str.find("fn existing_function").unwrap();
        assert!(
            idx_new_block < idx_existing_fn,
            "The top block should appear before fn existing_function()"
        );
    }

    /// 3) If we have top-level x! macros only (and no real items),
    ///    then those macros are removed and replaced by the new block.
    #[test]
    fn test_macros_only_replaced() {
        let old_text = r#"
x!{foo}
x!{bar}
"#;
        let new_block = "// my top\nx!{new_stuff}";
        let final_str = run_rebuild(old_text, new_block).unwrap();

        // old macros should be gone
        assert!(!final_str.contains("x!{foo}"));
        assert!(!final_str.contains("x!{bar}"));
        // new block should be present
        assert!(final_str.contains("// my top\nx!{new_stuff}"));
        // No real items or doc lines => final text is basically that block, plus some newlines
        // We'll do a trim check to be sure:
        assert_eq!(final_str.trim(), new_block.trim());
    }

    /// 4) If a real item appears, then any x! macros after that item => error
    #[test]
    fn test_macro_after_item_is_error() {
        let old_text = r#"
fn something() {}

x!{foo}
"#;
        let new_block = "// top block\nx!{new}";
        let result = run_rebuild(old_text, new_block);
        match result {
            Err(SourceFileRegistrationError::EncounteredAnXMacroAfterWeAlreadySawANonAttributeItem_NotRewritingSafely) => {}
            other => panic!("Expected 'macro after item' error, got {:?}", other),
        }
    }

    /// 5) doc comment, attribute, x! macro => once a real item is found, no macros allowed after
    #[test]
    fn test_doc_comments_and_attr_skipped() {
        let old_text = r#"
// Some doc comment
#![allow(dead_code)]

// Next line is our x! macro
x!{prelude}

// Then a real item
fn the_real_item() {}

// Then another x! => error
x!{late}
"#;
        let new_block = "// top block\nx!{unified}";
        let result = run_rebuild(old_text, new_block);
        match result {
            Err(SourceFileRegistrationError::EncounteredAnXMacroAfterWeAlreadySawANonAttributeItem_NotRewritingSafely) => {}
            other => panic!("Expected macro-after-item error, got {:?}", other),
        }
    }

    /// 6) Multiple macros at top => remove them all, then we succeed once we hit a real item
    #[test]
    fn test_multiple_macros_at_top_followed_by_real_item() {
        let old_text = r#"
x!{alpha}
x!{beta}
fn real_thing() {}
"#;
        let new_block = "// new top\nx!{gamma}";
        let final_str = run_rebuild(old_text, new_block).unwrap();

        // macros gone
        assert!(!final_str.contains("x!{alpha}"));
        assert!(!final_str.contains("x!{beta}"));

        // new block present
        assert!(final_str.contains("// new top\nx!{gamma}"));

        // real item remains
        assert!(final_str.contains("fn real_thing() {}"));

        // check that new block appears before "fn real_thing" in text
        let idx_new = final_str.find("// new top").unwrap();
        let idx_fn = final_str.find("fn real_thing").unwrap();
        assert!(idx_new < idx_fn, "new block inserted above the real item");
    }

    /// 7) If the file is nothing but whitespace or doc comments, the new block is appended at the end
    #[test]
    fn test_only_doc_comments_and_whitespace() {
        let old_text = r#"
// Some doc
// Another comment

"#;
        let new_block = "// top block\nx!{stuff}";
        let final_str = run_rebuild(old_text, new_block).unwrap();

        // We keep the doc lines
        assert!(final_str.contains("// Some doc"));
        assert!(final_str.contains("// Another comment"));
        // new block appended
        assert!(final_str.contains("// top block\nx!{stuff}"));
        // let's ensure the doc lines appear *before* the new block
        let idx_doc = final_str.find("// Some doc").unwrap();
        let idx_new = final_str.find("// top block").unwrap();
        assert!(idx_doc < idx_new);
    }

    #[test]
    fn test_macro_among_comments() {
        let old_text = r#"
    // Some doc
    x!{foo}
    // Another doc
    "#;
        let new_block = "// top block\nx!{updated}";
        let final_str = run_rebuild(old_text, new_block).unwrap();

        // old macro removed
        assert!(!final_str.contains("x!{foo}"), "Should remove macro foo");

        // Because `// Some doc` might be recognized as a doc comment for that macro item,
        // removing the macro can remove that doc. So we do NOT assert it remains.

        // But we can still check if the second line remains, if it’s separate enough 
        // that the parser doesn't treat it as doc for foo:
        assert!(final_str.contains("// Another doc"),
            "Expected the line after x! macro to remain, unless it also got attached to the macro. If it fails, remove this.");

        // new block appended
        assert!(final_str.contains("// top block\nx!{updated}"),
            "The new block must appear somewhere in the final text");
    }
}
