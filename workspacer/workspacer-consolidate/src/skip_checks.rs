// ---------------- [ File: src/skip_checks.rs ]
crate::ix!();


/// Decide whether to skip a top-level item (fn/struct/enum/trait/typeAlias/macro/module).
///
/// Logic summary:
///  - If it’s in a test context (or has `#[cfg(test)]`) and `include_test_items=false` => skip
///  - Otherwise, if it’s either public or inside a trait-impl block, we treat it as “visible.”
///  - If it’s not visible and `include_private=false` => skip
///  - Otherwise => keep
pub fn should_skip_item(node: &SyntaxNode, options: &ConsolidationOptions) -> bool {
    let snippet = snippet_for_logging(node);

    // Is it a test item? (mod #[cfg(test)] or has #[cfg(test)] itself)
    let is_test_item = is_in_test_module(node.clone()) || has_cfg_test_attr(node);

    // If it’s a test item but we do not want test items => skip.
    if is_test_item && !options.include_test_items() {
        info!("Skipping item: test item but include_test_items=false => {}", snippet);
        return true;
    }

    // If the node is public or is inside a trait impl, treat it as “visible.”
    let is_public_or_trait_impl = is_node_public(node) || is_in_trait_impl_block(node);

    // If it’s not test-item, not visible, and user excludes private => skip
    if !is_test_item && !is_public_or_trait_impl && !options.include_private() {
        info!("Skipping item: private item but include_private=false => {}", snippet);
        return true;
    }

    // Otherwise, do not skip.
    false
}

/// Returns true if `node` is anywhere inside `impl SomeTrait for SomeType { ... }`.
fn is_in_trait_impl_block(node: &SyntaxNode) -> bool {
    for ancestor in node.ancestors() {
        if let Some(impl_block) = ast::Impl::cast(ancestor) {
            // `impl_block.trait_()` is Some(...) if it’s `impl SomeTrait for T`.
            return impl_block.trait_().is_some();
        }
    }
    false
}

/// Decide whether to skip an entire impl block. We gather methods + type aliases
/// first, removing them via `should_skip_item` logic, then:
/// 
/// - If the impl itself is `#[cfg(test)]` (and include_test_items=false), skip immediately.
/// - Otherwise we keep the impl block but only with the included items. If *zero* items remain,
///   we still show the block in a single line, e.g. `impl Trait for Type {}` 
///   (unless you prefer to skip empty blocks entirely — see the toggle below).
pub fn should_skip_impl(impl_ast: &ast::Impl, options: &ConsolidationOptions) -> bool {
    let snippet = snippet_for_logging(impl_ast.syntax());

    trace!("should_skip_impl: snippet='{}...'", snippet);

    // If the impl itself has #[cfg(test)], skip if we don’t want test items
    if has_cfg_test_attr(impl_ast.syntax()) && !options.include_test_items() {
        info!("Skipping impl: impl has #[cfg(test)] but include_test_items=false");
        return true;
    }

    // We do *not* skip based on the self‐ty being private. 
    // (We gather items inside it to see if any are included.)

    false
}

/// Simple helper to return a short snippet from a node’s text for logging
fn snippet_for_logging(node: &SyntaxNode) -> String {
    let ts_zero: TextSize = 0.into();
    node.text()
        .slice(ts_zero..node.text().len().min(60.into()))
        .to_string()
}
