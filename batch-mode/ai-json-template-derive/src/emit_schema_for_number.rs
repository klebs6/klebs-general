// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_number.rs ]
crate::ix!();

pub fn emit_schema_for_number(
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_number invoked");
    quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    }
}
