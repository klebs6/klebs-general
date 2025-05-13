// ---------------- [ File: ai-json-template-derive/src/create_flat_justification_idents_for_enum.rs ]
crate::ix!();

/// Creates four Ident values used when generating a flat-justified enum.
/// They are:
///  - FlatJustified{Type}
///  - Justified{Type}
///  - {Type}Justification
///  - {Type}Confidence
///
/// The name alone conveys that it’s producing identifiers for our “flat justification” expansions.
pub fn create_flat_justification_idents_for_enum(
    enum_ident: &Ident,
    span: proc_macro2::Span
) -> (Ident, Ident, Ident, Ident) {
    trace!("Creating four Ident values for enum: {}", enum_ident);

    let flat_enum_ident = Ident::new(&format!("FlatJustified{}", enum_ident), span);
    let justified_ident = Ident::new(&format!("Justified{}", enum_ident), span);
    let justification_id = Ident::new(&format!("{}Justification", enum_ident), span);
    let confidence_id = Ident::new(&format!("{}Confidence", enum_ident), span);

    debug!("Flat enum ident:   {}", flat_enum_ident);
    debug!("Justified ident:   {}", justified_ident);
    debug!("Justification ident: {}", justification_id);
    debug!("Confidence ident:    {}", confidence_id);

    (flat_enum_ident, justified_ident, justification_id, confidence_id)
}

#[cfg(test)]
mod test_create_flat_justification_idents_for_enum {
    use super::*;

    #[traced_test]
    fn test_generates_four_idents_correctly() {
        let span = proc_macro2::Span::call_site();
        let input_enum = Ident::new("FooEnum", span);
        trace!("Starting test: test_generates_four_idents_correctly");

        let (flat_id, just_id, j_id, c_id) 
            = create_flat_justification_idents_for_enum(&input_enum, span);

        assert_eq!(flat_id.to_string(), "FlatJustifiedFooEnum");
        assert_eq!(just_id.to_string(), "JustifiedFooEnum");
        assert_eq!(j_id.to_string(), "FooEnumJustification");
        assert_eq!(c_id.to_string(), "FooEnumConfidence");
    }

    #[traced_test]
    fn test_standard_enum_name() {
        trace!("Beginning test_standard_enum_name");
        let enum_ident = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let (flat_enum_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justification_idents_for_enum(&enum_ident, enum_ident.span());

        debug!("Asserting returned Ident values for a typical enum name");
        assert_eq!(flat_enum_ident.to_string(), "FlatJustifiedMyEnum");
        assert_eq!(justified_ident.to_string(), "JustifiedMyEnum");
        assert_eq!(justification_ident.to_string(), "MyEnumJustification");
        assert_eq!(confidence_ident.to_string(), "MyEnumConfidence");

        info!("test_standard_enum_name passed successfully");
    }

    #[traced_test]
    fn test_underscored_enum_name() {
        trace!("Beginning test_underscored_enum_name");
        let enum_ident = Ident::new("Some_Complex_Enum", proc_macro2::Span::call_site());
        let (flat_enum_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justification_idents_for_enum(&enum_ident, enum_ident.span());

        debug!("Asserting returned Ident values for an underscored enum name");
        assert_eq!(flat_enum_ident.to_string(), "FlatJustifiedSome_Complex_Enum");
        assert_eq!(justified_ident.to_string(), "JustifiedSome_Complex_Enum");
        assert_eq!(justification_ident.to_string(), "Some_Complex_EnumJustification");
        assert_eq!(confidence_ident.to_string(), "Some_Complex_EnumConfidence");

        info!("test_underscored_enum_name passed successfully");
    }

    #[traced_test]
    fn test_empty_enum_name() {
        trace!("Beginning test_empty_enum_name");
        // While unusual, let's see how our function behaves with an empty ident:
        let enum_ident = Ident::new("", proc_macro2::Span::call_site());
        let (flat_enum_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justification_idents_for_enum(&enum_ident, enum_ident.span());

        debug!("Asserting returned Ident values for an empty enum name");
        // The function simply appends to an empty string:
        assert_eq!(flat_enum_ident.to_string(), "FlatJustified");
        assert_eq!(justified_ident.to_string(), "Justified");
        assert_eq!(justification_ident.to_string(), "Justification");
        assert_eq!(confidence_ident.to_string(), "Confidence");

        info!("test_empty_enum_name passed successfully");
    }

    #[traced_test]
    fn test_symbolic_enum_name() {
        trace!("Beginning test_symbolic_enum_name");
        // Not typical, but let's see if we can create an Ident with non-alphabetic characters:
        let enum_ident = Ident::new("Enum_123", proc_macro2::Span::call_site());
        let (flat_enum_ident, justified_ident, justification_ident, confidence_ident) =
            create_flat_justification_idents_for_enum(&enum_ident, enum_ident.span());

        debug!("Asserting returned Ident values for a symbolic/numbered enum name");
        assert_eq!(flat_enum_ident.to_string(), "FlatJustifiedEnum_123");
        assert_eq!(justified_ident.to_string(), "JustifiedEnum_123");
        assert_eq!(justification_ident.to_string(), "Enum_123Justification");
        assert_eq!(confidence_ident.to_string(), "Enum_123Confidence");

        info!("test_symbolic_enum_name passed successfully");
    }
}
