crate::ix!();

/// If the `variant_ident`'s string is "Unit", returns a new Ident with the `replacement_name`.
/// Otherwise, returns `variant_ident` unchanged.
pub fn rename_variant_ident_if_unit(
    variant_ident:     &syn::Ident,
    replacement_name:  &str
) -> syn::Ident {
    let original_name = variant_ident.to_string();
    if original_name == "Unit" {
        trace!(
            "rename_variant_ident_if_unit: found 'Unit', using replacement='{}'",
            replacement_name
        );
        syn::Ident::new(replacement_name, variant_ident.span())
    } else {
        trace!(
            "rename_variant_ident_if_unit: variant '{}' not 'Unit', leaving as-is",
            variant_ident
        );
        variant_ident.clone()
    }
}
