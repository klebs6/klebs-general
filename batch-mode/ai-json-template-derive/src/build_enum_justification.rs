crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_justification(
    enum_just_ident: &syn::Ident,
    just_variants: &[proc_macro2::TokenStream],
    first_variant_ident: Option<&syn::Ident>,
    first_variant_just_fields: &[String]
) -> proc_macro2::TokenStream
{
    debug!(
        "Building justification enum '{}' with {} variant(s)",
        enum_just_ident,
        just_variants.len()
    );

    let variants_ts = quote::quote! { #( #just_variants ),* };
    let default_impl = if let Some(first_variant) = first_variant_ident {
        let init_fields: Vec<_> = first_variant_just_fields.iter().map(|f_str| {
            let f_id = syn::Ident::new(f_str, proc_macro2::Span::call_site());
            quote::quote! { #f_id: ::core::default::Default::default() }
        }).collect();

        quote::quote! {
            impl ::core::default::Default for #enum_just_ident {
                fn default() -> Self {
                    #enum_just_ident::#first_variant { #( #init_fields ),* }
                }
            }
        }
    } else {
        quote::quote!()
    };

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_just_ident {
            #variants_ts
        }
        #default_impl
    }
}
