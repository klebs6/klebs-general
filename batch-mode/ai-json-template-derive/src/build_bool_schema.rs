crate::ix!();

/// Builds a schema snippet for `bool` => "boolean".
pub fn build_bool_schema(
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    })
}
