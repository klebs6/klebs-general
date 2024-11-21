crate::ix!();

/// Check if any `NestedMeta` in the iterator matches the "Display" attribute.
pub fn has_display_attribute<'a, I>(nested_meta_iter: I) -> bool
where
    I: IntoIterator<Item = &'a NestedMeta>,
{
    nested_meta_iter.into_iter().any(is_display_attribute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, MetaList};

    #[test]
    fn test_has_display_attribute() {
        // Case: The list contains "Display"
        let meta_list: MetaList = parse_quote!(ai(Display, Other));
        assert!(has_display_attribute(&meta_list.nested));

        // Case: The list does not contain "Display"
        let meta_list: MetaList = parse_quote!(ai(Other, Another));
        assert!(!has_display_attribute(&meta_list.nested));

        // Case: The list is empty
        let meta_list: MetaList = parse_quote!(ai());
        assert!(!has_display_attribute(&meta_list.nested));
    }
}
