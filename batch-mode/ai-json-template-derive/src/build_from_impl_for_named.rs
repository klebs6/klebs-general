crate::ix!();

pub fn build_from_impl_for_named(
    flat_ident: &syn::Ident,
    justified_ident: &syn::Ident,
    ty_ident: &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
    item_inits: &[proc_macro2::TokenStream],
    just_inits: &[proc_macro2::TokenStream],
    conf_inits: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    trace!(
        "build_from_impl_for_named: building From<{}> for {}",
        flat_ident,
        justified_ident
    );

    quote! {
        impl From<#flat_ident> for #justified_ident {
            fn from(flat: #flat_ident) -> Self {
                let item = #ty_ident {
                    #(#item_inits, )*
                };
                let justification = #justification_ident {
                    #(#just_inits, )*
                    ..Default::default()
                };
                let confidence = #confidence_ident {
                    #(#conf_inits, )*
                    ..Default::default()
                };
                Self {
                    item,
                    justification,
                    confidence,
                }
            }
        }
    }
}
