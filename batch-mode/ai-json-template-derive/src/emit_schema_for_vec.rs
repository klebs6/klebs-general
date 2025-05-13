// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_vec.rs ]
crate::ix!();

pub fn emit_schema_for_vec(
    elem_ty: &syn::Type,
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_vec invoked");
    if is_numeric(elem_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_numbers".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_bool(elem_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_booleans".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else if is_string_type(elem_ty) {
        return quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        };
    } else {
        quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

                let nested_t = <#elem_ty as AiJsonTemplate>::to_template();
                obj.insert("item_template".to_string(), nested_t);

                serde_json::Value::Object(obj)
            }
        }
    }
}

#[cfg(test)]
mod test_emit_schema_for_vec_exhaustively {
    use super::*;

    #[traced_test]
    fn test_vec_of_i32() {
        trace!("Starting test_vec_of_i32 for emit_schema_for_vec");
        let elem_ty: syn::Type = parse_quote! { i32 };
        let generation_instructions = "Generate an array of integers";
        let required_bool = quote! { true };

        let result = emit_schema_for_vec(&elem_ty, generation_instructions, &required_bool);
        debug!("Resulting token stream: {}", result);

        let ts_string = result.to_string();
        info!("Checking if output matches expected content for array_of_numbers");
        assert!(
            ts_string.contains("\"array_of_numbers\""),
            "Expected schema to identify array_of_numbers but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"required\": serde_json :: Value :: Bool ( true )"),
            "Expected required: true in schema but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"generation_instructions\": serde_json :: Value :: String ( \"Generate an array of integers\".to_string())"),
            "Missing generation_instructions in schema but got: {}",
            ts_string
        );
    }

    #[traced_test]
    fn test_vec_of_bool() {
        trace!("Starting test_vec_of_bool for emit_schema_for_vec");
        let elem_ty: syn::Type = parse_quote! { bool };
        let generation_instructions = "Generate a list of booleans";
        let required_bool = quote! { false };

        let result = emit_schema_for_vec(&elem_ty, generation_instructions, &required_bool);
        debug!("Resulting token stream: {}", result);

        let ts_string = result.to_string();
        info!("Checking if output matches expected content for array_of_booleans");
        assert!(
            ts_string.contains("\"array_of_booleans\""),
            "Expected schema to identify array_of_booleans but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"required\": serde_json :: Value :: Bool ( false )"),
            "Expected required: false in schema but got: {}",
            ts_string
        );
    }

    #[traced_test]
    fn test_vec_of_strings() {
        trace!("Starting test_vec_of_strings for emit_schema_for_vec");
        let elem_ty: syn::Type = parse_quote! { String };
        let generation_instructions = "Generate a list of strings";
        let required_bool = quote! { true };

        let result = emit_schema_for_vec(&elem_ty, generation_instructions, &required_bool);
        debug!("Resulting token stream: {}", result);

        let ts_string = result.to_string();
        info!("Checking if output matches expected content for array_of_strings");
        assert!(
            ts_string.contains("\"array_of_strings\""),
            "Expected schema to identify array_of_strings but got: {}",
            ts_string
        );
    }

    #[traced_test]
    fn test_vec_of_nested_custom_type() {
        trace!("Starting test_vec_of_nested_custom_type for emit_schema_for_vec");
        // Simulate a user-defined type named MyCustomType
        let elem_ty: syn::Type = parse_quote! { MyCustomType };
        let generation_instructions = "Generate an array of MyCustomType";
        let required_bool = quote! { true };

        let result = emit_schema_for_vec(&elem_ty, generation_instructions, &required_bool);
        debug!("Resulting token stream: {}", result);

        let ts_string = result.to_string();
        info!("Checking if output matches expected content for array_of + item_template for nested type");
        assert!(
            ts_string.contains("\"type\"") && ts_string.contains("\"array_of\""),
            "Expected schema to identify array_of for nested type but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"nested_template\""),
            "Expected nested_template but not found in: {}",
            ts_string
        );
    }

    #[traced_test]
    fn test_vec_of_f64_with_custom_instructions() {
        trace!("Starting test_vec_of_f64_with_custom_instructions for emit_schema_for_vec");
        let elem_ty: syn::Type = parse_quote! { f64 };
        let generation_instructions = "Generate double-precision floats in an array";
        let required_bool = quote! { false };

        let result = emit_schema_for_vec(&elem_ty, generation_instructions, &required_bool);
        debug!("Resulting token stream: {}", result);

        let ts_string = result.to_string();
        info!("Checking final schema content for array_of_numbers with required=false");
        assert!(
            ts_string.contains("\"array_of_numbers\""),
            "Expected schema to identify array_of_numbers for f64 but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"required\": serde_json :: Value :: Bool ( false )"),
            "Expected required: false in schema but got: {}",
            ts_string
        );
        assert!(
            ts_string.contains("\"Generate double-precision floats in an array\""),
            "Expected custom generation_instructions but got: {}",
            ts_string
        );
    }
}
