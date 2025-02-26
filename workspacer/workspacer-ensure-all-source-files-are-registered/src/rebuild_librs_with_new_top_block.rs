// ---------------- [ File: src/rebuild_librs_with_new_top_block.rs ]
crate::ix!();

/// We’ll store references to existing x! macros in the order they appear.
#[derive(Debug)]
struct ExistingMacro {
    pub text: String,       // the full macro call text, e.g. "x!{command_runner}"
    pub range: TextRange,   // so we can remove it from the file
}

/// Return true if this is `#[macro_use] mod imports; use imports::*;`
fn is_imports_line(item: &ast::Item) -> bool {
    // If it's a `mod imports;`
    if let Some(module_item) = ast::Module::cast(item.syntax().clone()) {
        if let Some(name_ident) = module_item.name() {
            return name_ident.text() == "imports";
        }
    } else if let Some(use_item) = ast::Use::cast(item.syntax().clone()) {
        // e.g. "use imports::*;"
        let text = use_item.syntax().text().to_string();
        if text.contains("imports::*") {
            return true;
        }
    }
    false
}

/// Return Some("x!{foo}") if this item is an x! macro, or None otherwise.
fn is_x_macro(item: &ast::Item) -> Option<String> {
    let mac_call = ast::MacroCall::cast(item.syntax().clone())?;
    let path = mac_call.path()?;
    let path_text = path.syntax().text().to_string();
    if path_text.trim() == "x" {
        // The entire item text
        let macro_text = item.syntax().text().to_string();
        Some(macro_text)
    } else {
        None
    }
}

/// Rebuilds `lib.rs` so that:
///  1) `#[macro_use] mod imports; use imports::*;` lines stay at the very top
///  2) Then a single block of x! macros (old + new)
///  3) Everything else remains exactly as-is in the original text
///
/// `existing_new_stems` is the set of new `.rs` stems we discovered, e.g. ["my_new_file"].
pub fn rebuild_lib_rs_with_new_top_block(
    parsed_file: &SourceFile,
    old_text: &str,
    existing_new_stems: &[String],
    file_path: &Path, // optional, for error messages
) -> Result<String, SourceFileRegistrationError> 
{
    let file_syntax: SyntaxNode = parsed_file.syntax().clone();

    // 1) Identify lines that are `imports` items => track their position
    //    Identify existing x! macros => remove them, but store them in an ordered list.
    //    We'll keep everything else in place.
    let mut imports_ranges = vec![]; 
    let mut existing_macros = Vec::new(); 

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

    // 2) Remove the existing macros from the text
    //    We'll keep the import lines as is, so we do NOT remove them from the file.
    //    That means they'll remain exactly where they are in `old_text`.
    //    Sort by descending start offset so we can remove them without messing up earlier offsets.
    existing_macros.sort_by_key(|m| m.range.start());
    existing_macros.reverse();

    let mut edited_text = old_text.to_string();
    for mac in &existing_macros {
        let start = usize::from(mac.range.start());
        let end   = usize::from(mac.range.end());
        if end <= edited_text.len() {
            edited_text.replace_range(start..end, "");
        }
    }

    // 3) We'll unify the existing macros with new stems. We keep the existing order for old macros,
    //    then append any new stems that aren't already in the file. We'll detect which stems are already present?
    //    For simplicity, let's just append them. If you want to skip duplicates, do so.
    
    // (a) Gather the existing stems from the macros we found
    // e.g. "x!{command_runner}" => "command_runner"
    // We'll do a small parse. If the item has "x!{my_stem}", we parse out `my_stem`.
    let mut existing_stems = Vec::new();
    for mac in existing_macros.iter().rev() {
        // The macros are reversed, so we re-reverse them here if we want the original top-down order.
        if let Some(stem) = parse_x_macro_stem(&mac.text) {
            existing_stems.push(stem);
        }
    }
    // now `existing_stems` is in the original top-down order. The earliest macro is first.

    // (b) unify them with `existing_new_stems`
    // We'll skip duplicates if desired:
    for new_stem in existing_new_stems {
        if !existing_stems.contains(new_stem) {
            existing_stems.push(new_stem.clone());
        }
    }

    // 4) Build a single block of `x!{...}` lines
    //    e.g. 
    //    // ---------------- [ File: src/lib.rs ]
    //    x!{command_runner}
    //    x!{exit_status}
    //    x!{my_new_file}
    let mut lines = vec![];
    lines.push("// ---------------- [ File: src/lib.rs ]".to_string());
    for stem in existing_stems {
        lines.push(format!("x!{{{}}}", stem));
    }
    let new_top_block = lines.join("\n");

    // 5) Insert that new block right after the last import item (i.e. below all `imports`).
    //    If there are no import lines, we can place it at the top or do some fallback.
    //    We'll find the highest end offset among them and insert after that offset.
    let mut max_import_end: Option<usize> = None;
    for rng in &imports_ranges {
        let end_usize = usize::from(rng.end());
        if max_import_end.map_or(true, |old| end_usize > old) {
            max_import_end = Some(end_usize);
        }
    }
    let insertion_offset = max_import_end.unwrap_or(0);

    // 6) Now we splice the block into the `edited_text`.
    let mut final_text = String::new();
    final_text.push_str(&edited_text[..insertion_offset]);
    final_text.push('\n');
    final_text.push_str(&new_top_block);
    final_text.push('\n');
    final_text.push_str(&edited_text[insertion_offset..]);

    Ok(final_text)
}

/// This tries to parse e.g. "x!{command_runner}" => "command_runner".
fn parse_x_macro_stem(macro_text: &str) -> Option<String> {
    // We can do a naive approach. 
    // If macro_text is "x!{my_stem}", we find the substring between '{' and '}'.
    // This won't handle complex cases, but works for typical usage.
    let start = macro_text.find('{')?;
    let end = macro_text.rfind('}')?;
    if start+1 < end {
        let chunk = macro_text[start+1..end].trim().to_string();
        // e.g. "command_runner"
        Some(chunk)
    } else {
        None
    }
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
