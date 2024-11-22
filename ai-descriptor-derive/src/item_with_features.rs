crate::ix!();


pub(crate) fn impl_item_with_features(input: &DeriveInput, data: &DataStruct) -> TokenStream2 {
    let struct_name = &input.ident;

    // Extract the header from the #[ai("...")] attribute
    let header = match find_ai_attr(&input.attrs) {
        Some(text) => text,
        None => {
            return Error::new_spanned(
                input.ident.clone(),
                "Structs deriving ItemWithFeatures must have an #[ai(\"...\")] attribute for the header",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut feature_expressions = vec![];

    match &data.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                match process_field(field) {
                    Ok(feature_expr) => feature_expressions.push(feature_expr),
                    Err(error) => return error,
                }
            }
        }
        _ => {
            return Error::new_spanned(
                struct_name.clone(),
                "ItemWithFeatures can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    }

    // Check for #[ai(Display)] attribute to implement Display
    let implement_display = has_ai_display(&input.attrs);

    let display_impl = if implement_display {
        quote! {
            impl std::fmt::Display for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.ai())
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl ItemWithFeatures for #struct_name {
            fn header(&self) -> std::borrow::Cow<'_, str> {
                std::borrow::Cow::Borrowed(#header)
            }

            fn features(&self) -> Vec<std::borrow::Cow<'_, str>> {
                use std::borrow::Cow;
                let mut features = Vec::new();
                #(#feature_expressions)*
                features
            }
        }

        #display_impl
    };

    TokenStream2::from(expanded)
}
