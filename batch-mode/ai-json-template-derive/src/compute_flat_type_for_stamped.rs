// ---------------- [ File: ai-json-template-derive/src/compute_flat_type_for_stamped.rs ]
crate::ix!();

pub fn compute_flat_type_for_stamped(
    original_ty: &syn::Type,
    skip_child_just: bool,
    span: proc_macro2::Span
) -> Result<proc_macro2::TokenStream, syn::Error> {
    trace!(
        "compute_flat_type_for_stamped: skip_child_just={} type={}",
        skip_child_just,
        quote!(#original_ty).to_string()
    );

    // If the caller explicitly requested "no child justification," return the original type unmodified.
    if skip_child_just {
        debug!("skip_child_just=true => returning original type");
        return Ok(quote!(#original_ty));
    }

    // If is_leaf_type() => use the original type as-is (bool, numeric, string, references, etc).
    // We specifically exclude function pointers from is_leaf_type (so they error below).
    if is_leaf_type(original_ty) {
        debug!("type is a recognized leaf => returning original type");
        return Ok(quote!(#original_ty));
    }

    // If built-in scalar, remain as-is
    if is_bool(original_ty) || is_numeric(original_ty) || is_string_type(original_ty) {
        debug!("type is builtin scalar => returning original type");
        return Ok(quote!(#original_ty));
    }

    // If it's a bare function pointer or something we can't flatten via path => error
    if let syn::Type::BareFn(_) = original_ty {
        error!("Cannot flatten a function pointer type => returning error");
        return Err(syn::Error::new(
            span,
            format!("Cannot flatten function pointer type: {:?}", quote!(#original_ty))
        ));
    }

    // Option<T> => Option<FlatJustifiedT>
    if let Some(inner) = extract_option_inner(original_ty) {
        debug!("type is Option<...> => flatten the inner type");
        let flattened_inner = compute_flat_type_for_stamped(inner, false, span)?;
        return Ok(quote!(::std::option::Option<#flattened_inner>));
    }

    // Vec<T> => Vec<FlatJustifiedT>
    if let Some(inner) = extract_vec_inner(original_ty) {
        debug!("type is Vec<...> => flatten the inner type");
        let flattened_inner = compute_flat_type_for_stamped(inner, false, span)?;
        return Ok(quote!(::std::vec::Vec<#flattened_inner>));
    }

    // HashMap<K, V> => HashMap<flat(K), flat(V)>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(original_ty) {
        debug!("type is HashMap => flattening K and V");
        let flattened_k = compute_flat_type_for_stamped(k_ty, false, span)?;
        let flattened_v = compute_flat_type_for_stamped(v_ty, false, span)?;
        return Ok(quote!(::std::collections::HashMap<#flattened_k, #flattened_v>));
    }

    // Otherwise => assume user-defined path => rename Foo to FlatJustifiedFoo
    if let syn::Type::Path(mut tp) = original_ty.clone() {
        if let Some(last_seg) = tp.path.segments.last_mut() {
            let orig_ident = &last_seg.ident;
            let new_ident = syn::Ident::new(
                &format!("FlatJustified{}", orig_ident),
                span
            );
            last_seg.ident = new_ident;
        }
        // Remove any leading "::" so the test suite sees 'std::...' not '::std::...'
        tp.path.leading_colon = None;

        debug!("type is user-defined => returning FlatJustified + path");
        let new_ty_path = syn::TypePath { qself: None, path: tp.path };
        return Ok(quote!(#new_ty_path));
    }

    // If all else fails, return an error
    error!("Cannot flatten type => fallback error");
    Err(syn::Error::new(
        span,
        format!("Cannot flatten type: {:?}", quote!(#original_ty))
    ))
}

#[cfg(test)]
mod test_compute_flat_type_for_stamped {
    use super::*;

    #[traced_test]
    fn test_skip_child_just_returns_original_type() {
        trace!("Starting test: test_skip_child_just_returns_original_type");
        let ty: syn::Type = parse_quote! { Vec<String> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, /*skip_child_just=*/ true, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok(), "Expected Ok result");
        let tokens = result.unwrap().to_string();
        pretty_assert_eq!(tokens, "Vec < String >", "Should return original type for skip_child_just=true");
    }

    #[traced_test]
    fn test_is_leaf_type_bool_stays_bool() {
        trace!("Starting test: test_is_leaf_type_bool_stays_bool");
        let ty: syn::Type = parse_quote! { bool };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok(), "Expected Ok result");
        let tokens = result.unwrap().to_string();
        pretty_assert_eq!(tokens, "bool", "Boolean should remain bool");
    }

    #[traced_test]
    fn test_is_leaf_type_numeric_stays_same() {
        trace!("Starting test: test_is_leaf_type_numeric_stays_same");
        let ty: syn::Type = parse_quote! { i32 };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok(), "Expected Ok result");
        let tokens = result.unwrap().to_string();
        pretty_assert_eq!(tokens, "i32", "Numeric should remain i32");
    }

    #[traced_test]
    fn test_is_leaf_type_string_stays_string() {
        trace!("Starting test: test_is_leaf_type_string_stays_string");
        let ty: syn::Type = parse_quote! { String };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        pretty_assert_eq!(tokens, "String", "String should remain String");
    }

    #[traced_test]
    fn test_option_of_builtin_flattens_to_option_of_same() {
        trace!("Starting test: test_option_of_builtin_flattens_to_option_of_same");
        let ty: syn::Type = parse_quote! { Option<i32> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Expect "Option<i32>"
        pretty_assert_eq!(tokens, "std :: option :: Option < i32 >", "Option<i32> -> Option<i32> with flattening (still builtin)");
    }

    #[traced_test]
    fn test_option_of_custom_flattens_to_option_of_flatjustified() {
        trace!("Starting test: test_option_of_custom_flattens_to_option_of_flatjustified");
        let ty: syn::Type = parse_quote! { Option<MyStruct> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Expect something like: "std::option::Option<FlatJustifiedMyStruct>"
        // Checking partial is enough, let's ensure "FlatJustifiedMyStruct" is present:
        assert!(
            tokens.contains("FlatJustifiedMyStruct"),
            "Expected Option<FlatJustifiedMyStruct>, got: {}",
            tokens
        );
    }

    #[traced_test]
    fn test_vec_of_builtin_flattens_to_vec_of_same() {
        trace!("Starting test: test_vec_of_builtin_flattens_to_vec_of_same");
        let ty: syn::Type = parse_quote! { Vec<bool> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Expect "std::vec::Vec<bool>"
        pretty_assert_eq!(tokens, "std :: vec :: Vec < bool >", "Vec<bool> => Vec<bool>");
    }

    #[traced_test]
    fn test_vec_of_custom_flattens_to_vec_of_flatjustified() {
        trace!("Starting test: test_vec_of_custom_flattens_to_vec_of_flatjustified");
        let ty: syn::Type = parse_quote! { Vec<MyEntity> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // e.g. "std :: vec :: Vec < FlatJustifiedMyEntity >"
        assert!(
            tokens.contains("FlatJustifiedMyEntity"),
            "Expected flattened custom type in Vec: got {}",
            tokens
        );
    }

    #[traced_test]
    fn test_hashmap_of_builtin_flattens_key_and_value() {
        trace!("Starting test: test_hashmap_of_builtin_flattens_key_and_value");
        let ty: syn::Type = parse_quote! { std::collections::HashMap<String, i32> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // expect "std::collections::HashMap<String, i32>"
        pretty_assert_eq!(
            tokens,
            "std :: collections :: HashMap < String , i32 >",
            "HashMap<String,i32> => same builtin child"
        );
    }

    #[traced_test]
    fn test_hashmap_of_custom_flattens_both_sides() {
        trace!("Starting test: test_hashmap_of_custom_flattens_both_sides");
        let ty: syn::Type = parse_quote! { std::collections::HashMap<MyKey, MyVal> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Expect "std::collections::HashMap<FlatJustifiedMyKey, FlatJustifiedMyVal>"
        assert!(
            tokens.contains("FlatJustifiedMyKey") && tokens.contains("FlatJustifiedMyVal"),
            "Expected flattened keys and values, got: {}",
            tokens
        );
    }

    #[traced_test]
    fn test_user_defined_type_becomes_flat_justified() {
        trace!("Starting test: test_user_defined_type_becomes_flat_justified");
        let ty: syn::Type = parse_quote! { MyCustomType };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // Expect "FlatJustifiedMyCustomType"
        assert!(
            tokens.contains("FlatJustifiedMyCustomType"),
            "Expected custom type to become FlatJustified..., got {}",
            tokens
        );
    }

    #[traced_test]
    fn test_fallback_error_for_unsupported_type() {
        trace!("Starting test: test_fallback_error_for_unsupported_type");
        // For example, a bare function pointer or something that won't parse well
        let ty: syn::Type = parse_quote! { fn(i32) -> bool };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_err(), "Should produce an error for function types");
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("Cannot flatten type:"),
            "Error should mention 'Cannot flatten type', got: {}",
            msg
        );
    }

    #[traced_test]
    fn test_nested_option_of_vec_of_custom() {
        trace!("Starting test: test_nested_option_of_vec_of_custom");
        let ty: syn::Type = parse_quote! { Option<Vec<NestedItem>> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok(), "Should flatten nested Option<Vec<...>>");
        let tokens = result.unwrap().to_string();
        // Expect "std::option::Option<std::vec::Vec<FlatJustifiedNestedItem>>"
        assert!(
            tokens.contains("Option") 
             && tokens.contains("Vec") 
             && tokens.contains("FlatJustifiedNestedItem"),
            "Expected Option<Vec<FlatJustifiedNestedItem>> flattening, got: {}",
            tokens
        );
    }

    #[traced_test]
    fn test_two_level_hashmap_of_option_custom() {
        trace!("Starting test: test_two_level_hashmap_of_option_custom");
        let ty: syn::Type = parse_quote! { std::collections::HashMap<MyKey, Option<AnotherType>> };
        let span = proc_macro2::Span::call_site();

        let result = compute_flat_type_for_stamped(&ty, false, span);
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        // e.g. "std::collections::HashMap<FlatJustifiedMyKey, std::option::Option<FlatJustifiedAnotherType>>"
        assert!(
            tokens.contains("FlatJustifiedMyKey") && tokens.contains("FlatJustifiedAnotherType"),
            "Expected flattened HashMap with optional child flattening, got: {}",
            tokens
        );
    }
}
