// ---------------- [ File: src/gather_assoc_type_aliases.rs ]
crate::ix!();

/// Gathers all associated type aliases in an impl block, respecting skip logic and collecting docs/attrs.
pub fn gather_assoc_type_aliases(
    impl_ast: &ast::Impl, 
    options: &ConsolidationOptions
) -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>> 
{
    let mut out = Vec::new();
    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(ty_alias) = ast::TypeAlias::cast(item.syntax().clone()) {
                if !crate::skip_checks::should_skip_item(ty_alias.syntax(), options) {
                    let docs = if *options.include_docs() {
                        extract_docs(ty_alias.syntax())
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(ty_alias.syntax());
                    
                    out.push(crate::crate_interface_item::CrateInterfaceItem::new(
                        ty_alias,
                        docs,
                        attrs,
                        None
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
#[disable]
mod test_gather_assoc_type_aliases {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxNode, SyntaxKind, Edition};

    // If your code references them from your crate, import them, for example:
    // use crate::{
    //     gather_assoc_type_aliases, // the function being tested
    //     ConsolidationOptions,
    //     crate_interface_item::CrateInterfaceItem,
    //     extract_docs,
    //     gather_all_attrs,
    //     skip_checks::should_skip_item,
    // };

    /// Helper to parse a Rust snippet into a `SyntaxNode`.
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Extracts the first `ast::Impl` from the syntax tree, or None if none is found.
    fn find_first_impl(root: &SyntaxNode) -> Option<ast::Impl> {
        for node in root.descendants() {
            if let Some(impl_node) = ast::Impl::cast(node) {
                return Some(impl_node);
            }
        }
        None
    }

    /// A convenience function to create default `ConsolidationOptions`.
    /// Adjust toggles as needed for your real usage.
    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new().with_docs()
        // .with_test_items() or any other toggles your real code might need
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) No associated items in impl => gather_assoc_type_aliases should return empty.
    #[test]
    fn test_impl_with_no_items() {
        let snippet = r#"
            impl MyStruct {
                // no associated items
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert!(aliases.is_empty(), "No assoc items => no type aliases");
    }

    /// 2) Impl has associated items, but no type aliases => empty result.
    ///    This also confirms it ignores other item types (e.g. fn, const).
    #[test]
    fn test_impl_with_non_type_alias_items() {
        let snippet = r#"
            impl MyStruct {
                fn some_fn() {}
                const VAL: i32 = 10;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert!(aliases.is_empty(), "No type aliases => empty result");
    }

    /// 3) An impl with a single type alias => we gather exactly one item, with docs/attrs if present.
    #[test]
    fn test_single_type_alias() {
        let snippet = r#"
            impl MyStruct {
                type AliasA = i32;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert_eq!(aliases.len(), 1, "One type alias in the impl block");
        // Check that there's no doc or attr by default
        let alias_item = &aliases[0];
        assert_eq!(*alias_item.docs(), None, "No doc comment by default");
        assert_eq!(*alias_item.attributes(), None, "No attributes by default");
    }

    /// 4) Multiple type aliases => gather them all, unless skip logic says otherwise.
    #[test]
    fn test_multiple_type_aliases() {
        let snippet = r#"
            impl MyStruct {
                type AliasA = i32;
                type AliasB = String;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert_eq!(aliases.len(), 2, "Should gather two type aliases");
    }

    /// 5) If `should_skip_item` says to skip certain aliases (e.g., test or private),
    ///    we confirm they're not included in the result. We'll define a snippet
    ///    with one normal alias, one test alias, and a consolidation option that
    ///    presumably leads `should_skip_item` to skip the test alias.
    #[test]
    fn test_skip_logic_for_aliases() {
        let snippet = r#"
            impl MyStruct {
                type NormalAlias = u64;

                #[cfg(test)]
                type TestAlias = i32;
            }
        "#;
        // We'll assume `should_skip_item` sees `#[cfg(test)]` and decides to skip it if test items are excluded.
        // We'll define an opts that excludes test items, for demonstration.

        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");

        let mut opts = ConsolidationOptions::new().with_docs();
        // Hypothetically, if you have `.without_test_items()` or similar:
        // opts = opts.without_test_items();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        // We expect only 1 if `#[cfg(test)]` is skipped
        // but if your real skip logic differs, adapt the check
        assert_eq!(aliases.len(), 1, "Skipped the #[cfg(test)] alias, kept the normal one");
    }

    /// 6) A type alias with doc comments and attributes => they appear in docs/attrs if `include_docs()` is on.
    #[test]
    fn test_type_alias_with_docs_and_attrs() {
        let snippet = r#"
            impl MyStruct {
                /// This is a doc for the type alias
                #[some_attr]
                type FancyAlias = (i32, i32);
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let mut opts = ConsolidationOptions::new().with_docs();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert_eq!(aliases.len(), 1, "One alias present");
        let alias_item = &aliases[0];

        let docs = alias_item.docs().expect("we should have docs included");
        assert!(
            docs.contains("/// This is a doc for the type alias"),
            "Doc comment should appear in docs()"
        );

        let attr_opt = alias_item.attributes();
        assert!(attr_opt.is_some(), "We have at least one attribute");
        let attrs = attr_opt.unwrap();
        assert!(
            attrs.contains("#[some_attr]"),
            "The attribute should appear in attributes()"
        );
    }

    /// 7) If docs are disabled in options, doc comments are omitted.
    #[test]
    fn test_skip_docs_in_options() {
        let snippet = r#"
            impl Something {
                /// doc line
                type WithDoc = i32;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        let opts = ConsolidationOptions::new(); // no .with_docs()

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert_eq!(aliases.len(), 1);
        let alias_item = &aliases[0];
        assert_eq!(*alias_item.docs(), None, "Docs are disabled => docs None");
    }

    /// 8) If there's no assoc_item_list in the impl, we get an empty result.
    #[test]
    fn test_impl_no_assoc_item_list() {
        let snippet = r#"
            impl MyStruct;
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert!(aliases.is_empty(), "No assoc_item_list => no type aliases");
    }

    /// 9) If the impl is for a trait (e.g. `impl MyTrait for Foo { ... }`) that includes type aliases,
    ///    it works the same: gather any type aliases not skipped.
    #[test]
    fn test_trait_impl_with_type_aliases() {
        let snippet = r#"
            impl MyTrait for Foo {
                type Associated = i64;
                type Another = Result<(), String>;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        assert_eq!(aliases.len(), 2, "We have two type aliases in the trait impl");
    }

    /// 10) A more complex snippet with multiple type aliases, some with attributes,
    ///     some doc comments, some test/skip, to confirm partial inclusion.
    #[test]
    fn test_complex_impl_alias_scenario() {
        let snippet = r#"
            impl ComplexImpl {
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
        let impl_ast = find_first_impl(&root).expect("Expected an impl");
        // Suppose we skip `#[cfg(test)]` items
        let mut opts = ConsolidationOptions::new().with_docs();
        // e.g. opts = opts.without_test_items() if that’s how your code does skipping test items

        let aliases = gather_assoc_type_aliases(&impl_ast, &opts);
        // We expect to keep A and C, skip B and D
        assert_eq!(aliases.len(), 2, "Kept 2, skipped 2 test items");
        // Check that the docs/attrs exist for the ones we kept if relevant
        let doc_a = aliases[0].docs().unwrap_or_default();
        assert!(
            doc_a.contains("doc for A"),
            "We keep doc for A"
        );
        let attr_c = aliases[1].attributes().unwrap();
        assert!(
            attr_c.contains("#[some_attr]"),
            "We keep attribute for C"
        );
    }
}
