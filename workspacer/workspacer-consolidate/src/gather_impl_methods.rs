// ---------------- [ File: workspacer-consolidate/src/gather_impl_methods.rs ]
crate::ix!();

/// Gathers all `fn` items from within an `impl` block, excluding any that
/// `should_skip_item_fn` determines should be skipped (e.g., private methods,
/// test methods if not `.include_test_items()`, or with `#[some_other_attr]`, etc.).
pub fn gather_impl_methods(
    impl_ast: &ast::Impl,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Vec<CrateInterfaceItem<ast::Fn>> {
    let mut out = Vec::new();

    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(fn_ast) = ast::Fn::cast(item.syntax().clone()) {
                // We'll pass the `ast::Fn` object to a skip-check specialized for functions.
                if !should_skip_item_fn(&fn_ast, options) {
                    out.push(gather_fn_item(&fn_ast, options,file_path,crate_path));
                } else {
                    debug!(
                        "Skipping fn in impl: private/test/other => {:?}",
                        fn_ast.syntax().text().to_string()
                    );
                }
            }
        }
    }

    out
}

#[tracing::instrument(level="trace", skip_all)]
fn should_skip_item_fn(fn_ast: &ast::Fn, options: &ConsolidationOptions) -> bool {
    let node = fn_ast.syntax().clone();
    let snippet = snippet_for_logging(&node);
    trace!("Entering should_skip_item_fn; snippet = {:?}", snippet);

    // 1) If we're only collecting test items, skip any non-test function.
    //    Or if it *is* a test fn but `include_test_items` is false, skip it.
    let is_test_item = is_in_test_module(node.clone()) || has_cfg_test_attr(&node);
    if *options.only_test_items() && !is_test_item {
        trace!("Skipping fn because only_test_items=true but this fn isn't a test item: {:?}", snippet);
        return true;
    }
    if is_test_item && !options.include_test_items() {
        trace!("Skipping fn because it's a test item but include_test_items=false: {:?}", snippet);
        return true;
    }

    // 2) Skip if it has an explicit `#[some_other_attr]` that we don't want.
    for attr in fn_ast.attrs() {
        let raw = attr.syntax().text().to_string();
        if raw.contains("#[some_other_attr]") {
            trace!("Skipping fn because it has #[some_other_attr]: {:?}", snippet);
            return true;
        }
    }

    // 3) If it’s private (no `pub`) and not inside a trait impl, we skip it
    //    unless `.with_private_items()` is set.
    if !options.include_private() {
        if !is_in_trait_impl_block(&node) {
            // In a normal (inherent) impl or free function scenario,
            // we require `pub` unless user asked for private items.
            if fn_ast.visibility().is_none() {
                trace!("Skipping fn because it's private and user didn't ask for private items: {:?}", snippet);
                return true;
            }
        }
    }

    trace!("Not skipping fn: {:?}", snippet);
    false
}

// The rest of your import boilerplate & test suite below...
#[cfg(test)]
mod test_gather_impl_methods {
    use super::*;

    // Minimally define or import ConsolidationOptions, CrateInterfaceItem, etc.
    // e.g.:
    // use crate::{ConsolidationOptions, CrateInterfaceItem, ...};

    // Helper to parse a snippet
    fn parse_source(snippet: &str) -> SyntaxNode {
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        parse.tree().syntax().clone()
    }

    // Find first impl
    fn find_first_impl(root: &SyntaxNode) -> Option<ast::Impl> {
        for node in root.descendants() {
            if let Some(impl_ast) = ast::Impl::cast(node) {
                return Some(impl_ast);
            }
        }
        None
    }

    // e.g. default test options
    fn default_options() -> ConsolidationOptions {
        // includes private, not test => skip test fns, skip some_other_attr
        ConsolidationOptions::new()
            .with_private_items()
            // .include_test_items(false)
            .with_docs()
    }

    // ------------------------------------------------------------------------
    //  Tests
    // ------------------------------------------------------------------------

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

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert!(fns.is_empty());
    }

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

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert!(fns.is_empty());
    }

    #[test]
    fn test_impl_with_multiple_fns() {
        let snippet = r#"
            impl MyStruct {
                fn first(&self) {}
                fn second() -> i32 { 0 }
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert_eq!(fns.len(), 2, "We expect 2 normal fns");
    }

    #[test]
    fn test_impl_with_skipped_fn() {
        let snippet = r#"
            impl MyStruct {
                fn normal_fn() {}
                #[cfg(test)]
                fn test_fn() {}
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("impl block");
        // We skip test => so we only keep normal_fn
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert_eq!(fns.len(), 1, "Should skip test_fn");
    }

    #[test]
    fn test_impl_partial_skips() {
        let snippet = r#"
            impl MyStruct {
                fn keep_me() {}

                #[cfg(test)]
                fn skip_me_test() {}

                fn also_keep_me() {}

                #[some_other_attr]
                fn skip_me_something_else() {}
            }
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("impl block");
        let opts = default_options(); // skip test, skip #some_other_attr, keep private

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());

        // We expect keep_me & also_keep_me => 2
        // skip_me_test => has #[cfg(test)], skip
        // skip_me_something_else => has #[some_other_attr], skip
        assert_eq!(fns.len(), 2, "We keep 2, skip 2");
    }

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
        let impl_ast = find_first_impl(&root).expect("impl block");

        let opts = default_options(); // skip test => keep only trait_method
        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert_eq!(fns.len(), 1);
    }

    #[test]
    fn test_impl_missing_assoc_item_list() {
        let snippet = r#"
            impl MyStruct;
        "#;
        let root = parse_source(snippet);
        let impl_ast = find_first_impl(&root).expect("impl block");
        let opts = default_options();

        let fns = gather_impl_methods(&impl_ast, &opts, file_path.clone(), crate_path.clone());
        assert!(fns.is_empty());
    }
}
