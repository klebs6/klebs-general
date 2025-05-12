// ---------------- [ File: ai-json-template-derive/src/field_just_conf_mapping.rs ]
crate::ix!();

/// Restored logic that, for each named field, creates e.g. `fieldname_justification: String` and
/// `fieldname_confidence: f32`. If the field is a nested type, we produce e.g. `SomeTypeJustification` 
/// and `SomeTypeConfidence`.
#[derive(Builder, Debug, Clone, Getters, Setters)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct FieldJustConfMapping {
    field_ident:               syn::Ident,
    justification_field_ident: syn::Ident,
    confidence_field_ident:    syn::Ident,
    justification_field_type:  proc_macro2::TokenStream,
    confidence_field_type:     proc_macro2::TokenStream,
}
