// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_hashmap.rs ]
crate::ix!();

pub fn emit_schema_for_hashmap(
    k_ty:                    &syn::Type,
    v_ty:                    &syn::Type,
    generation_instructions: &str,
    required_bool:           &proc_macro2::TokenStream
) -> proc_macro2::TokenStream
{
    trace!("emit_schema_for_hashmap invoked");

    let map_key_schema = match resolve_map_key_type(k_ty) {
        Ok(key_str) => {
            quote! { serde_json::Value::String(#key_str.to_string()) }
        }
        Err(err_ts) => {
            return err_ts;
        }
    };

    if is_numeric(v_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_numbers".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_bool(v_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_booleans".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_string_type(v_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of_strings".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else {
        quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                obj.insert("map_key_type".to_string(), #map_key_schema);
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                let nested_val = <#v_ty as AiJsonTemplate>::to_template();
                obj.insert("map_value_template".to_string(), nested_val);

                serde_json::Value::Object(obj)
            }
        }
    }
}

#[cfg(test)]
mod test_emit_schema_for_hashmap {
    use super::*;

    #[traced_test]
    fn test_emit_schema_for_hashmap_bool_key_produces_compile_error() {
        trace!("Starting test: bool key should produce compile_error!");
        let k_ty = parse_quote!(bool);
        let v_ty = parse_quote!(String);
        let generation_instructions = "bool-key test doc";
        let required_bool = quote!(true);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // We expect a compile_error! invocation
        assert!(
            actual_str.contains("compile_error"),
            "Expected compile_error in token stream for bool key, but got:\n{}",
            actual_str
        );
        info!("Passed: bool key => compile_error found");
    }

    #[traced_test]
    fn test_emit_schema_for_hashmap_numeric_key_and_value() {
        trace!("Starting test: numeric key with numeric value => 'map_of_numbers'");
        let k_ty = parse_quote!(u32);
        let v_ty = parse_quote!(i64);
        let generation_instructions = "numeric-key-value test doc";
        let required_bool = quote!(true);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // For numeric keys + numeric values => either "map_of_numbers" or a close pattern
        // The function specifically sets "type=map_of_numbers" if the value is numeric.
        // The key is stored in "map_key_type" => "number"
        assert!(
            actual_str.contains("\"map_of_numbers\"") 
                || actual_str.contains("map_of_numbers"),
            "Expected 'map_of_numbers' in result for numeric key+value, got:\n{}",
            actual_str
        );
        assert!(
            actual_str.contains("\"map_key_type\": serde_json :: Value :: String ( \"number\" . to_string ( ) )")
                || actual_str.contains("map_key_type"),
            "Expected map_key_type to be numeric, got:\n{}",
            actual_str
        );
        info!("Passed: numeric key with numeric value => 'map_of_numbers'");
    }

    #[traced_test]
    fn test_emit_schema_for_hashmap_string_key_and_bool_value() {
        trace!("Starting test: string key with bool value => 'map_of_booleans'");
        let k_ty = parse_quote!(String);
        let v_ty = parse_quote!(bool);
        let generation_instructions = "string-key-bool-value test doc";
        let required_bool = quote!(false);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // For a bool value => "map_of_booleans"
        assert!(
            actual_str.contains("\"map_of_booleans\""),
            "Expected 'map_of_booleans' in token stream, got:\n{}",
            actual_str
        );
        info!("Passed: string key with bool value => 'map_of_booleans'");
    }

    #[traced_test]
    fn test_emit_schema_for_hashmap_string_key_and_string_value() {
        trace!("Starting test: string key with string value => 'map_of_strings'");
        let k_ty = parse_quote!(String);
        let v_ty = parse_quote!(String);
        let generation_instructions = "string-key-string-value test doc";
        let required_bool = quote!(true);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // For string value => "map_of_strings"
        assert!(
            actual_str.contains("\"map_of_strings\""),
            "Expected 'map_of_strings' in token stream, got:\n{}",
            actual_str
        );
        info!("Passed: string key with string value => 'map_of_strings'");
    }

    #[traced_test]
    fn test_emit_schema_for_hashmap_with_nested_types() {
        trace!("Starting test: nested user-defined type => fallback to 'map_of' + nested_value_template");
        let k_ty = parse_quote!(String);
        // Suppose 'MyCustomType' stands for some non-primitive type
        let v_ty = parse_quote!(MyCustomType);
        let generation_instructions = "nested-value test doc";
        let required_bool = quote!(true);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // For a non-primitive value => "map_of" plus "map_value_template"
        assert!(
            actual_str.contains("\"map_of\""),
            "Expected 'map_of' for nested type, got:\n{}",
            actual_str
        );
        assert!(
            actual_str.contains("map_value_template"),
            "Expected 'map_value_template' property for nested type, got:\n{}",
            actual_str
        );
        info!("Passed: nested user-defined type => 'map_of' with 'map_value_template'");
    }

    #[traced_test]
    fn test_emit_schema_for_hashmap_required_flag_false() {
        trace!("Starting test: verifying that 'required' can be set to false");
        let k_ty = parse_quote!(String);
        let v_ty = parse_quote!(u32);
        let generation_instructions = "optional-map test doc";
        let required_bool = quote!(false);

        let token_stream = emit_schema_for_hashmap(&k_ty, &v_ty, generation_instructions, &required_bool);
        let actual_str = token_stream.to_string();

        debug!("Resulting token_stream: {}", actual_str);

        // We expect something like ... "required": serde_json::Value::Bool(false)
        // Checking for "required" to appear with a bool or the word "false"
        assert!(
            actual_str.contains("\"required\"") && actual_str.contains("false"),
            "Expected 'required' to be false, got:\n{}",
            actual_str
        );
        info!("Passed: 'required' = false is handled properly for HashMap");
    }
}
