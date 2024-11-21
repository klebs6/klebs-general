crate::ix!();

/// Generate field patterns for unnamed fields in an enum variant.
/// Example: `field0, field1, ...` for unnamed fields.
pub fn generate_field_patterns_for_unnamed_fields(
    field_count: usize,
    span: Span,
) -> Vec<Ident> {
    (0..field_count)
        .map(|i| Ident::new(&format!("field{}", i), span))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Ident;
    use proc_macro2::Span;

    #[test]
    fn test_generate_field_patterns_for_no_fields() {
        let field_count = 0;
        let span = Span::call_site(); // Use a valid default span
        let field_patterns = generate_field_patterns_for_unnamed_fields(field_count, span);
        assert_eq!(field_patterns.len(), 0);
    }

    #[test]
    fn test_generate_field_patterns_for_single_field() {
        let field_count = 1;
        let span = Span::call_site(); // Use a valid default span
        let field_patterns = generate_field_patterns_for_unnamed_fields(field_count, span);
        assert_eq!(field_patterns.len(), 1);
        assert_eq!(field_patterns[0].to_string(), "field0");
    }

    #[test]
    fn test_generate_field_patterns_for_multiple_fields() {
        let field_count = 3;
        let span = Span::call_site(); // Use a valid default span
        let field_patterns = generate_field_patterns_for_unnamed_fields(field_count, span);
        assert_eq!(field_patterns.len(), 3);
        assert_eq!(field_patterns[0].to_string(), "field0");
        assert_eq!(field_patterns[1].to_string(), "field1");
        assert_eq!(field_patterns[2].to_string(), "field2");
    }
}

