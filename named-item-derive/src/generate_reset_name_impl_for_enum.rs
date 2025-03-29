// ---------------- [ File: src/generate_reset_name_impl_for_enum.rs ]
crate::ix!();

pub fn generate_reset_name_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics ResetName for #enum_ident #ty_generics #where_clause {
            fn reset_name(&mut self) -> Result<(), NameError> {
                trace!("Called reset_name() on enum '{}'", stringify!(#enum_ident));
                self.set_name(&Self::default_name())
            }
        }
    }
}
