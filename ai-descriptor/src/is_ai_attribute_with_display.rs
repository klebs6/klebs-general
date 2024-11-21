crate::ix!();

/// Check if an attribute has the path "ai" and contains the "Display" attribute.
pub fn is_ai_attribute_with_display(attr: &Attribute) -> bool {
    attr.path.is_ident("ai") && has_display_in_attribute(attr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_is_ai_attribute_with_display() {
        // Case: The attribute has path "ai" and contains "Display"
        let attr: Attribute = parse_quote!(#[ai(Display, Other)]);
        assert!(is_ai_attribute_with_display(&attr));

        // Case: The attribute has path "ai" but does not contain "Display"
        let attr: Attribute = parse_quote!(#[ai(Other)]);
        assert!(!is_ai_attribute_with_display(&attr));

        // Case: The attribute does not have path "ai" but contains "Display"
        let attr: Attribute = parse_quote!(#[not_ai(Display)]);
        assert!(!is_ai_attribute_with_display(&attr));

        // Case: The attribute does not have path "ai" and does not contain "Display"
        let attr: Attribute = parse_quote!(#[not_ai(Other)]);
        assert!(!is_ai_attribute_with_display(&attr));

        // Case: The attribute has path "ai" but is not a Meta::List
        let attr: Attribute = parse_quote!(#[ai = "value"]);
        assert!(!is_ai_attribute_with_display(&attr));

        // Case: The attribute is invalid (e.g., syntax error)
        let attr: Attribute = parse_quote!(#[ai]);
        assert!(!is_ai_attribute_with_display(&attr));
    }
}

