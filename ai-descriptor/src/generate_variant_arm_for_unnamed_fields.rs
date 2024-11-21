crate::ix!();

/// Generate a match arm for an enum variant with unnamed fields.
pub fn generate_variant_arm_for_unnamed_fields(
    enum_name: &Ident,
    variant_name: &Ident,
    field_count: usize,
) -> TokenStream2 {
    if field_count == 0 {
        // Handle case where there are no fields
        let variant_string = format!("{}()", variant_name);
        quote! {
            #enum_name::#variant_name() => {
                let description = format!(#variant_string);
                std::borrow::Cow::Owned(description)
            }
        }
    } else {
        // Generate field patterns, bindings, and AI identifiers
        let field_patterns = generate_field_patterns_for_unnamed_fields(field_count, variant_name.span());
        let field_bindings = generate_field_bindings(&field_patterns);
        let field_ai_idents = generate_field_ai_idents(&field_patterns);
        let format_string = generate_format_string_for_variant(variant_name, field_ai_idents.len());

        // Generate the match arm
        quote! {
            #enum_name::#variant_name(#(#field_patterns),*) => {
                #(#field_bindings)*
                let description = format!(#format_string, #(#field_ai_idents),*);
                std::borrow::Cow::Owned(description)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_variant_arm_for_unnamed_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant_name: Ident = parse_quote!(MyVariant);
        let field_count = 2;

        let tokens = generate_variant_arm_for_unnamed_fields(&enum_name, &variant_name, field_count);

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
    fn test_generate_variant_arm_for_unnamed_fields_no_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant_name: Ident = parse_quote!(MyVariant);
        let field_count = 0;

        let tokens = generate_variant_arm_for_unnamed_fields(&enum_name, &variant_name, field_count);

        let expected = quote! {
            MyEnum::MyVariant() => {
                let description = format!("MyVariant()");
                std::borrow::Cow::Owned(description)
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
