crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine E: Build the final “flat variant” snippet for named variants
// ---------------------------------------------------------------------------
pub fn build_flat_variant_snippet_named(
    variant_ident:       &syn::Ident,
    field_decls_top:     &[proc_macro2::TokenStream],
    field_decls_fields:  &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream
{
    trace!(
        "build_flat_variant_snippet_named: variant='{}'",
        variant_ident
    );

    let mut all_fields = Vec::new();
    all_fields.extend_from_slice(field_decls_top);
    all_fields.extend_from_slice(field_decls_fields);

    if !all_fields.is_empty() {
        quote::quote! {
            #variant_ident {
                #(#all_fields),*
            },
        }
    } else {
        // no fields at all
        quote::quote! {
            #variant_ident {},
        }
    }
}
