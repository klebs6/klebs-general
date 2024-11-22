pub(crate) fn find_ai_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                // Handle #[ai("value")]
                for nested in meta_list.nested.iter() {
                    if let NestedMeta::Lit(Lit::Str(lit_str)) = nested {
                        return Some(lit_str.value());
                    }
                }
            } else if let Ok(Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. })) = attr.parse_meta() {
                // Handle #[ai = "value"]
                return Some(lit_str.value());
            }
        }
    }
    None
}

pub(crate) fn find_feature_if_none(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested_meta in meta_list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit: Lit::Str(lit_str), .. })) = nested_meta {
                        if path.is_ident("feature_if_none") {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_option_inner_type(ty: &Type) -> &Type {
    if let Type::Path(TypePath { path, .. }) = ty {
        for segment in &path.segments {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty;
                    }
                }
            }
        }
    }
    ty
}

pub(crate) fn has_ai_display(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path.is_ident("ai") {
            if let Ok(Meta::List(MetaList { nested, .. })) = attr.parse_meta() {
                for meta in nested {
                    if let NestedMeta::Meta(Meta::Path(path)) = meta {
                        if path.is_ident("Display") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub(crate) fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

pub(crate) fn impl_item_feature(input: &DeriveInput, data: &DataEnum) -> TokenStream2 {
    let enum_name = &input.ident;
    let mut variant_matches = vec![];

    for variant in &data.variants {
        match process_variant(variant) {
            Ok(variant_match) => variant_matches.push(variant_match),
            Err(error) => return error,
        }
    }

    let expanded = quote! {
        impl ItemFeature for #enum_name {
            fn text(&self) -> std::borrow::Cow<'_, str> {
                match self {
                    #(#variant_matches)*
                }
            }
        }
    };

    TokenStream2::from(expanded)
}

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
                    Err(error) => return error.into(),
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

#[proc_macro_derive(ItemFeature, attributes(ai))]
pub fn item_feature_derive(input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Ensure the input is an enum
    let enum_data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Error::new_spanned(input.ident, "ItemFeature can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    // Generate the implementation
    impl_item_feature(&input, enum_data).into()
}

#[proc_macro_derive(ItemWithFeatures, attributes(ai))]
pub fn item_with_features_derive(input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Ensure the input is a struct
    let struct_data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Error::new_spanned(input.ident, "ItemWithFeatures can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate the implementation
    impl_item_with_features(&input, struct_data).into()
}

pub(crate) fn process_field(field: &Field) -> Result<TokenStream2, TokenStream> {
    let field_name = field.ident.as_ref().unwrap();
    let ty = &field.ty;

    let is_option = is_option_type(ty);
    let feature_if_none = find_feature_if_none(&field.attrs);

    if is_option {
        // Optional field
        if let Some(default_text) = feature_if_none {
            Ok(quote! {
                match &self.#field_name {
                    Some(value) => features.push(value.text()),
                    None => features.push(std::borrow::Cow::Borrowed(#default_text)),
                }
            })
        } else {
            Ok(quote! {
                if let Some(value) = &self.#field_name {
                    features.push(value.text());
                }
            })
        }
    } else {
        // Non-optional field
        Ok(quote! {
            features.push(self.#field_name.text());
        })
    }
}

pub(crate) fn process_variant(variant: &Variant) -> Result<TokenStream2, TokenStream2> {
    match &variant.fields {
        Fields::Unit => {
            // Unit variant, should have #[ai("...")]
            let ai_attr = find_ai_attr(&variant.attrs);
            let ai_text = match ai_attr {
                Some(text) => text,
                None => {
                    let span = variant.ident.span();
                    return Err(quote_spanned! { span =>
                        compile_error!("Unit variants must have an #[ai(\"...\")] attribute");
                    });
                }
            };

            let variant_ident = &variant.ident;
            Ok(quote! {
                Self::#variant_ident => Cow::Borrowed(#ai_text),
            })
        }
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            // Tuple variant with one unnamed field
            let variant_ident = &variant.ident;
            Ok(quote! {
                Self::#variant_ident(inner) => inner.text(),
            })
        }
        _ => {
            let span = variant.ident.span();
            return Err(quote_spanned! { span =>
                compile_error!("Variants must be unit variants with #[ai(\"...\")] or single-field tuple variants wrapping a type that implements ItemFeature");
            });
        }
    }
}


