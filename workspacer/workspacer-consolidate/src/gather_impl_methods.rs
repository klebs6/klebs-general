crate::ix!();

/// Gathers associated methods from an impl, skipping test/private if needed.
pub fn gather_impl_methods(
    impl_ast: &ast::Impl,
    options:  &ConsolidationOptions,
) -> Vec<CrateInterfaceItem<ast::Fn>> {
    let mut out = vec![];
    if let Some(assoc_list) = impl_ast.assoc_item_list() {
        for assoc_item in assoc_list.assoc_items() {
            if let Some(fn_ast) = ast::Fn::cast(assoc_item.syntax().clone()) {
                if !options.include_test_items() && has_cfg_test_attr(fn_ast.syntax()) {
                    continue;
                }
                if !options.include_private() && !is_node_public(fn_ast.syntax()) {
                    continue;
                }
                out.push(gather_fn_item(&fn_ast, options));
            }
        }
    }
    out
}
