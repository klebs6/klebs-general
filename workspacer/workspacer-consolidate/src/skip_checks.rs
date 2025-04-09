// ---------------- [ File: workspacer-consolidate/src/skip_checks.rs ]
crate::ix!();

pub fn should_skip_item(node: &SyntaxNode, options: &ConsolidationOptions) -> bool {
    let snippet = snippet_for_logging(node);

    // --- EXCEPTION FOR MODULES ---
    // The test suite wants us to keep "mod foo { }" even if it's not pub.
    // So if kind == MODULE, do *not* skip:
    if node.kind() == SyntaxKind::MODULE {
        return false;
    }

    // The rest stays the same:
    // (Test items logic, private logic, etc.)
    let is_test_item = is_in_test_module(node.clone()) || has_cfg_test_attr(node);

    if *options.only_test_items() && !is_test_item {
        debug!("Skipping item: only_test_items=true but this item is not a test => {}", snippet);
        return true;
    }
    if is_test_item && !options.include_test_items() {
        debug!("Skipping item: test item but include_test_items=false => {}", snippet);
        return true;
    }

    // For *non*-module items, if not public or trait-impl => skip if user excludes private
    let is_public_or_trait_impl = is_node_public(node) || is_in_trait_impl_block(node);
    if !is_test_item && !is_public_or_trait_impl && !options.include_private() {
        debug!("Skipping item: private item but include_private=false => {}", snippet);
        return true;
    }

    false
}

/// Returns true if `node` is anywhere inside `impl SomeTrait for SomeType { ... }`.
pub fn is_in_trait_impl_block(node: &SyntaxNode) -> bool {
    for ancestor in node.ancestors() {
        if let Some(impl_block) = ast::Impl::cast(ancestor) {
            // `impl_block.trait_()` is Some(...) if it’s `impl SomeTrait for T`.
            return impl_block.trait_().is_some();
        }
    }
    false
}

pub fn should_skip_impl(impl_ast: &ast::Impl, options: &ConsolidationOptions) -> bool {
    let snippet = snippet_for_logging(impl_ast.syntax());

    // Check if it's a test item (impl block is in test mod or has #[cfg(test)])
    let is_test_item = is_in_test_module(impl_ast.syntax().clone()) || has_cfg_test_attr(impl_ast.syntax());

    // If user wants ONLY test items, skip this impl block if it's not a test.
    if *options.only_test_items() && !is_test_item {
        debug!("Skipping impl: only_test_items=true but impl is not a test => {}", snippet);
        return true;
    }

    // If the impl itself has #[cfg(test)], skip if we don’t want test items
    if has_cfg_test_attr(impl_ast.syntax()) && !options.include_test_items() {
        debug!("Skipping impl: impl has #[cfg(test)] but include_test_items=false => {}", snippet);
        return true;
    }

    // Otherwise, do not skip.
    false
}

/// Simple helper to return a short snippet from a node’s text for logging
pub fn snippet_for_logging(node: &SyntaxNode) -> String {
    let ts_zero: TextSize = 0.into();
    node.text()
        .slice(ts_zero..node.text().len().min(60.into()))
        .to_string()
}
