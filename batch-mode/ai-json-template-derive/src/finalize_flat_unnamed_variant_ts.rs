// ---------------- [ File: ai-json-template-derive/src/finalize_flat_unnamed_variant_ts.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn finalize_flat_unnamed_variant_ts(
    variant_ident: &syn::Ident,
    expansions: &UnnamedVariantExpansion
) -> TokenStream2
{
    trace!(
        "Constructing flattened unnamed variant definition for variant '{}'",
        variant_ident
    );

    let field_decls = expansions.field_declarations();

    if !field_decls.is_empty() {
        quote! {
            #variant_ident {
                #(#field_decls),*
            },
        }
    } else {
        quote! {
            #variant_ident {},
        }
    }
}
