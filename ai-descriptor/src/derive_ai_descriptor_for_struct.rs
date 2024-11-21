crate::ix!();

// Function to handle structs
pub fn derive_ai_descriptor_for_struct(
    struct_name: &syn::Ident,
    data_struct: &syn::DataStruct,
    attrs: &[syn::Attribute],
) -> TokenStream2 {
    // Extract the ai attribute from the struct
    let mut struct_ai_template = String::new();
    let mut derive_display = false;

    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(meta) = attr.parse_meta() {
                match meta {
                    Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. }) => {
                        struct_ai_template = lit_str.value();
                    }
                    Meta::List(MetaList { nested, .. }) => {
                        for nested_meta in nested {
                            match nested_meta {
                                NestedMeta::Lit(Lit::Str(lit_str)) => {
                                    struct_ai_template = lit_str.value();
                                }
                                NestedMeta::Meta(Meta::Path(path)) => {
                                    if path.is_ident("Display") {
                                        derive_display = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let fields = match &data_struct.fields {
        Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
        Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed.iter().collect::<Vec<_>>(),
        Fields::Unit => Vec::new(),
    };

    let mut field_bindings = Vec::new();
    let mut format_args = Vec::new();

    for field in &fields {
        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| syn::Ident::new("unnamed", field.span()));
        let field_name_str = field_name.to_string();

        // Create a new identifier for the ai variable, e.g., field_name_ai
        let field_ai_ident = syn::Ident::new(&format!("{}_ai", field_name_str), field.span());

        // Extract #[ai(none="...")] attribute
        let mut none_message = None;
        for attr in &field.attrs {
            if attr.path.is_ident("ai") {
                if let Ok(meta) = attr.parse_meta() {
                    match meta {
                        Meta::NameValue(MetaNameValue { path, lit: Lit::Str(lit_str), .. }) => {
                            if path.is_ident("none") {
                                none_message = Some(lit_str.value());
                            }
                        }
                        Meta::List(MetaList { nested, .. }) => {
                            for nested_meta in nested {
                                match nested_meta {
                                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                        path,
                                        lit: Lit::Str(lit_str),
                                        ..
                                    })) => {
                                        if path.is_ident("none") {
                                            none_message = Some(lit_str.value());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check if field is Option<T>
        if let Some(_inner_ty) = is_option_type(&field.ty) {
            // Field is Option<T>
            if let Some(message) = none_message {
                field_bindings.push(quote! {
                    let #field_ai_ident = match &self.#field_name {
                        Some(value) => value.ai(),
                        None => std::borrow::Cow::Borrowed(#message),
                    };
                });
            } else {
                field_bindings.push(quote! {
                    let #field_ai_ident = match &self.#field_name {
                        Some(value) => value.ai(),
                        None => std::borrow::Cow::Borrowed(""),
                    };
                });
            }
        } else {
            // Field is not Option<T>
            field_bindings.push(quote! {
                let #field_ai_ident = self.#field_name.ai();
            });
        }

        // Add to format_args
        format_args.push(quote! {
            #field_name = #field_ai_ident
        });
    }

    let ai_impl = if !struct_ai_template.is_empty() {
        quote! {
            fn ai(&self) -> std::borrow::Cow<'_, str> {
                #(#field_bindings)*

                let description = format!(#struct_ai_template, #(#format_args),*);
                std::borrow::Cow::Owned(description)
            }
        }
    } else {
        // Generate default ai() implementation
        let field_names: Vec<_> = fields
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();

        let format_string = format!(
            "{} {{ {{ {} }} }}",
            struct_name,
            field_names
            .iter()
            .map(|n| format!("{}: {{}}", n))
            .collect::<Vec<_>>()
            .join(", ")
        );
        let field_ai_idents: Vec<_> = fields
            .iter()
            .map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                syn::Ident::new(&format!("{}_ai", field_name), field_name.span())
            })
            .collect();
        quote! {
            fn ai(&self) -> std::borrow::Cow<'_, str> {
                #(#field_bindings)*

                let description = format!(#format_string, #(#field_ai_idents),*);
                std::borrow::Cow::Owned(description)
            }
        }
    };

    let gen = quote! {
        impl AIDescriptor for #struct_name {
            #ai_impl
        }
    };

    // If #[ai(Display)] is present, implement Display
    let display_impl = if derive_display {
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

    quote! {
        #gen
        #display_impl
    }
    .into()
}
