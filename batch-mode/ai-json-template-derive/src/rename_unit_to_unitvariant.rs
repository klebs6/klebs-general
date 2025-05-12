crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine B: Possibly rename "Unit" => "UnitVariant"
// ---------------------------------------------------------------------------
pub fn rename_unit_to_unitvariant(variant_ident: &syn::Ident) -> syn::Ident {
    let real_name = variant_ident.to_string();
    if real_name == "Unit" {
        let renamed = syn::Ident::new("UnitVariant", variant_ident.span());
        trace!(
            "Renaming variant '{}' => '{}'",
            real_name,
            renamed.to_string()
        );
        renamed
    } else {
        variant_ident.clone()
    }
}
