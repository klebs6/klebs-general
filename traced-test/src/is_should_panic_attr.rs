// ---------------- [ File: src/is_should_panic_attr.rs ]
crate::ix!();

pub trait IsShouldPanicAttr {

    fn is_should_panic_attr(&self) -> bool;
}

impl IsShouldPanicAttr for syn::Attribute {

    fn is_should_panic_attr(&self) -> bool {
        // Check if the path is exactly `should_panic`
        if self.path().is_ident("should_panic") {
            // Parse the nested meta attributes, e.g., expected = "..."
            let _ = self.parse_nested_meta(|meta| {
                if meta.path.is_ident("expected") {
                    // Handle the expected key (or other keys) here if necessary
                    let _ = meta.value()?.parse::<syn::LitStr>()?;
                }
                Ok(())
            });

            return true;
        }

        false
    }
}

#[cfg(test)]
mod should_panic_attr_tests {
    use super::*;
    use syn::{Attribute, parse_quote};

    // Test a simple `#[should_panic]` attribute
    #[test]
    fn test_is_should_panic_attr_simple() {
        let attr: Attribute = parse_quote!(#[should_panic]);
        assert!(attr.is_should_panic_attr(), "Expected the attribute to be recognized as `should_panic`.");
    }

    // Test `#[should_panic(expected = "error")]` with expected argument
    #[test]
    fn test_is_should_panic_attr_with_expected() {
        let attr: Attribute = parse_quote!(#[should_panic(expected = "error")]);
        assert!(attr.is_should_panic_attr(), "Expected the attribute to be recognized as `should_panic`.");
    }

    // Test an attribute that looks like `should_panic` but with an invalid format
    #[test]
    fn test_is_should_panic_attr_invalid_format() {
        let attr: Attribute = parse_quote!(#[should_panic(invalid = "some_value")]);
        assert!(attr.is_should_panic_attr(), "Even with an invalid argument, should be recognized as `should_panic`.");
    }

    // Test an attribute with incorrect name
    #[test]
    fn test_is_not_should_panic_attr() {
        let attr: Attribute = parse_quote!(#[not_should_panic]);
        assert!(!attr.is_should_panic_attr(), "Expected the attribute to not be recognized as `should_panic`.");
    }

    // Test `#[should_panic]` with multiple arguments
    #[test]
    fn test_is_should_panic_attr_with_multiple_arguments() {
        let attr: Attribute = parse_quote!(#[should_panic(expected = "error", another_arg = "value")]);
        assert!(attr.is_should_panic_attr(), "Expected the attribute to be recognized as `should_panic` even with multiple arguments.");
    }

    // Test an attribute with complex meta that is not `should_panic`
    #[test]
    fn test_is_not_should_panic_attr_with_complex_meta() {
        let attr: Attribute = parse_quote!(#[some_other_attr(arg = "value")]);
        assert!(!attr.is_should_panic_attr(), "Expected the attribute to not be recognized as `should_panic`.");
    }

    // Test an empty attribute (no meta)
    #[test]
    fn test_is_not_should_panic_attr_empty() {
        let attr: Attribute = parse_quote!(#[some_attr]);
        assert!(!attr.is_should_panic_attr(), "Expected the attribute to not be recognized as `should_panic`.");
    }
}
