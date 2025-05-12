crate::ix!();

/// Builds `FlatJustified{ParentEnum}` ident, e.g. `FlatJustifiedMyEnum`.
pub fn build_flat_parent_ident(parent_enum_ident: &syn::Ident) -> syn::Ident {
    let new_name = format!("FlatJustified{}", parent_enum_ident);
    trace!("build_flat_parent_ident => '{}'", new_name);
    syn::Ident::new(&new_name, parent_enum_ident.span())
}
