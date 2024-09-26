crate::ix!();

pub trait EnsureItemHasNoTestAttribute {

    type Error;
    fn ensure_no_test_attribute(&self) -> Result<(),Self::Error>;
    fn generate_error() -> Self::Error;
}

#[cfg(test)]
impl EnsureItemHasNoTestAttribute for ItemFn {

    type Error = String;

    fn generate_error() -> String {
        "The `traced_test` attribute should be used in place of `#[test]` or `#[tokio::test]`, not alongside them.".to_string()
    }

    fn ensure_no_test_attribute(&self) -> Result<(), String> {
        let has_test_attr = self.attrs.iter().any(|attr| attr.is_test_attr());

        if has_test_attr {
            return Err(Self::generate_error());
        }

        Ok(())
    }
}

#[cfg(not(test))]
impl EnsureItemHasNoTestAttribute for ItemFn {

    type Error = TokenStream;

    fn generate_error() -> Self::Error {
        TokenStream::from(quote! {
            compile_error!("The `traced_test` attribute should be used in place of `#[test]` or `#[tokio::test]`, not alongside them.");
        })
    }

    fn ensure_no_test_attribute(&self) -> Result<(), Self::Error> {
        let has_test_attr = self.attrs.iter().any(|attr| attr.is_test_attr());

        if has_test_attr {
            return Err(Self::generate_error());
        }

        Ok(())
    }
}

#[cfg(test)]
mod ensure_item_has_no_test_attribute_tests {

    use super::*;
    use syn::{ItemFn, Attribute, parse_quote};
    use proc_macro2::TokenStream;
    use quote::quote;

    // Tests for `ensure_no_test_attribute`
    #[test]
    fn test_ensure_no_test_attribute_with_no_test_attribute() {
        let function: ItemFn = parse_quote! {
            #[some_attr]
            fn example() {}
        };
        assert!(function.ensure_no_test_attribute().is_ok());
    }

    #[test]
    fn test_ensure_no_test_attribute_with_test_attribute() {
        let function: ItemFn = parse_quote! {
            #[test]
            fn example() {}
        };
        let result = function.ensure_no_test_attribute();
        assert!(result.is_err());
        let err_message = result.unwrap_err().to_string();
        assert!(err_message.contains("The `traced_test` attribute should be used"));
    }

    #[test]
    fn test_ensure_no_test_attribute_with_tokio_test() {
        let function: ItemFn = parse_quote! {
            #[tokio::test]
            fn example() {}
        };
        let result = function.ensure_no_test_attribute();
        assert!(result.is_err());
        let err_message = result.unwrap_err().to_string();
        assert!(err_message.contains("The `traced_test` attribute should be used"));
    }

    #[test]
    fn test_ensure_no_test_attribute_with_mixed_attributes() {
        let function: ItemFn = parse_quote! {
            #[test]
            #[tokio::test]
            #[some_attr]
            fn example() {}
        };
        let result = function.ensure_no_test_attribute();
        assert!(result.is_err());
        let err_message = result.unwrap_err().to_string();
        assert!(err_message.contains("The `traced_test` attribute should be used"));
    }

    #[test]
    fn test_error_handling_in_proc_macro() {
        // Intentionally trigger the error condition in ensure_no_test_attribute
        let function: ItemFn = syn::parse_quote! {
            #[test]
            fn sample_test() {}
        };

        assert!(function.ensure_no_test_attribute().is_err());
    }
}
