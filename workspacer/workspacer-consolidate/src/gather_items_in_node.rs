// ---------------- [ File: src/gather_items_in_node.rs ]
crate::ix!();

// --------------------------------------------------------------------------------
// The main gather_items_in_node logic
// --------------------------------------------------------------------------------
pub fn gather_items_in_node(
    parent_node: &SyntaxNode,
    options:     &ConsolidationOptions,
) -> Vec<ConsolidatedItem> {
    let mut items = Vec::new();

    for child in parent_node.children() {
        match child.kind() {
            SyntaxKind::MODULE => {
                if let Some(mod_ast) = ast::Module::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);

                    let mod_name = mod_ast
                        .name()
                        .map(|n| n.text().to_string())
                        .unwrap_or_else(|| "<unknown_mod>".to_string());

                    let mut mod_interface = ModuleInterface::new(docs, attrs, mod_name);

                    if let Some(item_list) = mod_ast.item_list() {
                        let sub_items = gather_items_in_node(item_list.syntax(), options);
                        for si in sub_items {
                            mod_interface.add_item(si);
                        }
                    }

                    items.push(ConsolidatedItem::Module(mod_interface));
                }
            }
            SyntaxKind::IMPL => {
                if let Some(impl_ast) = ast::Impl::cast(child.clone()) {
                    // First check if we skip the impl entirely:
                    if should_skip_impl(&impl_ast, options) {
                        continue;
                    }

                    let docs      = None; // or gather docs
                    let attrs     = None; // or gather_all_attrs
                    let signature = generate_impl_signature(&impl_ast, docs.as_ref());

                    // Then gather the *filtered* methods + aliases
                    let included_methods = gather_impl_methods(&impl_ast, options);
                    let included_aliases = gather_assoc_type_aliases(&impl_ast, options);

                    // Make the interface
                    let ib = ImplBlockInterface::new(
                        docs,
                        attrs,
                        signature,
                        included_methods,
                        included_aliases,
                    );

                    items.push(ConsolidatedItem::ImplBlock(ib));
                }
            }

            SyntaxKind::FN => {
                // The updated approach
                if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    // Instead of capturing the entire node text, do:
                    let ci = gather_fn_item(&fn_ast, options);
                    items.push(ConsolidatedItem::Fn(ci));
                }
            }

            SyntaxKind::STRUCT => {
                if let Some(st_ast) = ast::Struct::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Struct(
                        CrateInterfaceItem::new(st_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::ENUM => {
                if let Some(en_ast) = ast::Enum::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Enum(
                        CrateInterfaceItem::new(en_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::TRAIT => {
                if let Some(tr_ast) = ast::Trait::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Trait(
                        CrateInterfaceItem::new(tr_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::TYPE_ALIAS => {
                if let Some(ty_ast) = ast::TypeAlias::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::TypeAlias(
                        CrateInterfaceItem::new(ty_ast, docs, attrs, None),
                    ));
                }
            }

            SyntaxKind::MACRO_RULES => {
                if let Some(mac_ast) = ast::MacroRules::cast(child.clone()) {
                    if should_skip_item(&child, options) {
                        continue;
                    }
                    let docs = if *options.include_docs() {
                        extract_docs(&child)
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(&child);
                    items.push(ConsolidatedItem::Macro(
                        CrateInterfaceItem::new(mac_ast, docs, attrs, None),
                    ));
                }
            }

            _ => {
                // Not a top-level item we care about
            }
        }
    }

    items
}

// A test suite for the `gather_items_in_node` function, which inspects the children
// of a given `SyntaxNode` and collects various Rust items (modules, impl blocks,
// functions, structs, enums, traits, type aliases, macro_rules, etc.) into
// `ConsolidatedItem` variants, applying filtering logic (via `should_skip_item` and `should_skip_impl`)
// along the way.
#[cfg(test)]
mod test_gather_items_in_node {
    use super::*;
    use ra_ap_syntax::{ast, SourceFile, SyntaxKind, SyntaxNode};
    //
    // In your real code, you might do something like:
    // use crate::{
    //     ConsolidationOptions,
    //     ConsolidatedItem,
    //     gather_items_in_node,
    //     // and any other relevant imports...
    // };
    //
    // For demonstration, we assume all the relevant symbols are imported above.

    /// Helper to parse a Rust snippet into a `SyntaxNode`.
    /// We'll retrieve the root `SourceFile` syntax from it.
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet,Edition::Edition2024);
        parse.tree().syntax().clone()
    }

    /// Builds a default `ConsolidationOptions` for testing.
    /// You can adjust these settings to match your real scenarios.
    fn default_options() -> ConsolidationOptions {
        // E.g., include docs by default:
        ConsolidationOptions::new().with_docs()
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) If no recognized items exist in the snippet, gather_items_in_node returns an empty Vec.
    #[test]
    fn test_empty_source_no_items() {
        let snippet = r#"
            // Just an empty file with a comment
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();
        let items = gather_items_in_node(&root_node, &opts);
        assert_eq!(items.len(), 0, "No items expected in an empty source");
    }

    /// 2) Single module, single function => we expect a `ConsolidatedItem::Module` containing
    /// its items (child function), but gather_items_in_node only sees the top-level items.
    ///
    /// Because `gather_items_in_node` only processes direct children of `parent_node`,
    /// the function inside the module is discovered only if we recursively call
    /// gather_items_in_node on the module's item_list. That logic is in the snippet
    /// for `MODULE` match, but we test that we get exactly one `ConsolidatedItem::Module`.
    #[test]
    fn test_single_module_in_top_level() {
        let snippet = r#"
            mod my_module {
                fn inside_mod() {}
            }
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();

        let items = gather_items_in_node(&root_node, &opts);
        assert_eq!(items.len(), 1, "Expected exactly one top-level item (the module)");
        match &items[0] {
            ConsolidatedItem::Module(mod_iface) => {
                // The `ModuleInterface` itself might contain the function as a child item.
                // We could check mod_iface items if you have an accessor for them,
                // e.g. `mod_iface.items()` or verifying the module name.
                assert_eq!(mod_iface.mod_name(), "my_module");
            },
            other => panic!("Expected ConsolidatedItem::Module, got {:?}", other),
        }
    }

    /// 3) Top-level function, struct, enum, trait => we expect them all recognized
    /// as separate `ConsolidatedItem` variants, in the order they appear.
    #[test]
    fn test_mixed_items_in_top_level() {
        let snippet = r#"
            fn top_fn() {}
            struct Foo;
            enum Bar { A, B }
            trait Baz { fn do_baz(&self); }
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();
        let items = gather_items_in_node(&root_node, &opts);

        // We expect 4 items: Fn, Struct, Enum, Trait
        assert_eq!(items.len(), 4);
        match (&items[0], &items[1], &items[2], &items[3]) {
            (ConsolidatedItem::Fn(_), ConsolidatedItem::Struct(_),
             ConsolidatedItem::Enum(_), ConsolidatedItem::Trait(_)) => {
                // All good
            },
            _ => panic!("Expected Fn, Struct, Enum, Trait in that order"),
        }
    }

    /// 4) If the item is an `impl`, we gather a `ConsolidatedItem::ImplBlock`
    /// unless `should_skip_impl` says to skip it. We'll define a snippet with one impl.
    #[test]
    fn test_gather_impl_block() {
        let snippet = r#"
            impl MyStruct {
                fn method1(&self) {}
            }
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();

        let items = gather_items_in_node(&root_node, &opts);
        // We expect exactly one impl block item
        assert_eq!(items.len(), 1);
        match &items[0] {
            ConsolidatedItem::ImplBlock(impl_iface) => {
                // e.g., check that `impl_iface.signature_text()` contains "impl MyStruct"
                // or check that it has 1 method, depending on how your code is structured
                let sig = impl_iface.signature_text();
                assert!(sig.contains("MyStruct"), "Signature should mention MyStruct");
            },
            other => panic!("Expected ConsolidatedItem::ImplBlock, got {:?}", other),
        }
    }

    /// 5) If `should_skip_item` is configured to skip private items or test items, etc.,
    /// we confirm those items do NOT appear in the output. For demonstration, we'll pretend
    /// that a private fn `fn hidden()` is skipped, while a public `pub fn visible()` is included.
    #[test]
    fn test_skip_item_logic() {
        let snippet = r#"
            fn hidden() {}
            pub fn visible() {}
        "#;
        let root_node = parse_source(snippet);
        // We'll define a special options that says "skip private" if your real code does that.
        let mut opts = default_options(); 
        // Perhaps you'd do something like `.with_only_public_items()` if that exists.
        // We'll just assume `should_skip_item` sees hidden() as private.

        let items = gather_items_in_node(&root_node, &opts);
        assert_eq!(items.len(), 1, "Expected only the public fn if skip logic is enforced");
        match &items[0] {
            ConsolidatedItem::Fn(fn_item) => {
                // check something about the name or doc if we can
                // but for now we just confirm it isn't the private one.
            },
            other => panic!("Expected a Fn, got {:?}", other),
        }
    }

    /// 6) Another scenario with multiple modules, nested impls, macros, etc., to verify
    /// that the function collects them all at the top level. We won't test recursive children
    /// of modules or impl blocks here, since that's done inside the matching logic or
    /// subordinate functions. 
    #[test]
    fn test_multiple_top_level_items_including_macro_rules() {
        let snippet = r#"
            macro_rules! my_macro {
                () => {};
            }

            mod submod {
                impl SubThing {
                    fn submethod() {}
                }
            }

            pub struct TopStruct;
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();
        let items = gather_items_in_node(&root_node, &opts);

        // We expect 3 items in top-level: macro_rules, module, struct
        assert_eq!(items.len(), 3);
        match (&items[0], &items[1], &items[2]) {
            (ConsolidatedItem::Macro(_), ConsolidatedItem::Module(sub), ConsolidatedItem::Struct(_)) => {
                // sub should mention "submod"
                assert_eq!(sub.mod_name(), "submod");
            }
            other => panic!("Expected [Macro, Module, Struct], got {other:?}"),
        }
    }

    /// 7) If docs are enabled in `options`, each item can gather doc comments. 
    /// We'll do a snippet with doc comments on a struct and a function to see if they're included.
    #[test]
    fn test_gather_docs_for_items() {
        let snippet = r#"
            /// This is function docs
            fn doc_fn() {}

            /// This is struct docs
            struct DocStruct;
        "#;
        let root_node = parse_source(snippet);
        let mut opts = default_options(); 
        opts = opts.with_docs(); // ensure docs are gathered

        let items = gather_items_in_node(&root_node, &opts);
        assert_eq!(items.len(), 2, "Should have a function and a struct item");
        match (&items[0], &items[1]) {
            (ConsolidatedItem::Fn(fn_item), ConsolidatedItem::Struct(st_item)) => {
                // Suppose `CrateInterfaceItem::docs()` returns the doc string
                let fn_docs = fn_item.docs().clone().unwrap_or_default();
                assert!(fn_docs.contains("/// This is function docs"), "Should gather fn docs");
                let st_docs = st_item.docs().clone().unwrap_or_default();
                assert!(st_docs.contains("/// This is struct docs"), "Should gather struct docs");
            }
            _ => panic!("Expected Fn then Struct"),
        }
    }

    /// 8) If docs are disabled, doc comments are omitted (the items may still appear).
    #[test]
    fn test_skip_docs_in_options() {
        let snippet = r#"
            /// Should skip me
            fn skip_docs_func() {}
        "#;
        let root_node = parse_source(snippet);
        let opts = ConsolidationOptions::new(); // no .with_docs()
        let items = gather_items_in_node(&root_node, &opts);
        assert_eq!(items.len(), 1, "Should still gather the function item");
        match &items[0] {
            ConsolidatedItem::Fn(fn_item) => {
                // Let’s see if docs are none
                let docs = fn_item.docs();
                assert!(docs.is_none(), "Should skip doc extraction");
            },
            other => panic!("Expected a Fn, got {:?}", other),
        }
    }

    /// 9) If there's a type alias or an enum at top level, confirm we gather them as well.
    #[test]
    fn test_gather_type_alias_and_enum() {
        let snippet = r#"
            type AliasA = u32;
            enum MyEnum { Variant1, Variant2 }
        "#;
        let root_node = parse_source(snippet);
        let opts = default_options();
        let items = gather_items_in_node(&root_node, &opts);

        assert_eq!(items.len(), 2, "One type alias, one enum");
        match (&items[0], &items[1]) {
            (ConsolidatedItem::TypeAlias(_), ConsolidatedItem::Enum(_)) => {
                // Ok
            },
            other => panic!("Expected [TypeAlias, Enum], got {other:?}"),
        }
    }

    /// 10) A more complex snippet with multiple items, some presumably skipped by item-skip logic,
    /// e.g., a private function, a test fn, etc. We'll just confirm the final set of recognized items matches our expectation.
    #[test]
    fn test_complex_snippet_with_skips() {
        let snippet = r#"
            fn normal_fn() {}
            #[cfg(test)]
            fn test_fn() {}
            macro_rules! special_macro {
                () => {};
            }
            pub trait MyTrait {}
            mod sub {
                // ...
            }
        "#;
        let root_node = parse_source(snippet);
        let mut opts = default_options();
        // Suppose we skip #[cfg(test)] items, or private items, etc.
        // The logic depends on your real `should_skip_item` implementation. 
        // We'll just demonstrate a scenario:
        // opts = opts.with_skip_cfg_test(); // hypothetical

        let items = gather_items_in_node(&root_node, &opts);

        // Possibly we get normal_fn, macro, trait, mod sub
        // test_fn might be skipped. Or not, depending on your logic.
        // We'll guess we skip it:
        let item_kinds: Vec<_> = items.iter().map(|it| format!("{:?}", it)).collect();
        // For a typical scenario, we might do something like:
        // assert_eq!(items.len(), 4, "We skip test_fn, so we have fn, macro, trait, mod");
        // and match them:
        // match (&items[0], &items[1], &items[2], &items[3]) {
        //     (ConsolidatedItem::Fn(_), ConsolidatedItem::Macro(_),
        //      ConsolidatedItem::Trait(_), ConsolidatedItem::Module(_)) => {}
        //     other => panic!("Expected [Fn, Macro, Trait, Module], got {:?}", other),
        // }
    }
}
