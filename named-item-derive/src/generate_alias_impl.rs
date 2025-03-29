// ---------------- [ File: src/generate_alias_impl.rs ]
crate::ix!();

pub fn generate_alias_impl(
    struct_name:   &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics:   &syn::TypeGenerics<'_>,
    where_clause:  Option<&syn::WhereClause>,
    cfg:           &NamedItemConfig
) -> proc_macro2::TokenStream {
    if !cfg.aliases() {
        return quote!{};
    }

    let arr_tokens = cfg
        .default_aliases()
        .iter()
        .map(|s| quote! { #s.to_owned() });

    quote! {
        impl #impl_generics NamedAlias for #struct_name #ty_generics #where_clause {
            fn add_alias(&mut self, alias: &str) {
                trace!("add_alias('{}') called on '{}'", alias, stringify!(#struct_name));
                self.aliases.push(alias.to_string());
            }
            fn aliases(&self) -> Vec<std::borrow::Cow<'_, str>> {
                trace!("aliases() called on '{}'", stringify!(#struct_name));
                self.aliases
                    .iter()
                    .map(|s| std::borrow::Cow::from(&s[..]))
                    .collect()
            }
            fn clear_aliases(&mut self) {
                trace!("clear_aliases() called on '{}'", stringify!(#struct_name));
                self.aliases.clear();
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            fn default_aliases() -> Vec<std::borrow::Cow<'static, str>> {
                trace!("default_aliases() called on '{}'", stringify!(#struct_name));
                vec![
                    #(std::borrow::Cow::from(#arr_tokens)),*
                ]
            }
        }
    }
}
