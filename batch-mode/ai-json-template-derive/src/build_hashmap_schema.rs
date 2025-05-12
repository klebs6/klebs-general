// ---------------- [ File: ai-json-template-derive/src/build_hashmap_schema.rs ]
crate::ix!();

/// Builds the schema for a `HashMap<K, V>`, handling special cases for K=bool, K=number, etc.
pub fn build_hashmap_schema(
    k_ty: &syn::Type,
    v_ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    trace!("build_hashmap_schema => K: {:?}, V: {:?}", k_ty, v_ty);

    // Key handling
    let key_schema = if is_bool(k_ty) {
        let err_msg = format!("Unsupported key type in HashMap<bool, _> for AiJsonTemplateWithJustification");
        let err = syn::Error::new(k_ty.span(), &err_msg);
        return Some(err.to_compile_error());
    } else if is_numeric(k_ty) {
        quote::quote! {
            serde_json::Value::String("number".to_string())
        }
    } else if is_string_type(k_ty) {
        quote::quote! {
            serde_json::Value::String("string".to_string())
        }
    } else {
        // Non-primitive key => treat as nested
        quote::quote! {
            {
                let mut k_obj = serde_json::Map::new();
                k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                k_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                k_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));

                let nested_k = <#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification();
                k_obj.insert("nested_template".to_string(), nested_k);
                serde_json::Value::Object(k_obj)
            }
        }
    };

    // Value handling
    let val_schema = if is_bool(v_ty) {
        quote::quote! {
            serde_json::Value::String("boolean".to_string())
        }
    } else if is_numeric(v_ty) {
        quote::quote! {
            serde_json::Value::String("number".to_string())
        }
    } else if is_string_type(v_ty) {
        quote::quote! {
            serde_json::Value::String("string".to_string())
        }
    } else {
        // Non-primitive => nested
        quote::quote! {
            {
                let mut v_obj = serde_json::Map::new();
                v_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
                v_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
                v_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));

                let nested_v = <#v_ty as AiJsonTemplateWithJustification>::to_template_with_justification();
                v_obj.insert("nested_template".to_string(), nested_v);
                serde_json::Value::Object(v_obj)
            }
        }
    };

    // Build final map_of
    Some(quote::quote! {
        {
            let mut map_obj = serde_json::Map::new();
            map_obj.insert("type".to_string(), serde_json::Value::String("map_of".to_string()));
            map_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            map_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            map_obj.insert("map_key_template".to_string(), #key_schema);
            map_obj.insert("map_value_template".to_string(), #val_schema);
            serde_json::Value::Object(map_obj)
        }
    })
}

#[cfg(test)]
mod test_build_hashmap_schema {
    use super::*;

    #[traced_test]
    fn test_bool_key_error() {
        trace!("Testing HashMap<bool, _> key => should produce compile_error");
        let k_ty: Type = parse_quote!(bool);
        let v_ty: Type = parse_quote!(String);

        // We simulate the required_bool token stream and doc literal
        let required_bool = quote!(true);
        let doc_lit = proc_macro2::Literal::string("Doc string for bool-key error test");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit);

        assert!(result.is_some(), "Expected Some(...) from build_hashmap_schema.");
        let output_ts = result.unwrap().to_string();
        debug!("Output TS for bool-key error => {}", output_ts);

        // Check for compile_error snippet
        // We do a rough contains() check because the exact formatting may vary.
        assert!(
            output_ts.contains("compile_error!"),
            "Expected compile_error! for HashMap<bool,_>, got: {}",
            output_ts
        );
    }

    #[traced_test]
    fn test_numeric_key_and_bool_value() {
        trace!("Testing HashMap<i32, bool> => numeric key => 'number', bool => 'boolean'");
        let k_ty: Type = parse_quote!(i32);
        let v_ty: Type = parse_quote!(bool);

        let required_bool = quote!(true);
        let doc_lit = proc_macro2::Literal::string("Doc numeric-key/bool-value");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit).unwrap();
        let result_str = result.to_string();
        debug!("Numeric key + bool value => result: {}", result_str);

        assert!(result_str.contains("\"type\":\"map_of\""));
        assert!(result_str.contains("\"map_key_template\":\"number\""), "Expected 'number' key schema");
        assert!(result_str.contains("\"map_value_template\":\"boolean\""), "Expected 'boolean' value schema");
        assert!(result_str.contains("\"required\":true"));
        assert!(result_str.contains("Doc numeric-key/bool-value"));
    }

    #[traced_test]
    fn test_string_key_and_numeric_value() {
        trace!("Testing HashMap<String, f64> => 'string' key => 'number' value");
        let k_ty: Type = parse_quote!(String);
        let v_ty: Type = parse_quote!(f64);

        let required_bool = quote!(false);
        let doc_lit = proc_macro2::Literal::string("Doc string-key/float-value");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit).unwrap();
        let result_str = result.to_string();
        debug!("String key + float value => result: {}", result_str);

        assert!(result_str.contains("\"type\":\"map_of\""));
        assert!(result_str.contains("\"map_key_template\":\"string\""));
        assert!(result_str.contains("\"map_value_template\":\"number\""));
        assert!(result_str.contains("\"required\":false"));
        assert!(result_str.contains("Doc string-key/float-value"));
    }

    #[traced_test]
    fn test_nested_key_and_value() {
        trace!("Testing HashMap<CustomKey, CustomValue> => nested schemas for both");
        let k_ty: Type = parse_quote!(MyCustomKeyType);
        let v_ty: Type = parse_quote!(MyCustomValueType);

        let required_bool = quote!(true);
        let doc_lit = proc_macro2::Literal::string("Doc nested key/value test");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit).unwrap();
        let result_str = result.to_string();
        debug!("Nested key + nested value => result: {}", result_str);

        // We expect:
        //   "type":"map_of"
        //   "map_key_template":{"type":"nested_struct_or_enum", ...}
        //   "map_value_template":{"type":"nested_struct_or_enum", ...}
        //   "required":true
        //   "generation_instructions":"Doc nested key/value test"
        assert!(result_str.contains("\"type\":\"map_of\""));
        assert!(
            result_str.contains("\"map_key_template\":{\"type\":\"nested_struct_or_enum\"")
            || result_str.contains("\"map_key_template\": { \"type\":\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum for key"
        );
        assert!(
            result_str.contains("\"map_value_template\":{\"type\":\"nested_struct_or_enum\"")
            || result_str.contains("\"map_value_template\": { \"type\":\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum for value"
        );
        assert!(result_str.contains("\"required\":true"));
        assert!(result_str.contains("Doc nested key/value test"));
    }

    #[traced_test]
    fn test_nested_value_only() {
        trace!("Testing HashMap<String, MyCustomValueType> => 'string' key => nested value");
        let k_ty: Type = parse_quote!(String);
        let v_ty: Type = parse_quote!(MyCustomValueType);

        let required_bool = quote!(true);
        let doc_lit = proc_macro2::Literal::string("Doc for nested value only");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit).unwrap();
        let result_str = result.to_string();
        debug!("String key + nested value => result: {}", result_str);

        // Key => "string"
        // Value => "nested_struct_or_enum"
        assert!(result_str.contains("\"type\":\"map_of\""));
        assert!(result_str.contains("\"map_key_template\":\"string\""));
        assert!(
            result_str.contains("\"map_value_template\":{\"type\":\"nested_struct_or_enum\"")
            || result_str.contains("\"map_value_template\": { \"type\":\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum for value"
        );
        assert!(result_str.contains("\"required\":true"));
        assert!(result_str.contains("Doc for nested value only"));
    }

    #[traced_test]
    fn test_optional_value() {
        trace!("Testing HashMap<String, Option<i32>> => 'string' key => 'number' if unwrapped, but not required");
        // This doesn't directly change the key, but let's see how the function reacts
        // to v_ty=Option<i32>. It's currently not part of the primary logic for build_hashmap_schema,
        // but let's test the scenario anyway for coverage.
        let k_ty: Type = parse_quote!(String);
        let v_ty: Type = parse_quote!(Option<i32>);

        let required_bool = quote!(false);
        let doc_lit = proc_macro2::Literal::string("Doc optional value test");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit).unwrap();
        let result_str = result.to_string();
        debug!("String key + Option<i32> => result: {}", result_str);

        // Because the logic doesn't specifically unwrap Option<T> in this function, we expect that
        // it will treat the entire type as "nested_struct_or_enum" or similar. Let's just check we
        // didn't crash or produce nonsense.
        assert!(result_str.contains("\"type\":\"map_of\""));
        assert!(result_str.contains("\"map_key_template\":\"string\""));
        // We expect a nested fallback for the value (since the function doesn't detect Option).
        // So we see "nested_struct_or_enum" for the map_value_template.
        // The doc string should match as well.
        assert!(
            result_str.contains("\"map_value_template\":{\"type\":\"nested_struct_or_enum\"")
            || result_str.contains("\"map_value_template\": { \"type\":\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum for Option<T> fallback"
        );
        assert!(result_str.contains("\"required\":false"));
        assert!(result_str.contains("Doc optional value test"));
    }
}
