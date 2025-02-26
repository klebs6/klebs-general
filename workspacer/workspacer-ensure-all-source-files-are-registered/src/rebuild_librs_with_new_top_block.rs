// ---------------- [ File: src/rebuild_librs_with_new_top_block.rs ]
crate::ix!();

// A small helper to detect `#[macro_use] mod imports; use imports::*;`
fn is_imports_item(item: &ast::Item) -> bool {
    if let Some(module_item) = ast::Module::cast(item.syntax().clone()) {
        if let Some(name_ident) = module_item.name() {
            return name_ident.text() == "imports";
        }
    } else if let Some(use_item) = ast::Use::cast(item.syntax().clone()) {
        let text = use_item.syntax().text().to_string();
        return text.contains("imports::*");
    }
    false
}

// A small helper to detect a `#[cfg(test)] mod something { ... }` item.
fn is_cfg_test_module(item: &ast::Item) -> bool {
    // The item must be a module
    let mod_item = if let Some(m) = ast::Module::cast(item.syntax().clone()) {
        m
    } else {
        return false;
    };
    // Then it must have an attribute with cfg(test)
    for attr in mod_item.attrs() {
        let text = attr.syntax().text().to_string();
        if text.contains("cfg(test") {
            return true;
        }
    }
    false
}

fn short_item_description(item: &ast::Item) -> String {
    let text = item.syntax().text().to_string();
    text.lines().next().unwrap_or("").trim().into()
}

/// Rebuilds `lib.rs` by:
/// - Keeping imports at the top,
/// - Then putting x! macros,
/// - Then test modules,
/// - Everything else is either appended or triggers an error (your choice).
/// The `file_path` is used for error messages only.
pub fn rebuild_lib_rs_with_new_top_block(
    parsed_file: &SourceFile,
    old_text: &str,
    new_top_block: &str,
    file_path: &Path, // so we can mention in errors
) -> Result<String, SourceFileRegistrationError> {
    let file_syntax: SyntaxNode = parsed_file.syntax().clone();

    // We'll store these items + their text ranges.
    let mut imports_items = vec![];
    let mut macros_items = vec![];
    let mut test_modules = vec![];
    let mut others = vec![];

    // 1) Classify items
    for item in parsed_file.items() {
        if is_imports_item(&item) {
            imports_items.push(item);
        } else if let Some(mac_item) = ast::MacroCall::cast(item.syntax().clone()) {
            // Check if it’s x! macro
            if let Some(path) = mac_item.path() {
                if path.syntax().text().to_string().trim() == "x" {
                    // It's an x! macro => store in macros_items
                    macros_items.push(item);
                } else {
                    // If you want to skip e.g. "foo!{}" macros, or treat them as others, your choice
                    others.push(item);
                }
            }
        } else if is_cfg_test_module(&item) {
            test_modules.push(item);
        } else {
            // e.g. a real function or something
            // Your policy: do you want to keep them? error? reorder them? 
            // Let's do "error" if we find a real item at top. 
            // Or you can store them in `others` to re-insert them below everything else.
            others.push(item);
        }
    }

    // 2) Remove the macros + test modules from text (since we want to re-insert them).
    // If we also want to remove imports from text so we can re-insert them first, do so. 
    // In your snippet, you want to keep imports in place. So let's remove only macros & test mods.
    let mut segments_to_remove = vec![];
    for it in macros_items.iter().chain(test_modules.iter()) {
        segments_to_remove.push(it.syntax().text_range());
    }
    // Sort by start offset descending
    segments_to_remove.sort_by_key(|r| u32::from(r.start()));
    let mut edited_text = old_text.to_string();
    for range in segments_to_remove.into_iter().rev() {
        let start = usize::from(range.start());
        let end = usize::from(range.end());
        if end <= edited_text.len() {
            edited_text.replace_range(start..end, "");
        }
    }

    // 3) Now we build new segments for macros + test modules
    //    Instead of re-inserting macros in alphabetical order, let's keep original order:
    //    But you can do macros_items.sort_by(...) if you want to reorder them.
    let macros_str = macros_items
        .iter()
        .map(|m| m.syntax().text().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let test_mods_str = test_modules
        .iter()
        .map(|tm| tm.syntax().text().to_string())
        .collect::<Vec<_>>()
        .join("\n\n");

    // 4) Insert `new_top_block` near the top
    //    Then macros, then test modules, then the remainder of the file.
    //    If you want to keep imports in place, don't remove them from text. 
    //    Our code hasn’t removed them, so they remain where they are. 
    //    We'll just insert macros/test after the top block. 
    let insertion_offset = find_top_block_insertion_offset(parsed_file, &edited_text)?;

    let mut final_text = String::new();
    // everything before insertion
    final_text.push_str(&edited_text[..insertion_offset]);
    // now our top block lines
    final_text.push_str(new_top_block);
    final_text.push('\n');
    // the macros we extracted from the file
    final_text.push_str(&macros_str);
    final_text.push('\n');
    // the test modules
    final_text.push_str(&test_mods_str);
    final_text.push('\n');
    // then the remainder of the file
    final_text.push_str(&edited_text[insertion_offset..]);

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
