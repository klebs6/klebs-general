crate::ix!();

pub fn build_justified_struct(
    justified_ident: &syn::Ident,
    ty_ident: &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    trace!(
        "Building the main 'Justified' struct type for '{}'",
        ty_ident
    );

    let expanded = quote::quote! {
        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            // do not use pub fields
            item:          #ty_ident,
            justification: #justification_ident,
            confidence:    #confidence_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };
    debug!(
        "Constructed 'Justified' struct definition for '{}'",
        justified_ident
    );
    expanded
}
