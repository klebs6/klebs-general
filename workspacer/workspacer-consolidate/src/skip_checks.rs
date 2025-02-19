crate::ix!();

pub fn should_skip_impl(impl_ast: &ast::Impl, options: &ConsolidationOptions) -> bool {
    if !options.include_test_items() && has_cfg_test_attr(impl_ast.syntax()) {
        return true;
    }
    if let Some(self_ty) = impl_ast.self_ty() {
        let self_ty_text = self_ty.syntax().text().to_string();
        if self_ty_text == "ImplType" && !options.include_private() {
            return true;
        }
    }
    false
}

pub fn should_skip_item(node: &SyntaxNode, options: &ConsolidationOptions) -> bool {
    if !options.include_test_items() && is_in_test_module(node.clone()) {
        return true;
    }
    if !options.include_test_items() && has_cfg_test_attr(node) {
        return true;
    }
    if !options.include_private() && !is_node_public(node) {
        return true;
    }
    false
}
