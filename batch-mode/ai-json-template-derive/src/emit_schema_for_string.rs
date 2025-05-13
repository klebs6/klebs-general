// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_string.rs ]
crate::ix!();

pub fn emit_schema_for_string(
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_string invoked");
    quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    }
}

#[cfg(test)]
mod test_emit_schema_for_string_schema_output {
    use super::*;

    #[traced_test]
    fn generates_required_string_schema_with_instructions() {
        trace!("Testing string schema generation with required = true");

        let instructions = "Please enter a valid name";
        let required = quote::quote!(true);
        let schema_ts = emit_schema_for_string(instructions, &required);

        let schema_str = schema_ts.to_string();
        debug!(%schema_str, "Generated schema token stream");

        // Evaluate into a JSON value
        let json_expr = schema_ts.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&json_expr.replace("# [", "\"").replace("]", "\"")).expect("Failed to parse generated schema");

        assert_eq!(parsed["type"], "string");
        assert_eq!(parsed["generation_instructions"], instructions);
        assert_eq!(parsed["required"], true);
    }

    #[traced_test]
    fn generates_optional_string_schema_with_instructions() {
        trace!("Testing string schema generation with required = false");

        let instructions = "Provide optional alias";
        let required = quote::quote!(false);
        let schema_ts = emit_schema_for_string(instructions, &required);

        let schema_str = schema_ts.to_string();
        debug!(%schema_str, "Generated schema token stream");

        let json_expr = schema_ts.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&json_expr.replace("# [", "\"").replace("]", "\"")).expect("Failed to parse generated schema");

        assert_eq!(parsed["type"], "string");
        assert_eq!(parsed["generation_instructions"], instructions);
        assert_eq!(parsed["required"], false);
    }

    #[traced_test]
    fn string_schema_preserves_all_fields() {
        trace!("Ensuring all expected fields exist in the output schema");

        let instructions = "Example field doc";
        let required = quote::quote!(true);
        let schema_ts = emit_schema_for_string(instructions, &required);

        let schema_str = schema_ts.to_string();
        debug!(%schema_str, "Schema token stream as string");

        let json_expr = schema_ts.to_string();
        let parsed: serde_json::Value = serde_json::from_str(&json_expr.replace("# [", "\"").replace("]", "\"")).expect("Failed to parse schema");

        assert!(parsed.get("type").is_some(), "Missing 'type' key");
        assert!(parsed.get("generation_instructions").is_some(), "Missing 'generation_instructions' key");
        assert!(parsed.get("required").is_some(), "Missing 'required' key");
    }
}
