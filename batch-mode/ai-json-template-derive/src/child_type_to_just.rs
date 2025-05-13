// ---------------- [ File: ai-json-template-derive/src/child_type_to_just.rs ]
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

#[cfg(test)]
mod verify_child_ty_to_just_exhaustive {
    use super::*;

    #[traced_test]
    fn verify_single_segment_path_type_identifiers() {
        trace!("Starting test: verify_single_segment_path_type_identifiers");
        // We'll try a few single-segment path types like Foo, String, i32, bool, etc.
        // Expectation: child_ty_to_just should append "Justification".
        
        // 1) A generic single-segment type Foo
        let ty_foo: Type = parse_quote!(Foo);
        debug!("Testing single-segment path: Foo => expecting FooJustification");
        let result = child_ty_to_just(&ty_foo);
        info!(?result, "child_ty_to_just(Foo) returned");
        assert_eq!(quote::quote!(#result).to_string(), "FooJustification");

        // 2) Built-in type i32
        let ty_i32: Type = parse_quote!(i32);
        debug!("Testing single-segment path: i32 => expecting i32Justification");
        let result = child_ty_to_just(&ty_i32);
        info!(?result, "child_ty_to_just(i32) returned");
        assert_eq!(quote::quote!(#result).to_string(), "i32Justification");

        // 3) Built-in bool
        let ty_bool: Type = parse_quote!(bool);
        debug!("Testing single-segment path: bool => expecting boolJustification");
        let result = child_ty_to_just(&ty_bool);
        info!(?result, "child_ty_to_just(bool) returned");
        assert_eq!(quote::quote!(#result).to_string(), "boolJustification");

        // 4) Standard library String
        let ty_string: Type = parse_quote!(String);
        debug!("Testing single-segment path: String => expecting StringJustification");
        let result = child_ty_to_just(&ty_string);
        info!(?result, "child_ty_to_just(String) returned");
        assert_eq!(quote::quote!(#result).to_string(), "StringJustification");
    }

    #[traced_test]
    fn verify_multi_segment_path_returns_fail() {
        trace!("Starting test: verify_multi_segment_path_returns_fail");
        // If the type path has more than one segment, we expect __FlattenChildJustFail.
        // For example: crate::my_mod::Foo
        let ty_multi: Type = parse_quote!(crate::my_mod::Foo);
        debug!("Testing multi-segment path: crate::my_mod::Foo => expecting __FlattenChildJustFail");
        let result = child_ty_to_just(&ty_multi);
        info!(?result, "child_ty_to_just(crate::my_mod::Foo) returned");
        assert_eq!(quote::quote!(#result).to_string(), "__FlattenChildJustFail");
    }

    #[traced_test]
    fn verify_non_path_types_return_fail() {
        trace!("Starting test: verify_non_path_types_return_fail");

        // A reference type &str or &u8:
        let ty_ref: Type = parse_quote!(&str);
        debug!("Testing reference type &str => expecting __FlattenChildJustFail");
        let result = child_ty_to_just(&ty_ref);
        info!(?result, "child_ty_to_just(&str) returned");
        assert_eq!(quote::quote!(#result).to_string(), "__FlattenChildJustFail");

        // A bare function type fn() -> i32:
        let ty_barefn: Type = parse_quote!(fn() -> i32);
        debug!("Testing bare function type => expecting __FlattenChildJustFail");
        let result = child_ty_to_just(&ty_barefn);
        info!(?result, "child_ty_to_just(fn() -> i32) returned");
        assert_eq!(quote::quote!(#result).to_string(), "__FlattenChildJustFail");

        // A tuple type (Foo, Bar):
        let ty_tuple: Type = parse_quote!((Foo, Bar));
        debug!("Testing tuple type => expecting __FlattenChildJustFail");
        let result = child_ty_to_just(&ty_tuple);
        info!(?result, "child_ty_to_just((Foo, Bar)) returned");
        assert_eq!(quote::quote!(#result).to_string(), "__FlattenChildJustFail");
    }
}
