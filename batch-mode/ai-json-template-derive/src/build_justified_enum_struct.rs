crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_justified_enum_struct(
    ty_ident: &syn::Ident,
    enum_just_ident: &syn::Ident,
    enum_conf_ident: &syn::Ident,
    justified_ident: &syn::Ident
) -> proc_macro2::TokenStream
{
    debug!(
        "Building the final Justified struct '{}' for enum '{}'",
        justified_ident,
        ty_ident
    );

    quote::quote! {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Getters, Setters)]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            item: #ty_ident,
            justification: #enum_just_ident,
            confidence: #enum_conf_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: ::core::default::Default::default(),
                    confidence: ::core::default::Default::default(),
                }
            }
        }
    }
}
