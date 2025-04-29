// ---------------- [ File: ai-json-template-derive/src/is_builtin_scalar.rs ]
crate::ix!();

/// Stub: bool, numeric, string => "basic"
pub fn is_builtin_scalar(ty: &syn::Type) -> bool {
    // e.g. if type is bool, or i32, or u8, or f32, or String => return true
    // You can replicate your earlier checks from `classify_field_type`.
    let s = quote!(#ty).to_string();
    matches!(s.as_str(),
        "bool"
        | "i8" | "i16" | "i32" | "i64"
        | "u8" | "u16" | "u32" | "u64"
        | "f32" | "f64"
        | "String"
    )
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
