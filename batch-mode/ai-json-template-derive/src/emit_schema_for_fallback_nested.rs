// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_fallback_nested.rs ]
crate::ix!();

pub fn emit_schema_for_fallback_nested(
    ty: &syn::Type,
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
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

#[cfg(test)]
mod emit_schema_for_fallback_nested_exhaustive_validation {
    use super::*;
    use ai_json_template::*;
    use serde_derive::*;
    use save_load_derive::*;
    use save_load_traits::*;
    use std::fs;

    // A simple "fake" type that returns an object with "enum_name" key
    // to simulate a nested enum scenario.
    #[derive(Debug,SaveLoad,Clone,Deserialize,Serialize)]
    struct FakeEnumType;
    impl AiJsonTemplate for FakeEnumType {
        fn to_template() -> serde_json::Value {
            // Produce a JSON object with "enum_name" to trigger "nested_enum"
            json!({
                "enum_name": "FakeEnum",
            })
        }
    }

    // Another "fake" type that returns an object with "type"="complex_enum"
    // to also trigger a nested enum classification.
    #[derive(Debug,SaveLoad,Clone,Deserialize,Serialize)]
    struct FakeComplexEnumType;
    impl AiJsonTemplate for FakeComplexEnumType {
        fn to_template() -> serde_json::Value {
            // Produce a JSON object with "type":"complex_enum"
            json!({
                "type": "complex_enum",
                "other_data": 123
            })
        }
    }

    // Another "fake" type that returns an object with "struct_name" to trigger "nested_struct".
    #[derive(Debug,SaveLoad,Clone,Deserialize,Serialize)]
    struct FakeStructType;
    impl AiJsonTemplate for FakeStructType {
        fn to_template() -> serde_json::Value {
            // Produce a JSON object with "struct_name" => "FakeStruct"
            json!({
                "struct_name": "FakeStruct"
            })
        }
    }

    // A fallback "fake" type that returns something non-object or an empty object
    // so that we default to "nested_struct" classification.
    #[derive(Debug,SaveLoad,Clone,Deserialize,Serialize)]
    struct FakeFallbackType;
    impl AiJsonTemplate for FakeFallbackType {
        fn to_template() -> serde_json::Value {
            // Return either an empty JSON object or something else
            // so we have no "enum_name"/"struct_name" and thus fallback to "nested_struct"
            json!({})
        }
    }

    #[traced_test]
    fn confirm_nested_enum_for_enum_name_key() {
        info!("Testing emit_schema_for_fallback_nested for a type that returns 'enum_name'...");
        let ty: Type = parse_str("FakeEnumType").expect("Failed to parse 'FakeEnumType'");
        let required_bool = quote!(true);

        let ts = emit_schema_for_fallback_nested(&ty, "Instructions for enum-like", &required_bool);
        let result_str = ts.to_string();
        debug!("Token stream: {}", result_str);

        // Because FakeEnumType includes "enum_name", we expect "nested_enum"
        assert!(
            result_str.contains("\"nested_enum\""),
            "Expected 'nested_enum' in the output for 'enum_name', got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"required\": true"),
            "Expected required=true, got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"generation_instructions\": \"Instructions for enum-like\""),
            "Expected generation instructions to match, got: {}",
            result_str
        );
        info!("Completed confirm_nested_enum_for_enum_name_key successfully.");
    }

    #[traced_test]
    fn confirm_nested_enum_for_complex_enum_type() {
        info!("Testing emit_schema_for_fallback_nested for a type with 'type':'complex_enum'...");
        let ty: Type = parse_str("FakeComplexEnumType").expect("Failed to parse 'FakeComplexEnumType'");
        let required_bool = quote!(false);

        let ts = emit_schema_for_fallback_nested(&ty, "Complex enum instructions", &required_bool);
        let result_str = ts.to_string();
        trace!("Token stream: {}", result_str);

        // Because FakeComplexEnumType includes "type":"complex_enum", we expect "nested_enum"
        assert!(
            result_str.contains("\"nested_enum\""),
            "Expected 'nested_enum' for complex_enum, got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"required\": false"),
            "Expected required=false, got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"Complex enum instructions\""),
            "Expected correct generation_instructions, got: {}",
            result_str
        );
        info!("Completed confirm_nested_enum_for_complex_enum_type successfully.");
    }

    #[traced_test]
    fn confirm_nested_struct_for_struct_name_key() {
        info!("Testing emit_schema_for_fallback_nested for a type that returns 'struct_name'...");
        let ty: Type = parse_str("FakeStructType").expect("Failed to parse 'FakeStructType'");
        let required_bool = quote!(true);

        let ts = emit_schema_for_fallback_nested(&ty, "Some struct instructions", &required_bool);
        let result_str = ts.to_string();
        debug!("Token stream: {}", result_str);

        // Because FakeStructType includes "struct_name", we expect "nested_struct"
        assert!(
            result_str.contains("\"nested_struct\""),
            "Expected 'nested_struct', got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"required\": true"),
            "Expected required=true, got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"Some struct instructions\""),
            "Expected generation_instructions, got: {}",
            result_str
        );
        info!("Completed confirm_nested_struct_for_struct_name_key successfully.");
    }

    #[traced_test]
    fn confirm_nested_struct_for_fallback() {
        info!("Testing emit_schema_for_fallback_nested fallback scenario...");
        let ty: Type = parse_str("FakeFallbackType").expect("Failed to parse 'FakeFallbackType'");
        let required_bool = quote!(false);

        let ts = emit_schema_for_fallback_nested(&ty, "Fallback instructions", &required_bool);
        let result_str = ts.to_string();
        trace!("Token stream: {}", result_str);

        // Because FakeFallbackType returns an empty object (no enum_name/struct_name/type=complex_enum),
        // we expect "nested_struct" fallback
        assert!(
            result_str.contains("\"nested_struct\""),
            "Expected fallback to 'nested_struct', got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"required\": false"),
            "Expected required=false, got: {}",
            result_str
        );
        assert!(
            result_str.contains("\"Fallback instructions\""),
            "Expected generation_instructions, got: {}",
            result_str
        );
        info!("Completed confirm_nested_struct_for_fallback successfully.");
    }
}
