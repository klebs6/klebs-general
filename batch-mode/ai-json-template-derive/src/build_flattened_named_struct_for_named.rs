crate::ix!();

pub fn build_flattened_named_struct_for_named(
    flat_ident: &syn::Ident,
    flat_fields: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    trace!("build_flattened_named_struct_for_named: building '{}'", flat_ident);

    quote! {
        #[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
        pub struct #flat_ident {
            #(#flat_fields)*
        }
    }
}
