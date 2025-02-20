// ---------------- [ File: src/gather_assoc_type_aliases.rs ]
crate::ix!();

/// Gathers all associated type aliases in an impl block, respecting skip logic and collecting docs/attrs.
pub fn gather_assoc_type_aliases(
    impl_ast: &ast::Impl, 
    options: &ConsolidationOptions
) -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>> 
{
    let mut out = Vec::new();
    if let Some(assoc_items) = impl_ast.assoc_item_list() {
        for item in assoc_items.assoc_items() {
            if let Some(ty_alias) = ast::TypeAlias::cast(item.syntax().clone()) {
                if !crate::skip_checks::should_skip_item(ty_alias.syntax(), options) {
                    let docs = if *options.include_docs() {
                        extract_docs(ty_alias.syntax())
                    } else {
                        None
                    };
                    let attrs = gather_all_attrs(ty_alias.syntax());
                    
                    out.push(crate::crate_interface_item::CrateInterfaceItem::new(
                        ty_alias,
                        docs,
                        attrs,
                        None
                    ));
                } else {
                    info!("Skipping type_alias in impl: either test item or private item was disallowed");
                }
            }
        }
    }
    out
}
