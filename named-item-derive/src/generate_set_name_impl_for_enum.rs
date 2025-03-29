// ---------------- [ File: src/generate_set_name_impl_for_enum.rs ]
crate::ix!();

pub fn generate_set_name_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    set_name_arms: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics SetName for #enum_ident #ty_generics #where_clause {
            fn set_name(&mut self, name: &str) -> Result<(), NameError> {
                match self {
                    #( #set_name_arms ),*
                }
            }
        }
    }
}
