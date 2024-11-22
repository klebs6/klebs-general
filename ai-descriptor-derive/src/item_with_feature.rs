crate::ix!();

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

