// ---------------- [ File: ai-json-template-derive/src/is_leaf_type.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn is_leaf_type(ty: &syn::Type) -> bool {
    trace!("Evaluating if type is considered a leaf: {:?}", ty);

    // 1) If `Option<T>` => treat as non-leaf
    if crate::extract_option_inner(ty).is_some() {
        trace!("Type is Option<T>, treating as non-leaf");
        return false;
    }

    // 2) If `Vec<T>` => treat as non-leaf
    if crate::extract_vec_inner(ty).is_some() {
        trace!("Type is Vec<T>, treating as non-leaf");
        return false;
    }

    // 3) If `HashMap<K, V>` => treat as leaf (test suite says so)
    if crate::extract_hashmap_inner(ty).is_some() {
        trace!("Type is HashMap<K, V>, treating as leaf");
        return true;
    }

    // 4) If reference => treat as leaf (the tests say &str is leaf)
    if let syn::Type::Reference(_) = ty {
        trace!("Type is a reference, treating as leaf");
        return true;
    }

    // 5) If bare function pointer => leaf
    if let syn::Type::BareFn(_) = ty {
        trace!("Type is a bare function pointer, treating as leaf");
        return true;
    }

    // 6) If it’s a `Type::Path`, check the last segment. If it's a known built-in leaf type, return true.
    //    Otherwise return false (likely a user-defined type).
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_seg) = type_path.path.segments.last() {
            let ident_s = last_seg.ident.to_string();
            trace!("Type::Path => last segment: {}", ident_s);

            match ident_s.as_str() {
                // The tests specifically want `str` to be leaf:
                "str"
                // "String", "bool", numeric
                | "String" | "bool"
                | "u8" | "u16" | "u32" | "u64"
                | "i8" | "i16" | "i32" | "i64"
                | "f32" | "f64"
                // Also allow trailing "HashMap" as a leaf, e.g. `my::outer::HashMap`.
                | "HashMap" => {
                    trace!("Matched known built-in leaf type: {}", ident_s);
                    return true;
                }
                _ => {
                    trace!("Type '{}' is not recognized as a built-in leaf, treating as non-leaf", ident_s);
                    return false;
                }
            }
        }
    }

    trace!("Type does not match any known leaf criteria, treating as non-leaf");
    false
}

#[cfg(test)]
mod test_is_leaf_type {
    use super::*;

    #[traced_test]
    fn test_is_leaf_type_bool() {
        trace!("Starting test_is_leaf_type_bool");
        let ty: Type = parse_quote!(bool);
        let result = is_leaf_type(&ty);
        debug!("For type=bool, expected=true, got={}", result);
        assert!(result, "bool should be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_i32() {
        trace!("Starting test_is_leaf_type_i32");
        let ty: Type = parse_quote!(i32);
        let result = is_leaf_type(&ty);
        debug!("For type=i32, expected=true, got={}", result);
        assert!(result, "i32 should be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_f64() {
        trace!("Starting test_is_leaf_type_f64");
        let ty: Type = parse_quote!(f64);
        let result = is_leaf_type(&ty);
        debug!("For type=f64, expected=true, got={}", result);
        assert!(result, "f64 should be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_string() {
        trace!("Starting test_is_leaf_type_string");
        let ty: Type = parse_quote!(String);
        let result = is_leaf_type(&ty);
        debug!("For type=String, expected=true, got={}", result);
        assert!(result, "String should be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_reference() {
        trace!("Starting test_is_leaf_type_reference");
        let ty: Type = parse_quote!(&str);
        let result = is_leaf_type(&ty);
        debug!("For type=&str, expected=true, got={}", result);
        assert!(result, "&str should be considered a leaf type by current logic");
    }

    #[traced_test]
    fn test_is_leaf_type_hashmap_generic() {
        trace!("Starting test_is_leaf_type_hashmap_generic");
        let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
        let result = is_leaf_type(&ty);
        debug!("For type=HashMap<String, i32>, expected=true, got={}", result);
        assert!(result, "HashMap should be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_user_defined_struct() {
        trace!("Starting test_is_leaf_type_user_defined_struct");
        // Simulate a user-defined struct type
        let ty: Type = parse_quote!(MyCustomStruct);
        let result = is_leaf_type(&ty);
        debug!("For type=MyCustomStruct, expected=false, got={}", result);
        assert!(!result, "User-defined struct should NOT be considered a leaf type");
    }

    #[traced_test]
    fn test_is_leaf_type_user_defined_modded_struct() {
        trace!("Starting test_is_leaf_type_user_defined_modded_struct");
        // Something with multiple segments, but not recognized as a builtin
        let ty: Type = parse_quote!(crate::some_module::CustomThing);
        let result = is_leaf_type(&ty);
        debug!("For type=crate::some_module::CustomThing, expected=false, got={}", result);
        assert!(
            !result,
            "Any path ending with 'CustomThing' not recognized as builtin should NOT be a leaf"
        );
    }

    #[traced_test]
    fn test_is_leaf_type_bare_fn() {
        trace!("Starting test_is_leaf_type_bare_fn");
        let ty: Type = parse_quote!(fn(i32) -> bool);
        let result = is_leaf_type(&ty);
        debug!("For type=fn(i32)->bool, expected=true, got={}", result);
        assert!(
            result,
            "Function pointer types are currently treated as leaf by the implementation"
        );
    }

    #[traced_test]
    fn test_is_leaf_type_generic_path_but_not_hashmap() {
        trace!("Starting test_is_leaf_type_generic_path_but_not_hashmap");
        // This ensures any generic path not recognized explicitly should return false
        let ty: Type = parse_quote!(std::vec::Vec<String>);
        let result = is_leaf_type(&ty);
        debug!("For type=Vec<String>, expected=false, got={}", result);
        assert!(
            !result,
            "A generic path like Vec<String> is not recognized as a leaf type in the current logic"
        );
    }

    #[traced_test]
    fn test_is_leaf_type_unknown_builtin_like_strslice() {
        trace!("Starting test_is_leaf_type_unknown_builtin_like_strslice");
        // This tests if something that looks built-in but isn't exactly known is recognized
        let ty: Type = parse_quote!(str);
        let result = is_leaf_type(&ty);
        debug!("For type=str, expected=true, got={}", result);
        assert!(
            result,
            "Implementation explicitly treats str as true (leaf) per match"
        );
    }

    #[traced_test]
    fn test_is_leaf_type_trailing_path_segment_recognized() {
        trace!("Starting test_is_leaf_type_trailing_path_segment_recognized");
        // Checking if the last segment is recognized => "HashMap"
        let ty: Type = parse_quote!(my::outer::HashMap);
        let result = is_leaf_type(&ty);
        debug!("For type=my::outer::HashMap, expected=true, got={}", result);
        assert!(
            result,
            "Implementation checks the last path segment for built-in type naming"
        );
    }

    #[traced_test]
    fn test_is_leaf_type_trailing_path_segment_unrecognized() {
        trace!("Starting test_is_leaf_type_trailing_path_segment_unrecognized");
        let ty: Type = parse_quote!(std::somewhere::NotKnown);
        let result = is_leaf_type(&ty);
        debug!("For type=std::somewhere::NotKnown, expected=false, got={}", result);
        assert!(
            !result,
            "Implementation should return false if the last path segment is not recognized"
        );
    }
}
