crate::ix!();

/// Generate `let` bindings for named fields in a struct or enum variant.
/// Example: For `field1`, generate `let field1_ai = field1.ai();`.
pub fn generate_named_field_bindings(field_names: &[&Ident]) -> Vec<TokenStream2> {
    field_names
        .iter()
        .map(|name| {
            let field_ai_ident = Ident::new(&format!("{}_ai", name), name.span());
            quote! {
                let #field_ai_ident = format!("{}", #name);
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_named_field_bindings_no_fields() {
        let field_names: Vec<&Ident> = vec![];
        let bindings = generate_named_field_bindings(&field_names);
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_generate_named_field_bindings_single_field() {
        let f1 = parse_quote!(field1);
        let field_names: Vec<&Ident> = vec![&f1];
        let bindings = generate_named_field_bindings(&field_names);

        let expected = vec![
            quote! {
                let field1_ai = format!("{}", field1);
            },
        ];

        assert_eq!(
            bindings.iter().map(|ts| ts.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|ts| ts.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_generate_named_field_bindings_multiple_fields() {
        let f1 = parse_quote!(field1);
        let f2 = parse_quote!(field2);
        let field_names: Vec<&Ident> = vec![&f1, &f2];
        let bindings = generate_named_field_bindings(&field_names);

        let expected = vec![
            quote! {
                let field1_ai = format!("{}", field1);
            },
            quote! {
                let field2_ai = format!("{}", field2);
            },
        ];

        assert_eq!(
            bindings.iter().map(|ts| ts.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|ts| ts.to_string()).collect::<Vec<_>>()
        );
    }
}
