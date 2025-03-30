// ---------------- [ File: src/extract_enum_variants.rs ]
crate::ix!();

pub fn extract_enum_variants(input: &syn::DeriveInput) 
    -> &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> 
{
    if let syn::Data::Enum(ref data_enum) = input.data {
        &data_enum.variants
    } else {
        panic!("Expected enum data");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, DeriveInput};

    #[test]
    fn test_extract_enum_variants_with_valid_enum() {
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                Variant1,
                Variant2,
            }
        };

        let variants = extract_enum_variants(&input);
        assert_eq!(variants.len(), 2); // Two variants in the enum
        assert_eq!(variants[0].ident, "Variant1");
        assert_eq!(variants[1].ident, "Variant2");
    }

    #[test]
    #[should_panic(expected = "Expected enum data")]
    fn test_extract_enum_variants_with_non_enum() {
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                field: i32,
            }
        };

        extract_enum_variants(&input); // Should panic
    }
}
