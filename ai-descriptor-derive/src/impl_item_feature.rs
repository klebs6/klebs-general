crate::ix!();

pub(crate) fn impl_item_feature(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Enum(data_enum) => impl_item_feature_enum(input, data_enum),
        Data::Struct(data_struct) => impl_item_feature_struct(input, data_struct),
        _ => {
            Error::new_spanned(input.ident.clone(), "ItemFeature can only be derived for enums and structs")
                .to_compile_error()
                .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, DeriveInput};
    use quote::ToTokens;

    #[test]
    fn test_item_feature_struct_named_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(ItemFeature)]
            #[ai("Follows a custom rhyme scheme: {scheme}.")]
            pub struct CustomRhymeScheme {
                scheme: String,
            }
        };
        let expanded = impl_item_feature(&input);
        let expected = quote! {
            impl ItemFeature for CustomRhymeScheme {
                fn text(&self) -> std::borrow::Cow<'_, str> {
                    std::borrow::Cow::Owned(format!("Follows a custom rhyme scheme: {scheme}.", scheme = &self.scheme))
                }
            }
        };
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    #[test]
    fn test_item_feature_struct_unnamed_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(ItemFeature)]
            #[ai("Follows a custom rhyme scheme: {0}.")]
            pub struct CustomRhymeScheme(String);
        };
        let expanded = impl_item_feature(&input);
        let expected = quote! {
            impl ItemFeature for CustomRhymeScheme {
                fn text(&self) -> std::borrow::Cow<'_, str> {
                    std::borrow::Cow::Owned(format!("Follows a custom rhyme scheme: {0}.", &self.0))
                }
            }
        };
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    #[test]
    fn test_item_feature_struct_no_fields() {
        let input: DeriveInput = parse_quote! {
            #[derive(ItemFeature)]
            #[ai("No fields")]
            pub struct NoFields;
        };
        let expanded = impl_item_feature(&input);
        let expected = quote! {
            impl ItemFeature for NoFields {
                fn text(&self) -> std::borrow::Cow<'_, str> {
                    std::borrow::Cow::Borrowed("No fields")
                }
            }
        };
        assert_eq!(expanded.to_string(), expected.to_string());
    }
}
