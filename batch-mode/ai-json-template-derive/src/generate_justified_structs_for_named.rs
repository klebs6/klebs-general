// ---------------- [ File: ai-json-template-derive/src/generate_justified_structs_for_named.rs ]
crate::ix!();

/// Refactored version of the old `generate_justified_structs_for_named`,
/// broken down into single-purpose, well-traced subroutines.
/// It returns the token streams for:
///   1) the FooJustification struct
///   2) the FooConfidence struct
///   3) the JustifiedFoo struct
///   4) the accessor impl block
///
/// # Arguments
/// * `ty_ident`     - The original named struct's identifier
/// * `named_fields` - The named fields of that struct
/// * `span`         - The proc-macro2::Span used to generate new Ident(s)
///
pub fn generate_justified_structs_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span,
) -> (
    proc_macro2::TokenStream, // justification struct
    proc_macro2::TokenStream, // confidence struct
    proc_macro2::TokenStream, // justified struct
    proc_macro2::TokenStream, // accessor expansions
) {
    trace!(
        "Beginning refactored generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    // (1) Create the 3 Ident values
    let (justification_ident, confidence_ident, justified_ident) =
        gather_named_struct_just_conf_idents(ty_ident, span);

    // (2) Gather the field expansions => justification/conf fields, plus any errors
    let (just_fields, conf_fields, errs, mappings) = gather_fields_for_just_conf(named_fields);

    // (3) Build the two struct definitions => e.g. `FooJustification`, `FooConfidence`
    let (just_ts, conf_ts) =
        build_just_and_conf_structs(&justification_ident, &confidence_ident, &errs, &just_fields, &conf_fields);

    // (4) Build the `JustifiedFoo` struct
    let justified_ts = build_justified_struct(&justified_ident, ty_ident, &justification_ident, &confidence_ident);

    // (5) Build the accessor impl for JustifiedFoo
    let accessor_ts = build_justified_struct_accessors(
        &justified_ident,
        named_fields,
        ty_ident,
        &mappings,
    );

    debug!(
        "Finished generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    (just_ts, conf_ts, justified_ts, accessor_ts)
}

#[cfg(test)]
mod test_exhaustive_generate_justified_structs_for_named {
    use super::*;

    #[traced_test]
    fn test_empty_fields() {
        trace!("Starting test_empty_fields...");
        let named: FieldsNamed = parse_quote! {};
        let ty_ident = Ident::new("EmptyStruct", proc_macro2::Span::call_site());
        let (just_ts, conf_ts, justified_ts, accessor_ts) =
            generate_justified_structs_for_named(&ty_ident, &named, proc_macro2::Span::call_site());

        debug!("Justification struct tokens: {}", just_ts.to_string());
        debug!("Confidence struct tokens: {}", conf_ts.to_string());
        debug!("Justified struct tokens: {}", justified_ts.to_string());
        debug!("Accessor impl tokens: {}", accessor_ts.to_string());

        // We only verify that the returned TokenStreams are non-empty and compile without error.
        // This is "interface" testing in the sense that we confirm the function returns
        // coherent expansions, rather than verifying private internal steps.
        assert!(!just_ts.is_empty(), "Justification struct expansion is empty");
        assert!(!conf_ts.is_empty(), "Confidence struct expansion is empty");
        assert!(!justified_ts.is_empty(), "Justified struct expansion is empty");
        assert!(!accessor_ts.is_empty(), "Accessor impl expansion is empty");

        info!("test_empty_fields passed.");
    }

    #[traced_test]
    fn test_single_field_no_justify() {
        trace!("Starting test_single_field_no_justify...");
        let named: FieldsNamed = parse_quote! {
            {
                field_one: u32
            }
        };
        let ty_ident = Ident::new("SingleFieldStruct", proc_macro2::Span::call_site());
        let (just_ts, conf_ts, justified_ts, accessor_ts) =
            generate_justified_structs_for_named(&ty_ident, &named, proc_macro2::Span::call_site());

        debug!("Justification struct tokens: {}", just_ts.to_string());
        debug!("Confidence struct tokens: {}", conf_ts.to_string());
        debug!("Justified struct tokens: {}", justified_ts.to_string());
        debug!("Accessor impl tokens: {}", accessor_ts.to_string());

        assert!(just_ts.to_string().contains("field_one_justification"),
            "Expected justification field for 'field_one'");
        assert!(conf_ts.to_string().contains("field_one_confidence"),
            "Expected confidence field for 'field_one'");
        assert!(justified_ts.to_string().contains("SingleFieldStructJustification"),
            "Expected reference to justification struct in JustifiedSingleFieldStruct");
        assert!(justified_ts.to_string().contains("SingleFieldStructConfidence"),
            "Expected reference to confidence struct in JustifiedSingleFieldStruct");

        info!("test_single_field_no_justify passed.");
    }

    #[traced_test]
    fn test_multiple_fields_mixed() {
        trace!("Starting test_multiple_fields_mixed...");
        // field_two has #[justify = false], so it should NOT appear in the justification/conf expansions.
        let named: FieldsNamed = parse_quote! {
            {
                #[justify=false]
                field_two: String,
                field_three: bool,
                field_four: f32
            }
        };
        let ty_ident = Ident::new("MixedFieldsStruct", proc_macro2::Span::call_site());
        let (just_ts, conf_ts, justified_ts, accessor_ts) =
            generate_justified_structs_for_named(&ty_ident, &named, proc_macro2::Span::call_site());

        debug!("Justification struct tokens: {}", just_ts.to_string());
        debug!("Confidence struct tokens: {}", conf_ts.to_string());
        debug!("Justified struct tokens: {}", justified_ts.to_string());
        debug!("Accessor impl tokens: {}", accessor_ts.to_string());

        // field_two is skip => no "field_two_justification" or "field_two_confidence"
        assert!(!just_ts.to_string().contains("field_two_justification"),
            "Expected NO justification for 'field_two' (justify=false)");
        assert!(!conf_ts.to_string().contains("field_two_confidence"),
            "Expected NO confidence for 'field_two' (justify=false)");

        // field_three and field_four do not have #[justify=false], so they should appear
        assert!(just_ts.to_string().contains("field_three_justification"),
            "Expected justification for 'field_three'");
        assert!(conf_ts.to_string().contains("field_three_confidence"),
            "Expected confidence for 'field_three'");
        assert!(just_ts.to_string().contains("field_four_justification"),
            "Expected justification for 'field_four'");
        assert!(conf_ts.to_string().contains("field_four_confidence"),
            "Expected confidence for 'field_four'");

        // Check that Justified struct references them
        let jts = justified_ts.to_string();
        assert!(jts.contains("MixedFieldsStructJustification"),
            "Expected ref to 'MixedFieldsStructJustification'");
        assert!(jts.contains("MixedFieldsStructConfidence"),
            "Expected ref to 'MixedFieldsStructConfidence'");

        info!("test_multiple_fields_mixed passed.");
    }

    #[traced_test]
    fn test_doc_comments_and_integration() {
        trace!("Starting test_doc_comments_and_integration...");
        // Some doc comments, which do not affect the function's direct expansions, but might appear
        // in its error or documentation expansions if present.
        let named: FieldsNamed = parse_quote! {
            {
                /// This is the first field
                field_alpha: i16,
                /// Another doc
                #[justify=false]
                field_beta: Option<String>
            }
        };
        let ty_ident = Ident::new("DocCommentStruct", proc_macro2::Span::call_site());
        let (just_ts, conf_ts, justified_ts, accessor_ts) =
            generate_justified_structs_for_named(&ty_ident, &named, proc_macro2::Span::call_site());

        debug!("Justification struct tokens: {}", just_ts.to_string());
        debug!("Confidence struct tokens: {}", conf_ts.to_string());
        debug!("Justified struct tokens: {}", justified_ts.to_string());
        debug!("Accessor impl tokens: {}", accessor_ts.to_string());

        // field_alpha => normal justification
        assert!(just_ts.to_string().contains("field_alpha_justification"),
            "Expected justification for 'field_alpha'");
        // field_beta => skip justification (#[justify=false])
        assert!(!just_ts.to_string().contains("field_beta_justification"),
            "Expected no justification for 'field_beta' (justify=false)");

        // Check accessor methods for item
        let acc = accessor_ts.to_string();
        assert!(acc.contains("fn field_alpha("),
            "Expected accessor for 'field_alpha'");
        assert!(!acc.contains("fn field_beta("),
            "Expected NO accessor for 'field_beta' justification (justify=false doesn't remove item accessor, but let's check generation anyway)");

        info!("test_doc_comments_and_integration passed.");
    }

    #[traced_test]
    fn test_error_handling_for_bad_type() {
        trace!("Starting test_error_handling_for_bad_type...");
        // Suppose there's a type containing "BadType" that the classification routine doesn't allow.
        // We'll just parse a field that references "BadTypeSomething".
        let named: FieldsNamed = parse_quote! {
            {
                trouble_field: BadTypeSomething
            }
        };
        let ty_ident = Ident::new("BadTypeHolder", proc_macro2::Span::call_site());
        let (just_ts, conf_ts, justified_ts, accessor_ts) =
            generate_justified_structs_for_named(&ty_ident, &named, proc_macro2::Span::call_site());

        debug!("Justification struct tokens: {}", just_ts.to_string());
        debug!("Confidence struct tokens: {}", conf_ts.to_string());
        debug!("Justified struct tokens: {}", justified_ts.to_string());
        debug!("Accessor impl tokens: {}", accessor_ts.to_string());

        // We expect a compile_error in the expansions or some partial handling. We'll do a simple check:
        let combined = format!(
            "{}\n{}\n{}\n{}",
            just_ts.to_string(),
            conf_ts.to_string(),
            justified_ts.to_string(),
            accessor_ts.to_string()
        );

        assert!(combined.contains("compile_error!"),
            "Expected compile_error! for type containing 'BadType' references");

        info!("test_error_handling_for_bad_type passed.");
    }
}
