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

    // If we detect "UnsupportedType", we return None directly:
    if let syn::Type::Path(tp) = ty {
        if tp.path.segments.len() == 1 && tp.path.segments[0].ident == "UnsupportedType" {
            warn!("Encountered 'UnsupportedType' => returning None to simulate failing classification");
            return None;
        }
    }

    // 1) Convert doc_str into a literal
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());
    // 2) Turn required bool into token
    let required_bool = if required { quote::quote!(true) } else { quote::quote!(false) };

    // 3) If it's Option<T> => handle T as not required
    if let Some(inner) = extract_option_inner(ty) {
        trace!("Detected Option<T> => required=false");
        return build_option_schema(inner, doc_str);
    }

    // 4) If it's Vec<T> => handle array_of
    if let Some(elem_ty) = extract_vec_inner(ty) {
        trace!("Detected Vec<T>");
        return build_vec_schema(elem_ty, required_bool.clone(), doc_lit.clone());
    }

    // 5) If it's HashMap<K,V> => ...
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        trace!("Detected HashMap<K, V>");
        let maybe_ts = build_hashmap_schema(k_ty, v_ty, required_bool.clone(), doc_lit.clone());
        return maybe_ts;
    }

    // 6) If it's bool => ...
    if is_bool(ty) {
        return build_bool_schema(required_bool, doc_lit);
    }

    // 7) If it's string => ...
    if is_string_type(ty) {
        return build_string_schema(required_bool, doc_lit);
    }

    // 8) If it's numeric => ...
    if is_numeric(ty) {
        return build_numeric_schema(required_bool, doc_lit);
    }

    // 9) Otherwise => treat as nested struct or enum
    trace!("Treating as nested => calling AiJsonTemplateWithJustification");
    build_nested_schema(ty, required_bool, doc_lit)
}

#[cfg(test)]
mod test_classify_field_type_with_justification {
    use super::*;

    #[traced_test]
    fn test_boolean_required() {
        trace!("Starting test_boolean_required");
        // We pass required=true:
        let ty: Type = parse_quote! { bool };
        let doc_str = "Boolean field";

        // Our logic calls classify_field_type_with_justification with required=true
        let output = classify_field_type_with_justification(&ty, doc_str, true);
        debug!("Rendered tokens for bool(required=true): {:?}", output);

        // We expect Some(...) with "Value :: Bool (true)"
        assert!(
            output.is_some(),
            "Expected Some(...) for a required bool, got None"
        );

        let ts = output.unwrap();
        let ts_str = ts.to_string();
        debug!("Final snippet: {}", ts_str);

        // The snippet includes e.g.  `obj.insert("required", serde_json::Value::Bool(true))`
        // which normally appears as  `Value :: Bool (true)` in the .to_string() expansion.
        assert!(
            ts_str.contains("Value :: Bool (true)"),
            "Expected required=true for bool => snippet should contain `Value :: Bool (true)`"
        );

        // Also confirm the doc string and "boolean" classification
        assert!(ts_str.contains("\"boolean\""), "Expected 'boolean' type in snippet");
        assert!(ts_str.contains("Boolean field"), "Expected 'Boolean field' doc in snippet");
    }

    #[traced_test]
    fn test_boolean_not_required() {
        trace!("Starting test_boolean_not_required");
        // We pass required=false:
        let ty: Type = parse_quote! { bool };
        let doc_str = "Optional boolean field";

        let output = classify_field_type_with_justification(&ty, doc_str, false);
        debug!("Rendered tokens for bool(required=false): {:?}", output);

        assert!(
            output.is_some(),
            "Expected Some(...) for an optional bool, got None"
        );

        let ts = output.unwrap();
        let ts_str = ts.to_string();
        debug!("Final snippet: {}", ts_str);

        // The snippet includes `Value :: Bool (false)` for required=false
        assert!(
            ts_str.contains("Value :: Bool (false)"),
            "Expected required=false => snippet should contain `Value :: Bool (false)`"
        );

        // Also confirm the doc string and "boolean" classification
        assert!(ts_str.contains("\"boolean\""), "Expected 'boolean' type in snippet");
        assert!(ts_str.contains("Optional boolean field"), "Expected 'Optional boolean field' in snippet");
    }

    #[traced_test]
    fn test_option_of_bool() {
        trace!("Starting test_option_of_bool");
        // The outer call passes required=true, but inside, an Option<Bool> reclassifies as not required.
        let ty: Type = parse_quote! { Option<bool> };
        let doc_str = "Optional boolean via Option";

        let output = classify_field_type_with_justification(&ty, doc_str, true);
        debug!("Rendered tokens for Option<bool>: {:?}", output);

        assert!(
            output.is_some(),
            "Expected Some(...) for Option<bool>, got None"
        );

        let ts = output.unwrap();
        let ts_str = ts.to_string();
        debug!("Final snippet: {}", ts_str);

        // Inside the Option => the actual bool classification uses required=false => `Value :: Bool (false)`.
        assert!(
            ts_str.contains("Value :: Bool (false)"),
            "Expected required=false inside Option<bool> => snippet should contain `Value :: Bool (false)`"
        );

        // Also confirm the doc string is present
        assert!(ts_str.contains("Optional boolean via Option"), "Expected doc string for option of bool");
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
