// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_type.rs ]
crate::ix!();

pub fn emit_schema_for_type(
    ty: &syn::Type,
    doc_lit: proc_macro2::Literal,
    required: bool
) -> Option<proc_macro2::TokenStream> {
    trace!("Starting emit_schema_for_type");

    let required_bool = if required {
        quote!(true)
    } else {
        quote!(false)
    };
    let generation_instructions = format!("{}", doc_lit);
    let type_str = quote!(#ty).to_string();

    trace!(
        "emit_schema_for_type => required={} type={}",
        required,
        type_str
    );

    // (A) Handle bool, string, numeric
    if is_bool(ty) {
        return Some(emit_schema_for_bool(&generation_instructions, &required_bool));
    }
    if is_string_type(ty) {
        return Some(emit_schema_for_string(&generation_instructions, &required_bool));
    }
    if is_numeric(ty) {
        return Some(emit_schema_for_number(&generation_instructions, &required_bool));
    }

    // (B) Handle Vec<T>
    if let Some(elem_ty) = extract_vec_inner(ty) {
        debug!("Detected Vec<T> => specialized array-of expansions");
        return Some(emit_schema_for_vec(
            elem_ty,
            &generation_instructions,
            &required_bool,
        ));
    }

    // (C) Handle HashMap<K, V>
    if let Some((k_ty, v_ty)) = extract_hashmap_inner(ty) {
        debug!("Detected HashMap => specialized expansions for map");
        return Some(emit_schema_for_hashmap(
            k_ty,
            v_ty,
            &generation_instructions,
            &required_bool,
        ));
    }

    // (D) Fallback => treat as a nested struct or enum
    trace!(
        "Falling back to nested struct/enum expansion for type={}",
        type_str
    );
    Some(emit_schema_for_fallback_nested(
        ty,
        &generation_instructions,
        &required_bool,
    ))
}

#[cfg(test)]
mod test_emit_schema_for_type {
    use super::*;

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
        assert!(
            ts_str.contains("\"boolean\""),
            "Expected '\"boolean\"' in bool schema"
        );
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(true)"),
            "Expected 'required=true' for bool schema"
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
        assert!(
            ts_str.contains("\"string\""),
            "Expected '\"string\"' in string schema"
        );
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected 'required=false' for string schema"
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

            assert!(
                ts_str.contains("\"number\""),
                "Expected '\"number\"' in numeric schema for '{}'",
                nty
            );
            assert!(
                ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(true)"),
                "Expected 'required=true' for numeric schema '{}'",
                nty
            );
        }
        info!("test_emit_schema_for_type_numeric passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_option() {
        trace!("Starting test_emit_schema_for_type_option...");

        let ty: Type = parse_str("Option<String>").expect("Unable to parse Option<String> type");
        let doc_lit = proc_macro2::Literal::string("Testing Option<String> schema");
        let required = true; 
        // The function will see it's an Option and treat the inner as not required.

        let token_stream_opt = emit_schema_for_type(&ty, doc_lit, required);
        assert!(
            token_stream_opt.is_some(),
            "Expected Some(...) for Option<String> type, got None"
        );

        let token_stream = token_stream_opt.unwrap();
        let ts_str = token_stream.to_string();
        debug!("Generated token stream for Option<String>: {}", ts_str);

        // We expect something about "string" or "nested".
        // The function's approach is to treat Option<T> as T with 'required=false' internally.
        assert!(
            ts_str.contains("\"string\"") || ts_str.contains("\"nested_struct_or_enum\""),
            "Expected either 'string' or 'nested_struct_or_enum' in Option<String> schema"
        );

        // Because it's an Option, required might show up as false for the child:
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected 'required=false' for the child schema inside Option<String>"
        );
        info!("test_emit_schema_for_type_option passed.");
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

        assert!(
            ts_str.contains("\"array_of_numbers\"") || ts_str.contains("\"array_of\""),
            "Expected 'array_of_numbers' or 'array_of' for Vec<i32>"
        );
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected 'required=false' for Vec<i32>"
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

        // We expect to see "map_of" or something referencing "map_of_numbers" possibly.
        assert!(
            ts_str.contains("\"map_of\"")
                || ts_str.contains("\"map_of_numbers\"")
                || ts_str.contains("\"map_of_strings\""),
            "Expected a 'map_of' style schema for HashMap<String, i32>"
        );
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(true)"),
            "Expected 'required=true' for HashMap<String, i32>"
        );
        info!("test_emit_schema_for_type_hashmap passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_custom_nested() {
        trace!("Starting test_emit_schema_for_type_custom_nested...");

        // We simulate a custom type e.g. "MyType", which isn't recognized as a builtin.
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

        // For fallback nested, we expect "nested_struct_or_enum" or something similar:
        assert!(
            ts_str.contains("\"nested_struct_or_enum\""),
            "Expected 'nested_struct_or_enum' in custom nested schema"
        );
        assert!(
            ts_str.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
            "Expected 'required=false' for MyCustomStruct fallback"
        );
        info!("test_emit_schema_for_type_custom_nested passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_type_bad_type_handling() {
        trace!("Starting test_emit_schema_for_type_bad_type_handling...");

        // If it's a pointer or something unusual, we might get None or a compile_error
        // We'll try to parse an unusual type signature:
        // e.g. "fn() -> i32"
        let ty_result = parse_str::<Type>("fn() -> i32");

        match ty_result {
            Ok(ty) => {
                // We expect fallback might just treat it as None or produce a compile error token stream
                let doc_lit = proc_macro2::Literal::string("Testing function pointer type");
                let token_stream_opt = emit_schema_for_type(&ty, doc_lit, true);

                // If the function pointer is not recognized, the function might produce None or an error code snippet
                // We'll just check that it does not panic and see if it's Some or None.
                if token_stream_opt.is_none() {
                    warn!("Function pointer gave None, which is acceptable for unknown types.");
                } else {
                    // The function might produce a compile_error token stream. Let's see:
                    let ts_str = token_stream_opt.unwrap().to_string();
                    debug!("Got token stream for function pointer: {}", ts_str);

                    // We'll look for compile_error
                    if ts_str.contains("compile_error!") {
                        info!("Recognized compile_error for function pointer type, which is acceptable.");
                    } else {
                        // It's also possible the fallback is used. We'll accept that as well.
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
