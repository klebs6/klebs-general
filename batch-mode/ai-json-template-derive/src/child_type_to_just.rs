crate::ix!();

/// If the child is a non-leaf custom type, we produce something like `ChildJustification`.
/// Otherwise we never call this at all. E.g. for `String` or numeric, we skip.
pub fn child_ty_to_just(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 {
            let original_ident = &tp.path.segments[0].ident;
            let just_ident = syn::Ident::new(
                &format!("{}Justification", original_ident),
                original_ident.span()
            );
            return syn::parse_quote!( #just_ident );
        }
    }
    syn::parse_quote! { __FlattenChildJustFail }
}

pub fn child_ty_to_conf(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 {
            let original_ident = &tp.path.segments[0].ident;
            let conf_ident = syn::Ident::new(
                &format!("{}Confidence", original_ident),
                original_ident.span()
            );
            return syn::parse_quote!( #conf_ident );
        }
    }
    syn::parse_quote! { __FlattenChildConfFail }
}
