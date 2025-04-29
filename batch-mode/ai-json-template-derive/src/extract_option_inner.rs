// ---------------- [ File: ai-json-template-derive/src/extract_option_inner.rs ]
crate::ix!();

/// If `ty` is Option<T> (any path leading to `Option`), returns Some(&T). Else None.
pub fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        // e.g. std::option::Option
        if let Some(last) = type_path.path.segments.last() {
            if last.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref bracketed) = last.arguments {
                    if bracketed.args.len() == 1 {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = bracketed.args.first() {
                            tracing::trace!("Detected Option<{}>", quote!(#inner_ty).to_string());
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}
