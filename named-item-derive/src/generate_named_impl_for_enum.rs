// ---------------- [ File: src/generate_named_impl_for_enum.rs ]
crate::ix!();

pub fn generate_named_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    name_arms: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics Named for #enum_ident #ty_generics #where_clause {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                trace!("Called name() on enum '{}'", stringify!(#enum_ident));
                match self {
                    #( #name_arms ),*
                }
            }
        }
    }
}
