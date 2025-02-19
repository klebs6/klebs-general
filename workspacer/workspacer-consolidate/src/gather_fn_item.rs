crate::ix!();

pub fn gather_fn_item(
    fn_ast: &ast::Fn,
    options: &ConsolidationOptions,
) -> CrateInterfaceItem<ast::Fn> {
    let docs = if *options.include_docs() {
        extract_docs(fn_ast.syntax())
    } else {
        None
    };
    let attributes = gather_all_attrs(fn_ast.syntax());

    let body_source = if *options.include_fn_bodies() {
        if let Some(block_expr) = fn_ast.body() {
            Some(block_expr.syntax().text().to_string())
        } else {
            None
        }
    } else {
        None
    };

    CrateInterfaceItem::new(fn_ast.clone(), docs, attributes, body_source)
}
