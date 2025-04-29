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
