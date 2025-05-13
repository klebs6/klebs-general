// ---------------- [ File: ai-json-template-derive/src/build_just_and_conf_structs.rs ]
crate::ix!();

pub fn build_just_and_conf_structs(
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
    errs: &proc_macro2::TokenStream,
    justification_fields: &[proc_macro2::TokenStream],
    confidence_fields: &[proc_macro2::TokenStream],
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
    use syn::{
        parse2, File, Item, ItemMacro, ItemStruct, Field, Attribute, Meta,
        Fields, Type, Visibility,
    };
    use tracing::{trace, debug, info};

    /// Return a reference to the `ItemStruct` with the given name from the file's AST.
    /// Panics if not found.
    fn find_struct_by_name<'a>(ast: &'a File, struct_name: &str) -> &'a ItemStruct {
        for item in &ast.items {
            if let Item::Struct(s) = item {
                if s.ident == struct_name {
                    return s;
                }
            }
        }
        panic!("Did not find a `struct {}` in the generated tokens!", struct_name);
    }

    /// Returns `true` if there's a top-level `compile_error!(...)` macro containing `expected_msg`.
    fn has_compile_error_top_level(ast: &File, expected_msg: &str) -> bool {
        for item in &ast.items {
            if let Item::Macro(ItemMacro { mac, .. }) = item {
                // For example: `compile_error!("some message")`
                if mac.path.is_ident("compile_error") {
                    let token_str = mac.tokens.to_string();
                    if token_str.contains(expected_msg) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Parse the `#[derive(...)]` attribute from a struct to see if it includes all required traits.
    /// In syn 2.x, `Attribute` has `meta: Option<Meta>`. If `meta` is `Some(Meta::List(list))`,
    /// we can parse nested metas using `list.parse_nested_meta(...)`.
    fn assert_derive_traits(attrs: &[syn::Attribute], struct_name: &str, required_traits: &[&str]) {
        use syn::{Meta, MetaList};
        let mut found_any_derive = false;
        let mut found_traits: Vec<String> = Vec::new();

        for attr in attrs {
            // `attr.meta` is Option<Meta> in syn 2.x
            if let Some(meta) = &attr.meta {
                // e.g. #[derive(...)]
                if let Meta::List(meta_list) = meta {
                    // Check if it's actually `#[derive(...)]`
                    if meta_list.path.is_ident("derive") {
                        found_any_derive = true;

                        // Use parse_nested_meta to get each nested item, e.g. Builder, Debug, etc.
                        meta_list.parse_nested_meta(|nested| {
                            // `nested` is a ParseNestedMeta struct in syn 2.x
                            // If you want the path ident, do:
                            if let Some(ident) = nested.path.get_ident() {
                                found_traits.push(ident.to_string());
                            }
                            Ok(())
                        }).ok(); // Ignore errors
                    }
                }
            }
        }

        assert!(
            found_any_derive,
            "No #[derive(...)] attribute found on struct {struct_name}"
        );

        for &trait_name in required_traits {
            assert!(
                found_traits.iter().any(|t| t == trait_name),
                "Trait '{trait_name}' not found in #[derive(...)] for struct {struct_name}. Found: {found_traits:?}"
            );
        }
    }

    /// Confirm that a struct has exactly `expected_fields.len()` fields,
    /// each with the matching `(name, type_substring)` pair.
    fn assert_struct_fields(
        item_struct: &ItemStruct,
        expected_fields: &[(&str, &str)],
        struct_name: &str,
    ) {
        let Fields::Named(named) = &item_struct.fields else {
            panic!("Struct {} does not have named fields!", struct_name);
        };

        let actual = named.named.iter().collect::<Vec<_>>();
        assert_eq!(
            actual.len(),
            expected_fields.len(),
            "Expected {} fields in struct {}, found {}",
            expected_fields.len(),
            struct_name,
            actual.len()
        );

        for (i, (exp_name, exp_type)) in expected_fields.iter().enumerate() {
            let Field { ident, ty, vis, .. } = actual[i];
            let name_found = ident.as_ref().map(|x| x.to_string()).unwrap_or_default();
            assert_eq!(
                name_found, *exp_name,
                "Expected field name '{}' but found '{}'",
                exp_name, name_found
            );
            // Check the type substring
            let type_str = ty.to_token_stream().to_string();
            assert!(
                type_str.contains(exp_type),
                "Field '{}' in struct '{}': expected type containing '{}', got '{}'",
                exp_name, struct_name, exp_type, type_str
            );
            // In your code you said "we do not want pub fields". If so, check the Visibility:
            match vis {
                Visibility::Inherited => { /* OK, it's not pub */ }
                _ => panic!("Field '{}' in {} is not inherited (private). Found: {:?}", exp_name, struct_name, vis),
            }
        }
    }

    // --------------------------------------------------
    //  TESTS
    // --------------------------------------------------

    #[traced_test]
    fn test_empty_fields_no_error_tokens() {
        info!("test_empty_fields_no_error_tokens");
        let justification_ident = syn::Ident::new("EmptyJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("EmptyConfidence",    proc_macro2::Span::call_site());

        let errs = quote::quote! {};
        let justification_fields = vec![];
        let confidence_fields    = vec![];

        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        let just_ast: File = parse2(just_ts.clone())
            .expect("Justification tokens parse error");
        let conf_ast: File = parse2(conf_ts.clone())
            .expect("Confidence tokens parse error");

        // Justification struct => no fields
        let js_struct = find_struct_by_name(&just_ast, "EmptyJustification");
        assert_struct_fields(js_struct, &[], "EmptyJustification");
        assert_derive_traits(
            &js_struct.attrs,
            "EmptyJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // No compile_error
        assert!(!has_compile_error_top_level(&just_ast, ""), "Found unexpected compile_error in justification snippet!");

        // Confidence struct => no fields
        let cf_struct = find_struct_by_name(&conf_ast, "EmptyConfidence");
        assert_struct_fields(cf_struct, &[], "EmptyConfidence");
        assert_derive_traits(
            &cf_struct.attrs,
            "EmptyConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        assert!(!has_compile_error_top_level(&conf_ast, ""), "Found unexpected compile_error in confidence snippet!");
    }

    #[traced_test]
    fn test_populated_fields_no_error_tokens() {
        info!("test_populated_fields_no_error_tokens");
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

        let just_ast: File = parse2(just_ts.clone()).expect("Justification parse");
        let conf_ast: File = parse2(conf_ts.clone()).expect("Confidence parse");

        // Justification
        let js_struct = find_struct_by_name(&just_ast, "PopulatedJustification");
        assert_struct_fields(
            js_struct,
            &[
                ("some_field", "String"),
                ("another_field", "i32"),
            ],
            "PopulatedJustification"
        );
        assert_derive_traits(
            &js_struct.attrs,
            "PopulatedJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        assert!(!has_compile_error_top_level(&just_ast, "Simulated"), "Unexpected compile_error in justification snippet!");

        // Confidence
        let cf_struct = find_struct_by_name(&conf_ast, "PopulatedConfidence");
        assert_struct_fields(
            cf_struct,
            &[
                ("conf_val", "f32"),
                ("alpha_level", "f64"),
            ],
            "PopulatedConfidence"
        );
        assert_derive_traits(
            &cf_struct.attrs,
            "PopulatedConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        assert!(!has_compile_error_top_level(&conf_ast, "Simulated"), "Unexpected compile_error in confidence snippet!");
    }

    #[traced_test]
    fn test_with_error_tokens() {
        info!("test_with_error_tokens");
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

        let just_ast: File = parse2(just_ts.clone()).expect("Justification parse");
        let conf_ast: File = parse2(conf_ts.clone()).expect("Confidence parse");

        // Justification: should have compile_error
        assert!(
            has_compile_error_top_level(&just_ast, "Simulated error from prior step."),
            "Expected compile_error in justification snippet"
        );
        let js_struct = find_struct_by_name(&just_ast, "ErrorfulJustification");
        assert_struct_fields(js_struct, &[("j1","String")], "ErrorfulJustification");
        assert_derive_traits(
            &js_struct.attrs,
            "ErrorfulJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );

        // Confidence: If your code re-injects the same errs, it also contains compile_error. 
        // Or maybe not: check if it does, or not. We'll skip for demonstration:
        let cf_struct = find_struct_by_name(&conf_ast, "ErrorfulConfidence");
        assert_struct_fields(cf_struct, &[("c1","i64")], "ErrorfulConfidence");
        assert_derive_traits(
            &cf_struct.attrs,
            "ErrorfulConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
    }

    #[traced_test]
    fn test_minimal_overall_validity() {
        info!("test_minimal_overall_validity");
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

        let just_ast: File = parse2(just_ts.clone()).expect("Just parse");
        let conf_ast: File = parse2(conf_ts.clone()).expect("Conf parse");

        // Justification => must have compile_error
        assert!(
            has_compile_error_top_level(&just_ast, "Mixed scenario error"),
            "Expected compile_error in justification snippet for Mixed scenario"
        );
        let js_struct = find_struct_by_name(&just_ast, "MixedJustification");
        assert_struct_fields(
            js_struct,
            &[("field_a","String"), ("field_b","bool")],
            "MixedJustification"
        );
        assert_derive_traits(
            &js_struct.attrs,
            "MixedJustification",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );

        // Confidence => also has fields, check them
        let cf_struct = find_struct_by_name(&conf_ast, "MixedConfidence");
        assert_struct_fields(
            cf_struct,
            &[("field_x","f32"), ("field_y","u64")],
            "MixedConfidence"
        );
        assert_derive_traits(
            &cf_struct.attrs,
            "MixedConfidence",
            &["Builder","Debug","Clone","PartialEq","Default","Serialize","Deserialize","Getters","Setters"]
        );
        // If your code injects the same compile_error in confidence, test that too:
        // assert!(has_compile_error_top_level(&conf_ast, "Mixed scenario error"), "... ");
    }
}
