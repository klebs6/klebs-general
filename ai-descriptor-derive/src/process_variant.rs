crate::ix!();

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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{parse_quote, Variant};

    #[test]
    fn test_process_variant_unit_with_ai() {
        let variant: Variant = parse_quote! {
            #[ai("It is the default.")]
            Default
        };
        let result = process_variant(&variant);
        assert!(result.is_ok());
        let expected = quote! {
            Self::Default => Cow::Borrowed("It is the default."),
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }

    #[test]
    fn test_process_variant_unit_without_ai() {
        let variant: Variant = parse_quote! {
            Default
        };
        let result = process_variant(&variant);
        assert!(result.is_err());
        // Optionally, check the error message
        let error = result.err().unwrap();
        println!("error={}",error);
        assert!(error.to_string().contains("Unit variants must have an #[ai(\\\"...\\\")] attribute"));
    }

    #[test]
    fn test_process_variant_tuple_single_field() {
        let variant: Variant = parse_quote! {
            Standard(StandardLyricalMeter)
        };
        let result = process_variant(&variant);
        assert!(result.is_ok());
        let expected = quote! {
            Self::Standard(inner) => inner.text(),
        };
        assert_eq!(result.unwrap().to_string(), expected.to_string());
    }

    #[test]
    fn test_process_variant_tuple_multiple_fields() {
        let variant: Variant = parse_quote! {
            Complex(String, i32)
        };
        let result = process_variant(&variant);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("Variants must be unit variants"));
    }

    #[test]
    fn test_process_variant_struct_variant() {
        let variant: Variant = parse_quote! {
            Complex { a: String, b: i32 }
        };
        let result = process_variant(&variant);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert!(error.to_string().contains("Variants must be unit variants"));
    }
}
