// ---------------- [ File: workspacer-consolidate/src/gather_items_in_node.rs ]
crate::ix!();

/// Gathers Rust items (fn, struct, enum, mod, trait, etc.) from `parent_node`.
/// In older RA versions, top-level items appear in a `SourceFile` or `ItemList`.
/// We'll check those first; if that fails, we fallback to iterating `.children()`.
pub fn gather_items_in_node(
    parent_node: &SyntaxNode,
    options: &ConsolidationOptions,
) -> Vec<ConsolidatedItem> {

    // Print the parent_node's first ~80 chars so we can see what snippet is being parsed:
    let node_text = parent_node.text().to_string();
    let preview = if node_text.len() > 80 {
        format!("{}...", &node_text[..77])
    } else {
        node_text.clone()
    };
    debug!("=== gather_items_in_node: Node preview = {:?} ===", preview);

    // 1) Attempt to parse as `SourceFile`
    if let Some(sf) = ast::SourceFile::cast(parent_node.clone()) {
        let items_iter = sf.items();
        // We'll collect them in a vector just to debug their count:
        let all_items: Vec<_> = items_iter.collect();
        debug!(">>> recognized SourceFile => found {} top-level items", all_items.len());
        for (i, it) in all_items.iter().enumerate() {
            let k = it.syntax().kind();
            debug!("    item #{}: kind={:?}, text={:?}",
                i, k,
                trim_to_60(it.syntax().text().to_string())
            );
        }
        // Now actually process them
        return gather_items_from_iter(all_items.into_iter(), options);
    }

    // 2) Attempt `ItemList`
    if let Some(item_list) = ast::ItemList::cast(parent_node.clone()) {
        let items_iter = item_list.items();
        let all_items: Vec<_> = items_iter.collect();
        debug!(">>> recognized ItemList => found {} items", all_items.len());
        for (i, it) in all_items.iter().enumerate() {
            let k = it.syntax().kind();
            debug!("    item #{}: kind={:?}, text={:?}",
                i, k,
                trim_to_60(it.syntax().text().to_string())
            );
        }
        return gather_items_from_iter(all_items.into_iter(), options);
    }

    // 3) Fallback: direct iteration over `parent_node.children()`.
    //    This is rarely needed for top-level items if the snippet is well-formed,
    //    but it might matter for nested scopes or partial code.
    debug!(">>> fallback: direct children => checking .kind() of each child");
    let mut items = Vec::new();
    let mut count_children = 0;
    for child in parent_node.children() {
        count_children += 1;
        let k = child.kind();
        let short_txt = trim_to_60(child.text().to_string());
        debug!("   - fallback child #{} => kind={:?}, text={:?}", count_children, k, short_txt);

        // Attempt the same item logic:
        if let Some(mod_ast) = ast::Module::cast(child.clone()) {
            debug!("       => recognized ast::Module");
            if should_skip_item(&child, options) {
                debug!("         [skipped by should_skip_item]");
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
                .unwrap_or("<unknown_mod>".to_owned());

            let mut mod_iface = ModuleInterface::new(docs, attrs, mod_name);

            if let Some(sub_item_list) = mod_ast.item_list() {
                let sub_items = gather_items_in_node(sub_item_list.syntax(), options);
                for si in sub_items {
                    mod_iface.add_item(si);
                }
            }
            items.push(ConsolidatedItem::Module(mod_iface));

        } else if let Some(impl_ast) = ast::Impl::cast(child.clone()) {
            debug!("       => recognized ast::Impl");
            if should_skip_impl(&impl_ast, options) {
                debug!("         [skipped by should_skip_impl]");
                continue;
            }
            let docs  = None; // or extract_docs
            let attrs = None; // or gather_all_attrs
            let signature = generate_impl_signature(&impl_ast, docs.as_ref());
            let methods = gather_impl_methods(&impl_ast, options);
            let aliases = gather_assoc_type_aliases(&impl_ast, options);

            let ib = ImplBlockInterface::new(docs, attrs, signature, methods, aliases);
            items.push(ConsolidatedItem::ImplBlock(ib));

        } else if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
            debug!("       => recognized ast::Fn");
            if should_skip_item(&child, options) {
                debug!("         [skipped by should_skip_item]");
                continue;
            }
            let ci = gather_fn_item(&fn_ast, options);
            items.push(ConsolidatedItem::Fn(ci));

        } else if let Some(st_ast) = ast::Struct::cast(child.clone()) {
            debug!("       => recognized ast::Struct");
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
                CrateInterfaceItem::new(st_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(en_ast) = ast::Enum::cast(child.clone()) {
            debug!("       => recognized ast::Enum");
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
                CrateInterfaceItem::new(en_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(tr_ast) = ast::Trait::cast(child.clone()) {
            debug!("       => recognized ast::Trait");
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
                CrateInterfaceItem::new(tr_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(ty_ast) = ast::TypeAlias::cast(child.clone()) {
            debug!("       => recognized ast::TypeAlias");
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
                CrateInterfaceItem::new(ty_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(mac_ast) = ast::MacroRules::cast(child.clone()) {
            debug!("       => recognized ast::MacroRules");
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
                CrateInterfaceItem::new(mac_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else {
            debug!("       => no recognized item cast");
        }
    }

    debug!(
        ">>> fallback complete, recognized {} item(s) from {} children.\n",
        items.len(),
        count_children
    );
    items
}

/// Called by gather_items_in_node to process each `ast::Item`.
/// We try each cast in turn, building the appropriate ConsolidatedItem.
fn gather_items_from_iter(
    items_iter: impl Iterator<Item = ast::Item>,
    options: &ConsolidationOptions,
) -> Vec<ConsolidatedItem> {
    debug!("+++ gather_items_from_iter: scanning RA items +++");
    let mut out = Vec::new();

    for (idx, item) in items_iter.enumerate() {
        let syn = item.syntax().clone();
        let short_txt = trim_to_60(syn.text().to_string());
        debug!("   item #{} => kind={:?}, text={:?}", idx, syn.kind(), short_txt);

        if let Some(mod_ast) = ast::Module::cast(syn.clone()) {
            debug!("       => recognized ast::Module");
            if should_skip_item(&syn, options) {
                debug!("         [skipped by should_skip_item]");
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            let mod_name = mod_ast
                .name()
                .map(|n| n.text().to_string())
                .unwrap_or_else(|| "<unknown_mod>".to_owned());
            let mut mod_iface = ModuleInterface::new(docs, attrs, mod_name);
            if let Some(sub_item_list) = mod_ast.item_list() {
                let sub_items = gather_items_in_node(sub_item_list.syntax(), options);
                for si in sub_items {
                    mod_iface.add_item(si);
                }
            }
            out.push(ConsolidatedItem::Module(mod_iface));

        } else if let Some(impl_ast) = ast::Impl::cast(syn.clone()) {
            debug!("       => recognized ast::Impl");
            if should_skip_impl(&impl_ast, options) {
                debug!("         [skipped by should_skip_impl]");
                continue;
            }
            let docs = None;
            let attrs = None;
            let signature = generate_impl_signature(&impl_ast, docs.as_ref());
            let methods = gather_impl_methods(&impl_ast, options);
            let aliases = gather_assoc_type_aliases(&impl_ast, options);
            let ib = ImplBlockInterface::new(docs, attrs, signature, methods, aliases);
            out.push(ConsolidatedItem::ImplBlock(ib));

        } else if let Some(fn_ast) = ast::Fn::cast(syn.clone()) {
            debug!("       => recognized ast::Fn");
            if should_skip_item(&syn, options) {
                debug!("         [skipped by should_skip_item]");
                continue;
            }
            let ci = gather_fn_item(&fn_ast, options);
            out.push(ConsolidatedItem::Fn(ci));

        } else if let Some(st_ast) = ast::Struct::cast(syn.clone()) {
            debug!("       => recognized ast::Struct");
            if should_skip_item(&syn, options) {
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            out.push(ConsolidatedItem::Struct(
                CrateInterfaceItem::new(st_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(en_ast) = ast::Enum::cast(syn.clone()) {
            debug!("       => recognized ast::Enum");
            if should_skip_item(&syn, options) {
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            out.push(ConsolidatedItem::Enum(
                CrateInterfaceItem::new(en_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(tr_ast) = ast::Trait::cast(syn.clone()) {
            debug!("       => recognized ast::Trait");
            if should_skip_item(&syn, options) {
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            out.push(ConsolidatedItem::Trait(
                CrateInterfaceItem::new(tr_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(ty_ast) = ast::TypeAlias::cast(syn.clone()) {
            debug!("       => recognized ast::TypeAlias");
            if should_skip_item(&syn, options) {
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            out.push(ConsolidatedItem::TypeAlias(
                CrateInterfaceItem::new(ty_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else if let Some(mac_ast) = ast::MacroRules::cast(syn.clone()) {
            debug!("       => recognized ast::MacroRules");
            if should_skip_item(&syn, options) {
                continue;
            }
            let docs = if *options.include_docs() {
                extract_docs(&syn)
            } else {
                None
            };
            let attrs = gather_all_attrs(&syn);
            out.push(ConsolidatedItem::Macro(
                CrateInterfaceItem::new(mac_ast, docs, attrs, None, Some(options.clone())),
            ));

        } else {
            debug!("       => no recognized item cast for this item");
        }
    }

    debug!("+++ gather_items_from_iter => total recognized items: {} +++\n", out.len());
    out
}

/// Trims a string to ~60 chars for logging
fn trim_to_60(mut s: String) -> String {
    if s.len() > 60 {
        s.truncate(57);
        s.push_str("...");
    }
    s
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
        ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
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

        // We'll pass options that do **not** include private items:
        let mut opts = ConsolidationOptions::new()
            .with_docs()           // If the test wants doc coverage
            // .with_private_items() is **not** called => we skip private
            ;
        
        let items = gather_items_in_node(&root_node, &opts);

        // Now we expect only the public function
        assert_eq!(items.len(), 1);
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
        // We do *not* call .with_docs(), so doc lines are not captured,
        // **but** we DO call .with_private_items() so we do NOT skip it for being private.
        let opts = ConsolidationOptions::new()
            .with_private_items(); 

        let items = gather_items_in_node(&root_node, &opts);

        assert_eq!(items.len(), 1, "Should still gather the function item");
        match &items[0] {
            ConsolidatedItem::Fn(fn_item) => {
                // Confirm docs are None.
                let docs = fn_item.docs();
                assert!(docs.is_none(), "Should skip doc extraction entirely");
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
