// ---------------- [ File: ai-json-template-derive/src/extract_hashmap_inner.rs ]
crate::ix!();

/// If `ty` is HashMap<K, V> (any path leading to `HashMap`), returns (K, V). Else None.
pub fn extract_hashmap_inner(ty: &syn::Type) -> Option<(&syn::Type, &syn::Type)> {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == "HashMap" {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = last.arguments {
                    if bracketed.args.len() == 2 {
                        let mut args_iter = bracketed.args.iter();
                        if let (Some(syn::GenericArgument::Type(k_ty)), Some(syn::GenericArgument::Type(v_ty))) =
                            (args_iter.next(), args_iter.next())
                        {
                            return Some((k_ty, v_ty));
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod test_extract_hashmap_inner {
    use super::*;

    #[traced_test]
    fn test_extracts_std_collections_hashmap_string_i32() {
        trace!("Starting test_extracts_std_collections_hashmap_string_i32");
        let ty: Type = parse_str("std::collections::HashMap<String, i32>")
            .expect("Failed to parse type for test_extracts_std_collections_hashmap_string_i32");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_some(),
            "Expected Some(..) for valid HashMap<String, i32>, got None"
        );
        let (k_ty, v_ty) = extracted.unwrap();
        debug!("Extracted key type: {:?}, value type: {:?}", k_ty, v_ty);
        info!("Finished test_extracts_std_collections_hashmap_string_i32 successfully");
    }

    #[traced_test]
    fn test_extracts_simple_hashmap_string_i32() {
        trace!("Starting test_extracts_simple_hashmap_string_i32");
        let ty: Type = parse_str("HashMap<String, i32>")
            .expect("Failed to parse type for test_extracts_simple_hashmap_string_i32");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_some(),
            "Expected Some(..) for valid HashMap<String, i32>, got None"
        );
        let (k_ty, v_ty) = extracted.unwrap();
        debug!("Extracted key type: {:?}, value type: {:?}", k_ty, v_ty);
        info!("Finished test_extracts_simple_hashmap_string_i32 successfully");
    }

    #[traced_test]
    fn test_extracts_custom_hashmap_types() {
        trace!("Starting test_extracts_custom_hashmap_types");
        let ty: Type = parse_str("std::collections::HashMap<MyKeyType, MyValueType>")
            .expect("Failed to parse type for test_extracts_custom_hashmap_types");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_some(),
            "Expected Some(..) for valid HashMap<MyKeyType, MyValueType>, got None"
        );
        let (k_ty, v_ty) = extracted.unwrap();
        debug!("Extracted key type: {:?}, value type: {:?}", k_ty, v_ty);
        info!("Finished test_extracts_custom_hashmap_types successfully");
    }

    #[traced_test]
    fn test_none_for_tuple_struct() {
        trace!("Starting test_none_for_tuple_struct");
        let ty: Type = parse_str("(i32, i32)")
            .expect("Failed to parse type for test_none_for_tuple_struct");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None for a non-HashMap type (tuple), got Some(..)"
        );
        info!("Finished test_none_for_tuple_struct successfully");
    }

    #[traced_test]
    fn test_none_for_vec_of_i32() {
        trace!("Starting test_none_for_vec_of_i32");
        let ty: Type = parse_str("Vec<i32>")
            .expect("Failed to parse type for test_none_for_vec_of_i32");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None for Vec<i32>, got Some(..)"
        );
        info!("Finished test_none_for_vec_of_i32 successfully");
    }

    #[traced_test]
    fn test_none_for_option_hashmap_string_i32() {
        trace!("Starting test_none_for_option_hashmap_string_i32");
        let ty: Type = parse_str("Option<HashMap<String, i32>>")
            .expect("Failed to parse type for test_none_for_option_hashmap_string_i32");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None for Option<HashMap<String, i32>>, got Some(..)"
        );
        info!("Finished test_none_for_option_hashmap_string_i32 successfully");
    }

    #[traced_test]
    fn test_none_for_wrong_number_of_generic_args() {
        trace!("Starting test_none_for_wrong_number_of_generic_args");
        let ty: Type = parse_str("HashMap<String>")
            .expect("Failed to parse type for test_none_for_wrong_number_of_generic_args");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_none(),
            "Expected None for HashMap with one generic, got Some(..)"
        );
        info!("Finished test_none_for_wrong_number_of_generic_args successfully");
    }

    #[traced_test]
    fn test_returns_some_for_hashmap_bool_i32() {
        trace!("Starting test_returns_some_for_hashmap_bool_i32");
        let ty: Type = parse_str("HashMap<bool, i32>")
            .expect("Failed to parse type for test_returns_some_for_hashmap_bool_i32");
        let extracted = extract_hashmap_inner(&ty);
        assert!(
            extracted.is_some(),
            "Expected Some(..) for valid HashMap<bool, i32>, got None"
        );
        let (k_ty, v_ty) = extracted.unwrap();
        debug!("Extracted key type: {:?}, value type: {:?}", k_ty, v_ty);
        info!("Finished test_returns_some_for_hashmap_bool_i32 successfully");
    }

    #[traced_test]
    fn test_none_for_extraneous_generics() {
        trace!("Starting test_none_for_extraneous_generics");
        // This is invalid Rust syntax, so we'll see if parse_str fails or if we handle it gracefully.
        let parsed = parse_str::<Type>("HashMap<String, i32, f64>");
        if parsed.is_ok() {
            let ty = parsed.unwrap();
            let extracted = extract_hashmap_inner(&ty);
            assert!(
                extracted.is_none(),
                "Expected None for HashMap with three generics, got Some(..)"
            );
            debug!("Parsed and got an unexpected Some(..) extraction");
        } else {
            debug!("Parsing HashMap<String, i32, f64> failed as expected. No extraction needed.");
        }
        info!("Finished test_none_for_extraneous_generics successfully");
    }
}
