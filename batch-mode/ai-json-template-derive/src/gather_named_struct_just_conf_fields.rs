// ---------------- [ File: ai-json-template-derive/src/gather_named_struct_just_conf_fields.rs ]
crate::ix!();

pub fn gather_named_struct_just_conf_idents(
    ty_ident: &syn::Ident,
    span: proc_macro2::Span,
) -> (syn::Ident, syn::Ident, syn::Ident) {
    trace!(
        "Constructing justification/conf/justified idents for '{}'",
        ty_ident
    );
    let justification_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident = syn::Ident::new(&format!("{}Confidence", ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);
    debug!(
        " -> justification_ident='{}', confidence_ident='{}', justified_ident='{}'",
        justification_ident, confidence_ident, justified_ident
    );
    (justification_ident, confidence_ident, justified_ident)
}

#[cfg(test)]
mod test_suite_for_gather_named_struct_just_conf_idents {
    use super::*;

    #[traced_test]
    fn should_produce_expected_idents_for_simple_type_name() {
        trace!("Starting test: should_produce_expected_idents_for_simple_type_name");
        let ty_name = "Person";
        let ty_ident = Ident::new(ty_name, proc_macro2::Span::call_site());

        debug!("Calling gather_named_struct_just_conf_idents(...) with '{}'", ty_name);
        let (just_id, conf_id, just_struct_id) =
            gather_named_struct_just_conf_idents(&ty_ident, proc_macro2::Span::call_site());

        debug!("Received => just_id='{}', conf_id='{}', just_struct_id='{}'", just_id, conf_id, just_struct_id);

        assert_eq!(just_id.to_string(),        "PersonJustification", "Incorrect Justification Ident for {}", ty_name);
        assert_eq!(conf_id.to_string(),        "PersonConfidence",    "Incorrect Confidence Ident for {}",    ty_name);
        assert_eq!(just_struct_id.to_string(), "JustifiedPerson",     "Incorrect Justified Ident for {}",     ty_name);

        info!("Test passed: should_produce_expected_idents_for_simple_type_name");
    }

    #[traced_test]
    fn should_handle_longer_type_name_with_numbers() {
        trace!("Starting test: should_handle_longer_type_name_with_numbers");
        let ty_name = "Http2Request";
        let ty_ident = Ident::new(ty_name, proc_macro2::Span::call_site());

        debug!("Calling gather_named_struct_just_conf_idents(...) with '{}'", ty_name);
        let (just_id, conf_id, just_struct_id) =
            gather_named_struct_just_conf_idents(&ty_ident, proc_macro2::Span::call_site());

        debug!("Received => just_id='{}', conf_id='{}', just_struct_id='{}'", just_id, conf_id, just_struct_id);

        assert_eq!(just_id.to_string(),        "Http2RequestJustification", "Should handle numeric parts in type name correctly");
        assert_eq!(conf_id.to_string(),        "Http2RequestConfidence",    "Should handle numeric parts in type name correctly");
        assert_eq!(just_struct_id.to_string(), "JustifiedHttp2Request",     "Should handle numeric parts in type name correctly");

        info!("Test passed: should_handle_longer_type_name_with_numbers");
    }

    #[traced_test]
    fn should_handle_type_name_with_underscore() {
        trace!("Starting test: should_handle_type_name_with_underscore");
        let ty_name = "My_StructType";
        let ty_ident = Ident::new(ty_name, proc_macro2::Span::call_site());

        debug!("Calling gather_named_struct_just_conf_idents(...) with '{}'", ty_name);
        let (just_id, conf_id, just_struct_id) =
            gather_named_struct_just_conf_idents(&ty_ident, proc_macro2::Span::call_site());

        debug!("Received => just_id='{}', conf_id='{}', just_struct_id='{}'", just_id, conf_id, just_struct_id);

        assert_eq!(just_id.to_string(),        "My_StructTypeJustification", "Underscore in type name should carry over correctly");
        assert_eq!(conf_id.to_string(),        "My_StructTypeConfidence",    "Underscore in type name should carry over correctly");
        assert_eq!(just_struct_id.to_string(), "JustifiedMy_StructType",     "Underscore in type name should carry over correctly");

        info!("Test passed: should_handle_type_name_with_underscore");
    }

    #[traced_test]
    fn should_work_for_single_char_type_name() {
        trace!("Starting test: should_work_for_single_char_type_name");
        let ty_name = "X";
        let ty_ident = Ident::new(ty_name, proc_macro2::Span::call_site());

        debug!("Calling gather_named_struct_just_conf_idents(...) with '{}'", ty_name);
        let (just_id, conf_id, just_struct_id) =
            gather_named_struct_just_conf_idents(&ty_ident, proc_macro2::Span::call_site());

        debug!("Received => just_id='{}', conf_id='{}', just_struct_id='{}'", just_id, conf_id, just_struct_id);

        assert_eq!(just_id.to_string(),        "XJustification", "Should handle single-char type name gracefully");
        assert_eq!(conf_id.to_string(),        "XConfidence",    "Should handle single-char type name gracefully");
        assert_eq!(just_struct_id.to_string(), "JustifiedX",     "Should handle single-char type name gracefully");

        info!("Test passed: should_work_for_single_char_type_name");
    }

    #[traced_test]
    fn should_handle_complex_mixed_alphanumeric() {
        trace!("Starting test: should_handle_complex_mixed_alphanumeric");
        let ty_name = "V3Alpha_16Bit";
        let ty_ident = Ident::new(ty_name, proc_macro2::Span::call_site());

        debug!("Calling gather_named_struct_just_conf_idents(...) with '{}'", ty_name);
        let (just_id, conf_id, just_struct_id) =
            gather_named_struct_just_conf_idents(&ty_ident, proc_macro2::Span::call_site());

        debug!("Received => just_id='{}', conf_id='{}', just_struct_id='{}'", just_id, conf_id, just_struct_id);

        assert_eq!(just_id.to_string(),        "V3Alpha_16BitJustification");
        assert_eq!(conf_id.to_string(),        "V3Alpha_16BitConfidence");
        assert_eq!(just_struct_id.to_string(), "JustifiedV3Alpha_16Bit");

        info!("Test passed: should_handle_complex_mixed_alphanumeric");
    }
}
