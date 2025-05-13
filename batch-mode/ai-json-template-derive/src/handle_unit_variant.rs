// ---------------- [ File: ai-json-template-derive/src/handle_unit_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_unit_variant(
    var_ident: &syn::Ident,
    skip_self_just: bool
) -> (
    proc_macro2::TokenStream, // variant in the Justification enum
    proc_macro2::TokenStream, // variant in the Confidence enum
    Option<String>,           // first-variant justification field name
    Option<String>            // first-variant confidence field name
)
{
    debug!(
        "Handling unit variant '{}', skip_self_just={}",
        var_ident,
        skip_self_just
    );

    if skip_self_just {
        let jvar = quote::quote! { #var_ident {} };
        let cvar = quote::quote! { #var_ident {} };
        (jvar, cvar, None, None)
    } else {
        let jvar = quote::quote! { #var_ident { variant_justification: String } };
        let cvar = quote::quote! { #var_ident { variant_confidence: f32 } };
        (
            jvar,
            cvar,
            Some("variant_justification".to_string()),
            Some("variant_confidence".to_string())
        )
    }
}

#[cfg(test)]
mod test_handle_unit_variant {
    use super::*;

    #[traced_test]
    fn test_no_justification_fields_when_skip_self_just_true() {
        trace!("Starting test_no_justification_fields_when_skip_self_just_true");

        let var_ident = Ident::new("SampleVariant", proc_macro2::Span::call_site());
        let skip_self_just = true;

        debug!("Calling handle_unit_variant with skip_self_just={}", skip_self_just);
        let (just_ts, conf_ts, first_just, first_conf) = handle_unit_variant(&var_ident, skip_self_just);

        info!("Asserting the returned Option<String> values are None...");
        assert_eq!(first_just, None, "Expected no top-level justification field name for skip_self_just=true");
        assert_eq!(first_conf, None, "Expected no top-level confidence field name for skip_self_just=true");

        let just_str = just_ts.to_token_stream().to_string();
        let conf_str = conf_ts.to_token_stream().to_string();

        debug!("just_str='{}'", just_str);
        debug!("conf_str='{}'", conf_str);

        info!("Checking for the presence of an empty brace block in each token stream...");
        // In skip_self_just=true, we expect something like `<VariantIdent> { }`.
        assert!(
            just_str.contains("{}"),
            "When skip_self_just=true, expected the Justification variant to have no fields"
        );
        assert!(
            conf_str.contains("{}"),
            "When skip_self_just=true, expected the Confidence variant to have no fields"
        );
    }

    #[traced_test]
    fn test_with_justification_fields_when_skip_self_just_false() {
        trace!("Starting test_with_justification_fields_when_skip_self_just_false");

        let var_ident = Ident::new("AnotherVariant", proc_macro2::Span::call_site());
        let skip_self_just = false;

        debug!("Calling handle_unit_variant with skip_self_just={}", skip_self_just);
        let (just_ts, conf_ts, first_just, first_conf) = handle_unit_variant(&var_ident, skip_self_just);

        info!("Asserting the returned Option<String> values are Some(...)...");
        assert_eq!(
            first_just,
            Some("variant_justification".to_string()),
            "Expected top-level justification field for skip_self_just=false"
        );
        assert_eq!(
            first_conf,
            Some("variant_confidence".to_string()),
            "Expected top-level confidence field for skip_self_just=false"
        );

        let just_str = just_ts.to_token_stream().to_string();
        let conf_str = conf_ts.to_token_stream().to_string();

        debug!("just_str='{}'", just_str);
        debug!("conf_str='{}'", conf_str);

        info!("Checking for presence of 'variant_justification' and 'variant_confidence' fields...");
        assert!(
            just_str.contains("variant_justification : String"),
            "When skip_self_just=false, expected the Justification variant to contain 'variant_justification : String'"
        );
        assert!(
            conf_str.contains("variant_confidence : f32"),
            "When skip_self_just=false, expected the Confidence variant to contain 'variant_confidence : f32'"
        );
    }

    #[traced_test]
    fn test_explicit_unit_variant_name() {
        trace!("Starting test_explicit_unit_variant_name");

        let var_ident = Ident::new("Unit", proc_macro2::Span::call_site());
        let skip_self_just = false;

        debug!(
            "Calling handle_unit_variant with variant='{}' and skip_self_just={}",
            var_ident, skip_self_just
        );
        let (just_ts, conf_ts, first_just, first_conf) = handle_unit_variant(&var_ident, skip_self_just);

        info!("Verifying the Option<String> are Some(...) since skip_self_just=false...");
        assert_eq!(
            first_just,
            Some("variant_justification".to_string()),
            "Expected top-level justification field for skip_self_just=false with 'Unit'"
        );
        assert_eq!(
            first_conf,
            Some("variant_confidence".to_string()),
            "Expected top-level confidence field for skip_self_just=false with 'Unit'"
        );

        let just_str = just_ts.to_token_stream().to_string();
        let conf_str = conf_ts.to_token_stream().to_string();

        debug!("just_str='{}'", just_str);
        debug!("conf_str='{}'", conf_str);

        info!("Ensuring we renamed 'Unit' properly and included the fields...");
        // Implementation detail: handle_unit_variant does not rename it in the returned TokenStream,
        // but we expect the fields to be present, so let's just check them.
        assert!(just_str.contains("Unit { variant_justification : String }"), "Expected 'Unit {{ variant_justification : String }}'");
        assert!(conf_str.contains("Unit { variant_confidence : f32 }"), "Expected 'Unit {{ variant_confidence : f32 }}'");
    }

    #[traced_test]
    fn test_various_ident_and_skip_values() {
        trace!("Starting test_various_ident_and_skip_values");

        let test_cases = vec![
            ("EmptyVariant", true),
            ("EmptyVariant", false),
            ("AnotherOne",   true),
            ("AnotherOne",   false),
        ];

        for (name, skip) in test_cases {
            debug!("Processing test case => variant='{}', skip_self_just={}", name, skip);
            let ident = Ident::new(name, proc_macro2::Span::call_site());

            let (just_ts, conf_ts, first_just, first_conf) = handle_unit_variant(&ident, skip);

            if skip {
                debug!("skip_self_just=true => expect None for first_just/conf, and empty braced fields");
                assert_eq!(first_just, None);
                assert_eq!(first_conf, None);

                let j_str = just_ts.to_token_stream().to_string();
                let c_str = conf_ts.to_token_stream().to_string();
                assert!(j_str.contains("{}"), "Expected empty braces for justification variant");
                assert!(c_str.contains("{}"), "Expected empty braces for confidence variant");
            } else {
                debug!("skip_self_just=false => expect Some(...) for first_just/conf, and actual fields");
                assert_eq!(first_just, Some("variant_justification".to_string()));
                assert_eq!(first_conf, Some("variant_confidence".to_string()));

                let j_str = just_ts.to_token_stream().to_string();
                let c_str = conf_ts.to_token_stream().to_string();
                assert!(
                    j_str.contains("variant_justification : String"),
                    "Expected 'variant_justification : String' in justification variant"
                );
                assert!(
                    c_str.contains("variant_confidence : f32"),
                    "Expected 'variant_confidence : f32' in confidence variant"
                );
            }
        }
    }
}
