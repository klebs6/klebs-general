crate::ix!();

// Function to check if any variant has a primitive type field
pub fn variant_has_primitive_type(variants: &Punctuated<Variant, Comma>) -> bool {
    variants.iter().any(|variant| {
        match &variant.fields {
            Fields::Named(fields_named) => {
                fields_named.named.iter().any(|field| is_primitive_type(&field.ty))
            }
            Fields::Unnamed(fields_unnamed) => {
                fields_unnamed.unnamed.iter().any(|field| is_primitive_type(&field.ty))
            }
            Fields::Unit => false, // Unit variants have no fields
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, DeriveInput};

    #[test]
    fn test_variant_has_primitive_type_with_primitive_field() {
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                VariantOne(u8),
                VariantTwo { value: String },
                VariantThree,
            }
        };

        let variants = extract_enum_variants(&input);
        assert!(variant_has_primitive_type(&variants));
    }

    #[test]
    fn test_variant_has_primitive_type_without_primitive_field() {
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                VariantOne(String),
                VariantTwo { value: Vec<u8> },
                VariantThree,
            }
        };

        let variants = extract_enum_variants(&input);
        assert!(!variant_has_primitive_type(&variants));
    }

    #[test]
    fn test_variant_has_primitive_type_with_mixed_fields() {
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                VariantOne(i32, String),
                VariantTwo { value: Vec<u8>, flag: bool },
                VariantThree,
            }
        };

        let variants = extract_enum_variants(&input);
        assert!(variant_has_primitive_type(&variants));
    }

    #[test]
    fn test_variant_has_primitive_type_with_unit_variants() {
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                VariantOne,
                VariantTwo,
                VariantThree,
            }
        };

        let variants = extract_enum_variants(&input);
        assert!(!variant_has_primitive_type(&variants));
    }
}
