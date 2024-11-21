crate::ix!();

/// Check if a `NestedMeta` object matches the identifier "Display".
pub fn is_display_attribute(nested_meta: &NestedMeta) -> bool {
    if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
        path.is_ident("Display")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Meta, NestedMeta};

    #[test]
    fn test_is_display_attribute() {
        // Case: The attribute matches "Display"
        let nested_meta: NestedMeta = parse_quote!(Display);
        assert!(is_display_attribute(&nested_meta));

        // Case: The attribute does not match "Display"
        let nested_meta: NestedMeta = parse_quote!(Other);
        assert!(!is_display_attribute(&nested_meta));

        // Case: The attribute is not a Meta::Path
        let nested_meta: NestedMeta = parse_quote!(key = "value");
        assert!(!is_display_attribute(&nested_meta));
    }
}
