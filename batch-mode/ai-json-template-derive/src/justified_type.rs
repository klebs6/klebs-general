crate::ix!();

//TODO: maybe want to clean this up and test it
pub fn justified_type(ty: &syn::Type) -> TokenStream2 {

    if let Some(inner) = extract_option_inner(ty) {

        if is_leaf_type(&inner) {
            return ty.to_token_stream();
        }

        let justified_inner = syn::Ident::new(
            &format!("Justified{}", quote!{ #inner }.to_string()),
            ty.span()
        );
        return quote!{ Option < #justified_inner > };
    }

    if let Some(inner) = extract_vec_inner(ty) {

        if is_leaf_type(&inner) {
            return ty.to_token_stream();
        }

        let justified_inner = syn::Ident::new(
            &format!("Justified{}", quote!{ #inner }.to_string()),
            ty.span()
        );
        return quote!{ Vec < #justified_inner > };
    }

    if is_leaf_type(ty) {
        return ty.to_token_stream();
    }

    syn::Ident::new(
        &format!("Justified{}", quote!{ #ty }.to_string()),
        ty.span()
    ).to_token_stream()
}


