crate::ix!();

/// Generate bindings for unnamed fields in an enum variant.
/// Example: `let field0_ai = field0.ai();`
pub fn generate_field_bindings(field_patterns: &[Ident]) -> Vec<TokenStream2> {
    field_patterns
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
    use syn::{parse_quote, Ident};
    use quote::quote;

    #[test]
    fn test_generate_field_bindings_no_fields() {
        let field_patterns: Vec<Ident> = vec![];
        let bindings = generate_field_bindings(&field_patterns);
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_generate_field_bindings_single_field() {
        let field_patterns: Vec<Ident> = vec![parse_quote!(field0)];
        let bindings = generate_field_bindings(&field_patterns);

        let expected = vec![
            quote! {
                let field0_ai = format!("{}", field0);
            },
        ];

        assert_eq!(
            bindings.iter().map(|ts| ts.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|ts| ts.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_generate_field_bindings_multiple_fields() {
        let field_patterns: Vec<Ident> = vec![parse_quote!(field0), parse_quote!(field1)];
        let bindings = generate_field_bindings(&field_patterns);

        let expected = vec![
            quote! {
                let field0_ai = format!("{}", field0);
            },
            quote! {
                let field1_ai = format!("{}", field1);
            },
        ];

        assert_eq!(
            bindings.iter().map(|ts| ts.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|ts| ts.to_string()).collect::<Vec<_>>()
        );
    }
}
