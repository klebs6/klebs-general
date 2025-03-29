// ---------------- [ File: src/generate_baseline_impl.rs ]
crate::ix!();

pub fn generate_baseline_impl(
    struct_name:   &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics:   &syn::TypeGenerics<'_>,
    where_clause:  Option<&syn::WhereClause>,
    fallback_name: &str
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics Named for #struct_name #ty_generics #where_clause {
            fn name(&self) -> std::borrow::Cow<'_, str> {
                trace!("Called name() on '{}'", stringify!(#struct_name));
                std::borrow::Cow::from(&self.name)
            }
        }

        impl #impl_generics DefaultName for #struct_name #ty_generics #where_clause {
            fn default_name() -> std::borrow::Cow<'static, str> {
                trace!("Called default_name() on '{}'", stringify!(#struct_name));
                std::borrow::Cow::from(#fallback_name)
            }
        }

        impl #impl_generics ResetName for #struct_name #ty_generics #where_clause {
            fn reset_name(&mut self) -> Result<(), NameError> {
                trace!("Called reset_name() on '{}'", stringify!(#struct_name));
                self.set_name(&Self::default_name())
            }
        }
    }
}
