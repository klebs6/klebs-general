// ---------------- [ File: ai-json-template-derive/src/is_leaf_type.rs ]
crate::ix!();

/// Return true if `ty` should *not* be treated as a nested justification-enabled type.
/// For example, if `ty` is just a `String`, or a numeric, or a `HashMap<_, _>`, etc.
pub fn is_leaf_type(ty: &syn::Type) -> bool {
    // 1) If it's a simple path of "String", "u8", "bool", etc, return true.
    // 2) If it's "HashMap<..., ...>", also return true.
    // 3) Otherwise, by default, return false => might be a user-defined struct that DOES have justification.

    match ty {
        syn::Type::Path(type_path) => {
            let segs = &type_path.path.segments;
            if segs.len() == 1 {
                let ident = &segs[0].ident;
                let ident_s = ident.to_string();

                // e.g. "String", "bool", "f32", "HashMap", etc:
                match ident_s.as_str() {
                    "String" | "str" 
                    | "bool" 
                    | "u8" | "u16" | "u32" | "u64" 
                    | "i8" | "i16" | "i32" | "i64" 
                    | "f32" | "f64" => true,
                    "HashMap" => true, // or parse generics, if needed
                    _ => false, // e.g. "InnerPart" => not a leaf
                }
            } else {
                // multiple path segments? Possibly std::collections::HashMap
                // or user code: "crate::InnerPart"? So do more pattern matches if you want.
                // For demonstration, let's just do a quick hack:
                let last = &segs.last().unwrap().ident.to_string();
                match last.as_str() {
                    "HashMap" | "String" | "bool" 
                        | "u8" | "u16" | "u32" | "u64"
                        | "i8" | "i16" | "i32" | "i64"
                        | "f32" | "f64" => true,
                    _ => false,
                }
            }
        }
        // Option<T>?  If you want Option to be treated as leaf or not, up to you. 
        // For demonstration, let's just do:
        syn::Type::Reference(_) => true,
        syn::Type::BareFn(_) => true,
        // etc.
        _ => false,
    }
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
