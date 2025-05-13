// ---------------- [ File: ai-json-template-derive/src/classify_field_type_for_child.rs ]
crate::ix!();

pub fn classify_field_type_for_child(
    field_ty: &syn::Type,
    doc_str: &str,
    required: bool,
    skip_child_just: bool,
) -> Option<proc_macro2::TokenStream> {
    let doc_lit = proc_macro2::Literal::string(doc_str.trim());
    let required_bool = if required {
        quote::quote!(true)
    } else {
        quote::quote!(false)
    };

    if let Some(inner) = extract_option_inner(field_ty) {
        let snippet = classify_field_type_for_child(inner, doc_str, false, skip_child_just)?;
        return Some(quote::quote!({
            #snippet
        }));
    }

    if let Some(elem) = extract_vec_inner(field_ty) {
        let item_snippet = classify_field_type_for_child(elem, doc_str, true, skip_child_just)?;
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                obj.insert("item_template".to_string(), #item_snippet);
                serde_json::Value::Object(obj)
            }
        });
    }

    if let Some((k_ty, v_ty)) = extract_hashmap_inner(field_ty) {
        if is_bool(k_ty) {
            let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
            let e = syn::Error::new(k_ty.span(), &err_msg);
            return Some(e.to_compile_error());
        }

        let key_schema = if is_numeric(k_ty) {
            quote::quote!(serde_json::Value::String("number".to_string()))
        } else if is_string_type(k_ty) {
            quote::quote!(serde_json::Value::String("string".to_string()))
        } else {
            let child_expr = if skip_child_just {
                quote::quote!(<#k_ty as AiJsonTemplate>::to_template())
            } else {
                quote::quote!(<#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
            };
            quote::quote! {
                {
                    let mut k_obj = serde_json::Map::new();
                    k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    k_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    k_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    let nested_k = #child_expr;
                    k_obj.insert("nested_template".to_string(), nested_k);
                    serde_json::Value::Object(k_obj)
                }
            }
        };

        let val_schema = if is_bool(v_ty) {
            quote::quote!(serde_json::Value::String("boolean".to_string()))
        } else if is_numeric(v_ty) {
            quote::quote!(serde_json::Value::String("number".to_string()))
        } else if is_string_type(v_ty) {
            quote::quote!(serde_json::Value::String("string".to_string()))
        } else {
            let child_expr = if skip_child_just {
                quote::quote!(<#v_ty as AiJsonTemplate>::to_template())
            } else {
                quote::quote!(<#v_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
            };
            quote::quote! {
                {
                    let mut v_obj = serde_json::Map::new();
                    v_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                    v_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                    v_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    let nested_v = #child_expr;
                    v_obj.insert("nested_template".to_string(), nested_v);
                    serde_json::Value::Object(v_obj)
                }
            }
        };

        return Some(quote::quote! {
            {
                let mut map_obj = serde_json::Map::new();
                map_obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
                map_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                map_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                map_obj.insert("map_key_template".to_string(), #key_schema);
                map_obj.insert("map_value_template".to_string(), #val_schema);
                serde_json::Value::Object(map_obj)
            }
        });
    }

    if is_bool(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    if is_string_type(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    if is_numeric(field_ty) {
        return Some(quote::quote! {
            {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                serde_json::Value::Object(obj)
            }
        });
    }

    let child_expr = if skip_child_just {
        quote::quote!(<#field_ty as AiJsonTemplate>::to_template())
    } else {
        quote::quote!(<#field_ty as AiJsonTemplateWithJustification>::to_template_with_justification())
    };
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

            let nested = #child_expr;
            obj.insert("nested_template".to_string(), nested);
            serde_json::Value::Object(obj)
        }
    })
}

#[cfg(test)]
mod classify_field_type_for_child_exhaustive_validation {
    use super::*;

    #[traced_test]
    fn confirm_handling_of_numeric_field() {
        info!("Starting confirm_handling_of_numeric_field...");
        let ty: Type = parse_str("i32").expect("Unable to parse i32");
        let doc_str = "Example numeric doc";
        let required = false;
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying numeric type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        debug!("Resulting token stream: {}", tokens);

        // We expect "type": "number", plus "required": false => 
        // typically "Value :: Bool (false)" or "Value::Bool(false)"
        assert!(
            tokens.contains("\"number\""),
            "Expected output to contain 'number', got: {}",
            tokens
        );
        assert!(
            tokens.contains("Bool (false)") || tokens.contains("Bool(false)"),
            "Expected 'required' to be false for numeric, got: {}",
            tokens
        );
        assert!(
            tokens.contains("Example numeric doc"),
            "Expected doc string in snippet, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_numeric_field successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_string_field() {
        info!("Starting confirm_handling_of_string_field...");
        let ty: Type = parse_str("String").expect("Unable to parse String");
        let doc_str = "Example string doc";
        let required = true;
        let skip_child_just = true;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying string type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        debug!("Resulting token stream: {}", tokens);

        // We expect "type": "string", plus "required": true => 
        // typically "Bool (true)" or "Bool(true)"
        assert!(
            tokens.contains("\"string\""),
            "Expected output to contain 'string', got: {}",
            tokens
        );
        assert!(
            tokens.contains("Bool (true)") || tokens.contains("Bool(true)"),
            "Expected 'required' to be true for string, got: {}",
            tokens
        );
        assert!(
            tokens.contains("Example string doc"),
            "Expected doc string in snippet, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_string_field successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_option_field() {
        info!("Starting confirm_handling_of_option_field...");
        let ty: Type = parse_str("Option<i64>").expect("Unable to parse Option<i64>");
        let doc_str = "Example optional doc";
        let required = true; // though the Option should override to false inside
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying Option type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        trace!("Resulting token stream: {}", tokens);

        // Because it's Option<...>, we expect the final snippet to contain "required": false 
        // and some reference to "number" for i64.
        // The actual snippet might wrap it in an extra block => e.g. "{ { ... } }".
        assert!(
            tokens.contains("\"number\""),
            "Expected the Option's inner numeric type to appear, got: {}",
            tokens
        );
        assert!(
            tokens.contains("Bool (false)") || tokens.contains("Bool(false)"),
            "Expected 'required' to be false for an Option field, got: {}",
            tokens
        );
        assert!(
            tokens.contains("Example optional doc"),
            "Expected doc string in snippet, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_option_field successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_vec_field() {
        info!("Starting confirm_handling_of_vec_field...");
        let ty: Type = parse_str("Vec<String>").expect("Unable to parse Vec<String>");
        let doc_str = "Example vector doc";
        let required = false;
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying Vec type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        debug!("Resulting token stream: {}", tokens);

        // The parent is "array_of", required=false
        // The child "item_template" might be a string => required=true
        // The test only cares that the *parent* is required=false
        assert!(
            tokens.contains("\"array_of\""),
            "Expected the result to contain 'array_of', got: {}",
            tokens
        );
        // confirm parent's `required` is false:
        // Typically => `obj.insert("required", serde_json::Value::Bool(false))`
        assert!(
            tokens.contains("Bool (false)") || tokens.contains("Bool(false)"),
            "Expected 'required' to be false for parent Vec field, got: {}",
            tokens
        );
        assert!(
            tokens.contains("Example vector doc"),
            "Expected doc string in snippet, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_vec_field successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_hashmap_field_with_string_key_and_bool_value() {
        info!("Starting confirm_handling_of_hashmap_field_with_string_key_and_bool_value...");
        let ty: Type = parse_str("std::collections::HashMap<String, bool>")
            .expect("Unable to parse HashMap<String, bool>");
        let doc_str = "HashMap doc";
        let required = true;
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying HashMap, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        debug!("Resulting token stream: {}", tokens);

        assert!(
            tokens.contains("\"map_of\""),
            "Expected 'map_of' in the classification, got: {}",
            tokens
        );
        // Check key=string, value=boolean
        assert!(
            tokens.contains("\"string\""),
            "Expected 'string' for map key, got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"boolean\""),
            "Expected 'boolean' for map value, got: {}",
            tokens
        );
        // required=true for the parent 
        assert!(
            tokens.contains("Bool (true)") || tokens.contains("Bool(true)"),
            "Expected required=true for the parent HashMap field, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_hashmap_field_with_string_key_and_bool_value successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_hashmap_field_with_unsupported_bool_key() {
        info!("Starting confirm_handling_of_hashmap_field_with_unsupported_bool_key...");
        let ty: Type = parse_str("HashMap<bool, i32>").expect("Unable to parse HashMap<bool, i32>");
        let doc_str = "Invalid bool-keyed map";
        let required = false;
        let skip_child_just = true;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        // The code returns Some(TokenStream) containing compile_error if bool is used as a key
        assert!(
            result.is_some(),
            "We still expect Some(TokenStream) containing compile_error for bool keys"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        trace!("Resulting token stream: {}", tokens);

        assert!(
            tokens.contains("compile_error"),
            "Expected a compile_error snippet for bool key type, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_hashmap_field_with_unsupported_bool_key successfully.");
    }

    #[traced_test]
    fn confirm_handling_of_fallback_nested_struct_or_enum() {
        info!("Starting confirm_handling_of_fallback_nested_struct_or_enum...");
        let ty: Type = parse_str("UserType").expect("Unable to parse user-defined type");
        let doc_str = "Example user type doc";
        let required = true;
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) for user-defined fallback nested type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        debug!("Resulting token stream: {}", tokens);

        // We expect it to contain "nested_struct_or_enum", plus call AiJsonTemplateWithJustification if skip_child_just=false
        assert!(
            tokens.contains("\"nested_struct_or_enum\""),
            "Expected fallback to nested_struct_or_enum, got: {}",
            tokens
        );
        assert!(
            tokens.contains("AiJsonTemplateWithJustification"),
            "Expected call to AiJsonTemplateWithJustification, got: {}",
            tokens
        );
        // required=true for the parent
        assert!(
            tokens.contains("Bool (true)") || tokens.contains("Bool(true)"),
            "Expected required=true for user type, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_fallback_nested_struct_or_enum successfully.");
    }

    /// A helper that runs `classify_field_type` and returns the stringified TokenStream,
    /// or panics if `None` is returned.
    fn run_classify_and_stringify(ty: &Type, doc_str: &str) -> String {
        trace!("Invoking classify_field_type on type: {:?}, doc_str={:?}", ty, doc_str);
        let result_opt = classify_field_type(ty, doc_str);
        match result_opt {
            Some(ts) => {
                let out = ts.to_string();
                debug!("Successfully obtained TokenStream: {}", out);
                out
            },
            None => {
                panic!("Expected Some(TokenStream) but got None for type={:?}, doc_str={:?}", ty, doc_str);
            }
        }
    }

    #[traced_test]
    fn test_option_bool() {
        info!("test_option_bool => Checking classification for Option<bool>");
        let ty: Type = parse_quote!(Option<bool>);
        let output = run_classify_and_stringify(&ty, "example doc for Option<bool>");
        // Should have "boolean", "required=false"
        assert!(
            output.contains("\"boolean\""),
            "Should contain 'boolean' for bool type, got: {}",
            output
        );
        // The actual code typically prints `Value :: Bool (false)`, or `Value::Bool(false)`
        // with no space around (false).
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<bool>, got: {}",
            output
        );
        // Also check we have the doc string as "\"example doc for Option<bool>\"" or similar
        // (the macros typically embed the doc with an extra quote).
        assert!(
            output.contains("example doc for Option<bool>"),
            "Expected doc string for Option<bool>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_bool() {
        info!("test_bool => Checking classification for bare bool");
        let ty: Type = parse_quote!(bool);
        let output = run_classify_and_stringify(&ty, "example doc for bool");
        // Should have "boolean", "required=true"
        assert!(
            output.contains("\"boolean\""),
            "Should contain 'boolean' for bool type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for bare bool, got: {}",
            output
        );
        assert!(
            output.contains("example doc for bool"),
            "Expected doc string for bool, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_string() {
        info!("test_option_string => Checking classification for Option<String>");
        let ty: Type = parse_quote!(Option<String>);
        let output = run_classify_and_stringify(&ty, "doc for Option<String>");
        // Should have "string", "required=false"
        assert!(
            output.contains("\"string\""),
            "Should contain 'string' for String type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<String>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<String>"),
            "Expected doc string for Option<String>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_string() {
        info!("test_string => Checking classification for bare String");
        let ty: Type = parse_quote!(String);
        let output = run_classify_and_stringify(&ty, "doc for bare String");
        // Should have "string", "required=true"
        assert!(
            output.contains("\"string\""),
            "Should contain 'string' for String type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for bare String, got: {}",
            output
        );
        assert!(
            output.contains("doc for bare String"),
            "Expected doc string for bare String, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_i32() {
        info!("test_i32 => Checking classification for i32");
        let ty: Type = parse_quote!(i32);
        let output = run_classify_and_stringify(&ty, "doc for i32");
        // Should have "number", "required=true"
        assert!(
            output.contains("\"number\""),
            "Should contain 'number' for numeric type i32, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for i32, got: {}",
            output
        );
        assert!(
            output.contains("doc for i32"),
            "Expected doc string for i32, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_f64() {
        info!("test_option_f64 => Checking classification for Option<f64>");
        let ty: Type = parse_quote!(Option<f64>);
        let output = run_classify_and_stringify(&ty, "doc for Option<f64>");
        // Should have "number", "required=false"
        assert!(
            output.contains("\"number\""),
            "Should contain 'number' for numeric type f64, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<f64>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<f64>"),
            "Expected doc string for Option<f64>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_nested_custom_type() {
        info!("test_nested_custom_type => Checking classification for a user-defined type");
        let ty: Type = parse_quote!(MyCustomType);
        let output = run_classify_and_stringify(&ty, "doc for MyCustomType");
        // Expect fallback => "nested_struct_or_enum", "required=true"
        // But the macro might produce a snippet that calls AiJsonTemplate::to_template() 
        // plus some logic to guess "nested_enum"/"nested_struct". We just check for "required=true" 
        // and "nested" in the snippet.
        assert!(
            output.contains("nested_struct_or_enum")
                || output.contains("nested_struct") 
                || output.contains("nested_enum"),
            "Expected fallback to 'nested_struct_or_enum' or 'nested_struct' for unknown user type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for MyCustomType, got: {}",
            output
        );
        assert!(
            output.contains("doc for MyCustomType"),
            "Expected doc for MyCustomType, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_nested_custom_type() {
        info!("test_option_nested_custom_type => Checking classification for Option of user-defined type");
        let ty: Type = parse_quote!(Option<AnotherType>);
        let output = run_classify_and_stringify(&ty, "doc for Option<AnotherType>");
        // Expect fallback => "nested_struct_or_enum", "required=false"
        assert!(
            output.contains("nested_struct_or_enum")
                || output.contains("nested_struct") 
                || output.contains("nested_enum"),
            "Expected fallback to 'nested_struct_or_enum', 'nested_struct', or 'nested_enum' for unknown user type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<AnotherType>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<AnotherType>"),
            "Expected doc string for Option<AnotherType>, got: {}",
            output
        );
    }

    #[traced_test]
    fn confirm_handling_of_bool_field() {

        info!("confirm_handling_of_bool_field...");

        // Suppose we call classify_field_type_for_child with required=true
        let required        = true;
        let doc_str         = "Example bool doc";
        let skip_child_just = false;

        let snippet         = classify_field_type_for_child(
            &parse_quote!(bool), 
            doc_str, 
            required, 
            skip_child_just
        ).expect("expected this to unwrap");

        debug!("Resulting token stream: {}", snippet.to_string());

        // The snippet includes something like `obj.insert("required", serde_json::Value::Bool(true))`
        assert!(
            snippet.to_string().contains("Bool (true)") 
            || snippet.to_string().contains("Bool(true)"),
            "Expected 'required' to be true for bool, got: {}",
            snippet.to_string()
        );
        // Also check "boolean" is present
        assert!(
            snippet.to_string().contains("\"boolean\""),
            "Expected 'boolean' classification in snippet, got: {}",
            snippet.to_string()
        );
        assert!(
            snippet.to_string().contains(doc_str),
            "Expected doc string in snippet, got: {}",
            snippet.to_string()
        );
    }
}
