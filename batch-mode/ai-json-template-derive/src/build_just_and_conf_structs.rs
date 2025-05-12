crate::ix!();

pub fn build_just_and_conf_structs(
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
    errs: &proc_macro2::TokenStream,
    justification_fields: &[proc_macro2::TokenStream],
    confidence_fields: &[proc_macro2::TokenStream],
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    trace!(
        "Building justification/conf structs: '{}' and '{}'",
        justification_ident,
        confidence_ident
    );

    // Make sure we do not use `pub` on the fields. The fields are already stripped of `pub` above.
    let just_ts = quote::quote! {
        #errs
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justification_ident {
            #(#justification_fields)*
        }
    };

    let conf_ts = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #confidence_ident {
            #(#confidence_fields)*
        }
    };

    debug!(
        "Finished building struct tokens for '{}' and '{}'",
        justification_ident, confidence_ident
    );
    (just_ts, conf_ts)
}
