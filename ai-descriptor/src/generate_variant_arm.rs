crate::ix!();

/// Generate the match arm for a specific enum variant.
/// Handles named, unnamed, and unit variants.
pub fn generate_variant_arm(
    enum_name: &Ident,
    variant: &Variant,
) -> TokenStream2 {
    let variant_name = &variant.ident;
    let fields = &variant.fields;

    match fields {
        Fields::Named(fields_named) => {
            generate_variant_arm_for_named_fields(enum_name, variant_name, fields_named)
        }
        Fields::Unnamed(fields_unnamed) => {
            generate_variant_arm_for_unnamed_fields(enum_name, variant_name, fields_unnamed.unnamed.len())
        }
        Fields::Unit => generate_unit_variant_arm(enum_name, variant_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Variant, Fields};

    #[test]
    fn test_generate_variant_arm_named_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant: Variant = parse_quote! {
            MyVariant { field1: i32, field2: String }
        };

        let tokens = generate_variant_arm(&enum_name, &variant);

        let expected = quote! {
            MyEnum::MyVariant { field1, field2 } => {
                let field1_ai = format!("{}", field1);
                let field2_ai = format!("{}", field2);
                let description = format!("MyVariant {{ field1: {{}}, field2: {{}} }}", field1_ai, field2_ai);
                std::borrow::Cow::Owned(description)
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_variant_arm_unnamed_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant: Variant = parse_quote! {
            MyVariant(i32, String)
        };

        let tokens = generate_variant_arm(&enum_name, &variant);

        let expected = quote! {
            MyEnum::MyVariant(field0, field1) => {
                let field0_ai = format!("{}", field0);
                let field1_ai = format!("{}", field1);
                let description = format!("MyVariant({{}}, {{}})", field0_ai, field1_ai);
                std::borrow::Cow::Owned(description)
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_variant_arm_unit() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant: Variant = parse_quote! {
            MyVariant
        };

        let tokens = generate_variant_arm(&enum_name, &variant);

        let expected = quote! {
            MyEnum::MyVariant => std::borrow::Cow::Borrowed(stringify!(MyVariant))
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
