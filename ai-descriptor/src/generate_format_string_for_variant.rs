crate::ix!();

/// Generate a format string for an enum variant with fields.
/// Example: `VariantName(field1, field2)` -> "VariantName({})"
/// Generate a format string for an enum variant with unnamed fields.
/// Example: `VariantName({}, {})`
pub fn generate_format_string_for_variant(variant_name: &Ident, field_count: usize) -> String {
    let placeholders = std::iter::repeat("{}")
        .take(field_count)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({})", variant_name, placeholders)
        .replace("{", "{{")
        .replace("}", "}}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_format_string_for_variant_no_fields() {
        let variant_name: Ident = parse_quote!(VariantName);
        let format_string = generate_format_string_for_variant(&variant_name, 0);
        assert_eq!(format_string, "VariantName()");
    }

    #[test]
    fn test_generate_format_string_for_variant_single_field() {
        let variant_name: Ident = parse_quote!(VariantName);
        let format_string = generate_format_string_for_variant(&variant_name, 1);
        assert_eq!(format_string, "VariantName({{}})");
    }

    #[test]
    fn test_generate_format_string_for_variant_multiple_fields() {
        let variant_name: Ident = parse_quote!(VariantName);
        let format_string = generate_format_string_for_variant(&variant_name, 3);
        assert_eq!(format_string, "VariantName({{}}, {{}}, {{}})");
    }
}
