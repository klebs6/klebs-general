crate::ix!();

pub trait IsTestAttribute {

    fn is_test_attribute(&self) -> bool;
}

impl IsTestAttribute for syn::Attribute {

    fn is_test_attribute(&self) -> bool {
        if self.path().is_ident("test") {
            return true;
        }

        // Check for paths like `tokio::test`
        if let Some(segment) = self.path().segments.last() {
            if segment.ident == "test" && self.path().segments.len() > 1 {
                return self.path().segments.iter().any(|seg| seg.ident == "tokio");
            }
        }

        false
    }
}

#[cfg(test)]
mod test_is_test_attribute {

    use super::*;
    use syn::{ItemFn, Attribute, parse_quote};
    use proc_macro2::TokenStream;
    use quote::quote;

    // Tests for `attr_is_test_attribute`
    #[test]
    fn test_attr_is_test_attribute_with_test() {
        let attr: Attribute = parse_quote!(#[test]);
        assert!(attr.is_test_attribute());
    }

    #[test]
    fn test_attr_is_test_attribute_with_tokio_test() {
        let attr: Attribute = parse_quote!(#[tokio::test]);
        assert!(attr.is_test_attribute());
    }

    #[test]
    fn test_attr_is_test_attribute_with_non_test_attribute() {
        let attr: Attribute = parse_quote!(#[some_other_attr]);
        assert!(!attr.is_test_attribute());
    }
}
