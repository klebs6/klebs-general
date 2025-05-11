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
    use traced_test::traced_test;

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
}
