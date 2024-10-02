crate::ix!();

pub trait ExtractAllAttributesExcept {

    fn extract_all_attributes_except(&self, kinds: &[AttributeKind]) 
        -> Vec<Attribute>;
}

impl ExtractAllAttributesExcept for ItemFn {

    /// Retain original attributes, removing attributes that match any in the `kinds` set
    fn extract_all_attributes_except(&self, kinds: &[AttributeKind]) -> Vec<Attribute> {
        self.attrs
            .iter()
            .filter(|attr| {
                // Try converting the attribute to AttributeKind and check if it's in the `kinds` set
                if let Ok(kind) = AttributeKind::try_from(*attr) {
                    !kinds.contains(&kind)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod extract_all_test_attributes_except_test_attribute_tests {

    use super::*;
    use syn::{ItemFn, Attribute, parse_quote};
    use proc_macro2::TokenStream;
    use quote::quote;

    // Tests for `all_attributes_except_test`
    #[test]
    fn test_all_attributes_except_test_with_no_test_attribute() {
        let function: ItemFn = parse_quote! {
            #[some_attr]
            fn example() {}
        };
        let result = function.extract_all_attributes_except(&[AttributeKind::TestAttr]);
        assert_eq!(result.len(), 1); // Expecting one attribute to be retained
        assert!(result.iter().any(|attr| attr.path().is_ident("some_attr")));
    }

    #[test]
    fn test_all_attributes_except_test_with_test_attribute() {
        let function: ItemFn = parse_quote! {
            #[test]
            #[some_attr]
            fn example() {}
        };
        let result = function.extract_all_attributes_except(&[AttributeKind::TestAttr]);
        assert_eq!(result.len(), 1); // Only `some_attr` should remain, `test` should be removed
        assert!(result.iter().any(|attr| attr.path().is_ident("some_attr")));
        assert!(!result.iter().any(|attr| attr.path().is_ident("test")));
    }

    #[test]
    fn test_all_attributes_except_test_with_multiple_test_attributes() {
        let function: ItemFn = parse_quote! {
            #[test]
            #[tokio::test]
            #[some_attr]
            fn example() {}
        };
        let result = function.extract_all_attributes_except(&[AttributeKind::TestAttr]);
        assert_eq!(result.len(), 1); // Only `some_attr` should remain
        assert!(result.iter().any(|attr| attr.path().is_ident("some_attr")));
        assert!(!result.iter().any(|attr| attr.path().is_ident("test")));
        assert!(!result.iter().any(|attr| attr.path().is_ident("tokio::test")));
    }
}
