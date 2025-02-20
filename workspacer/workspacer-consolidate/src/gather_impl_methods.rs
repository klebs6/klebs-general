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
