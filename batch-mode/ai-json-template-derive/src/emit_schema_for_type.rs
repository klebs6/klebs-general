// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_type.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn emit_schema_for_type(
    ty: &syn::Type,
    doc_lit: proc_macro2::Literal,
    required: bool
) -> Option<proc_macro2::TokenStream>
{
    trace!("Starting emit_schema_for_type");

    let required_bool = if required {
        quote::quote!(true)
    } else {
        quote::quote!(false)
    };
    let generation_instructions = format!("{}", doc_lit);
    let type_str = quote::quote!(#ty).to_string();

    trace!(
        "emit_schema_for_type => required={}, type={}",
        required,
        type_str
    );

    // 1) If Option<T> => handle T with required=false
    if let Some(inner) = extract_option_inner(ty) {
        trace!("Detected Option => classifying the inner type as 'required=false'");
        let child_ts = emit_schema_for_type(inner, doc_lit, false)?;
        return Some(quote::quote!({
            #child_ts
        }));
    }

    // 2) If Vec<T> => specialized "array_of*"
    if let Some(elem_ty) = extract_vec_inner(ty) {
        debug!("Detected Vec<T> => specialized array_of expansions");
        return Some(emit_schema_for_vec(
            elem_ty,
            &generation_instructions,
            &required_bool
        ));
    }

    // 3) If HashMap<K, V> => specialized "map_of*"
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        debug!("Detected HashMap => specialized expansions for map");
        return Some(emit_schema_for_hashmap(
            k_ty,
            v_ty,
            &generation_instructions,
            &required_bool
        ));
    }

    // 4) If bool => "boolean" schema
    if is_bool(ty) {
        return Some(emit_schema_for_bool(&generation_instructions, &required_bool));
    }

    // 5) If string => "string" schema
    if is_string_type(ty) {
        return Some(emit_schema_for_string(&generation_instructions, &required_bool));
    }

    // 6) If numeric => "number" schema
    if is_numeric(ty) {
        return Some(emit_schema_for_number(&generation_instructions, &required_bool));
    }

    // 7) Otherwise => treat as custom "nested_struct_or_enum"
    trace!("Falling back to nested struct/enum expansion for type={}", type_str);
    Some(emit_schema_for_fallback_nested(
        ty,
        &generation_instructions,
        &required_bool
    ))
}

#[cfg(test)]
mod test_emit_schema_for_type {
    use super::*;
    use quote::quote;
    use syn::{Type, parse_str};
    use traced_test::traced_test;
    use tracing::{trace, debug, info, warn, error};
    use serde_json::{Value, json};
    use ai_json_template::*;

    // Helper to remove all run-together whitespace so we can do substring checks
    // without worrying about how `quote::quote!` or `.to_string()` decides to space tokens.
    fn remove_ws(s: &str) -> String {
        s.split_whitespace().collect()
    }

    #[traced_test]
    fn test_emit_schema_for_type_bool() {
        trace!("Starting test_emit_schema_for_type_bool...");

        let ty: Type = parse_str("bool").expect("Unable to parse bool type");
        let doc_lit = proc_macro2::Literal::string("Testing boolean schema");
        let required = true;

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for bool type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for bool: {}", ts_str);

        // Compare canonical forms (no whitespace differences)
        let actual = remove_ws(&ts_str);
        let expected_bool = remove_ws("\"boolean\"");
        assert!(
            actual.contains(&expected_bool),
            "Expected '\"boolean\"' in bool schema, got: {}",
            ts_str
        );

        let expected_required = remove_ws("\"required\".to_string(),serde_json::Value::Bool(true)");
        assert!(
            actual.contains(&expected_required),
            "Expected 'required=true' for bool schema, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_bool passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_string() {
        trace!("Starting test_emit_schema_for_type_string...");

        let ty: Type = parse_str("String").expect("Unable to parse String type");
        let doc_lit = proc_macro2::Literal::string("Testing string schema");
        let required = false;

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for String type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for String: {}", ts_str);

        let actual = remove_ws(&ts_str);
        let expected_str = remove_ws("\"string\"");
        assert!(
            actual.contains(&expected_str),
            "Expected '\"string\"' in string schema, got: {}",
            ts_str
        );

        let expected_req_false = remove_ws("\"required\".to_string(),serde_json::Value::Bool(false)");
        assert!(
            actual.contains(&expected_req_false),
            "Expected 'required=false' for string schema, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_string passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_numeric() {
        trace!("Starting test_emit_schema_for_type_numeric...");

        // We will test multiple numeric types in a loop.
        let numeric_types = &["i32", "u64", "f32", "f64"];

        for &nty in numeric_types {
            debug!("Testing numeric type '{}'", nty);
            let ty: Type = parse_str(nty).expect("Unable to parse numeric type");
            let doc_lit = proc_macro2::Literal::string("Testing numeric schema");
            let required = true;

            let token_stream_opt = emit_schema_for_type(&ty, doc_lit.clone(), required);
            assert!(
                token_stream_opt.is_some(),
                "Expected Some(...) for numeric type '{}', got None",
                nty
            );

            let token_stream = token_stream_opt.unwrap();
            let ts_str = token_stream.to_string();
            debug!("Generated token stream for {}: {}", nty, ts_str);

            let actual = remove_ws(&ts_str);
            let expected_number = remove_ws("\"number\"");
            assert!(
                actual.contains(&expected_number),
                "Expected '\"number\"' in numeric schema for '{}'. Actual: {}",
                nty,
                ts_str
            );

            let expected_required_true =
                remove_ws("\"required\".to_string(),serde_json::Value::Bool(true)");
            assert!(
                actual.contains(&expected_required_true),
                "Expected 'required=true' for numeric schema '{}', got: {}",
                nty,
                ts_str
            );
        }
        info!("test_emit_schema_for_type_numeric passed.");
    }


    #[traced_test]
    fn test_emit_schema_for_type_vec() {
        trace!("Starting test_emit_schema_for_type_vec...");

        let ty: Type = parse_str("Vec<i32>").expect("Unable to parse Vec<i32> type");
        let doc_lit = proc_macro2::Literal::string("Testing Vec<i32> schema");
        let required = false;

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for Vec<i32> type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for Vec<i32>: {}", ts_str);

        let actual = remove_ws(&ts_str);
        let array_of_numbers = remove_ws("\"array_of_numbers\"");
        let array_of_fallback = remove_ws("\"array_of\"");

        assert!(
            actual.contains(&array_of_numbers) || actual.contains(&array_of_fallback),
            "Expected 'array_of_numbers' or 'array_of' for Vec<i32>, got: {}",
            ts_str
        );

        let expected_required_false =
            remove_ws("\"required\".to_string(),serde_json::Value::Bool(false)");
        assert!(
            actual.contains(&expected_required_false),
            "Expected 'required=false' for Vec<i32>, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_vec passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_hashmap() {
        trace!("Starting test_emit_schema_for_type_hashmap...");

        // We'll test HashMap<String, i32>
        let ty: Type = parse_str("std::collections::HashMap<String, i32>")
            .expect("Unable to parse HashMap<String, i32> type");
        let doc_lit = proc_macro2::Literal::string("Testing HashMap<String, i32> schema");
        let required = true;

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for HashMap<String, i32> type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for HashMap<String, i32>: {}", ts_str);

        let actual = remove_ws(&ts_str);
        let possible_map_of = remove_ws("\"map_of\"");
        let possible_map_of_numbers = remove_ws("\"map_of_numbers\"");
        let possible_map_of_strings = remove_ws("\"map_of_strings\"");

        assert!(
            actual.contains(&possible_map_of)
                || actual.contains(&possible_map_of_numbers)
                || actual.contains(&possible_map_of_strings),
            "Expected a 'map_of' style schema for HashMap<String, i32>, got: {}",
            ts_str
        );

        let expected_required_true =
            remove_ws("\"required\".to_string(),serde_json::Value::Bool(true)");
        assert!(
            actual.contains(&expected_required_true),
            "Expected 'required=true' for HashMap<String, i32>, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_hashmap passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_option() {
        trace!("Starting test_emit_schema_for_type_option...");

        let ty: Type = parse_str("Option<String>").expect("Unable to parse Option<String> type");
        let doc_lit = proc_macro2::Literal::string("Testing Option<String> schema");
        let required = true; 
        // The function sees Option<T> => T with required=false.

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for Option<String> type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for Option<String>: {}", ts_str);

        let actual = remove_ws(&ts_str);

        // We'll allow either "string", "nested_struct", "nested_enum", or "nested_struct_or_enum".
        let possible_string = remove_ws("\"string\"");
        let possible_nested_enum = remove_ws("\"nested_enum\"");
        let possible_nested_struct = remove_ws("\"nested_struct\"");
        let possible_nested_se = remove_ws("\"nested_struct_or_enum\"");

        assert!(
            actual.contains(&possible_string)
            || actual.contains(&possible_nested_enum)
            || actual.contains(&possible_nested_struct)
            || actual.contains(&possible_nested_se),
            "Expected 'string', 'nested_enum', 'nested_struct', or 'nested_struct_or_enum' in Option<String> schema, got: {}",
            ts_str
        );

        // Because it's Option, the *inner* required might appear as false:
        let expected_required_child_false =
            remove_ws("\"required\".to_string(),serde_json::Value::Bool(false)");
        assert!(
            actual.contains(&expected_required_child_false),
            "Expected 'required=false' for the child schema inside Option<String>, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_option passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_custom_nested() {
        trace!("Starting test_emit_schema_for_type_custom_nested...");

        // We simulate a custom type e.g. "MyCustomStruct"
        let ty: Type = parse_str("MyCustomStruct").expect("Unable to parse MyCustomStruct type");
        let doc_lit = proc_macro2::Literal::string("Testing custom nested schema");
        let required = false;

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for MyCustomStruct type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for MyCustomStruct: {}", ts_str);

        let actual = remove_ws(&ts_str);

        // We'll allow "nested_enum", "nested_struct", or "nested_struct_or_enum" in the fallback.
        let possible_enum = remove_ws("\"nested_enum\"");
        let possible_struct = remove_ws("\"nested_struct\"");
        let possible_se = remove_ws("\"nested_struct_or_enum\"");

        assert!(
            actual.contains(&possible_enum)
            || actual.contains(&possible_struct)
            || actual.contains(&possible_se),
            "Expected 'nested_enum', 'nested_struct', or 'nested_struct_or_enum' in custom nested schema, got: {}",
            ts_str
        );

        let expected_required_false =
            remove_ws("\"required\".to_string(),serde_json::Value::Bool(false)");
        assert!(
            actual.contains(&expected_required_false),
            "Expected 'required=false' for MyCustomStruct fallback, got: {}",
            ts_str
        );

        info!("test_emit_schema_for_type_custom_nested passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_bad_type_handling() {
        trace!("Starting test_emit_schema_for_type_bad_type_handling...");

        // e.g. "fn() -> i32"
        let ty_result = parse_str::<Type>("fn() -> i32");

        match ty_result {
            Ok(ty) => {
                let doc_lit = proc_macro2::Literal::string("Testing function pointer type");
                let token_stream_opt = emit_schema_for_type(&ty, doc_lit, true);

                // We expect either None or a compile_error snippet. We'll just confirm it doesn't panic.
                if token_stream_opt.is_none() {
                    warn!("Function pointer gave None, which is acceptable for unknown types.");
                } else {
                    let ts_str = token_stream_opt.unwrap().to_string();
                    debug!("Got token stream for function pointer: {}", ts_str);

                    let actual = remove_ws(&ts_str);
                    let compile_err = remove_ws("compile_error!");
                    if actual.contains(&compile_err) {
                        info!("Recognized compile_error for function pointer type, acceptable.");
                    } else {
                        info!("Fallback was used for function pointer type, also acceptable.");
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse function pointer type: {}", e);
                panic!("Unable to proceed with function pointer test");
            }
        }
        info!("test_emit_schema_for_type_bad_type_handling passed.");
    }
}
