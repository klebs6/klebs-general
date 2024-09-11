crate::ix!();

pub(crate) fn is_async_function(function: &syn::ItemFn) -> bool {
    function.sig.asyncness.is_some()
}

/// Retain the original attributes and remove the `test` attribute
pub(crate) fn all_attributes_except_test(function: &syn::ItemFn) -> Vec<syn::Attribute> {
    function.attrs.iter().filter(|attr| !attr.path().is_ident("test")).cloned().collect()
}

pub(crate) fn is_return_type_result(function: &syn::ItemFn) -> bool {
    match &function.sig.output {
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

pub(crate) fn generate_test_block(
    function_is_async: bool,
    function_returns_result: bool,
    original_block: &syn::Block,
    test_name: &str,
) -> proc_macro2::TokenStream {
    match (function_is_async, function_returns_result) {
        (true, true)   => generate_traced_block_async_with_result(original_block, test_name),
        (true, false)  => generate_traced_block_async_no_result(original_block, test_name),
        (false, true)  => generate_traced_block_sync_with_result(original_block, test_name),
        (false, false) => generate_traced_block_sync_no_result(original_block, test_name),
    }
}

pub(crate) fn parse_or_compile_error(block: proc_macro2::TokenStream) -> syn::Block {
    syn::parse2(block).unwrap_or_else(|e| {
        panic!("Failed to parse block: {}", e);
    })
}

pub(crate) fn generate_function_with_test_attr(
    function_is_async: bool,
    attrs: &[syn::Attribute],
    function: &syn::ItemFn,
) -> proc_macro2::TokenStream {
    if function_is_async {
        quote! {
            #(#attrs)*
            #[tokio::test]
            #function
        }
    } else {
        quote! {
            #(#attrs)*
            #[test]
            #function
        }
    }
}

pub(crate) fn ensure_no_test_attribute(function: &ItemFn) -> Result<(), TokenStream> {
    let has_test_attr = function.attrs.iter().any(|attr| attr.path().is_ident("test") || attr.path().is_ident("tokio::test"));

    if has_test_attr {
        Err(TokenStream::from(quote! {
            compile_error!("The `traced_test` attribute should be used in place of `#[test]` or `#[tokio::test]`, not alongside them.");
        }))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_error_handling_in_proc_macro() {
        // Intentionally trigger the error condition in ensure_no_test_attribute
        let function = syn::parse_quote! {
            #[test]
            fn sample_test() {}
        };

        assert!(crate::ensure_no_test_attribute(&function).is_err());
    }
}
