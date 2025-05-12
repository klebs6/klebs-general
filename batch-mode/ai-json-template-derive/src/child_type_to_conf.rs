crate::ix!();

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
