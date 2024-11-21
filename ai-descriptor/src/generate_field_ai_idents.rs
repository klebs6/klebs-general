crate::ix!();

/// Generate `_ai` identifiers for a list of field patterns.
/// Example: `field0` -> `field0_ai`
pub fn generate_field_ai_idents(field_patterns: &[Ident]) -> Vec<Ident> {
    field_patterns
        .iter()
        .map(|name| Ident::new(&format!("{}_ai", name), name.span()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Ident};

    #[test]
    fn test_generate_field_ai_idents_no_fields() {
        let field_patterns: Vec<Ident> = vec![];
        let ai_idents = generate_field_ai_idents(&field_patterns);
        assert!(ai_idents.is_empty());
    }

    #[test]
    fn test_generate_field_ai_idents_single_field() {
        let field_patterns: Vec<Ident> = vec![parse_quote!(field0)];
        let ai_idents = generate_field_ai_idents(&field_patterns);

        let expected: Vec<Ident> = vec![parse_quote!(field0_ai)];

        assert_eq!(
            ai_idents.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_generate_field_ai_idents_multiple_fields() {
        let field_patterns: Vec<Ident> = vec![parse_quote!(field0), parse_quote!(field1)];
        let ai_idents = generate_field_ai_idents(&field_patterns);

        let expected: Vec<Ident> = vec![parse_quote!(field0_ai), parse_quote!(field1_ai)];

        assert_eq!(
            ai_idents.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            expected.iter().map(|id| id.to_string()).collect::<Vec<_>>()
        );
    }
}
