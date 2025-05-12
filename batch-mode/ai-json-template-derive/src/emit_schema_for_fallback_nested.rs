// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_fallback_nested.rs ]
crate::ix!();

pub fn emit_schema_for_fallback_nested(
    ty: &syn::Type,
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_fallback_nested invoked");
    quote! {
        {
            let nested = <#ty as AiJsonTemplate>::to_template();
            let mut obj = serde_json::Map::new();

            let nested_as_obj = nested.as_object();
            let nested_type_str = if let Some(o) = nested_as_obj {
                if o.contains_key("enum_name") {
                    "nested_enum"
                } else if o.contains_key("struct_name") {
                    "nested_struct"
                } else if o.contains_key("type") && o["type"] == "complex_enum" {
                    "nested_enum"
                } else {
                    "nested_struct"
                }
            } else {
                "nested_struct"
            };

            obj.insert("type".to_string(), serde_json::Value::String(nested_type_str.to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("nested_template".to_string(), nested);

            serde_json::Value::Object(obj)
        }
    }
}
