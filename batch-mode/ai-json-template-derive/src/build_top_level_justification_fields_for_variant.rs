// ---------------- [ File: ai-json-template-derive/src/build_top_level_justification_fields_for_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_top_level_just_fields_for_variant(
    variant_ident: &syn::Ident,
    skip_self_just: bool
) -> TopLevelJustResult {
    trace!(
        "build_top_level_just_fields_for_variant: variant='{}', skip_self_just={}",
        variant_ident,
        skip_self_just
    );

    if skip_self_just {
        return TopLevelJustResultBuilder::default()
            .field_decls_top(vec![])
            .pattern_vars_top(vec![])
            .just_inits_top(vec![])
            .conf_inits_top(vec![])
            .build()
            .expect("Failed building TopLevelJustResult");
    }

    debug!(
        "Inserting top-level enum_variant_justification/enum_variant_confidence for variant '{}'",
        variant_ident
    );

    // Using `syn::parse_quote!` preserves the exact token spacing without extra spaces.
    // The test suite is very strict about the debug representation of the TokenStream.
    TopLevelJustResultBuilder::default()
        .field_decls_top(vec![
            syn::parse_quote!(#[serde(default)] enum_variant_justification:String),
            syn::parse_quote!(#[serde(default)] enum_variant_confidence:f32),
        ])
        .pattern_vars_top(vec![
            syn::parse_quote!(enum_variant_justification),
            syn::parse_quote!(enum_variant_confidence),
        ])
        .just_inits_top(vec![
            syn::parse_quote!(variant_justification: enum_variant_justification),
        ])
        .conf_inits_top(vec![
            syn::parse_quote!(variant_confidence: enum_variant_confidence),
        ])
        .build()
        .expect("Failed building TopLevelJustResult")
}

#[cfg(test)]
mod test_build_top_level_just_fields_for_variant {
    use super::*;

    #[traced_test]
    fn test_skip_self_just_true_returns_empty() {
        trace!("Starting test: skip_self_just = true => should yield empty fields.");
        let variant_ident = Ident::new("UnitVariant", proc_macro2::Span::call_site());
        debug!("Created test variant identifier: {:?}", variant_ident);

        let result = build_top_level_just_fields_for_variant(&variant_ident, true);
        info!("Obtained TopLevelJustResult => {:?}", result);

        // All Vec fields should be empty if skip_self_just = true
        assert_eq!(result.field_decls_top().len(), 0, "Expected no field declarations");
        assert_eq!(result.pattern_vars_top().len(), 0, "Expected no pattern vars");
        assert_eq!(result.just_inits_top().len(), 0, "Expected no just inits");
        assert_eq!(result.conf_inits_top().len(), 0, "Expected no conf inits");

        debug!("Completed test: skip_self_just = true => all expansions empty as expected.");
    }

    #[traced_test]
    fn test_skip_self_just_false_returns_fields() {
        trace!("Starting test: skip_self_just = false => should yield two top-level fields.");
        let variant_ident = Ident::new("StructVariant", proc_macro2::Span::call_site());
        debug!("Created test variant identifier: {:?}", variant_ident);

        let result = build_top_level_just_fields_for_variant(&variant_ident, false);
        info!("Obtained TopLevelJustResult => {:?}", result);

        // We expect exactly two field declarations for the top-level justification/confidence
        assert_eq!(result.field_decls_top().len(), 2, "Expected two field declarations");
        assert_eq!(result.pattern_vars_top().len(), 2, "Expected two pattern vars");
        assert_eq!(result.just_inits_top().len(), 1, "Expected one just init");
        assert_eq!(result.conf_inits_top().len(), 1, "Expected one conf init");

        // Check the actual content of the first field decl
        let field_1_str = result.field_decls_top()[0].to_string();
        assert!(
            field_1_str.contains("enum_variant_justification")
            && field_1_str.contains("String")
            && field_1_str.contains("# [ serde ( default ) ]"),
            "Expected the first field to reference enum_variant_justification:String with #[serde(default)]. Got: {}",
            field_1_str
        );

        // Check the content of the second field decl
        let field_2_str = result.field_decls_top()[1].to_string();
        assert!(
            field_2_str.contains("enum_variant_confidence")
            && field_2_str.contains("f32")
            && field_2_str.contains("# [ serde ( default ) ]"),
            "Expected the second field to reference enum_variant_confidence:f32 with #[serde(default)]. Got: {}",
            field_2_str
        );

        // Similarly, check pattern_vars_top for correct var names
        let pat_vars_joined = result
            .pattern_vars_top()
            .iter()
            .map(|ts| ts.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            pat_vars_joined.contains("enum_variant_justification")
            && pat_vars_joined.contains("enum_variant_confidence"),
            "Expected pattern vars to reference enum_variant_justification and enum_variant_confidence. Got: {}",
            pat_vars_joined
        );

        // Check justification inits
        let just_inits_str = result.just_inits_top()[0].to_string();
        assert!(
            just_inits_str.contains("variant_justification")
            && just_inits_str.contains("enum_variant_justification"),
            "Expected the justification init to map variant_justification -> enum_variant_justification. Got: {}",
            just_inits_str
        );

        // Check confidence inits
        let conf_inits_str = result.conf_inits_top()[0].to_string();
        assert!(
            conf_inits_str.contains("variant_confidence")
            && conf_inits_str.contains("enum_variant_confidence"),
            "Expected the confidence init to map variant_confidence -> enum_variant_confidence. Got: {}",
            conf_inits_str
        );

        debug!("Completed test: skip_self_just = false => expansions for top-level justification/conf present and valid.");
    }
}
