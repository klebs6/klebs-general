// ---------------- [ File: ai-json-template-derive/src/build_nested_schema.rs ]
crate::ix!();

/// Builds the fallback schema snippet for any nested struct/enum => call `AiJsonTemplateWithJustification`.
pub fn build_nested_schema(
    ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut nested_obj = serde_json::Map::new();
            nested_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
            nested_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            nested_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

            let nested = <#ty as AiJsonTemplateWithJustification>::to_template_with_justification();
            nested_obj.insert("nested_template".to_string(), nested);
            serde_json::Value::Object(nested_obj)
        }
    })
}
