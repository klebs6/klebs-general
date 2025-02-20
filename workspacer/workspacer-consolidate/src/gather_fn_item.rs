// ---------------- [ File: src/gather_fn_item.rs ]
crate::ix!();

pub fn gather_fn_item(
    fn_ast:  &ast::Fn,
    options: &ConsolidationOptions,
) -> CrateInterfaceItem<ast::Fn> {

    let docs = if *options.include_docs() {
        extract_docs(fn_ast.syntax())
    } else {
        None
    };

    let attributes = gather_all_attrs(fn_ast.syntax());

    let is_test_item = is_in_test_module(fn_ast.syntax().clone()) || has_cfg_test_attr(fn_ast.syntax());

    let body_source = if is_test_item {
        if *options.include_fn_bodies_in_tests() {
            if let Some(block_expr) = fn_ast.body() {
                Some(block_expr.syntax().text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        if *options.include_fn_bodies() {
            if let Some(block_expr) = fn_ast.body() {
                Some(block_expr.syntax().text().to_string())
            } else {
                None
            }
        } else {
            None
        }
    };

    CrateInterfaceItem::new(fn_ast.clone(), docs, attributes, body_source)
}
