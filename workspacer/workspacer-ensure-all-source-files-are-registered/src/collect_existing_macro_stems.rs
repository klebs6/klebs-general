// ---------------- [ File: src/collect_existing_macro_stems.rs ]
crate::ix!();

/// Collects any existing `x!{foo}` expansions found among the top-level items,
/// returning those “foo” stems.  
/// Bails out if it sees unexpected gating (e.g. `#[cfg] x!{...}`) or multiple x! items at once.
pub fn collect_existing_mod_macro_stems(
    parsed_file: &SourceFile
) -> Result<Vec<String>, SourceFileRegistrationError>
{
    let mut stems = vec![];

    // Iterate top-level items: these are `ast::Item` in ra_ap_syntax.
    for item in parsed_file.items() {
        // We want to see if `item.syntax()` is a MacroCall or a Module.
        if let Some(mac_item) = ast::MacroCall::cast(item.syntax().clone()) {
            // Check for any outer attributes
            if mac_item.attrs().count() > 0 {
                return Err(SourceFileRegistrationError::FoundAnUnhandlableTopLevelMacroCallWithAttributes);
            }

            // Make sure it's `x!{ ... }`
            if let Some(path) = mac_item.path() {
                let path_text = path.syntax().text().to_string();
                if path_text.trim() == "x" {
                    // Extract what's inside the macro
                    if let Some(tt) = mac_item.token_tree() {
                        let macro_inner = tt.syntax().text().to_string();
                        let chunk = macro_inner.trim_matches([' ', '{', '}', ',']).to_string();
                        if chunk.contains(',') {
                            return Err(
                                SourceFileRegistrationError::MultipleItemsInXMacroUnsupported {
                                    chunk
                                }
                            );
                        }
                        stems.push(chunk);
                    }
                }
            }
        } else if let Some(mod_item) = ast::Module::cast(item.syntax().clone()) {
            // If user wrote `pub mod foo;` we bail, since that might conflict with x! usage
            if let Some(name_ident) = mod_item.name() {
                let mod_name = name_ident.text().to_string();
                if !mod_name.is_empty() {
                    return Err(SourceFileRegistrationError::FoundARawModNameWhichWeDontHandlePleaseRemoveOrUnifyWithXMacros {
                        mod_name
                    });
                }
            }
        }
    }
    Ok(stems)
}

// ---------------------------------------------------------------------------
// Below is an exhaustive test suite for collect_existing_mod_macro_stems()
// ---------------------------------------------------------------------------
#[cfg(test)]
mod test_collect_existing_mod_macro_stems {
    use super::*;
    use ra_ap_syntax::{SourceFile, Edition};

    /// Helper to parse a string into a SourceFile.
    fn parse_source(input: &str) -> SourceFile {
        let parse = SourceFile::parse(input, Edition::Edition2021);
        // In real usage, you might check parse.errors() if you want
        parse.tree()
    }

    /// 1) Empty file => returns empty stems, no errors
    #[test]
    fn test_empty_file_no_macros() {
        let src = "";
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file)
            .expect("No errors for an empty file");
        assert_eq!(stems.len(), 0, "Should return an empty vec");
    }

    /// 2) Single x! macro with one name => we get that stem in the result
    #[test]
    fn test_single_macro() {
        let src = r#"
x!{hello}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file)
            .expect("Should parse a single macro fine");
        assert_eq!(stems, vec!["hello"]);
    }

    /// 3) Single x! macro but it has an attribute => we expect an error
    #[test]
    fn test_macro_with_attribute_is_error() {
        let src = r#"
#[cfg(feature="foo")]
x!{something}
"#;
        let file = parse_source(src);
        let result = collect_existing_mod_macro_stems(&file);
        match result {
            Err(SourceFileRegistrationError::FoundAnUnhandlableTopLevelMacroCallWithAttributes) => {
                // This is correct
            },
            other => panic!("Expected FoundAnUnhandlableTopLevelMacroCallWithAttributes, got: {:?}", other),
        }
    }

    /// 4) Single x! macro with multiple items => error
    #[test]
    fn test_macro_with_multiple_items_is_error() {
        let src = r#"
x!{foo, bar}
"#;
        let file = parse_source(src);
        let result = collect_existing_mod_macro_stems(&file);
        match result {
            Err(SourceFileRegistrationError::MultipleItemsInXMacroUnsupported{chunk}) => {
                assert_eq!(chunk, "foo, bar");
            }
            other => panic!("Expected MultipleItemsInXMacroUnsupported, got: {:?}", other),
        }
    }

    /// 5) Single macro but the path is not `x`, e.g. `y!{something}`, => we skip it
    ///    so stems is empty, no error
    #[test]
    fn test_macro_with_different_path_skipped() {
        let src = r#"
y!{not_x}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file)
            .expect("Should not fail with a non-x macro");
        assert!(stems.is_empty(), "No recognized macros => empty stems");
    }

    /// 6) If user wrote `pub mod foo;` => error
    #[test]
    fn test_raw_mod_foo_is_error() {
        let src = r#"
pub mod foo;
"#;
        let file = parse_source(src);
        let result = collect_existing_mod_macro_stems(&file);
        match result {
            Err(SourceFileRegistrationError::FoundARawModNameWhichWeDontHandlePleaseRemoveOrUnifyWithXMacros { mod_name }) => {
                assert_eq!(mod_name, "foo");
            }
            other => panic!("Expected FoundARawModNameWhichWeDontHandlePleaseRemoveOrUnifyWithXMacros, got: {:?}", other),
        }
    }

    /// 7) If user wrote `mod ;` without a name, we consider that zero-len => no error.
    #[test]
    fn test_module_no_name_ok() {
        let src = "mod ; // weird but let's see if it passes";
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file)
            .expect("Should not error for mod with no name");
        assert_eq!(stems.len(), 0);
    }

    /// 8) Multiple macros => we gather them in the order they appear
    #[test]
    fn test_multiple_macros_gather_all() {
        let src = r#"
x!{alpha}
x!{beta}
x!{gamma}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file).unwrap();
        assert_eq!(stems, vec!["alpha", "beta", "gamma"]);
    }

    /// 9) Macros that aren't x! => we ignore them, but if there's also an x! we gather it.
    #[test]
    fn test_mixed_macro_paths_only_x_collected() {
        let src = r#"
foo!{bar}
x!{hello}
zork!{something}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file).unwrap();
        assert_eq!(stems, vec!["hello"], "Only the x! macro is collected");
    }

    /// 10) We place doc comments or attributes that do NOT belong directly to macros => no effect
    #[test]
    fn test_doc_comments_unrelated_attributes_are_ignored() {
        let src = r#"
// Some doc comment
#![allow(dead_code)]
x!{stuff}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file).unwrap();
        // There's an x! macro with no attributes on it, so we gather "stuff"
        assert_eq!(stems, vec!["stuff"]);
    }

    /// 11) If there's an attribute on the macro call itself => error
    ///     (We covered a similar scenario above, but let's be explicit.)
    #[test]
    fn test_attribute_on_macro_call_itself() {
        let src = r#"
#[cfg(target_os="linux")]
x!{some_os_stuff}
"#;
        let file = parse_source(src);
        let result = collect_existing_mod_macro_stems(&file);
        match result {
            Err(SourceFileRegistrationError::FoundAnUnhandlableTopLevelMacroCallWithAttributes) => (),
            other => panic!("Expected FoundAnUnhandlableTopLevelMacroCallWithAttributes, got {:?}", other),
        }
    }

    /// 12) If there's an x! macro, but the token_tree is empty => we gather an empty string
    ///     (this is unusual, but let's confirm it doesn't fail).
    #[test]
    fn test_macro_call_with_empty_braces() {
        let src = r#"
x!{}
"#;
        let file = parse_source(src);
        let stems = collect_existing_mod_macro_stems(&file).unwrap();
        // Then chunk = "" inside braces
        assert_eq!(stems, vec![""], "We gather an empty string as the stem");
    }
}
