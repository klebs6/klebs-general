// ---------------- [ File: workspacer-consolidate/src/skip_checks.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip(node, options))]
pub fn should_skip_item(node: &SyntaxNode, options: &ConsolidationOptions) -> bool {
    trace!("Entered should_skip_item for snippet={}", snippet_for_logging(node));

    // 0) If this is a `mod` item, we currently do NOT enforce "pub" vs. "private" at all.
    //    Many user tests rely on collecting all modules by default, even if they're private.
    //    So we skip the normal private checks if `node` is SyntaxKind::MODULE.
    if node.kind() == SyntaxKind::MODULE {
        trace!("Item is a module => we do NOT skip it for being private, continuing with test checks only.");
        // Check test conditions only:
        let is_test_item = is_in_test_module(node.clone()) || has_cfg_test_attr(node);

        // If the user wants only test items, skip if it's NOT a test item:
        if *options.only_test_items() && !is_test_item {
            debug!(
                "Skipping module: only_test_items=true but not a test item => snippet={}",
                snippet_for_logging(node)
            );
            return true;
        }
        // If it IS a test item but user isn't including test items => skip
        if is_test_item && !options.include_test_items() {
            debug!(
                "Skipping module: it is a test module but include_test_items=false => snippet={}",
                snippet_for_logging(node)
            );
            return true;
        }
        // Otherwise, do not skip modules:
        trace!("Not skipping module => snippet={}", snippet_for_logging(node));
        return false;
    }

    // 1) Determine if this is a test item (#[cfg(test)] / #[test] / or in a test module)
    let is_test_item = is_in_test_module(node.clone()) || has_cfg_test_attr(node);

    // 2) If user wants *only* test items, skip all non-test
    if *options.only_test_items() && !is_test_item {
        debug!(
            "Skipping item: only_test_items=true but item is not a test item => snippet={}",
            snippet_for_logging(node)
        );
        return true;
    }

    // 3) If this *is* a test item, skip it if user isn't including test items
    if is_test_item && !options.include_test_items() {
        debug!(
            "Skipping item: it is a test item but include_test_items=false => snippet={}",
            snippet_for_logging(node)
        );
        return true;
    }

    // 4) If it's not a test item, we check if it's private in a non-trait-impl context
    //    and user did NOT ask for private => skip
    if !is_test_item && !is_node_public(node) && !is_in_trait_impl_block(node) && !options.include_private() {
        debug!(
            "Skipping item: private item but user did not ask for private => snippet={}",
            snippet_for_logging(node)
        );
        return true;
    }

    // Otherwise, do not skip
    trace!("Not skipping item => snippet={}", snippet_for_logging(node));
    false
}

#[tracing::instrument(level="trace", skip(impl_ast, options))]
pub fn should_skip_impl(impl_ast: &ast::Impl, options: &ConsolidationOptions) -> bool {
    let snippet = snippet_for_logging(impl_ast.syntax());
    trace!("Entered should_skip_impl for snippet: {}", snippet);

    let is_test_item = is_in_test_module(impl_ast.syntax().clone()) || has_cfg_test_attr(impl_ast.syntax());
    if *options.only_test_items() && !is_test_item {
        debug!(
            "Skipping impl: only_test_items=true but impl is not a test => snippet={}",
            snippet
        );
        return true;
    }

    if is_test_item && !options.include_test_items() {
        debug!(
            "Skipping impl: impl has test attribute but include_test_items=false => snippet={}",
            snippet
        );
        return true;
    }

    trace!("Not skipping impl => snippet={}", snippet);
    false
}

/// Returns true if `node` is anywhere inside `impl SomeTrait for SomeType { ... }`.
pub fn is_in_trait_impl_block(node: &SyntaxNode) -> bool {
    for ancestor in node.ancestors() {
        if let Some(impl_block) = ast::Impl::cast(ancestor) {
            if impl_block.trait_().is_some() {
                return true;
            }
        }
    }
    false
}

/// Simple helper to return a short snippet from a node’s text for logging
pub fn snippet_for_logging(node: &SyntaxNode) -> String {
    let max_len = 60;
    let full_text = node.text().to_string();
    if full_text.len() <= max_len {
        full_text
    } else {
        let mut end = max_len;
        while !full_text.is_char_boundary(end) {
            end -= 1;
        }
        let mut s = full_text[..end].to_owned();
        s.push_str("...");
        s
    }
}

#[cfg(test)]
mod test_should_skip_item_exhaustive {

    use super::*;

    /// Parse the provided Rust snippet into a `SyntaxNode`. We choose the first item or block we find
    /// to test. We rely on `should_skip_item` to do the real logic.
    fn parse_first_item_node(snippet: &str) -> SyntaxNode {
        trace!("Parsing snippet:\n{}", snippet);
        let parsed = SourceFile::parse(snippet, Edition::Edition2021);
        let file_syntax = parsed.tree().syntax().clone();

        // We attempt to find the *first* child item node. This is just for demonstration;
        // your real usage might need more robust searching or different indexing.
        for child in file_syntax.descendants() {
            match child.kind() {
                SyntaxKind::FN
                | SyntaxKind::STRUCT
                | SyntaxKind::ENUM
                | SyntaxKind::TRAIT
                | SyntaxKind::MODULE
                | SyntaxKind::TYPE_ALIAS
                | SyntaxKind::MACRO_RULES
                | SyntaxKind::IMPL => {
                    debug!("Found first item node: kind={:?}", child.kind());
                    return child;
                }
                _ => {}
            }
        }

        // If none found, we just return the root
        debug!("No recognized item node found; returning the entire file syntax node");
        file_syntax
    }

    /// Helper to call `should_skip_item` with the provided snippet and options,
    /// and then return the boolean result plus a short debug string for logging.
    fn check_skip(snippet: &str, options: &ConsolidationOptions) -> bool {
        trace!("check_skip => snippet:\n{}", snippet);
        let node = parse_first_item_node(snippet);
        let skip = should_skip_item(&node, options);
        debug!(
            "should_skip_item => skip={} for snippet:\n{:?}",
            skip,
            snippet_for_logging(&node)
        );
        skip
    }

    /// Demonstrates multiple scenarios of test vs. non-test, private vs. public, plus
    /// user toggles: only_test_items, include_test_items, include_private.
    #[traced_test]
    fn test_scenario_basic_public_non_test_no_flags() {
        info!("Starting test_scenario_basic_public_non_test_no_flags");
        let snippet = r#"
            pub fn hello_world() {}
        "#;

        let opts = ConsolidationOptions::new(); // all toggles off
        let result = check_skip(snippet, &opts);
        // The function is public, not a test item. We skip private items by default,
        // but it's public => we should NOT skip it. We also skip test items by default,
        // but it's not a test => no skip. So final => skip=false.
        assert_eq!(result, false, "A public non-test item should not be skipped with all defaults");
    }

    #[traced_test]
    fn test_scenario_private_fn_no_private_flag() {
        info!("Starting test_scenario_private_fn_no_private_flag");
        let snippet = r#"
            fn private_thing() {}
        "#;
        let opts = ConsolidationOptions::new(); 
        // private => not a test item => we do *not* have include_private => skip=true
        let result = check_skip(snippet, &opts);
        assert_eq!(result, true, "A private fn is skipped if we do not set include_private");
    }

    #[traced_test]
    fn test_scenario_private_fn_with_private_flag() {
        info!("Starting test_scenario_private_fn_with_private_flag");
        let snippet = r#"
            fn private_thing() {}
        "#;
        let opts = ConsolidationOptions::new().with_private_items();
        // now we do have include_private => skip=false
        let result = check_skip(snippet, &opts);
        assert_eq!(result, false, "A private fn is included if we set include_private=true");
    }

    #[traced_test]
    fn test_scenario_public_cfg_test_no_include_tests() {
        info!("Starting test_scenario_public_cfg_test_no_include_tests");
        let snippet = r#"
            #[cfg(test)]
            pub fn test_fn() {}
        "#;
        // The function is marked test, but we are not setting include_test_items => skip
        let opts = ConsolidationOptions::new(); 
        let result = check_skip(snippet, &opts);
        assert_eq!(result, true, "A test item is skipped if include_test_items=false");
    }

    #[traced_test]
    fn test_scenario_public_cfg_test_with_include_tests() {
        info!("Starting test_scenario_public_cfg_test_with_include_tests");
        let snippet = r#"
            #[cfg(test)]
            pub fn test_fn() {}
        "#;
        // The function is a test item; we set include_test_items => skip=false
        let opts = ConsolidationOptions::new().with_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(result, false, "A test item is included if include_test_items=true");
    }

    #[traced_test]
    fn test_scenario_private_cfg_test_no_private_flag_but_includes_tests() {
        info!("Starting test_scenario_private_cfg_test_no_private_flag_but_includes_tests");
        let snippet = r#"
            #[cfg(test)]
            fn private_test_fn() {}
        "#;
        // It's private, but also a test item. We do not set include_private, but we *do* set with_test_items.
        // In the adjusted logic, we do not skip test items for lacking pub. => skip=false
        let opts = ConsolidationOptions::new().with_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(
            result,
            false,
            "We do not skip a private test item if include_test_items=true, even though not include_private"
        );
    }

    #[traced_test]
    fn test_scenario_private_cfg_test_only_test_items() {
        info!("Starting test_scenario_private_cfg_test_only_test_items");
        let snippet = r#"
            #[cfg(test)]
            fn private_test_fn() {}
        "#;
        // now with_only_test_items => implies include_test_items => skip all non-test, but keep test items
        let opts = ConsolidationOptions::new().with_only_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(
            result, 
            false, 
            "A private test item is kept if with_only_test_items() is set (since test items pass, ignoring pub)."
        );
    }

    #[traced_test]
    fn test_scenario_private_non_test_with_only_test_items() {
        info!("Starting test_scenario_private_non_test_with_only_test_items");
        let snippet = r#"
            fn private_fn_normal() {}
        "#;
        // with_only_test_items => we skip all non-test, so skip => true
        let opts = ConsolidationOptions::new().with_only_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(
            result, 
            true, 
            "A private non-test item is skipped if with_only_test_items is set"
        );
    }

    #[traced_test]
    fn test_scenario_public_non_test_with_only_test_items() {
        info!("Starting test_scenario_public_non_test_with_only_test_items");
        let snippet = r#"
            pub fn normal_public_fn() {}
        "#;
        // It's public but not a test. with_only_test_items => skip => true
        let opts = ConsolidationOptions::new().with_only_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(result, true, "We skip all non-test items if only_test_items=true");
    }

    #[traced_test]
    fn test_scenario_test_attr_without_cfg_test() {
        info!("Starting test_scenario_test_attr_without_cfg_test");
        let snippet = r#"
            #[test]
            fn normal_test_attr() {}
        "#;
        // This is also recognized as a test item because has_cfg_test_attr checks for #[test]. 
        // With defaults => skip
        let opts = ConsolidationOptions::new(); 
        let result = check_skip(snippet, &opts);
        assert_eq!(result, true, "A #[test] item is recognized as test => skip if not include_test_items");
    }

    #[traced_test]
    fn test_scenario_test_attr_included() {
        info!("Starting test_scenario_test_attr_included");
        let snippet = r#"
            #[test]
            fn normal_test_attr() {}
        "#;
        let opts = ConsolidationOptions::new().with_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(result, false, "A #[test] item is included if with_test_items=true");
    }

    #[traced_test]
    fn test_scenario_mod_with_cfg_test() {
        info!("Starting test_scenario_mod_with_cfg_test");
        let snippet = r#"
            #[cfg(test)]
            mod test_mod {
                fn inside() {}
            }
        "#;
        // This mod is recognized as a test item => skip if we don't do include_test_items
        let opts = ConsolidationOptions::new();
        let result = check_skip(snippet, &opts);
        assert_eq!(result, true, "Module with #[cfg(test)] is treated as test => skip if no test items");
    }

    #[traced_test]
    fn test_scenario_mod_with_cfg_test_included() {
        info!("Starting test_scenario_mod_with_cfg_test_included");
        let snippet = r#"
            #[cfg(test)]
            mod test_mod {
                fn inside() {}
            }
        "#;
        let opts = ConsolidationOptions::new().with_test_items();
        let result = check_skip(snippet, &opts);
        assert_eq!(result, false, "Module with #[cfg(test)] is included if we set include_test_items=true");
    }

    #[traced_test]
    fn test_scenario_trait_impl_block_not_test() {
        info!("Starting test_scenario_trait_impl_block_not_test");
        let snippet = r#"
            impl SomeTrait for MyType {
                fn method(&self) {}
            }
        "#;
        // If it's not test, not private (impl blocks in trait are considered "public" in many sense?), so we keep it
        let opts = ConsolidationOptions::new();
        let node = parse_first_item_node(snippet);
        let skip = should_skip_impl(&ast::Impl::cast(node).unwrap(), &opts);
        assert_eq!(skip, false, "A normal trait impl is not skipped if we have no reason to skip it");
    }

    #[traced_test]
    fn test_scenario_impl_block_with_cfg_test() {
        info!("Starting test_scenario_impl_block_with_cfg_test");
        let snippet = r#"
            #[cfg(test)]
            impl MyStruct {
                fn do_stuff(&self) {}
            }
        "#;
        // This is recognized as a test item => skip if not include_test_items
        let opts = ConsolidationOptions::new();
        let node = parse_first_item_node(snippet);
        let skip = should_skip_impl(&ast::Impl::cast(node).unwrap(), &opts);
        assert_eq!(
            skip, 
            true, 
            "Impl with #[cfg(test)] is recognized as test => skip if no with_test_items"
        );
    }

    #[traced_test]
    fn test_scenario_impl_block_included_tests() {
        info!("Starting test_scenario_impl_block_included_tests");
        let snippet = r#"
            #[cfg(test)]
            impl MyStruct {
                fn do_stuff(&self) {}
            }
        "#;
        let opts = ConsolidationOptions::new().with_test_items();
        let node = parse_first_item_node(snippet);
        let skip = should_skip_impl(&ast::Impl::cast(node).unwrap(), &opts);
        assert_eq!(
            skip, 
            false, 
            "Impl with #[cfg(test)] is included if we set include_test_items=true"
        );
    }
}
