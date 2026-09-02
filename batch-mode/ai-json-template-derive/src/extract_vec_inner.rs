// ---------------- [ File: ai-json-template-derive/src/extract_vec_inner.rs ]
crate::ix!();

/// If `ty` is Vec<T> (any path leading to `Vec`), returns Some(&T). Else None.
pub fn extract_vec_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = last.arguments {
                    if bracketed.args.len() == 1 {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
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
mod evaluate_extract_vec_inner {
    use super::*;

    /// Helper to compare two `syn::Type` for equality by comparing their token output.
    /// This is a convenient way to "test the interface" of what `extract_vec_inner` returns,
    /// without directly relying on private implementation details of `syn::Type`.
    fn types_match(left: &syn::Type, right: &syn::Type) -> bool {
        quote!(#left).to_string() == quote!(#right).to_string()
    }

    #[traced_test]
    fn returns_inner_for_vec_of_i32() {
        trace!("Testing extract_vec_inner with Vec<i32>");
        let ty: syn::Type = parse_quote!(Vec<i32>);
        let extracted = extract_vec_inner(&ty);
        match extracted {
            Some(inner) => {
                let expected: syn::Type = parse_quote!(i32);
                assert!(
                    types_match(inner, &expected),
                    "Expected inner type i32, got {}",
                    quote!(#inner).to_string()
                );
            }
            None => {
                panic!("Expected Some(i32) but got None");
            }
        }
    }

    #[traced_test]
    fn returns_inner_for_vec_of_option_string() {
        trace!("Testing extract_vec_inner with Vec<Option<String>>");
        let ty: syn::Type = parse_quote!(Vec<Option<String>>);
        let extracted = extract_vec_inner(&ty);
        match extracted {
            Some(inner) => {
                let expected: syn::Type = parse_quote!(Option<String>);
                assert!(
                    types_match(inner, &expected),
                    "Expected inner type Option<String>, got {}",
                    quote!(#inner).to_string()
                );
            }
            None => {
                panic!("Expected Some(Option<String>) but got None");
            }
        }
    }

    #[traced_test]
    fn returns_inner_for_vec_of_vec_u8() {
        trace!("Testing extract_vec_inner with Vec<Vec<u8>>");
        let ty: syn::Type = parse_quote!(Vec<Vec<u8>>);
        let extracted = extract_vec_inner(&ty);
        match extracted {
            Some(inner) => {
                let expected: syn::Type = parse_quote!(Vec<u8>);
                assert!(
                    types_match(inner, &expected),
                    "Expected inner type Vec<u8>, got {}",
                    quote!(#inner).to_string()
                );
            }
            None => {
                panic!("Expected Some(Vec<u8>) but got None");
            }
        }
    }

    #[traced_test]
    fn none_for_option_vec_u8() {
        trace!("Testing extract_vec_inner with Option<Vec<u8>>");
        let ty: syn::Type = parse_quote!(Option<Vec<u8>>);
        let extracted = extract_vec_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None but got Some({})",
            quote!(#extracted.unwrap()).to_string()
        );
    }

    #[traced_test]
    fn none_for_plain_string() {
        trace!("Testing extract_vec_inner with String");
        let ty: syn::Type = parse_quote!(String);
        let extracted = extract_vec_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None but got Some({})",
            quote!(#extracted.unwrap()).to_string()
        );
    }

    #[traced_test]
    fn returns_inner_for_std_vec_of_custom_struct() {
        trace!("Testing extract_vec_inner with std::vec::Vec<MyStruct>");
        let ty: syn::Type = parse_quote!(std::vec::Vec<MyStruct>);
        let extracted = extract_vec_inner(&ty);
        match extracted {
            Some(inner) => {
                let expected: syn::Type = parse_quote!(MyStruct);
                assert!(
                    types_match(inner, &expected),
                    "Expected inner type MyStruct, got {}",
                    quote!(#inner).to_string()
                );
            }
            None => {
                panic!("Expected Some(MyStruct) but got None");
            }
        }
    }

    #[traced_test]
    fn none_for_hashmap_of_string_vec_u8() {
        trace!("Testing extract_vec_inner with HashMap<String, Vec<u8>>");
        let ty: syn::Type = parse_quote!(std::collections::HashMap<String, Vec<u8>>);
        let extracted = extract_vec_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None but got Some({})",
            quote!(#extracted.unwrap()).to_string()
        );
    }

    #[traced_test]
    fn none_for_vec_with_multiple_generic_params() {
        trace!("Testing extract_vec_inner with Vec<i32, i64> (invalid in standard usage)");
        // This is syntactically invalid in real code, but let's see that we do not falsely parse it:
        // We'll artificially parse something that mimics a 2-arg generic to confirm we get None.
        let ty: syn::Type = parse_quote!(Vec<i32, i64>);
        let extracted = extract_vec_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None but got Some({})",
            quote!(#extracted.unwrap()).to_string()
        );
    }
}
