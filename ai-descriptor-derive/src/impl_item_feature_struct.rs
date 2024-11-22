crate::ix!();

pub(crate) fn impl_item_feature_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream2 {
    let struct_name = &input.ident;

    // Extract the ai attribute
    let ai_text = match find_ai_attr(&input.attrs) {
        Some(text) => text,
        None => {
            return Error::new_spanned(
                input.ident.clone(),
                "Structs deriving ItemFeature must have an #[ai(\"...\")] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    let formatted_text = match &data.fields {
        Fields::Named(fields_named) => {
            let field_accesses: Vec<_> = fields_named.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote!(#ident = &self.#ident)
            }).collect();
            quote! {
                std::borrow::Cow::Owned(format!(#ai_text, #(#field_accesses),*))
            }
        },
        Fields::Unnamed(fields_unnamed) => {
            let field_accesses: Vec<_> = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                quote!(&self.#index)
            }).collect();
            quote! {
                std::borrow::Cow::Owned(format!(#ai_text, #(#field_accesses),*))
            }
        },
        Fields::Unit => {
            quote! {
                std::borrow::Cow::Borrowed(#ai_text)
            }
        },
    };

    let expanded = quote! {
        impl ItemFeature for #struct_name {
            fn text(&self) -> std::borrow::Cow<'_, str> {
                #formatted_text
            }
        }
    };

    TokenStream2::from(expanded)
}
