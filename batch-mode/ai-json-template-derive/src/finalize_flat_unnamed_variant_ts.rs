// ---------------- [ File: ai-json-template-derive/src/finalize_flat_unnamed_variant_ts.rs ]
crate::ix!();

pub fn finalize_flat_unnamed_variant_ts(
    variant_ident: &syn::Ident,
    expansions: &UnnamedVariantExpansion
) -> TokenStream2
{
    trace!(
        "Constructing flattened unnamed variant definition for variant '{}'",
        variant_ident
    );

    if !expansions.field_declarations().is_empty() {
        quote! {
            #variant_ident {
                #(#expansions.field_declarations()),*
            },
        }
    } else {
        quote! {
            #variant_ident {},
        }
    }
}
