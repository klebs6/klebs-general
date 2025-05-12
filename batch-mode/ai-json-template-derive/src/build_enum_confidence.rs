crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_enum_confidence(
    enum_conf_ident: &syn::Ident,
    conf_variants: &[proc_macro2::TokenStream],
    first_variant_ident: Option<&syn::Ident>,
    first_variant_conf_fields: &[String]
) -> proc_macro2::TokenStream
{
    debug!(
        "Building confidence enum '{}' with {} variant(s)",
        enum_conf_ident,
        conf_variants.len()
    );

    let variants_ts = quote::quote! { #( #conf_variants ),* };
    let default_impl = if let Some(first_variant) = first_variant_ident {
        let init_fields: Vec<_> = first_variant_conf_fields.iter().map(|f_str| {
            let f_id = syn::Ident::new(f_str, proc_macro2::Span::call_site());
            quote::quote! { #f_id: ::core::default::Default::default() }
        }).collect();

        quote::quote! {
            impl ::core::default::Default for #enum_conf_ident {
                fn default() -> Self {
                    #enum_conf_ident::#first_variant { #( #init_fields ),* }
                }
            }
        }
    } else {
        quote::quote!()
    };

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        enum #enum_conf_ident {
            #variants_ts
        }
        #default_impl
    }
}
