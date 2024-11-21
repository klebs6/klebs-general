crate::ix!();

// Helper function to check if a type is Option<T> and return the inner type if it is
pub fn is_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty {
        if path.segments.len() == 1 && path.segments[0].ident == "Option" {
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path.segments[0].arguments
            {
                if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                    return Some(inner_ty);
                }
            }
        }
    }
    None
}
