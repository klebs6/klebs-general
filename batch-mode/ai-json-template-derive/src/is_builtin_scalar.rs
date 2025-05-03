// ---------------- [ File: ai-json-template-derive/src/is_builtin_scalar.rs ]
crate::ix!();

/// Checks if this type is a recognized "builtin scalar" like bool/numeric/String or
/// a known alias for String (e.g. LeafName, LeafDescriptor).
///
/// We do this by parsing the final path segment. If it matches one of our known
/// built-in or alias names, we return true.
pub fn is_builtin_scalar(ty: &syn::Type) -> bool {
    // A small set of known aliases for "String"
    const ALIAS_FOR_STRING: &[&str] = &[
        "String",
        "LeafName",
        "LeafDescriptor",
        "LeafHolderName",
    ];

    if let syn::Type::Path(tp) = ty {
        // If there's exactly one path segment, we check if it's in the known set:
        if tp.path.segments.len() == 1 {
            let seg = &tp.path.segments[0];
            let ident_str = seg.ident.to_string();

            // Check if it's a known built-in numeric
            if matches!(ident_str.as_str(),
                "bool"
                | "i8" | "i16" | "i32" | "i64"
                | "u8" | "u16" | "u32" | "u64"
                | "f32" | "f64")
            {
                return true;
            }

            // Check if it's in ALIAS_FOR_STRING => treat as "String"
            if ALIAS_FOR_STRING.contains(&ident_str.as_str()) {
                return true;
            }
        }
    }

    false
}

// ----------------------------------------------------------------------
//  Test Modules for subroutines
// ----------------------------------------------------------------------
#[cfg(test)]
mod test_subroutine_is_builtin_scalar {
    use super::*;

    #[traced_test]
    fn test_builtin_ok() {
        let t_bool: syn::Type = parse_quote!{ bool };
        assert!(is_builtin_scalar(&t_bool));
        let t_str: syn::Type = parse_quote!{ String };
        assert!(is_builtin_scalar(&t_str));
        let t_u8: syn::Type = parse_quote!{ u8 };
        assert!(is_builtin_scalar(&t_u8));

        let t_custom: syn::Type = parse_quote!{ MyCustom };
        assert!(!is_builtin_scalar(&t_custom));
    }
}
