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
