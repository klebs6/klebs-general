crate::ix!();

/// Generate `_ai` identifiers for named fields.
/// Example: For `field1`, generate `field1_ai`.
pub fn generate_named_field_ai_idents(field_names: &[&Ident]) -> Vec<Ident> {
    field_names
        .iter()
        .map(|name| Ident::new(&format!("{}_ai", name), name.span()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_named_field_ai_idents_no_fields() {
        let field_names: Vec<&Ident> = vec![];
        let ai_idents = generate_named_field_ai_idents(&field_names);
        assert!(ai_idents.is_empty());
    }

    #[test]
    fn test_generate_named_field_ai_idents_single_field() {

        let f1 = parse_quote!(field1);

        let field_names: Vec<&Ident> = vec![&f1];
        let ai_idents = generate_named_field_ai_idents(&field_names);

        let f1ai = parse_quote!(field1_ai);
        let expected: Vec<Ident> = vec![f1ai];

        assert_eq!(
            ai_idents.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_generate_named_field_ai_idents_multiple_fields() {
        let f1 = parse_quote!(field1);
        let f2 = parse_quote!(field2);

        let field_names: Vec<&Ident> = vec![&f1, &f2];
        let ai_idents = generate_named_field_ai_idents(&field_names);

        let f1ai = parse_quote!(field1_ai);
        let f2ai = parse_quote!(field2_ai);

        let expected: Vec<Ident> = vec![f1ai, f2ai];

        assert_eq!(
            ai_idents.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        );
    }
}
