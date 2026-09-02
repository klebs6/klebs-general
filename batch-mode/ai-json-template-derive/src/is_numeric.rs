// ---------------- [ File: ai-json-template-derive/src/is_numeric.rs ]
crate::ix!();

/// Check if numeric type (u8,u16,u32,u64,i8,i16,i32,i64,usize,isize,f32,f64).
pub fn is_numeric(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 {
            let ident_str = tp.path.segments[0].ident.to_string();
            return matches!(
                ident_str.as_str(),
                "i8" | "i16" | "i32" | "i64" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "usize"
                | "f32" | "f64"
            );
        }
    }
    false
}

/// Check if `bool`.
pub fn is_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.len() == 1 && tp.path.segments[0].ident == "bool"
    } else {
        false
    }
}

/// Check if `String`.
pub fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.len() == 1 && tp.path.segments[0].ident == "String"
    } else {
        false
    }
}
