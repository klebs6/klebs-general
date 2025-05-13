// ---------------- [ File: ai-json-template-derive/src/create_flat_justified_idents_for_named.rs ]
crate::ix!();

pub fn create_flat_justified_idents_for_named(
    ty_ident: &syn::Ident,
    span: proc_macro2::Span,
) -> (syn::Ident, syn::Ident, syn::Ident, syn::Ident) {
    trace!("create_flat_justified_idents_for_named: starting for '{}'", ty_ident);

    let flat_ident = syn::Ident::new(&format!("FlatJustified{}", ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}", ty_ident), span);
    let justification_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let confidence_ident = syn::Ident::new(&format!("{}Confidence", ty_ident), span);

    debug!("create_flat_justified_idents_for_named => flat_ident='{}', justified_ident='{}', justification_ident='{}', confidence_ident='{}'",
        flat_ident, justified_ident, justification_ident, confidence_ident
    );
    (flat_ident, justified_ident, justification_ident, confidence_ident)
}

#[cfg(test)]
mod test_create_flat_justified_idents_for_named {
    use super::*;

    #[traced_test]
    fn check_typical_struct_name() {
        trace!("Starting check_typical_struct_name");
        let input_ident = Ident::new("MyStruct", proc_macro2::Span::call_site());
        debug!("Calling create_flat_justified_idents_for_named with 'MyStruct'");
        let (flat_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justified_idents_for_named(&input_ident, input_ident.span());

        info!("Asserting that returned flat ident is 'FlatJustifiedMyStruct'");
        assert_eq!(flat_ident.to_string(), "FlatJustifiedMyStruct");

        info!("Asserting that returned justified ident is 'JustifiedMyStruct'");
        assert_eq!(justified_ident.to_string(), "JustifiedMyStruct");

        info!("Asserting that returned justification ident is 'MyStructJustification'");
        assert_eq!(justification_ident.to_string(), "MyStructJustification");

        info!("Asserting that returned confidence ident is 'MyStructConfidence'");
        assert_eq!(confidence_ident.to_string(), "MyStructConfidence");

        debug!("check_typical_struct_name completed successfully");
    }

    #[traced_test]
    fn check_short_struct_name() {
        trace!("Starting check_short_struct_name");
        let input_ident = Ident::new("X", proc_macro2::Span::call_site());
        debug!("Calling create_flat_justified_idents_for_named with 'X'");
        let (flat_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justified_idents_for_named(&input_ident, input_ident.span());

        info!("Asserting that returned flat ident is 'FlatJustifiedX'");
        assert_eq!(flat_ident.to_string(), "FlatJustifiedX");

        info!("Asserting that returned justified ident is 'JustifiedX'");
        assert_eq!(justified_ident.to_string(), "JustifiedX");

        info!("Asserting that returned justification ident is 'XJustification'");
        assert_eq!(justification_ident.to_string(), "XJustification");

        info!("Asserting that returned confidence ident is 'XConfidence'");
        assert_eq!(confidence_ident.to_string(), "XConfidence");

        debug!("check_short_struct_name completed successfully");
    }

    #[traced_test]
    fn check_struct_name_with_numbers() {
        trace!("Starting check_struct_name_with_numbers");
        let input_ident = Ident::new("Type123", proc_macro2::Span::call_site());
        debug!("Calling create_flat_justified_idents_for_named with 'Type123'");
        let (flat_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justified_idents_for_named(&input_ident, input_ident.span());

        info!("Asserting flat ident is 'FlatJustifiedType123'");
        assert_eq!(flat_ident.to_string(), "FlatJustifiedType123");

        info!("Asserting justified ident is 'JustifiedType123'");
        assert_eq!(justified_ident.to_string(), "JustifiedType123");

        info!("Asserting justification ident is 'Type123Justification'");
        assert_eq!(justification_ident.to_string(), "Type123Justification");

        info!("Asserting confidence ident is 'Type123Confidence'");
        assert_eq!(confidence_ident.to_string(), "Type123Confidence");

        debug!("check_struct_name_with_numbers completed successfully");
    }

    #[traced_test]
    fn check_struct_name_with_underscores() {
        trace!("Starting check_struct_name_with_underscores");
        let input_ident = Ident::new("My_Struct_With_Underscores", proc_macro2::Span::call_site());
        debug!("Calling create_flat_justified_idents_for_named with 'My_Struct_With_Underscores'");
        let (flat_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justified_idents_for_named(&input_ident, input_ident.span());

        info!("Asserting flat ident is 'FlatJustifiedMy_Struct_With_Underscores'");
        assert_eq!(flat_ident.to_string(), "FlatJustifiedMy_Struct_With_Underscores");

        info!("Asserting justified ident is 'JustifiedMy_Struct_With_Underscores'");
        assert_eq!(justified_ident.to_string(), "JustifiedMy_Struct_With_Underscores");

        info!("Asserting justification ident is 'My_Struct_With_UnderscoresJustification'");
        assert_eq!(justification_ident.to_string(), "My_Struct_With_UnderscoresJustification");

        info!("Asserting confidence ident is 'My_Struct_With_UnderscoresConfidence'");
        assert_eq!(confidence_ident.to_string(), "My_Struct_With_UnderscoresConfidence");

        debug!("check_struct_name_with_underscores completed successfully");
    }

    #[traced_test]
    fn check_struct_name_with_mixed_cases() {
        trace!("Starting check_struct_name_with_mixed_cases");
        let input_ident = Ident::new("mIxEdCaSe", proc_macro2::Span::call_site());
        debug!("Calling create_flat_justified_idents_for_named with 'mIxEdCaSe'");
        let (flat_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justified_idents_for_named(&input_ident, input_ident.span());

        info!("Asserting flat ident is 'FlatJustifiedmIxEdCaSe'");
        assert_eq!(flat_ident.to_string(), "FlatJustifiedmIxEdCaSe");

        info!("Asserting justified ident is 'JustifiedmIxEdCaSe'");
        assert_eq!(justified_ident.to_string(), "JustifiedmIxEdCaSe");

        info!("Asserting justification ident is 'mIxEdCaSeJustification'");
        assert_eq!(justification_ident.to_string(), "mIxEdCaSeJustification");

        info!("Asserting confidence ident is 'mIxEdCaSeConfidence'");
        assert_eq!(confidence_ident.to_string(), "mIxEdCaSeConfidence");

        debug!("check_struct_name_with_mixed_cases completed successfully");
    }
}
