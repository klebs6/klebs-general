crate::ix!();

/// Check if an attribute contains a `MetaList` with the "Display" attribute.
pub fn has_display_in_attribute(attr: &Attribute) -> bool {
    if let Ok(meta) = attr.parse_meta() {
        if let Meta::List(MetaList { nested, .. }) = meta {
            return has_display_attribute(&nested);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Attribute};

    #[test]
    fn test_has_display_in_attribute() {
        // Case: The attribute contains "Display"
        let attr: Attribute = parse_quote!(#[ai(Display, Other)]);
        assert!(has_display_in_attribute(&attr));

        // Case: The attribute does not contain "Display"
        let attr: Attribute = parse_quote!(#[ai(Other)]);
        assert!(!has_display_in_attribute(&attr));

        // Case: The attribute is not a Meta::List
        let attr: Attribute = parse_quote!(#[ai = "value"]);
        assert!(!has_display_in_attribute(&attr));

        // Case: The attribute is invalid (e.g., syntax error)
        let attr: Attribute = parse_quote!(#[ai]);
        assert!(!has_display_in_attribute(&attr));
    }
}
