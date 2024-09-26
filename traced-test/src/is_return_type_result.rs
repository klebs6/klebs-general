crate::ix!();

pub trait ReturnsResult {

    fn returns_result(&self) -> bool;
}

impl ReturnsResult for syn::ItemFn {

    fn returns_result(&self) -> bool {
        match &self.sig.output {
            syn::ReturnType::Type(_, ty) => {
                if let syn::Type::Path(type_path) = &**ty {
                    type_path.path.segments.last().map_or(false, |segment| segment.ident == "Result")
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}

#[cfg(test)]
mod check_return_type_tests {
    use super::*;
    use syn::{ItemFn, parse_quote};

    #[test]
    fn test_function_returns_result() {
        // Function with a simple Result return type
        let function: ItemFn = parse_quote! {
            fn example() -> Result<(), ()> { Ok(()) }
        };
        assert!(function.returns_result());
    }

    #[test]
    fn test_function_does_not_return_result() {
        // Function with a non-Result return type
        let function: ItemFn = parse_quote! {
            fn example() -> Option<()> { Some(()) }
        };
        assert!(!function.returns_result());
    }

    #[test]
    fn test_function_with_no_return_type() {
        // Function with no return type (implicitly returns ())
        let function: ItemFn = parse_quote! {
            fn example() { }
        };
        assert!(!function.returns_result());
    }

    #[test]
    fn test_function_returns_complex_result() {
        // Function with a complex Result return type
        let function: ItemFn = parse_quote! {
            fn example() -> Result<Option<i32>, String> { Ok(Some(42)) }
        };
        assert!(function.returns_result());
    }

    #[test]
    fn test_function_with_generic_return_type() {
        // Function with a generic Result return type
        let function: ItemFn = parse_quote! {
            fn example<T>() -> Result<T, ()> where T: Default { Ok(T::default()) }
        };
        assert!(function.returns_result());
    }

    #[test]
    fn test_function_returns_associated_type() {
        // Function that returns an associated type from a trait, not a Result
        let function: ItemFn = parse_quote! {
            fn example() -> <T as Trait>::AssocType { }
        };
        assert!(!function.returns_result());
    }

    #[test]
    fn test_function_returns_result_with_lifetimes() {
        // Function that returns a Result type with lifetimes
        let function: ItemFn = parse_quote! {
            fn example<'a>() -> Result<&'a str, ()> { Ok("test") }
        };
        assert!(function.returns_result());
    }
}
