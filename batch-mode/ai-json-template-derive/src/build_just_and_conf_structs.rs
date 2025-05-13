// ---------------- [ File: ai-json-template-derive/src/build_just_and_conf_structs.rs ]
crate::ix!();

pub fn build_just_and_conf_structs(
    justification_ident:  &syn::Ident,
    confidence_ident:     &syn::Ident,
    errs:                 &proc_macro2::TokenStream,
    justification_fields: &[proc_macro2::TokenStream],
    confidence_fields:    &[proc_macro2::TokenStream],

) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {

    trace!(
        "Building justification/conf structs: '{}' and '{}'",
        justification_ident,
        confidence_ident
    );

    let just_ts = quote::quote! {
        #errs
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justification_ident {
            #(#justification_fields)*
        }
    };

    let conf_ts = quote::quote! {
        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #confidence_ident {
            #(#confidence_fields)*
        }
    };

    debug!(
        "Finished building struct tokens for '{}' and '{}'",
        justification_ident, confidence_ident
    );
    (just_ts, conf_ts)
}

#[cfg(test)]
mod test_build_just_and_conf_structs_exhaustively {
    use super::*;
    use quote::ToTokens;
    use syn::{parse2, File, Item, ItemMacro, ItemStruct, Field, Attribute, Meta, NestedMeta};
    use tracing::{trace, debug, info};

    /// Extract the single `ItemStruct` with the given name from the parsed File.
    /// Panics if not found.
    fn find_struct_by_name(ast: &File, struct_name: &str) -> &ItemStruct {
        for item in &ast.items {
            if let Item::Struct(s) = item {
                if s.ident == struct_name {
                    return s;
                }
            }
        }
        panic!("Did not find a `struct {}` in the generated tokens!", struct_name);
    }

    /// Returns `true` if we find a top-level `compile_error!( ... )` macro in the File.
    fn has_compile_error_top_level(ast: &File, expected_msg: &str) -> bool {
        for item in &ast.items {
            if let Item::Macro(ItemMacro { mac, .. }) = item {
                // e.g. `compile_error!("some message")`
                if mac.path.is_ident("compile_error") {
                    let tokens_str = mac.tokens.to_string();
                    // Just check if it contains the expected message substring:
                    if tokens_str.contains(expected_msg) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Checks that the `#[derive(...)]` attribute on a struct includes the desired traits.
    /// For example, we want to ensure `#[derive(Builder, Debug, Clone, ...)]` appears.
    fn assert_derive_traits(attrs: &[Attribute], struct_name: &str, required_traits: &[&str]) {
        // We'll look for an attribute of the form `#[derive(...)]`.
        // Then we parse out the nested meta items (Builder, Debug, etc.).
        // If any trait is missing, we'll panic.
        let mut found_any_derive = false;
        let mut found_traits = vec![];

        for attr in attrs {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                if meta_list.path.is_ident("derive") {
                    found_any_derive = true;
                    // Extract tokens like (Builder, Debug, Clone, ...)
                    for nested in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::Path(path)) = nested {
                            if let Some(ident) = path.get_ident() {
                                found_traits.push(ident.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Basic check: we expect at least one `#[derive(...)]` with some items
        assert!(
            found_any_derive,
            "No #[derive(...)] attribute found on struct {}",
            struct_name
        );

        // Now verify each required trait is in found_traits
        for &trait_name in required_traits {
            assert!(
                found_traits.iter().any(|f| f == trait_name),
                "Expected trait '{}' in #[derive(...)] for struct '{}'. Found: {:?}",
                trait_name, struct_name, found_traits
            );
        }
    }

    /// Helper to confirm that a struct has the given fields (as `(name, type_string)`).
    /// We do a best-effort check: compare the `ident` and the type's string form.
    fn assert_struct_fields(
        item_struct: &ItemStruct,
        expected_fields: &[(&str, &str)],
        struct_name: &str,
    ) {
        let actual_fields = match &item_struct.fields {
            syn::Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => panic!("Struct {} is not a named struct with braces!", struct_name),
        };

        assert_eq!(
            actual_fields.len(),
            expected_fields.len(),
            "Mismatch in field count for struct {}.\nExpected {}, found {}",
            struct_name,
            expected_fields.len(),
            actual_fields.len()
        );

        // For each expected field, ensure there's a matching field in `actual_fields`.
        for (i, (exp_name, exp_ty)) in expected_fields.iter().enumerate() {
            let Field { ident, ty, .. } = actual_fields[i];
            let found_name = ident.as_ref().map(|id| id.to_string()).unwrap_or_default();
            assert_eq!(
                &found_name, exp_name,
                "Field name mismatch in struct '{}'.\nExpected '{}', found '{}'",
                struct_name, exp_name, found_name
            );
            let ty_str = ty.to_token_stream().to_string();
            // We do a substring check for the type, or an exact match if we prefer.
            assert!(
                ty_str.contains(exp_ty),
                "Field '{}' in struct '{}' expected type containing '{}', got '{}'",
                exp_name,
                struct_name,
                exp_ty,
                ty_str
            );
        }
    }

    // -------------------------------------------------------------------
    // Below are the four tests from your code, reworked to parse the AST.
    // -------------------------------------------------------------------

    #[traced_test]
    fn test_empty_fields_no_error_tokens() {
        info!("Starting test_empty_fields_no_error_tokens");
        let justification_ident = syn::Ident::new("EmptyJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("EmptyConfidence",    proc_macro2::Span::call_site());

        let errs                = quote::quote! {};
        let justification_fields = vec![];
        let confidence_fields    = vec![];

        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        // Parse them.
        let just_ast: File = parse2(just_ts.clone())
            .expect("Justification tokens should parse successfully");
        let conf_ast: File = parse2(conf_ts.clone())
            .expect("Confidence tokens should parse successfully");

        // 1) Justification
        let just_struct = find_struct_by_name(&just_ast, "EmptyJustification");
        assert_struct_fields(just_struct, &[], "EmptyJustification");
        assert_derive_traits(
            &just_struct.attrs,
            "EmptyJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // Confirm no compile_error top-level in justification
        assert!(!has_compile_error_top_level(&just_ast, ""), "Did not expect compile_error in justification snippet");

        // 2) Confidence
        let conf_struct = find_struct_by_name(&conf_ast, "EmptyConfidence");
        assert_struct_fields(conf_struct, &[], "EmptyConfidence");
        assert_derive_traits(
            &conf_struct.attrs,
            "EmptyConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // Confirm no compile_error top-level in confidence
        assert!(!has_compile_error_top_level(&conf_ast, ""), "Did not expect compile_error in confidence snippet");
    }

    #[traced_test]
    fn test_populated_fields_no_error_tokens() {
        info!("Starting test_populated_fields_no_error_tokens");
        let justification_ident = syn::Ident::new("PopulatedJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("PopulatedConfidence",    proc_macro2::Span::call_site());

        let errs = quote::quote! {};
        let justification_fields = vec![
            quote::quote! { some_field: String, },
            quote::quote! { another_field: i32, },
        ];
        let confidence_fields = vec![
            quote::quote! { conf_val: f32, },
            quote::quote! { alpha_level: f64, },
        ];

        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        // Parse
        let just_ast: File = parse2(just_ts.clone())
            .expect("Justification tokens should parse successfully");
        let conf_ast: File = parse2(conf_ts.clone())
            .expect("Confidence tokens should parse successfully");

        // 1) Justification
        let just_struct = find_struct_by_name(&just_ast, "PopulatedJustification");
        // We expect 2 fields: (some_field -> String) and (another_field -> i32)
        let expected_just_fields = &[
            ("some_field", "String"),
            ("another_field", "i32"),
        ];
        assert_struct_fields(just_struct, expected_just_fields, "PopulatedJustification");
        assert_derive_traits(
            &just_struct.attrs,
            "PopulatedJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // No compile_error top-level
        assert!(!has_compile_error_top_level(&just_ast, "Simulated error"), "Should be no error tokens in justification");

        // 2) Confidence
        let conf_struct = find_struct_by_name(&conf_ast, "PopulatedConfidence");
        let expected_conf_fields = &[
            ("conf_val", "f32"),
            ("alpha_level", "f64"),
        ];
        assert_struct_fields(conf_struct, expected_conf_fields, "PopulatedConfidence");
        assert_derive_traits(
            &conf_struct.attrs,
            "PopulatedConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        assert!(!has_compile_error_top_level(&conf_ast, "Simulated error"), "Should be no error tokens in confidence");
    }

    #[traced_test]
    fn test_with_error_tokens() {
        info!("Starting test_with_error_tokens");
        let justification_ident = syn::Ident::new("ErrorfulJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("ErrorfulConfidence",    proc_macro2::Span::call_site());

        let errs = quote::quote! {
            compile_error!("Simulated error from prior step.");
        };
        let justification_fields = vec![
            quote::quote! { j1: String, },
        ];
        let confidence_fields = vec![
            quote::quote! { c1: i64, },
        ];

        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        // Parse
        let just_ast: File = parse2(just_ts.clone())
            .expect("Should parse justification even with compile_error");
        let conf_ast: File = parse2(conf_ts.clone())
            .expect("Should parse confidence even with compile_error");

        // Check compile_error presence in justification snippet
        assert!(
            has_compile_error_top_level(&just_ast, "Simulated error from prior step."),
            "Should have compile_error! with message in justification snippet"
        );
        // The struct itself should also exist
        let just_struct = find_struct_by_name(&just_ast, "ErrorfulJustification");
        let expected_just_fields = &[("j1", "String")];
        assert_struct_fields(just_struct, expected_just_fields, "ErrorfulJustification");
        assert_derive_traits(
            &just_struct.attrs,
            "ErrorfulJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );

        // Confidence
        // It's possible compile_error only appears once in the final tokens, 
        // but let's check if the code includes it for confidence, or not.
        let conf_struct = find_struct_by_name(&conf_ast, "ErrorfulConfidence");
        let expected_conf_fields = &[("c1", "i64")];
        assert_struct_fields(conf_struct, expected_conf_fields, "ErrorfulConfidence");
        assert_derive_traits(
            &conf_struct.attrs,
            "ErrorfulConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // There's only one `errs` insertion, so it will appear in justification snippet. 
        // But if your code duplicates `errs` for both, you could do:
        // assert!(has_compile_error_top_level(&conf_ast, "Simulated error from prior step."), "...");

        info!("Finished test_with_error_tokens");
    }

    #[traced_test]
    fn test_minimal_overall_validity() {
        info!("Starting test_minimal_overall_validity");
        let justification_ident = syn::Ident::new("MixedJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("MixedConfidence",    proc_macro2::Span::call_site());

        let errs = quote::quote! {
            compile_error!("Mixed scenario error");
        };
        let justification_fields = vec![
            quote::quote! { field_a: String, },
            quote::quote! { field_b: bool, },
        ];
        let confidence_fields = vec![
            quote::quote! { field_x: f32, },
            quote::quote! { field_y: u64, },
        ];

        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        // Parse
        let just_ast: File = parse2(just_ts.clone())
            .expect("Should parse the justification snippet in a mixed scenario");
        let conf_ast: File = parse2(conf_ts.clone())
            .expect("Should parse the confidence snippet in a mixed scenario");

        // Justification
        assert!(
            has_compile_error_top_level(&just_ast, "Mixed scenario error"),
            "Should include error tokens in justification output"
        );
        let just_struct = find_struct_by_name(&just_ast, "MixedJustification");
        let exp_just = &[("field_a","String"), ("field_b","bool")];
        assert_struct_fields(just_struct, exp_just, "MixedJustification");
        assert_derive_traits(
            &just_struct.attrs,
            "MixedJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );

        // Confidence
        let conf_struct = find_struct_by_name(&conf_ast, "MixedConfidence");
        let exp_conf = &[("field_x","f32"), ("field_y","u64")];
        assert_struct_fields(conf_struct, exp_conf, "MixedConfidence");
        assert_derive_traits(
            &conf_struct.attrs,
            "MixedConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // If you also expect the compile_error in confidence snippet, check that too:
        // assert!(has_compile_error_top_level(&conf_ast, "Mixed scenario error"), "...");

        info!("Finished test_minimal_overall_validity");
    }
}
