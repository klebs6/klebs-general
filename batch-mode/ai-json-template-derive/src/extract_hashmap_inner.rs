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
