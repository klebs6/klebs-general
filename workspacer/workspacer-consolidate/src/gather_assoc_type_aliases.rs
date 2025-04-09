// ---------------- [ File: workspacer-consolidate/src/gather_assoc_type_aliases.rs ]
crate::ix!();

/// Gathers all associated type aliases in an impl block, respecting skip logic and collecting docs/attrs.
pub fn gather_assoc_type_aliases(
    impl_ast:   &ast::Impl, 
    options:    &ConsolidationOptions,
    file_path:  &PathBuf,
    crate_path: &PathBuf,

) -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>> {

    let mut out = Vec::new();
    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(ty_alias) = ast::TypeAlias::cast(item.syntax().clone()) {
                if !crate::skip_checks::should_skip_item(ty_alias.syntax(), options) {

                    let raw_range = ty_alias.syntax().text_range();
                    // For now, we pass the same range for both raw & effective. If you
                    // want to exclude leading/trailing normal comments, you can compute
                    // a trimmed range. We'll keep it simple here:
                    let eff_range = raw_range;

                    let docs = if *options.include_docs() {
                        extract_docs(ty_alias.syntax())
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(ty_alias.syntax());
                    
                    out.push(CrateInterfaceItem::new_with_paths_and_ranges(
                        ty_alias,
                        docs,
                        attrs,
                        None,
                        Some(options.clone()),
                        file_path.clone(),
                        crate_path.clone(),
                        raw_range,
                        eff_range,
                    ));
                } else {
                    info!("Skipping type_alias in impl: either test item or private item was disallowed");
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod test_gather_assoc_type_aliases {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxNode, Edition};

    /// Helper: parse snippet => root SyntaxNode
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Find first ast::Impl if present.
    fn find_first_impl(root: &SyntaxNode) -> Option<ast::Impl> {
        for node in root.descendants() {
            if let Some(impl_block) = ast::Impl::cast(node) {
                return Some(impl_block);
            }
        }
        None
    }

    /// Default options: we include docs for these tests
    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new().with_docs()
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    #[test]
    fn test_impl_with_no_items() {
        let snippet = r#"
            trait MyTrait {}
            impl MyTrait for MyStruct {
                // no items
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert!(aliases.is_empty(), "No items => no type aliases");
    }

    #[test]
    fn test_impl_with_non_type_alias_items() {
        // Trait + impl that has a fn but no associated type => empty
        let snippet = r#"
            trait MyTrait {}
            impl MyTrait for MyStruct {
                fn some_fn() {}
                const VAL: i32 = 10;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert!(aliases.is_empty(), "No type aliases => empty");
    }

    #[test]
    fn test_single_type_alias() {
        // A trait that *requires* an associated type, plus an impl that defines it
        let snippet = r#"
            trait MyTrait { type AliasA; }
            impl MyTrait for MyStruct {
                type AliasA = i32;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert_eq!(aliases.len(), 1, "One type alias in the impl block");
        let alias_item = &aliases[0];
        assert_eq!(*alias_item.docs(), None, "No doc comment by default");
        assert_eq!(*alias_item.attributes(), None, "No attributes by default");
    }

    #[test]
    fn test_multiple_type_aliases() {
        // A trait with two required associated types => the impl defines both
        let snippet = r#"
            trait MyTrait {
                type A;
                type B;
            }
            impl MyTrait for MyStruct {
                type A = i32;
                type B = String;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert_eq!(aliases.len(), 2, "Should gather two type aliases");
    }

    #[test]
    fn test_skip_logic_for_aliases() {
        // We mark one associated type with #[cfg(test)] so that skip logic
        // might skip it if we exclude test items.
        let snippet = r#"
            trait MyTrait {
                type NormalAlias;
                #[cfg(test)]
                type TestAlias;
            }
            impl MyTrait for MyStruct {
                type NormalAlias = u64;

                #[cfg(test)]
                type TestAlias = i32;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");

        // Suppose our skip logic excludes test items
        let mut opts = ConsolidationOptions::new().with_docs();
        // e.g. if you have `opts = opts.without_test_items()` or no `.with_test_items()`

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        // We expect only the normal one if `#[cfg(test)]` is excluded
        assert_eq!(aliases.len(), 1, "Skipped the #[cfg(test)] alias, kept normal");
    }

    #[test]
    fn test_type_alias_with_docs_and_attrs() {
        // The trait has an associated type; the impl defines it with doc + an attribute
        let snippet = r#"
            trait MyTrait { type FancyAlias; }
            impl MyTrait for MyStruct {
                /// doc for fancy
                #[some_attr]
                type FancyAlias = (i32, i32);
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert_eq!(aliases.len(), 1, "One alias present");
        let alias_item = &aliases[0];

        let docs = alias_item.docs().clone().expect("Should have doc lines");
        assert!(docs.contains("/// doc for fancy"));

        let attr_opt = alias_item.attributes();
        assert!(attr_opt.is_some(), "We have at least one attribute");
        let attrs = attr_opt.clone().unwrap();
        assert!(attrs.contains("#[some_attr]"), "Should see the attribute line");
    }

    #[test]
    fn test_skip_docs_in_options() {
        // We do *not* enable .with_docs() => doc lines are ignored
        let snippet = r#"
            trait MyTrait { type WithDoc; }
            impl MyTrait for Something {
                /// doc line
                type WithDoc = i32;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        let opts = ConsolidationOptions::new(); // no .with_docs()

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert_eq!(aliases.len(), 1, "One type alias");
        let alias_item = &aliases[0];
        assert_eq!(*alias_item.docs(), None, "Docs are disabled => docs None");
    }

    #[test]
    fn test_impl_no_assoc_item_list() {
        // A weird partial snippet that might parse as an impl with no braces
        let snippet = r#"
            trait MyTrait { type T; }
            impl MyTrait for MyStruct;
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl block");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert!(aliases.is_empty(), "No assoc_item_list => no aliases");
    }

    #[test]
    fn test_trait_impl_with_type_aliases() {
        // Standard usage: trait with multiple associated types => the impl defines them
        let snippet = r#"
            trait AnotherTrait {
                type Associated;
                type Another;
            }
            impl AnotherTrait for Foo {
                type Associated = i64;
                type Another = Result<(), String>;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl block");
        let opts = default_options();

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        assert_eq!(aliases.len(), 2, "We have two type aliases in the trait impl");
    }

    #[test]
    fn test_complex_impl_alias_scenario() {
        // Some with doc lines, some with #[cfg(test)], etc.
        let snippet = r#"
            trait Complex {
                type A; 
                #[cfg(test)] type B;
                type C;
                #[cfg(test)] type D;
            }

            impl Complex for MyStruct {
                /// doc for A
                type A = i32;

                #[cfg(test)]
                type B = u32;

                #[some_attr]
                type C = String;

                /// doc for D
                #[cfg(test)]
                type D = f64;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected impl");
        // We skip #[cfg(test)] items
        let mut opts = ConsolidationOptions::new().with_docs();
        // e.g. no `.with_test_items()`
        // or some skip logic in should_skip_item that excludes them

        let file_path = PathBuf::from("TEST_ONLY_file_path.rs");
        let crate_path = PathBuf::from("TEST_ONLY_crate_path");

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts, &file_path, &crate_path);
        // Expect to keep A & C, skip B & D
        assert_eq!(aliases.len(), 2, "Kept 2 (A,C), skipped 2 test items (B,D)");
        let doc_a = aliases[0].docs().clone().unwrap_or_default();
        assert!(doc_a.contains("doc for A"), "We keep doc for A");
        let attr_c = aliases[1].attributes().clone().unwrap();
        assert!(attr_c.contains("#[some_attr]"), "We keep attribute for C");
    }
}
