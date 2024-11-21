crate::ix!();

/// Generate a format string for an enum variant with named fields.
/// Example: `VariantName { field1: {}, field2: {} }`
pub fn generate_named_field_format_string(variant_name: &Ident, field_names: &[&Ident]) -> String {
    let field_placeholders = field_names
        .iter()
        .map(|n| format!("{}: {{}}", n))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} {{ {} }}", variant_name, field_placeholders)
        .replace("{", "{{")
        .replace("}", "}}")
}


#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_named_field_format_string_no_fields() {
        let variant_name: Ident = parse_quote!(VariantName);
        let field_names: Vec<&Ident> = vec![];
        let format_string = generate_named_field_format_string(&variant_name, &field_names);

        assert_eq!(format_string, "VariantName {{  }}");
    }

    #[test]
    fn test_generate_named_field_format_string_single_field() {
        let variant_name: Ident = parse_quote!(VariantName);
        let f1 = parse_quote!(field1);
        let field_names: Vec<&Ident> = vec![&f1];
        let format_string = generate_named_field_format_string(&variant_name, &field_names);

        assert_eq!(format_string, "VariantName {{ field1: {{}} }}");
    }

    #[test]
    fn test_generate_named_field_format_string_multiple_fields() {
        let variant_name: Ident = parse_quote!(VariantName);
        let f1 = parse_quote!(field1);
        let f2 = parse_quote!(field2);
        let field_names: Vec<&Ident> = vec![&f1, &f2];
        let format_string = generate_named_field_format_string(&variant_name, &field_names);

        assert_eq!(format_string, "VariantName {{ field1: {{}}, field2: {{}} }}");
    }
}
