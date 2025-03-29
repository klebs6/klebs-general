// ---------------- [ File: src/generate_setname_impl.rs ]
crate::ix!();

pub fn generate_setname_impl(
    struct_name:   &syn::Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics:   &syn::TypeGenerics<'_>,
    where_clause:  Option<&syn::WhereClause>,
    cfg:           &NamedItemConfig

) -> proc_macro2::TokenStream {

    if *cfg.history() {
        quote! {
            impl #impl_generics SetName for #struct_name #ty_generics #where_clause {
                fn set_name(&mut self, name: &str) -> Result<(), NameError> {
                    trace!("set_name('{}') called on struct '{}'", name, stringify!(#struct_name));
                    self.name_history.push(name.to_string());
                    if name.is_empty() && name != &*Self::default_name() {
                        warn!("Attempted to set empty name on '{}'", stringify!(#struct_name));
                        return Err(NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }

            impl #impl_generics NameHistory for #struct_name #ty_generics #where_clause {
                fn add_name_to_history(&mut self, name: &str) {
                    trace!("add_name_to_history('{}') on '{}'", name, stringify!(#struct_name));
                    self.name_history.push(name.to_string());
                }

                fn name_history(&self) -> Vec<std::borrow::Cow<'_, str>> {
                    trace!("name_history() called on '{}'", stringify!(#struct_name));
                    self.name_history
                        .iter()
                        .map(|s| std::borrow::Cow::from(&s[..]))
                        .collect()
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics SetName for #struct_name #ty_generics #where_clause {
                fn set_name(&mut self, name: &str) -> Result<(), NameError> {
                    trace!("set_name('{}') called on struct '{}'", name, stringify!(#struct_name));
                    if name.is_empty() && name != &*Self::default_name() {
                        warn!("Attempted to set empty name on '{}'", stringify!(#struct_name));
                        return Err(NameError::EmptyName);
                    }
                    self.name = name.to_owned();
                    Ok(())
                }
            }
        }
    }
}
