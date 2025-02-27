// ---------------- [ File: hydro2-operator-derive/src/type_contains_lifetime_reference.rs ]
crate::ix!();

/// Return true if `ty` contains `&`.
pub fn contains_lifetime_reference(ty: &Type) -> bool {
    ty.to_token_stream().to_string().contains('&')
}

#[cfg(test)]
mod test_contains_lifetime_reference {
    use super::*;

    #[test]
    fn test_contains_lifetime_reference() {
        let ty_with_ref: syn::Type = parse_quote! { &[u32] };
        assert!(contains_lifetime_reference(&ty_with_ref));

        let ty_without_ref: syn::Type = parse_quote! { Vec<String> };
        assert!(!contains_lifetime_reference(&ty_without_ref));
    }

    //------------------------------------------------------------------
    // 1) Tests for contains_lifetime_reference
    //------------------------------------------------------------------
    #[test]
    fn test_contains_lifetime_reference_expanded() {
        // Already tested the basics, but let's add more coverage:
        let ty1: Type = parse_quote! { &str };
        let ty2: Type = parse_quote! { &&mut [u32] };
        let ty3: Type = parse_quote! { std::collections::HashMap<u32, String> };
        let ty4: Type = parse_quote! { fn(&mut T) -> bool };
        let ty5: Type = parse_quote! { (u32, &bool) };

        assert!(contains_lifetime_reference(&ty1));
        assert!(contains_lifetime_reference(&ty2));
        assert!(!contains_lifetime_reference(&ty3));
        // Even though `fn(&mut T)` has a reference in the signature, 
        // the textual test `to_string().contains('&')` will match it:
        assert!(contains_lifetime_reference(&ty4));
        // Likewise `(u32, &bool)` also has an ampersand:
        assert!(contains_lifetime_reference(&ty5));
    }
}
