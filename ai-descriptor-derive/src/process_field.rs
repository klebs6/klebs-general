crate::ix!();

pub(crate) fn process_field(field: &Field) -> Result<TokenStream2, TokenStream2> {
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
        if feature_if_none.is_some() {
            // Generate a compile-time error
            let span = field.span();
            return Err(quote_spanned! { span =>
                compile_error!("The `feature_if_none` attribute is only applicable to Option types");
            });
        } else {
            Ok(quote! {
                features.push(self.#field_name.text());
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Field};
    use quote::ToTokens;

    #[test]
    fn test_process_field_non_optional() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                availability: PotionAvailability
            }
        };

        let field = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        };

        let result = process_field(&field);
        assert!(result.is_ok());
        let expected = quote! {
            features.push(self.availability.text());
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }

    #[test]
    fn test_process_field_optional_without_feature_if_none() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                side_effects: Option<PotionSideEffects>
            }
        };

        let field = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        };

        let result = process_field(&field);
        assert!(result.is_ok());
        let expected = quote! {
            if let Some(value) = &self.side_effects {
                features.push(value.text());
            }
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }

    #[test]
    fn test_process_field_optional_with_feature_if_none() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_if_none = "This potion has no side effects.")]
                side_effects: Option<PotionSideEffects>
            }
        };

        let field = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        };

        let result = process_field(&field);
        assert!(result.is_ok());
        let expected = quote! {
            match &self.side_effects {
                Some(value) => features.push(value.text()),
                None => features.push(std::borrow::Cow::Borrowed("This potion has no side effects.")),
            }
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }

    #[test]
    fn test_process_field_non_optional_with_feature_if_none() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                #[ai(feature_if_none = "This should not be used.")]
                availability: PotionAvailability
            }
        };

        let field = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        };

        let result = process_field(&field);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("The `feature_if_none` attribute is only applicable to Option types"));
    }

    #[test]
    fn test_process_field_invalid_type() {
        let item_struct: syn::ItemStruct = parse_quote! {
            struct TestStruct {
                count: i32
            }
        };

        let field = match &item_struct.fields {
            syn::Fields::Named(fields_named) => fields_named.named.first().unwrap().clone(),
            _ => panic!("Expected named fields"),
        };

        let result = process_field(&field);
        assert!(result.is_ok());
        let expected = quote! {
            features.push(self.count.text());
        };
        // As before, note that this might cause an error elsewhere if `i32` doesn't implement `ItemFeature`.
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }
}
