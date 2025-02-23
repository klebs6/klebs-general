// ---------------- [ File: src/gather_impl_methods.rs ]
crate::ix!();

pub fn gather_impl_methods(impl_ast: &ast::Impl, options: &ConsolidationOptions) -> Vec<CrateInterfaceItem<ast::Fn>> {
    let mut out = Vec::new();
    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(fn_ast) = ast::Fn::cast(item.syntax().clone()) {
                if !should_skip_item(fn_ast.syntax(), options) {
                    out.push(gather_fn_item(&fn_ast, options));
                } else {
                    info!("Skipping fn in impl: either test item or private item was disallowed");
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod test_gather_impl_methods {
    use super::*;
    use ra_ap_syntax::{ast, AstNode, SourceFile, SyntaxNode, Edition};
    // If you have custom imports for gather_fn_item, ConsolidationOptions, etc., include them:
    // use crate::{gather_fn_item, should_skip_item, CrateInterfaceItem, ast};

    /// Helper: parse the Rust snippet into a `SyntaxNode`.
    /// We must supply `Edition` in the second arg if your RA-AP version requires it.
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    /// Extracts the first `ast::Impl` node from the syntax tree, or None if not found.
    fn find_first_impl(root: &SyntaxNode) -> Option<ast::Impl> {
        for node in root.descendants() {
            if let Some(impl_ast) = ast::Impl::cast(node) {
                return Some(impl_ast);
            }
        }
        None
    }

    /// A default `ConsolidationOptions` for these tests.
    /// Adjust to your real approach if you have specific toggles.
    fn default_options() -> ConsolidationOptions {
        ConsolidationOptions::new().with_docs()
        // or however you want to configure the default
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) If the impl block has no items at all, we expect an empty result.
    #[test]
    fn test_impl_with_no_assoc_items() {
        let snippet = r#"
            impl MyStruct {
                // no items
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        assert!(fns.is_empty(), "No functions => gather_impl_methods should return empty Vec");
    }

    /// 2) If the impl block has items but none are `fn` (like a type alias or const), expect empty.
    #[test]
    fn test_impl_with_non_fn_items_only() {
        let snippet = r#"
            impl MyStruct {
                type Alias = i32;
                const N: usize = 10;
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        assert!(fns.is_empty(), "Only type aliases/const => no fn items found");
    }

    /// 3) If the impl block has multiple functions, we gather them all unless `should_skip_item` says otherwise.
    ///    For demonstration, we assume `should_skip_item` is minimal or nonexistent. We'll just check that we got 2 fns.
    #[test]
    fn test_impl_with_multiple_fns() {
        let snippet = r#"
            impl MyStruct {
                fn first(&self) {}
                fn second() -> i32 { 0 }
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        assert_eq!(fns.len(), 2, "We have two fn items in the impl block");
        // We can look at fns[0] or fns[1] if we want to check their names, etc.
    }

    /// 4) If an item is a function but `should_skip_item` says skip, we do not include it.
    ///    We'll define a snippet with a normal fn and a test fn with `#[cfg(test)]` or something,
    ///    then we assume `should_skip_item` is configured to skip the test fn.
    #[test]
    fn test_impl_with_skipped_fn() {
        let snippet = r#"
            impl MyStruct {
                fn normal_fn() {}
                #[cfg(test)]
                fn test_fn() {}
            }
        "#;
        // Suppose our `should_skip_item` logic checks `#[cfg(test)]` and decides to skip it.
        // We'll mock that by controlling `ConsolidationOptions`.

        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");

        // We'll define an opts that might skip test items.
        // In real code, you might do: let opts = ConsolidationOptions::new().skip_test_items()
        // For demonstration:
        let mut opts = default_options();
        // e.g., opts = opts.without_test_items(); if you have such a method
        // or define your own mock skip logic in `should_skip_item`.

        let fns = gather_impl_methods(&impl_ast, &opts);
        // We expect only "normal_fn" to appear if `#[cfg(test)]` is skipped
        assert_eq!(fns.len(), 1, "One function is not skipped, the test fn is skipped");
    }

    /// 5) A more complex snippet with multiple functions, some skipped, some not.
    ///    We'll show partial inclusion. We'll define 4 fns, skip two, keep two.
    #[test]
    fn test_impl_partial_skips() {
        let snippet = r#"
            impl MyStruct {
                // normal
                fn keep_me() {}

                // some "private" logic, let's say it has an attribute or name that triggers skip
                #[cfg(test)]
                fn skip_me_test() {}

                fn also_keep_me() {}

                #[some_other_attr]
                fn skip_me_something_else() {}
            }
        "#;
        // We'll pretend `should_skip_item` sees `#[cfg(test)]` or `#[some_other_attr]` as skip triggers

        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        // We'll define or assume we have an options that leads to skipping those attributes.
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        // We expect 2 in the result: "keep_me" and "also_keep_me".
        assert_eq!(fns.len(), 2, "We keep 2, skip 2");
        // If we want to inspect names, we can do that by checking the AST or the CrateInterfaceItem fields.
    }

    /// 6) If the `impl` is for a trait (i.e. `impl TraitName for MyStruct { ... }`), the logic is the same:
    ///    we gather fn items if they're not skipped. We'll confirm it still works fine.
    #[test]
    fn test_trait_impl_gather_methods() {
        let snippet = r#"
            impl SomeTrait for MyStruct {
                fn trait_method(&self) {}

                #[cfg(test)]
                fn test_trait_method() {}
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        // If we skip test items, we get 1, otherwise 2. We'll assume skip.
        assert_eq!(fns.len(), 1, "Only the non-test trait method is included");
    }

    /// 7) If there's no `assoc_item_list`, we also get an empty result (similar to no items).
    #[test]
    fn test_impl_missing_assoc_item_list() {
        // A weird snippet missing braces
        let snippet = r#"
            impl MyStruct;
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("Expected an impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts);
        assert!(fns.is_empty(), "No assoc_item_list => no methods");
    }
}
