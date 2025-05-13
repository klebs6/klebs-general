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
    fn confirm_handling_of_bool_field() {
        info!("Starting confirm_handling_of_bool_field...");
        let ty: Type = parse_str("bool").expect("Unable to parse bool");
        let doc_str = "Example bool doc";
        let required = true;
        let skip_child_just = false;

        let result = classify_field_type_for_child(&ty, doc_str, required, skip_child_just);
        assert!(
            result.is_some(),
            "Expected Some(...) when classifying bool type, got None"
        );
        let tokens = result.unwrap().to_token_stream().to_string();
        trace!("Resulting token stream: {}", tokens);

        assert!(
            tokens.contains("\"boolean\""),
            "Expected output to contain 'boolean', got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"required\": true"),
            "Expected 'required' to be true for bool, got: {}",
            tokens
        );
        info!("Completed confirm_handling_of_bool_field successfully.");
    }

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

        assert!(
            tokens.contains("\"number\""),
            "Expected output to contain 'number', got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"required\": false"),
            "Expected 'required' to be false for numeric, got: {}",
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

        assert!(
            tokens.contains("\"string\""),
            "Expected output to contain 'string', got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"required\": true"),
            "Expected 'required' to be true for string, got: {}",
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

        // For Option, we expect an inner reclassification
        // Ultimately it should contain "number" (due to i64) and "required": false
        assert!(
            tokens.contains("\"number\""),
            "Expected the Option's inner numeric type to appear, got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"required\": false"),
            "Expected 'required' to be false for an Option field, got: {}",
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

        assert!(
            tokens.contains("\"array_of\""),
            "Expected the result to contain 'array_of', got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"string\""),
            "Expected the array item to be 'string', got: {}",
            tokens
        );
        assert!(
            tokens.contains("\"required\": false"),
            "Expected 'required' to be false for parent Vec field, got: {}",
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
        // In the code, it returns Some(...) containing a compile_error() if we see bool keys
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
        // We'll define a made-up user type: "UserType"
        // We only parse the type. We assume the type "UserType" is not recognized as a builtin.
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

        // We expect it to contain "nested_struct_or_enum"
        // plus an invocation of AiJsonTemplateWithJustification if skip_child_just=false
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
        info!("Completed confirm_handling_of_fallback_nested_struct_or_enum successfully.");
    }
}
