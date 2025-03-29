// ---------------- [ File: src/impl_named_item.rs ]
crate::ix!();

/// This is the refactored entry point that delegates to specialized subroutines
/// depending on whether we have a struct or an enum.
pub fn impl_named_item(ast: &DeriveInput, cfg: &NamedItemConfig) -> SynResult<TokenStream> {
    info!("Entering impl_named_item for '{}'", ast.ident);

    match &ast.data {
        Data::Struct(ds) => expand_struct_named_item(ast, ds, cfg),
        Data::Enum(de) => expand_enum_named_item(ast, de, cfg),
        _ => {
            error!("'NamedItem' can only be derived on a struct or enum");
            Err(SynError::new_spanned(
                &ast.ident,
                "NamedItem can only be derived on a struct or enum.",
            ))
        }
    }
}
