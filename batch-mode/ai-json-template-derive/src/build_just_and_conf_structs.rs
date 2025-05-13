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

    // Make sure we do not use `pub` on the fields. The fields are already stripped of `pub` above.
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

    #[traced_test]
    fn test_empty_fields_no_error_tokens() {
        trace!("Starting test: test_empty_fields_no_error_tokens");

        let justification_ident = syn::Ident::new("EmptyJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("EmptyConfidence",    proc_macro2::Span::call_site());

        let errs                = quote! {};
        let justification_fields = vec![];
        let confidence_fields    = vec![];

        trace!("Invoking build_just_and_conf_structs with empty fields and no errors");
        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        trace!("Validating generated tokens for justification");
        let just_syn: syn::File = parse2(just_ts.clone())
            .expect("Justification tokens should parse successfully");
        debug!("Parsed justification syntax: {:?}", just_syn);

        trace!("Validating generated tokens for confidence");
        let conf_syn: syn::File = parse2(conf_ts.clone())
            .expect("Confidence tokens should parse successfully");
        debug!("Parsed confidence syntax: {:?}", conf_syn);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        assert!(
            just_str.contains("struct EmptyJustification"),
            "Expected struct named EmptyJustification"
        );
        assert!(
            just_str.contains("#[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]"),
            "Should derive the correct traits on justification struct"
        );
        assert!(
            conf_str.contains("struct EmptyConfidence"),
            "Expected struct named EmptyConfidence"
        );
        assert!(
            conf_str.contains("#[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]"),
            "Should derive the correct traits on confidence struct"
        );

        trace!("Finished test: test_empty_fields_no_error_tokens");
    }

    #[traced_test]
    fn test_populated_fields_no_error_tokens() {
        trace!("Starting test: test_populated_fields_no_error_tokens");

        let justification_ident = syn::Ident::new("PopulatedJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("PopulatedConfidence",    proc_macro2::Span::call_site());

        let errs = quote! {};
        let justification_fields = vec![
            quote! { some_field: String, },
            quote! { another_field: i32, },
        ];
        let confidence_fields = vec![
            quote! { conf_val: f32, },
            quote! { alpha_level: f64, },
        ];

        trace!("Invoking build_just_and_conf_structs with multiple fields and no errors");
        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        trace!("Parsing the justification tokens");
        let just_syn: syn::File = parse2(just_ts.clone())
            .expect("Justification tokens should parse successfully");
        debug!("Parsed justification syntax: {:?}", just_syn);

        trace!("Parsing the confidence tokens");
        let conf_syn: syn::File = parse2(conf_ts.clone())
            .expect("Confidence tokens should parse successfully");
        debug!("Parsed confidence syntax: {:?}", conf_syn);

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        assert!(
            just_str.contains("struct PopulatedJustification"),
            "Expected struct named PopulatedJustification"
        );
        assert!(
            just_str.contains("some_field: String") && just_str.contains("another_field: i32"),
            "Expected field definitions in justification struct"
        );
        assert!(
            conf_str.contains("struct PopulatedConfidence"),
            "Expected struct named PopulatedConfidence"
        );
        assert!(
            conf_str.contains("conf_val: f32") && conf_str.contains("alpha_level: f64"),
            "Expected field definitions in confidence struct"
        );

        trace!("Finished test: test_populated_fields_no_error_tokens");
    }

    #[traced_test]
    fn test_with_error_tokens() {
        trace!("Starting test: test_with_error_tokens");

        let justification_ident = syn::Ident::new("ErrorfulJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("ErrorfulConfidence",    proc_macro2::Span::call_site());

        // Simulate a compile_error! invocation or any error tokens from prior processing
        let errs = quote! {
            compile_error!("Simulated error from prior step.");
        };

        let justification_fields = vec![
            quote! { j1: String, },
        ];
        let confidence_fields = vec![
            quote! { c1: i64, },
        ];

        trace!("Invoking build_just_and_conf_structs with an error token stream");
        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        trace!("Parsing the justification tokens");
        let just_syn_res = parse2::<syn::File>(just_ts.clone());
        debug!("Parsing result for justification: {:?}", just_syn_res);

        trace!("Parsing the confidence tokens");
        let conf_syn_res = parse2::<syn::File>(conf_ts.clone());
        debug!("Parsing result for confidence: {:?}", conf_syn_res);

        // We expect the presence of compile_error! but it should still parse
        assert!(just_syn_res.is_ok(), "The presence of compile_error! should not break parsing for justification");
        assert!(conf_syn_res.is_ok(), "The presence of compile_error! should not break parsing for confidence");

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        assert!(
            just_str.contains("compile_error! ( \"Simulated error from prior step.\" )"),
            "Should contain the simulated compile_error! tokens in justification"
        );
        assert!(
            just_str.contains("ErrorfulJustification"),
            "Expected struct name to appear in justification tokens"
        );
        assert!(
            conf_str.contains("compile_error! ( \"Simulated error from prior step.\" )"),
            "Should contain the simulated compile_error! tokens in confidence"
        );
        assert!(
            conf_str.contains("ErrorfulConfidence"),
            "Expected struct name to appear in confidence tokens"
        );

        trace!("Finished test: test_with_error_tokens");
    }

    #[traced_test]
    fn test_minimal_overall_validity() {
        trace!("Starting test: test_minimal_overall_validity");

        // Combine a bit of everything: some errors + some fields
        let justification_ident = syn::Ident::new("MixedJustification", proc_macro2::Span::call_site());
        let confidence_ident    = syn::Ident::new("MixedConfidence",    proc_macro2::Span::call_site());

        let errs = quote! {
            compile_error!("Mixed scenario error");
        };
        let justification_fields = vec![
            quote! { field_a: String, },
            quote! { field_b: bool, },
        ];
        let confidence_fields = vec![
            quote! { field_x: f32, },
            quote! { field_y: u64, },
        ];

        trace!("Invoking build_just_and_conf_structs for a mixed scenario");
        let (just_ts, conf_ts) = build_just_and_conf_structs(
            &justification_ident,
            &confidence_ident,
            &errs,
            &justification_fields,
            &confidence_fields
        );

        trace!("Parsing the returned justification tokens");
        let just_parse_res = parse2::<syn::File>(just_ts.clone());
        debug!("Parsed justification AST: {:?}", just_parse_res);
        assert!(just_parse_res.is_ok(), "Justification tokens should parse properly in a mixed scenario");

        trace!("Parsing the returned confidence tokens");
        let conf_parse_res = parse2::<syn::File>(conf_ts.clone());
        debug!("Parsed confidence AST: {:?}", conf_parse_res);
        assert!(conf_parse_res.is_ok(), "Confidence tokens should parse properly in a mixed scenario");

        let just_str = just_ts.to_string();
        let conf_str = conf_ts.to_string();

        assert!(just_str.contains("MixedJustification"), "Justification struct name must appear");
        assert!(just_str.contains("compile_error!"), "Should include error tokens in justification output");
        assert!(just_str.contains("field_a: String") && just_str.contains("field_b: bool"), "Should include provided justification fields");

        assert!(conf_str.contains("MixedConfidence"), "Confidence struct name must appear");
        assert!(conf_str.contains("compile_error!"), "Should include error tokens in confidence output");
        assert!(conf_str.contains("field_x: f32") && conf_str.contains("field_y: u64"), "Should include provided confidence fields");

        trace!("Finished test: test_minimal_overall_validity");
    }
}
