// ---------------- [ File: ai-json-template-derive/src/build_named_field_child_schema_expr.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn build_named_field_child_schema_expr(
    field: &syn::Field,
    doc_str: &str,
    is_required: bool,
    skip_child_just: bool,
) -> Option<proc_macro2::TokenStream> {
    tracing::trace!(
        "build_named_field_child_schema_expr: field='{}', required={}, skip_child_just={}",
        field
            .ident
            .as_ref()
            .map_or("<unnamed?>".to_string(), |i| i.to_string().to_string()),
        is_required,
        skip_child_just
    );

    classify_field_type_for_child(&field.ty, doc_str, is_required, skip_child_just)
}

#[cfg(test)]
mod test_build_named_field_child_schema_expr_exhaustive {
    use super::*;

    /// A helper function to parse a single named field of the form:
    /// `struct Temp { <field_tokens> }`
    /// then extract that single named field for testing.
    fn parse_single_named_field(field_tokens: proc_macro2::TokenStream) -> Field {
        trace!("parse_single_named_field: parsing field tokens = {}", field_tokens.to_string());
        let full_struct: syn::DeriveInput = parse_quote! {
            struct Temp {
                #field_tokens
            }
        };
        // Extract the sole field from the struct
        match full_struct.data {
            syn::Data::Struct(ds) => {
                match ds.fields {
                    syn::Fields::Named(named) => {
                        if named.named.len() != 1 {
                            panic!("Expected exactly one field in 'Temp' but found {}", named.named.len());
                        }
                        named.named.into_iter().next().unwrap()
                    }
                    _ => panic!("Expected named fields in parse_single_named_field"),
                }
            }
            _ => panic!("Expected a struct with named fields in parse_single_named_field"),
        }
    }

    #[traced_test]
    fn test_bool_required_skip_false() {
        trace!("Starting test_bool_required_skip_false");
        let field_tokens = quote! { my_bool: bool };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Documentation for bool", true, false);
        assert!(result.is_some(), "Expected a schema for bool field");
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        assert!(ts.to_string().contains("\"boolean\""));
        assert!(ts.to_string().contains("\"required\":true"));
        assert!(ts.to_string().contains("\"Documentation for bool"));
        info!("test_bool_required_skip_false passed.");
    }

    #[traced_test]
    fn test_bool_not_required_skip_false() {
        trace!("Starting test_bool_not_required_skip_false");
        let field_tokens = quote! { opt_bool: Option<bool> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for optional bool", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // Should still mention "boolean", but required=false
        assert!(ts.to_string().contains("\"boolean\""));
        assert!(ts.to_string().contains("\"required\":false"));
        assert!(ts.to_string().contains("\"Doc for optional bool"));
        info!("test_bool_not_required_skip_false passed.");
    }

    #[traced_test]
    fn test_bool_skip_child_just_true() {
        trace!("Starting test_bool_skip_child_just_true");
        let field_tokens = quote! { my_bool: bool };
        let field = parse_single_named_field(field_tokens);
        // is_required = true, skip_child_just = true
        let result = build_named_field_child_schema_expr(&field, "doc skip child", true, true);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // Should still produce a boolean schema, but skip_child_just doesn't really alter a primitive.
        assert!(ts.to_string().contains("\"boolean\""));
        assert!(ts.to_string().contains("\"required\":true"));
        assert!(ts.to_string().contains("\"doc skip child"));
        info!("test_bool_skip_child_just_true passed.");
    }

    #[traced_test]
    fn test_string_required() {
        trace!("Starting test_string_required");
        let field_tokens = quote! { name: String };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for string field", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // "string" type => must contain "type":"string"
        assert!(ts.to_string().contains("\"string\""));
        assert!(ts.to_string().contains("\"required\":true"));
        assert!(ts.to_string().contains("\"Doc for string field"));
        info!("test_string_required passed.");
    }

    #[traced_test]
    fn test_string_not_required() {
        trace!("Starting test_string_not_required");
        let field_tokens = quote! { maybe_name: Option<String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional string doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        assert!(ts.to_string().contains("\"string\""));
        assert!(ts.to_string().contains("\"required\":false"));
        info!("test_string_not_required passed.");
    }

    #[traced_test]
    fn test_numeric_required() {
        trace!("Starting test_numeric_required");
        let field_tokens = quote! { count: i32 };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Integer doc", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // "number" type => must contain "type":"number"
        assert!(ts.to_string().contains("\"number\""));
        assert!(ts.to_string().contains("\"required\":true"));
        assert!(ts.to_string().contains("\"Integer doc"));
        info!("test_numeric_required passed.");
    }

    #[traced_test]
    fn test_numeric_not_required() {
        trace!("Starting test_numeric_not_required");
        let field_tokens = quote! { maybe_count: Option<i64> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional number doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        assert!(ts.to_string().contains("\"number\""));
        assert!(ts.to_string().contains("\"required\":false"));
        info!("test_numeric_not_required passed.");
    }

    #[traced_test]
    fn test_vec_of_strings_required() {
        trace!("Starting test_vec_of_strings_required");
        let field_tokens = quote! { tags: Vec<String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for vector of strings", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // Should mention "array_of" and also "string" for item_template
        assert!(ts.to_string().contains("\"array_of\""));
        assert!(ts.to_string().contains("\"item_template\":"));
        assert!(ts.to_string().contains("\"string\""));
        info!("test_vec_of_strings_required passed.");
    }

    #[traced_test]
    fn test_vec_of_numeric_not_required() {
        trace!("Starting test_vec_of_numeric_not_required");
        let field_tokens = quote! { amounts: Option<Vec<u64>> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional vector doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // "array_of" with "number" inside
        assert!(ts.to_string().contains("\"array_of\""));
        assert!(ts.to_string().contains("\"number\""));
        assert!(ts.to_string().contains("\"required\":false"));
        info!("test_vec_of_numeric_not_required passed.");
    }

    #[traced_test]
    fn test_hashmap_string_key_not_required() {
        trace!("Starting test_hashmap_string_key_not_required");
        let field_tokens = quote! { map_data: Option<std::collections::HashMap<String, i32>> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for map", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // Expect "map_of"
        assert!(ts.to_string().contains("\"map_of\""));
        // key is "string", value is "number"
        assert!(ts.to_string().contains("\"map_key_template\""));
        assert!(ts.to_string().contains("\"map_value_template\""));
        assert!(ts.to_string().contains("\"string\""));
        assert!(ts.to_string().contains("\"number\""));
        assert!(ts.to_string().contains("\"Doc for map"));
        info!("test_hashmap_string_key_not_required passed.");
    }

    #[traced_test]
    fn test_hashmap_bool_key_should_error() {
        trace!("Starting test_hashmap_bool_key_should_error");
        let field_tokens = quote! { bad_map: std::collections::HashMap<bool, String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Bad map doc", true, false);
        // This should produce Some(...) containing a compile_error about unsupported bool keys
        match result {
            Some(ts) => {
                let s = ts.to_string();
                debug!("Generated TokenStream for error: {}", s);
                // Expect a compile_error
                assert!(s.contains("compile_error"));
                assert!(s.contains("Unsupported key type in HashMap<bool"));
                info!("test_hashmap_bool_key_should_error passed as expected.");
            }
            None => panic!("Expected Some(...) with compile_error for bool key"),
        }
    }

    #[traced_test]
    fn test_nested_custom_type() {
        trace!("Starting test_nested_custom_type");
        // We'll pretend there's a type named "MyStruct" in scope
        let field_tokens = quote! { custom: MyStruct };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for nested custom", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // Should produce "nested_struct_or_enum"
        assert!(ts.to_string().contains("\"nested_struct_or_enum\""));
        assert!(ts.to_string().contains("\"required\":true"));
        assert!(ts.to_string().contains("\"nested_template\""));
        info!("test_nested_custom_type passed.");
    }

    #[traced_test]
    fn test_optional_nested_custom_type() {
        trace!("Starting test_optional_nested_custom_type");
        // We'll pretend there's a type named "MyEnum" in scope
        let field_tokens = quote! { maybe_custom: Option<MyEnum> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for optional nested", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());
        // "nested_struct_or_enum"
        assert!(ts.to_string().contains("\"nested_struct_or_enum\""));
        // required should be false
        assert!(ts.to_string().contains("\"required\":false"));
        info!("test_optional_nested_custom_type passed.");
    }
}
