crate::ix!();

/// Check if any attribute in the slice has the path "ai" and contains the "Display" attribute.
pub fn has_ai_display_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(is_ai_attribute_with_display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_has_ai_display_attribute() {
        // Case: The attributes include #[ai(Display)]
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai(Display, Other)]),
            parse_quote!(#[other]),
        ];
        assert!(has_ai_display_attribute(&attrs));

        // Case: The attributes include #[ai] but without "Display"
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai(Other)]),
            parse_quote!(#[other]),
        ];
        assert!(!has_ai_display_attribute(&attrs));

        // Case: The attributes do not include #[ai]
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[other]),
        ];
        assert!(!has_ai_display_attribute(&attrs));

        // Case: The attributes are empty
        let attrs: Vec<Attribute> = vec![];
        assert!(!has_ai_display_attribute(&attrs));

        // Case: The attributes include #[ai = "value"] (not a Meta::List)
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[ai = "value"]),
            parse_quote!(#[other]),
        ];
        assert!(!has_ai_display_attribute(&attrs));
    }
}

