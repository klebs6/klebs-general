// ---------------- [ File: src/generate_name_history_impl_for_enum.rs ]
crate::ix!();

pub fn generate_name_history_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    history_arms_add: &[proc_macro2::TokenStream],
    history_arms_get: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics NameHistory for #enum_ident #ty_generics #where_clause {
            fn add_name_to_history(&mut self, name: &str) {
                trace!("Called add_name_to_history('{}') on enum '{}'", name, stringify!(#enum_ident));
                match self {
                    #( #history_arms_add ),*
                }
            }

            fn name_history(&self) -> Vec<std::borrow::Cow<'_, str>> {
                trace!("Called name_history() on enum '{}'", stringify!(#enum_ident));
                match self {
                    #( #history_arms_get ),*
                }
            }
        }
    }
}
