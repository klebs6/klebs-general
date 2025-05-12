// ---------------- [ File: ai-json-template-derive/src/gather_fields_for_just_conf.rs ]
crate::ix!();

pub fn gather_fields_for_just_conf(
    named_fields: &syn::FieldsNamed,
) -> (
    Vec<proc_macro2::TokenStream>, // justification fields
    Vec<proc_macro2::TokenStream>, // confidence fields
    proc_macro2::TokenStream,      // accumulated errors
    Vec<FieldJustConfMapping>,
) {
    trace!("Gathering fields for justification/conf from struct's named fields");

    let mut justification_struct_fields = Vec::new();
    let mut confidence_struct_fields = Vec::new();
    let mut errs = quote::quote!();
    let mut field_mappings = Vec::new();

    gather_justification_and_confidence_fields(
        named_fields,
        &mut justification_struct_fields,
        &mut confidence_struct_fields,
        &mut errs,
        &mut field_mappings,
    );

    debug!(
        "Completed gathering: justification_struct_fields={}, confidence_struct_fields={}, mappings={}",
        justification_struct_fields.len(),
        confidence_struct_fields.len(),
        field_mappings.len()
    );

    (justification_struct_fields, confidence_struct_fields, errs, field_mappings)
}
