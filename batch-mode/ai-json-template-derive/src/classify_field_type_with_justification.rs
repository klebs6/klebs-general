// ---------------- [ File: ai-json-template-derive/src/classify_field_type_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn classify_field_type_with_justification(
    ty: &syn::Type,
    doc_str: &str,
    required: bool
) -> Option<proc_macro2::TokenStream> {
    trace!(
        "classify_field_type_with_justification => type: {:?}, required: {}, doc_str: {:?}",
        ty,
        required,
        doc_str
    );

    // Convert doc_str into a proc-macro Literal
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // Turn the bool into a token we can embed in the final snippet
    let required_bool = if required {
        quote::quote!(true)
    } else {
        quote::quote!(false)
    };

    // 1) If it's Option<T>, handle T as not required
    if let Some(inner) = extract_option_inner(ty) {
        trace!("Detected Option<T> => required=false");
        return build_option_schema(inner, doc_str);
    }

    // 2) If it's Vec<T>, handle array_of
    if let Some(elem_ty) = extract_vec_inner(ty) {
        trace!("Detected Vec<T>");
        return build_vec_schema(elem_ty, required_bool.clone(), doc_lit.clone());
    }

    // 3) If it's HashMap<K,V>, decide how to represent K and V
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        trace!("Detected HashMap<K, V>");
        let maybe_ts = build_hashmap_schema(k_ty, v_ty, required_bool.clone(), doc_lit.clone());

        // If build_hashmap_schema returned None => direct failure
        let ts = maybe_ts?;

        // Check if it contains compile_error! => treat that as an “unsupported” scenario, returning None.
        let ts_str = ts.to_string();
        if ts_str.contains("compile_error !") {
            warn!("compile_error! snippet detected => returning None for classification");
            return None;
        }

        return Some(ts);
    }

    // 4) Builtin bool => "boolean"
    if is_bool(ty) {
        trace!("Detected bool => 'boolean'");
        return build_bool_schema(required_bool, doc_lit);
    }

    // 5) Builtin String => "string"
    if is_string_type(ty) {
        trace!("Detected String => 'string'");
        return build_string_schema(required_bool, doc_lit);
    }

    // 6) Builtin numeric => "number"
    if is_numeric(ty) {
        trace!("Detected numeric => 'number'");
        return build_numeric_schema(required_bool, doc_lit);
    }

    // 7) Otherwise => treat as nested struct/enum => call AiJsonTemplateWithJustification
    trace!("Treating as nested => calling AiJsonTemplateWithJustification");
    build_nested_schema(ty, required_bool, doc_lit)
}

#[cfg(test)]
mod test_classify_field_type_with_justification {
    use super::*;

    #[traced_test]
    fn test_boolean_required() {
        trace!("Starting test_boolean_required");
        let ty: Type = parse_str("bool").unwrap();
        let doc_str = "Boolean field";
        let required = true;

        trace!("Calling classify_field_type_with_justification with bool, required=true");
        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for bool");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for bool");

        assert!(
            rendered.contains("\"boolean\""),
            "Expected 'boolean' in rendered tokens for bool"
        );
        assert!(
            rendered.contains("\"required\".to_string(), serde_json::Value::Bool(true)"),
            "Expected required=true for bool"
        );
    }

    #[traced_test]
    fn test_boolean_not_required() {
        trace!("Starting test_boolean_not_required");
        // This scenario mimics an Option<bool>, so we pass required=false explicitly.
        let ty: Type = parse_str("bool").unwrap();
        let doc_str = "Optional boolean field";
        let required = false;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for bool (optional)");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for optional bool");

        assert!(
            rendered.contains("\"boolean\""),
            "Expected 'boolean' in rendered tokens for optional bool"
        );
        assert!(
            rendered.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected required=false for optional bool"
        );
    }

    #[traced_test]
    fn test_string_required() {
        trace!("Starting test_string_required");
        let ty: Type = parse_str("String").unwrap();
        let doc_str = "String field";
        let required = true;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for String");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for String");

        assert!(
            rendered.contains("\"string\""),
            "Expected 'string' in rendered tokens for String"
        );
        assert!(
            rendered.contains("\"generation_instructions\""),
            "Expected generation_instructions in rendered tokens"
        );
    }

    #[traced_test]
    fn test_numeric_type() {
        trace!("Starting test_numeric_type");
        let ty: Type = parse_str("u32").unwrap();
        let doc_str = "Numeric field";
        let required = true;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for numeric type");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for numeric type");

        assert!(
            rendered.contains("\"number\""),
            "Expected 'number' in rendered tokens for u32"
        );
    }

    #[traced_test]
    fn test_option_of_bool() {
        trace!("Starting test_option_of_bool");
        let ty: Type = parse_str("Option<bool>").unwrap();
        let doc_str = "Optional boolean via Option";
        let required = true; // the outer field might be required, but inside is not

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for Option<bool>");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for Option<bool>");

        // Expect that the classification yields the schema for the inner bool, but effectively not required
        assert!(
            rendered.contains("\"boolean\""),
            "Expected 'boolean' in rendered tokens for Option<bool>"
        );
        // The code treats the inner as required=false
        assert!(
            rendered.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected required=false inside Option<bool> tokens"
        );
    }

    #[traced_test]
    fn test_vec_of_numbers() {
        trace!("Starting test_vec_of_numbers");
        let ty: Type = parse_str("Vec<f64>").unwrap();
        let doc_str = "Vector of floating numbers";
        let required = true;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for Vec<f64>");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for Vec<f64>");

        assert!(
            rendered.contains("\"array_of\""),
            "Expected 'array_of' in rendered tokens for Vec<f64>"
        );
        assert!(
            rendered.contains("\"item_template\""),
            "Expected item_template in rendered tokens for Vec<f64>"
        );
    }

    #[traced_test]
    fn test_hashmap_of_string_to_number() {
        trace!("Starting test_hashmap_of_string_to_number");
        let ty: Type = parse_str("std::collections::HashMap<String, i32>").unwrap();
        let doc_str = "HashMap from String to i32";
        let required = false;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(
            result_tokens.is_some(),
            "Expected Some tokens for HashMap<String, i32>"
        );
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for HashMap<String, i32>");

        assert!(
            rendered.contains("\"map_of\""),
            "Expected 'map_of' in rendered tokens"
        );
        assert!(
            rendered.contains("\"string\"") && rendered.contains("\"number\""),
            "Expected key=string and value=number in rendered tokens"
        );
    }

    #[traced_test]
    fn test_hashmap_of_bool_key_should_fail() {
        trace!("Starting test_hashmap_of_bool_key_should_fail");
        let ty: Type = parse_str("std::collections::HashMap<bool, String>").unwrap();
        let doc_str = "Should produce compile_error for bool key";
        let required = true;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        // We expect Some(...) but with compile_error(...) inside
        assert!(result_tokens.is_some(), "Should yield Some compile_error tokens for bool key");

        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for HashMap<bool, String>");

        // It's a compile_error, so let's check for compile_error
        assert!(
            rendered.contains("compile_error"),
            "Expected compile_error for bool key in HashMap"
        );
    }

    #[traced_test]
    fn test_custom_type_nested() {
        trace!("Starting test_custom_type_nested");
        // Suppose "MyCustomType" is not recognized as bool, string, numeric, or any container
        // => classify_field_type_with_justification falls back to nested_struct_or_enum
        let ty: Type = parse_str("MyCustomType").unwrap();
        let doc_str = "A nested custom type that presumably implements AiJsonTemplateWithJustification";
        let required = true;

        let result_tokens = classify_field_type_with_justification(&ty, doc_str, required);
        assert!(result_tokens.is_some(), "Expected Some tokens for MyCustomType");
        let rendered = result_tokens.unwrap().to_string();
        debug!(?rendered, "Rendered tokens for MyCustomType fallback");

        assert!(
            rendered.contains("\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum fallback for custom type"
        );
        assert!(
            rendered.contains("\"nested_template\""),
            "Expected 'nested_template' field for custom type"
        );
    }
}
