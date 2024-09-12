crate::ix!();

pub trait IsAsync {
    fn is_async(&self) -> bool;
}

impl IsAsync for syn::ItemFn {
    fn is_async(&self) -> bool {
        self.sig.asyncness.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{ItemFn, parse_quote};

    #[test]
    fn test_function_is_async() {
        // Test an async function
        let function: ItemFn = parse_quote! {
            async fn example() {}
        };
        assert!(function.is_async(), "Expected the function to be async");
    }

    #[test]
    fn test_function_is_not_async() {
        // Test a non-async function
        let function: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(!function.is_async(), "Expected the function to not be async");
    }

    #[test]
    fn test_async_function_with_arguments() {
        // Test an async function with arguments
        let function: ItemFn = parse_quote! {
            async fn example(arg1: i32, arg2: String) {}
        };
        assert!(function.is_async(), "Expected the function to be async");
    }

    #[test]
    fn test_non_async_function_with_arguments() {
        // Test a non-async function with arguments
        let function: ItemFn = parse_quote! {
            fn example(arg1: i32, arg2: String) {}
        };
        assert!(!function.is_async(), "Expected the function to not be async");
    }

    #[test]
    fn test_async_function_with_return_type() {
        // Test an async function with a return type
        let function: ItemFn = parse_quote! {
            async fn example() -> i32 {
                42
            }
        };
        assert!(function.is_async(), "Expected the function to be async");
    }

    #[test]
    fn test_non_async_function_with_return_type() {
        // Test a non-async function with a return type
        let function: ItemFn = parse_quote! {
            fn example() -> i32 {
                42
            }
        };
        assert!(!function.is_async(), "Expected the function to not be async");
    }
}
