crate::ix!();

pub(crate) fn ensure_no_test_attribute(function: &ItemFn) -> Result<(), TokenStream> {
    let has_test_attr = function.attrs.iter().any(|attr| attr.path().is_ident("test"));

    if has_test_attr {
        Err(TokenStream::from(quote! {
            compile_error!("The `traced_test` attribute should be used in place of `#[test]`, not alongside it.");
        }))
    } else {
        Ok(())
    }
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

pub(crate) fn generate_traced_block(
    function_returns_result: bool, 
    original_block: &syn::Block, 
    test_name: &str

) -> proc_macro2::TokenStream {

    quote!({

        use tracing::*;
        use std::panic::AssertUnwindSafe;

        let local_subscriber = setup_buffered_tracing(Some(#test_name));
        let local_subscriber_clone = local_subscriber.clone();
        let _guard = tracing::subscriber::set_default(local_subscriber.clone());

        let span = span!(Level::INFO, "test_trace", test_name = #test_name);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let outcome = span.in_scope(|| {
                #original_block
            });
            if let Err(_) = &outcome {
                local_subscriber_clone.flush();
            }
            outcome
        }));

        if result.is_err() {
            local_subscriber.flush();
        }
        result.unwrap()
    })
}

/// Retain the original attributes and remove the `test` attribute
pub(crate) fn all_attributes_except_test(function: &syn::ItemFn) -> Vec<syn::Attribute> {
    function.attrs.iter().filter(|attr| !attr.path().is_ident("test")).cloned().collect()
}

pub(crate) fn parse_or_compile_error(block: proc_macro2::TokenStream) -> syn::Block {
    syn::parse2(block).unwrap_or_else(|e| {
        panic!("Failed to parse block: {}", e);
    })
}
