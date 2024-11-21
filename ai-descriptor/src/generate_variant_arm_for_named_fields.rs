crate::ix!();

/// Generate a match arm for an enum variant with named fields.
/// Example: `VariantName { field1, field2 } => { let field1_ai = ...; ... }`
pub fn generate_variant_arm_for_named_fields(
    enum_name: &Ident,
    variant_name: &Ident,
    fields_named: &syn::FieldsNamed,
) -> TokenStream2 {
    // Extract field names
    let field_names = extract_field_names(fields_named);

    if field_names.is_empty() {
        // Handle case where there are no fields
        let hardcoded_string = format!("{} {{  }}", variant_name);
        quote! {
            #enum_name::#variant_name { } => {
                let description = format!(#hardcoded_string);
                std::borrow::Cow::Owned(description)
            }
        }
    } else {
        // Generate components for the match arm
        let field_patterns = field_names.iter();
        let field_bindings = generate_named_field_bindings(&field_names);
        let field_ai_idents = generate_named_field_ai_idents(&field_names);
        let format_string = generate_named_field_format_string(variant_name, &field_names);

        // Generate the match arm
        quote! {
            #enum_name::#variant_name { #(#field_patterns),* } => {
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
    fn test_generate_variant_arm_for_named_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant_name: Ident = parse_quote!(MyVariant);
        let fields_named: syn::FieldsNamed = parse_quote!({ field1: i32, field2: String });

        let tokens = generate_variant_arm_for_named_fields(&enum_name, &variant_name, &fields_named);

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
    fn test_generate_variant_arm_for_named_fields_no_fields() {
        let enum_name: Ident = parse_quote!(MyEnum);
        let variant_name: Ident = parse_quote!(MyVariant);
        let fields_named: syn::FieldsNamed = parse_quote!({});

        let tokens = generate_variant_arm_for_named_fields(&enum_name, &variant_name, &fields_named);

        let expected = quote! {
            MyEnum::MyVariant { } => {
                let description = format!("MyVariant {  }");
                std::borrow::Cow::Owned(description)
            }
        };

        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
