// ---------------- [ File: src/generate_default_name_impl_for_enum.rs ]
crate::ix!();

pub fn generate_default_name_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    fallback_name: &str,
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics DefaultName for #enum_ident #ty_generics #where_clause {
            fn default_name() -> std::borrow::Cow<'static, str> {
                trace!("Called default_name() on enum '{}'", stringify!(#enum_ident));
                std::borrow::Cow::from(#fallback_name)
            }
        }
    }
}
