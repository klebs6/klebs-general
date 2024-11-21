crate::ix!();

/// Extract field names from a `FieldsNamed` struct.
/// Panics if any field is unnamed, which should not occur for named fields.
pub fn extract_field_names(fields_named: &FieldsNamed) -> Vec<&Ident> {
    fields_named
        .named
        .iter()
        .map(|f| {
            f.ident
                .as_ref()
                .filter(|ident| *ident != "_") // Filter out `_` as an invalid name
                .expect("Named fields should have identifiers")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, FieldsNamed};

    #[test]
    fn test_extract_field_names() {
        // Case: Fields are properly named
        let fields_named: FieldsNamed = parse_quote!({ field1: i32, field2: String });
        let field_names = extract_field_names(&fields_named);

        let expected: Vec<String> = vec!["field1".to_string(), "field2".to_string()];

        assert_eq!(
            field_names.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    #[should_panic(expected = "Named fields should have identifiers")]
    fn test_extract_field_names_unnamed_field() {
        // Case: Simulate an invalid unnamed field (shouldn't happen in valid `FieldsNamed`)
        let fields_named: FieldsNamed = parse_quote!({ _: i32 });
        let _ = extract_field_names(&fields_named); // This should panic
    }
}
