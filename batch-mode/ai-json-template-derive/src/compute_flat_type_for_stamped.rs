// ---------------- [ File: ai-json-template-derive/src/compute_flat_type_for_stamped.rs ]
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

/// Computes the \"flat\" type used in our FlatJustifiedX structs or enums:
///  - bool → bool
///  - String → String
///  - i32 → i32
///  - Option<T> → Option<FlatJustifiedT>
///  - Vec<T> → Vec<FlatJustifiedT>
///  - HashMap<K,V> → HashMap<flat(K), flat(V)>
///  - SomeStruct → FlatJustifiedSomeStruct
///
/// If `skip_child_just` is true, we do *not* flatten the child type; we just use the original type.
pub fn compute_flat_type_for_stamped(
    original_ty: &syn::Type,
    skip_child_just: bool,
    span: proc_macro2::Span
) -> Result<proc_macro2::TokenStream, syn::Error> {
    use quote::quote;

    // If the user explicitly said `#[justify_inner=false]`, we skip flattening:
    if skip_child_just {
        return Ok(quote!(#original_ty));
    }

    if is_leaf_type(original_ty) {
        // It's a leaf => we won't produce `FlatJustified<child>`.
        // Just return "the same type" or something.
        return Ok(quote!( #original_ty ));
    }

    // Built-in scalars remain as-is:
    if is_bool(original_ty) || is_numeric(original_ty) || is_string_type(original_ty) {
        return Ok(quote!(#original_ty));
    }

    // Option<T> => Option<FlatJustifiedT>
    if let Some(inner) = extract_option_inner(original_ty) {
        let flattened_inner = compute_flat_type_for_stamped(inner, false, span)?;
        return Ok(quote!(::std::option::Option<#flattened_inner>));
    }

    // Vec<T> => Vec<FlatJustifiedT>
    if let Some(inner) = extract_vec_inner(original_ty) {
        let flattened_inner = compute_flat_type_for_stamped(inner, false, span)?;
        return Ok(quote!(::std::vec::Vec<#flattened_inner>));
    }

    // HashMap<K,V> => HashMap<flat(K), flat(V)>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(original_ty) {
        let flattened_k = compute_flat_type_for_stamped(k_ty, false, span)?;
        let flattened_v = compute_flat_type_for_stamped(v_ty, false, span)?;
        return Ok(quote!(::std::collections::HashMap<#flattened_k, #flattened_v>));
    }

    // Otherwise, we treat it as a user-defined type Foo -> FlatJustifiedFoo
    if let syn::Type::Path(tp) = original_ty {
        let mut new_path = tp.path.clone();
        if let Some(last_seg) = new_path.segments.last_mut() {
            let orig_ident = &last_seg.ident;
            let new_ident = syn::Ident::new(
                &format!("FlatJustified{}", orig_ident),
                span
            );
            last_seg.ident = new_ident;
        }
        let new_ty_path = syn::TypePath {
            qself: None,
            path: new_path,
        };
        return Ok(quote!(#new_ty_path));
    }

    // Fallback error
    Err(syn::Error::new(
        span,
        format!("Cannot flatten type: {:?}", quote!(#original_ty)),
    ))
}
