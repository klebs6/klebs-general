// ---------------- [ File: ai-json-template-derive/src/extract_option_inner.rs ]
crate::ix!();

/// If `ty` is Option<T> (any path leading to `Option`), returns Some(&T). Else None.
pub fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        // e.g. std::option::Option
        if let Some(last) = type_path.path.segments.last() {
            if last.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = last.arguments {
                    if bracketed.args.len() == 1 {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                            tracing::trace!("Detected Option<{}>", quote!(#inner_ty).to_string());
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod test_extract_option_inner {
    use super::*;

    #[traced_test]
    fn test_extract_option_inner_with_option_of_string() {
        info!("Starting test_extract_option_inner_with_option_of_string");
        let ty: syn::Type = parse_quote! { Option<String> };
        trace!("Parsed type: {:?}", ty);

        let result = extract_option_inner(&ty);
        debug!("extract_option_inner result: {:?}", result);

        assert!(result.is_some(), "Expected Some(...) for Option<String>");
        let inner = result.unwrap();
        assert_eq!(quote::quote!(#inner).to_string(), "String", "Inner type should be 'String'");
        info!("Completed test_extract_option_inner_with_option_of_string");
    }

    #[traced_test]
    fn test_extract_option_inner_with_option_of_bool() {
        info!("Starting test_extract_option_inner_with_option_of_bool");
        let ty: syn::Type = parse_quote! { Option<bool> };
        trace!("Parsed type: {:?}", ty);

        let result = extract_option_inner(&ty);
        debug!("extract_option_inner result: {:?}", result);

        assert!(result.is_some(), "Expected Some(...) for Option<bool>");
        let inner = result.unwrap();
        assert_eq!(quote::quote!(#inner).to_string(), "bool", "Inner type should be 'bool'");
        info!("Completed test_extract_option_inner_with_option_of_bool");
    }

    #[traced_test]
    fn test_extract_option_inner_with_non_option_type() {
        info!("Starting test_extract_option_inner_with_non_option_type");
        let ty: syn::Type = parse_quote! { i32 };
        trace!("Parsed type: {:?}", ty);

        let result = extract_option_inner(&ty);
        debug!("extract_option_inner result: {:?}", result);

        assert!(result.is_none(), "Expected None for a non-Option type");
        info!("Completed test_extract_option_inner_with_non_option_type");
    }

    #[traced_test]
    fn test_extract_option_inner_with_nested_option() {
        info!("Starting test_extract_option_inner_with_nested_option");
        let ty: syn::Type = parse_quote! { Option<Option<u8>> };
        trace!("Parsed type: {:?}", ty);

        let first = extract_option_inner(&ty);
        debug!("extract_option_inner first call: {:?}", first);

        assert!(first.is_some(), "Expected Some(...) for Option<Option<u8>>");
        let inner = first.unwrap();

        let second = extract_option_inner(inner);
        debug!("extract_option_inner second call: {:?}", second);

        assert!(second.is_some(), "Expected Some(...) for the nested Option<u8>");
        let final_ty = second.unwrap();
        assert_eq!(quote::quote!(#final_ty).to_string(), "u8", "Innermost type should be 'u8'");
        info!("Completed test_extract_option_inner_with_nested_option");
    }

    #[traced_test]
    fn test_extract_option_inner_with_complex_generic_but_not_option() {
        info!("Starting test_extract_option_inner_with_complex_generic_but_not_option");
        let ty: syn::Type = parse_quote! { Vec<Option<String>> };
        trace!("Parsed type: {:?}", ty);

        let result = extract_option_inner(&ty);
        debug!("extract_option_inner result: {:?}", result);

        assert!(result.is_none(), "Expected None for a non-Option outer type");
        info!("Completed test_extract_option_inner_with_complex_generic_but_not_option");
    }
}
