// ---------------- [ File: src/generate_named_alias_impl_for_enum.rs ]
crate::ix!();

pub fn generate_named_alias_impl_for_enum(
    enum_ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    cfg: &NamedItemConfig,
    aliases_arms_add: &[proc_macro2::TokenStream],
    aliases_arms_get: &[proc_macro2::TokenStream],
    aliases_arms_clear: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let default_aliases = cfg
        .default_aliases()
        .iter()
        .map(|s| quote! { #s.to_owned() });

    quote! {
        impl #impl_generics NamedAlias for #enum_ident #ty_generics #where_clause {
            fn add_alias(&mut self, alias: &str) {
                match self {
                    #( #aliases_arms_add ),*
                }
            }

            fn aliases(&self) -> Vec<std::borrow::Cow<'_, str>> {
                match self {
                    #( #aliases_arms_get ),*
                }
            }

            fn clear_aliases(&mut self) {
                match self {
                    #( #aliases_arms_clear ),*
                }
            }
        }

        impl #impl_generics #enum_ident #ty_generics #where_clause {
            fn default_aliases() -> Vec<std::borrow::Cow<'static, str>> {
                trace!("Called default_aliases() on enum '{}'", stringify!(#enum_ident));
                vec![
                    #(std::borrow::Cow::from(#default_aliases)),*
                ]
            }
        }
    }
}
