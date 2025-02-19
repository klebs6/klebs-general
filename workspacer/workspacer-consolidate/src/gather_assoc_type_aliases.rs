crate::ix!();

// ---------------- [ File: workspacer-consolidate/src/imports.rs ] (or wherever your gather_assoc_type_aliases is)
pub fn gather_assoc_type_aliases(
    impl_ast: &ast::Impl,
    options: &ConsolidationOptions
) -> Vec<CrateInterfaceItem<ast::TypeAlias>> {
    let mut out = vec![];
    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(ty_alias) = ast::TypeAlias::cast(item.syntax().clone()) {
                if !options.include_test_items() && has_cfg_test_attr(ty_alias.syntax()) {
                    continue;
                }
                if !options.include_private() && !is_node_public(ty_alias.syntax()) {
                    continue;
                }
                let docs = if *options.include_docs() {
                    extract_docs(ty_alias.syntax())
                } else {
                    None
                };
                let attrs = gather_all_attrs(ty_alias.syntax());
                out.push(CrateInterfaceItem::new(ty_alias, docs, attrs, None));
            }
        }
    }
    out
}


