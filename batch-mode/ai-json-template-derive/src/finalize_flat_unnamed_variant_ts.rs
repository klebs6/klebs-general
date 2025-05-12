crate::ix!();

/// Builds the final “flat variant” snippet, e.g.:
/// ```ignore
/// TupleVar {
///    #[serde(default)] enum_variant_justification:String,
///    #[serde(default)] enum_variant_confidence:f32,
///    f0: f0,
///    f1: f1,
/// },
/// ```
pub fn finalize_flat_unnamed_variant_ts(
    variant_ident: &syn::Ident,
    expansions:    &UnnamedVariantExpansion
) -> TokenStream2 {
    trace!(
        "Constructing flattened unnamed variant definition for variant '{}'",
        variant_ident
    );
    if !expansions.field_declarations.is_empty() {
        quote! {
            #variant_ident {
                #( #expansions.field_declarations ),*
            },
        }
    } else {
        quote! {
            #variant_ident {},
        }
    }
}
