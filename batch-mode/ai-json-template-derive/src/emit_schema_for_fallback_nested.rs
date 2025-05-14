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

#[cfg(test)]
mod test_emit_schema_for_fallback_nested {
    use super::*;
    use quote::quote;
    use syn::parse_quote;
    use serde_json::{Value, json};
    use tracing::{trace, debug, info, warn, error};
    use traced_test::traced_test;
    use ai_json_template::*;
    use save_load_derive::*;
    use save_load_traits::*;
    use serde::*;
    use serde_derive::*;

    // We'll define some local mock types to simulate different
    // outputs for <T as AiJsonTemplate>::to_template().
    //
    // The function under test, emit_schema_for_fallback_nested,
    // inspects the JSON object returned by AiJsonTemplate::to_template()
    // to decide whether the result is "nested_enum" or "nested_struct".
    // Also, we want a scenario where the nested template is not even an object
    // to confirm that the code's fallback path is taken.

    // 1) Type that returns an object with "enum_name"
    //    => code should label it as "nested_enum".
    #[derive(SaveLoad,Clone,Serialize,Deserialize,Debug)]
    struct EnumNameMock;

    impl AiJsonTemplate for EnumNameMock {
        fn to_template() -> serde_json::Value {
            trace!("EnumNameMock => returning JSON object with 'enum_name' key");
            let mut obj = serde_json::Map::new();
            obj.insert("enum_name".to_string(), serde_json::Value::String("MyEnumName".to_string()));
            serde_json::Value::Object(obj)
        }
    }

    // 2) Type that returns an object with "type": "complex_enum"
    //    => code should label it as "nested_enum" as well.
    #[derive(SaveLoad,Clone,Serialize,Deserialize,Debug)]
    struct ComplexEnumMock;

    impl AiJsonTemplate for ComplexEnumMock {
        fn to_template() -> serde_json::Value {
            trace!("ComplexEnumMock => returning JSON object with 'type'='complex_enum'");
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("complex_enum".to_string()));
            serde_json::Value::Object(obj)
        }
    }

    // 3) Type that returns an object with "struct_name"
    //    => code should label it as "nested_struct".
    #[derive(SaveLoad,Clone,Serialize,Deserialize,Debug)]
    struct StructNameMock;

    impl AiJsonTemplate for StructNameMock {
        fn to_template() -> serde_json::Value {
            trace!("StructNameMock => returning JSON object with 'struct_name' key");
            let mut obj = serde_json::Map::new();
            obj.insert("struct_name".to_string(), serde_json::Value::String("MyStructName".to_string()));
            serde_json::Value::Object(obj)
        }
    }

    // 4) Type that returns an object with none of the recognized keys
    //    => code should label it as "nested_struct" by default.
    #[derive(SaveLoad,Clone,Serialize,Deserialize,Debug)]
    struct NoSpecialKeysMock;

    impl AiJsonTemplate for NoSpecialKeysMock {
        fn to_template() -> serde_json::Value {
            trace!("NoSpecialKeysMock => returning JSON object with no recognized keys");
            let mut obj = serde_json::Map::new();
            obj.insert("random_key".to_string(), serde_json::Value::String("random_value".to_string()));
            serde_json::Value::Object(obj)
        }
    }

    // 5) Type that returns a non-object (e.g. a string) => fallback is "nested_struct".
    #[derive(SaveLoad,Clone,Serialize,Deserialize,Debug)]
    struct NonObjectMock;

    impl AiJsonTemplate for NonObjectMock {
        fn to_template() -> serde_json::Value {
            trace!("NonObjectMock => returning a non-object value (string)");
            serde_json::Value::String("I'm not an object".to_string())
        }
    }

    #[traced_test]
    fn test_nested_enum_by_enum_name() {
        trace!("test_nested_enum_by_enum_name => Start");
        let ty: syn::Type = parse_quote!(EnumNameMock);
        let instructions = "doc instructions for enum_name mock";

        debug!("Invoking emit_schema_for_fallback_nested with required=false");
        let required_bool = quote!(false);

        let result = emit_schema_for_fallback_nested(&ty, instructions, &required_bool);
        debug!("Got result: {}", result);

        let code_str = result.to_string();
        debug!("Code string:\n{}", code_str);

        // Because "enum_name" is recognized, the code sets "nested_enum"
        assert!(
            code_str.contains("\"nested_enum\""),
            "Should set 'type' to nested_enum in generated code"
        );
        // Check we embed the correct generation_instructions
        assert!(
            code_str.contains("doc instructions for enum_name mock"),
            "Should embed 'doc instructions for enum_name mock' in the code"
        );
        // Check the "required" boolean
        assert!(
            code_str.contains("false"),
            "Should embed false as the 'required' value"
        );
        // Check presence of nested_template
        assert!(
            code_str.contains("\"nested_template\""),
            "Should embed 'nested_template' key in the code"
        );

        info!("test_nested_enum_by_enum_name => PASSED");
    }

    #[traced_test]
    fn test_nested_enum_by_complex_enum() {
        trace!("test_nested_enum_by_complex_enum => Start");
        let ty: syn::Type = parse_quote!(ComplexEnumMock);
        let instructions = "doc instructions for complex_enum mock";

        debug!("Invoking emit_schema_for_fallback_nested with required=true");
        let required_bool = quote!(true);

        let result = emit_schema_for_fallback_nested(&ty, instructions, &required_bool);
        debug!("Got result: {}", result);

        let code_str = result.to_string();
        debug!("Code string:\n{}", code_str);

        // Because "type" was "complex_enum", the code sets "nested_enum"
        assert!(
            code_str.contains("\"nested_enum\""),
            "Should set 'type' to nested_enum in generated code"
        );
        assert!(
            code_str.contains("doc instructions for complex_enum mock"),
            "Should embed 'doc instructions for complex_enum mock' in the code"
        );
        assert!(
            code_str.contains("true"),
            "Should embed true as the 'required' value"
        );
        assert!(
            code_str.contains("\"nested_template\""),
            "Should embed 'nested_template' key in the code"
        );

        info!("test_nested_enum_by_complex_enum => PASSED");
    }

    #[traced_test]
    fn test_nested_struct_by_struct_name() {
        trace!("test_nested_struct_by_struct_name => Start");
        let ty: syn::Type = parse_quote!(StructNameMock);
        let instructions = "doc instructions for struct_name mock";

        debug!("Invoking emit_schema_for_fallback_nested with required=false");
        let required_bool = quote!(false);

        let result = emit_schema_for_fallback_nested(&ty, instructions, &required_bool);
        debug!("Got result: {}", result);

        let code_str = result.to_string();
        debug!("Code string:\n{}", code_str);

        // Because "struct_name" is present, the code sets "nested_struct"
        assert!(
            code_str.contains("\"nested_struct\""),
            "Should set 'type' to nested_struct in generated code"
        );
        assert!(
            code_str.contains("doc instructions for struct_name mock"),
            "Should embed 'doc instructions for struct_name mock' in the code"
        );
        assert!(
            code_str.contains("false"),
            "Should embed false as the 'required' value"
        );
        assert!(
            code_str.contains("\"nested_template\""),
            "Should embed 'nested_template' key in the code"
        );

        info!("test_nested_struct_by_struct_name => PASSED");
    }

    #[traced_test]
    fn test_nested_struct_fallback() {
        trace!("test_nested_struct_fallback => Start");
        let ty: syn::Type = parse_quote!(NoSpecialKeysMock);
        let instructions = "doc instructions for no-special-keys mock";

        debug!("Invoking emit_schema_for_fallback_nested with required=true");
        let required_bool = quote!(true);

        let result = emit_schema_for_fallback_nested(&ty, instructions, &required_bool);
        debug!("Got result: {}", result);

        let code_str = result.to_string();
        debug!("Code string:\n{}", code_str);

        // Because none of the recognized keys are present, the code defaults to "nested_struct"
        assert!(
            code_str.contains("\"nested_struct\""),
            "Should set 'type' to nested_struct by default"
        );
        assert!(
            code_str.contains("doc instructions for no-special-keys mock"),
            "Should embed 'doc instructions for no-special-keys mock' in the code"
        );
        assert!(
            code_str.contains("true"),
            "Should embed true as the 'required' value"
        );
        assert!(
            code_str.contains("\"nested_template\""),
            "Should embed 'nested_template' key in the code"
        );

        info!("test_nested_struct_fallback => PASSED");
    }

    #[traced_test]
    fn test_nested_template_is_not_object() {
        trace!("test_nested_template_is_not_object => Start");
        let ty: syn::Type = parse_quote!(NonObjectMock);
        let instructions = "doc instructions for non-object mock";

        debug!("Invoking emit_schema_for_fallback_nested with required=false");
        let required_bool = quote!(false);

        let result = emit_schema_for_fallback_nested(&ty, instructions, &required_bool);
        debug!("Got result: {}", result);

        let code_str = result.to_string();
        debug!("Code string:\n{}", code_str);

        // Because the nested template is not an object at all, the code sets "nested_struct"
        assert!(
            code_str.contains("\"nested_struct\""),
            "Should set 'type' to nested_struct if the nested value is not an object"
        );
        assert!(
            code_str.contains("doc instructions for non-object mock"),
            "Should embed 'doc instructions for non-object mock' in the code"
        );
        assert!(
            code_str.contains("false"),
            "Should embed false as the 'required' value"
        );
        assert!(
            code_str.contains("\"nested_template\""),
            "Should embed 'nested_template' key in the code"
        );

        info!("test_nested_template_is_not_object => PASSED");
    }
}
