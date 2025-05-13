// ---------------- [ File: ai-json-template-derive/src/child_type_to_conf.rs ]
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

// ---------------- [ File: ai-json-template-derive/src/child_type_to_conf.rs ]
#[cfg(test)]
mod child_type_to_conf_tests {
    use super::*;

    #[traced_test]
    fn returns_correct_conf_type_for_simple_type() {
        let ty: Type = parse_quote!(MyFieldType);
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(MyFieldTypeConfidence);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected confidence type suffix for simple path"
        );
    }

    #[traced_test]
    fn returns_failure_type_for_complex_path() {
        let ty: Type = parse_quote!(crate::some::ComplexType);
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(__FlattenChildConfFail);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected fallback failure type for complex path"
        );
    }

    #[traced_test]
    fn returns_failure_type_for_tuple_type() {
        let ty: Type = parse_quote!((u32, String));
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(__FlattenChildConfFail);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected fallback failure type for tuple"
        );
    }

    #[traced_test]
    fn returns_failure_type_for_reference_type() {
        let ty: Type = parse_quote!(&str);
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(__FlattenChildConfFail);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected fallback failure type for reference"
        );
    }

    #[traced_test]
    fn returns_failure_type_for_array_type() {
        let ty: Type = parse_quote!([i32; 3]);
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(__FlattenChildConfFail);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected fallback failure type for array"
        );
    }

    #[traced_test]
    fn returns_failure_type_for_bare_function_type() {
        let ty: Type = parse_quote!(fn() -> i32);
        let result = child_ty_to_conf(&ty);

        let expected: Type = parse_quote!(__FlattenChildConfFail);
        assert_eq!(
            quote::quote!(#result).to_string(),
            quote::quote!(#expected).to_string(),
            "Expected fallback failure type for function pointer"
        );
    }
}
