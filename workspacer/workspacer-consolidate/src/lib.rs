// ---------------- [ File: workspacer-consolidate/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{compute_effective_range}
x!{formatting}
x!{consolidate_crate_interface}
x!{consolidated_crate_interface}
x!{consolidated_item}
x!{consolidation_options}
x!{crate_interface_item_serde}
x!{crate_interface_item}
x!{gather_all_attrs}
x!{gather_assoc_type_aliases}
x!{gather_crate_items}
x!{gather_fn_item}
x!{gather_impl_methods}
x!{gather_items_in_node}
x!{gather_module}
x!{generate_impl_signature}
x!{guess_is_function}
x!{has_cfg_test_attr}
x!{impl_block_interface}
x!{interstitial_segment}
x!{is_in_test_module}
x!{leading_spaces}
x!{maybe_build_enum}
x!{maybe_build_function}
x!{maybe_build_impl_block_node}
x!{maybe_build_macro_call}
x!{maybe_build_macro_rules}
x!{maybe_build_module}
x!{maybe_build_struct}
x!{maybe_build_trait}
x!{maybe_build_type_alias}
x!{merge_doc_attrs}
x!{merge}
x!{module_interface}
x!{skip_checks}
x!{trim_to_60}
x!{try_cast_and_build_item}

#[cfg(test)]
mod test_text_range {
    use super::*;

    /// Helpers: parse snippet -> gather crate items -> return them
    fn gather_single_file_items(snippet: &str) -> Vec<ConsolidatedItem> {
        // 1) Parse the snippet
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        let sf = parse.tree();

        // 2) We'll use a minimal ConsolidationOptions that doesn't skip anything.
        let opts = ConsolidationOptions::new()
            .with_private_items()
            .with_test_items()
            .with_docs()
            .with_fn_bodies()
            .with_fn_bodies_in_tests();

        // 3) Dummy file_path & crate_path
        let file_path = PathBuf::from("TEST_ONLY_file.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_root");

        // 4) Gather
        gather_crate_items(&sf, &opts, &file_path, &crate_path)
    }

    /// A helper to assert that the item’s text_range matches the underlying syntax node’s range.
    /// For a `CrateInterfaceItem<ast::Fn>`, for example, you can do `ci.item().syntax().text_range()`.
    /// Then compare that to `ci.text_range()`.
    fn assert_ranges_match(
        syntax_range: TextRange,
        consolidated_range: TextRange,
        context: &str,
    ) {
        assert_eq!(
            syntax_range, consolidated_range,
            "Mismatch in text_range for {}. Syntax node range = {:?}, item range = {:?}",
            context, syntax_range, consolidated_range
        );
    }

    #[test]
    fn test_fn_text_range() {
        let snippet = r#"
            fn example_fn() {
                let x = 10;
            }
        "#;
        let items = gather_single_file_items(snippet);

        // We expect exactly one item => a ConsolidatedItem::Fn
        assert_eq!(items.len(), 1, "Expected exactly one item (fn). Found: {:?}", items);
        match &items[0] {
            ConsolidatedItem::Fn(ci) => {
                // Compare the syntax node’s range with the stored text_range
                let syntax_range = ci.item().syntax().text_range();
                let item_range   = ci.text_range();
                assert_ranges_match(syntax_range, *item_range, "fn example_fn");
            },
            other => panic!("Expected a single fn item, got {:?}", other),
        }
    }

    #[test]
    fn test_struct_text_range() {
        let snippet = r#"
            struct MyStruct {
                field: i32,
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1, "Expected one struct item");
        match &items[0] {
            ConsolidatedItem::Struct(ci) => {
                let syntax_range = ci.item().syntax().text_range();
                let item_range   = ci.text_range();
                assert_ranges_match(syntax_range, *item_range, "struct MyStruct");
            },
            other => panic!("Expected struct, got {:?}", other),
        }
    }

    #[test]
    fn test_enum_text_range() {
        let snippet = r#"
            enum Color {
                Red,
                Green,
                Blue,
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1, "Expected one enum item");
        match &items[0] {
            ConsolidatedItem::Enum(ci) => {
                let syntax_range = ci.item().syntax().text_range();
                assert_ranges_match(syntax_range, *ci.text_range(), "enum Color");
            },
            other => panic!("Expected enum, got {:?}", other),
        }
    }

    #[test]
    fn test_trait_text_range() {
        let snippet = r#"
            trait Example {
                fn do_stuff(&self);
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::Trait(ci) => {
                let syntax_range = ci.item().syntax().text_range();
                assert_ranges_match(syntax_range, *ci.text_range(), "trait Example");
            },
            other => panic!("Expected trait, got {:?}", other),
        }
    }

    #[test]
    fn test_type_alias_text_range() {
        let snippet = r#"
            type AliasA = i64;
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::TypeAlias(ci) => {
                let syntax_range = ci.item().syntax().text_range();
                assert_ranges_match(syntax_range, *ci.text_range(), "type AliasA");
            },
            other => panic!("Expected type alias, got {:?}", other),
        }
    }

    #[test]
    fn test_macro_rules_text_range() {
        let snippet = r#"
            macro_rules! my_macro {
                () => {};
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::Macro(ci) => {
                let syntax_range = ci.item().syntax().text_range();
                assert_ranges_match(syntax_range, *ci.text_range(), "macro_rules my_macro");
            },
            other => panic!("Expected macro_rules, got {:?}", other),
        }
    }

    #[test]
    fn test_module_text_range() {
        let snippet = r#"
            mod submod {
                fn inside() {}
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::Module(mi) => {
                // For a ModuleInterface, we do `mi.text_range()`.
                // There's no `mi.item().syntax()` directly, but we know the underlying RA node is an `ast::Module`.
                // So we can parse the snippet again or do:
                //   let mod_node = ast::Module::cast(...) 
                // if needed. For a quick check, see if the text_range covers "mod submod { ... }".
                //
                // We can compare `mi.text_range()` to the entire snippet range or just do:
                let actual_range = mi.text_range();
                // You might do some length check:
                // For example:
                let text = snippet;
                let trimmed_len = text.trim().len() as u32; 
                // We'll just do a basic sanity check that the range length is > 0:
                assert!(actual_range.len() > 0.into());
            },
            other => panic!("Expected module, got {:?}", other),
        }
    }

    #[test]
    fn test_impl_block_text_range() {
        let snippet = r#"
            impl SomeTrait for Foo {
                fn method(&self) {}
            }
        "#;
        let items = gather_single_file_items(snippet);
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::ImplBlock(ib) => {
                let actual_range = ib.text_range();
                // Possibly you compare with your parse snippet length or do further analysis:
                assert!(actual_range.len() > 0.into(), "Impl block range should be > 0");
            },
            other => panic!("Expected impl block, got {:?}", other),
        }
    }
}
